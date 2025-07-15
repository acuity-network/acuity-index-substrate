use ahash::AHashMap;
use futures::future;
use num_format::{Locale, ToFormattedString};
use sled::Tree;
use std::{collections::HashMap, future::Future, sync::Mutex};
use subxt::{OnlineClient, backend::legacy::LegacyRpcMethods, blocks::Block, metadata::Metadata};
use tokio::{
    sync::{RwLock, mpsc, watch},
    time::{self, Duration, Instant, MissedTickBehavior},
};
use tracing::{debug, error, info};
use zerocopy::{AsBytes, BigEndian, FromBytes, byteorder::U32};

use crate::{shared::*, websockets::process_msg_status};

#[allow(clippy::type_complexity)]
pub struct Indexer<R: RuntimeIndexer + ?Sized> {
    trees: Trees<<R::ChainKey as IndexKey>::ChainTrees>,
    api: Option<OnlineClient<R::RuntimeConfig>>,
    rpc: Option<LegacyRpcMethods<R::RuntimeConfig>>,
    index_variant: bool,
    store_events: bool,
    metadata_map_lock: RwLock<AHashMap<u32, Metadata>>,
    status_sub: Mutex<Vec<mpsc::UnboundedSender<ResponseMessage<R::ChainKey>>>>,
    events_sub_map:
        Mutex<HashMap<Key<R::ChainKey>, Vec<mpsc::UnboundedSender<ResponseMessage<R::ChainKey>>>>>,
}

impl<R: RuntimeIndexer> Indexer<R> {
    fn new(
        trees: Trees<<R::ChainKey as IndexKey>::ChainTrees>,
        api: OnlineClient<R::RuntimeConfig>,
        rpc: LegacyRpcMethods<R::RuntimeConfig>,
        index_variant: bool,
        store_events: bool,
    ) -> Self {
        Indexer {
            trees,
            api: Some(api),
            rpc: Some(rpc),
            index_variant,
            store_events,
            metadata_map_lock: RwLock::new(AHashMap::new()),
            status_sub: Vec::new().into(),
            events_sub_map: HashMap::new().into(),
        }
    }

    pub fn new_test(trees: Trees<<R::ChainKey as IndexKey>::ChainTrees>) -> Self {
        Indexer {
            trees,
            api: None,
            rpc: None,
            index_variant: true,
            store_events: true,
            metadata_map_lock: RwLock::new(AHashMap::new()),
            status_sub: Vec::new().into(),
            events_sub_map: HashMap::new().into(),
        }
    }

    async fn index_head(
        &self,
        next: impl Future<
            Output = Option<
                Result<Block<R::RuntimeConfig, OnlineClient<R::RuntimeConfig>>, subxt::Error>,
            >,
        >,
    ) -> Result<(u32, u32, u32), IndexError> {
        let block = next.await.unwrap()?;
        self.index_block(block.number().into().try_into().unwrap())
            .await
    }

    async fn index_block(&self, block_number: u32) -> Result<(u32, u32, u32), IndexError> {
        let mut key_count = 0;
        let api = self.api.as_ref().unwrap();
        let rpc = self.rpc.as_ref().unwrap();

        let block_hash = match rpc.chain_get_block_hash(Some(block_number.into())).await? {
            Some(block_hash) => block_hash,
            None => return Err(IndexError::BlockNotFound(block_number)),
        };
        // Get the runtime version of the block.
        let runtime_version = rpc.state_get_runtime_version(Some(block_hash)).await?;

        let metadata_map = self.metadata_map_lock.read().await;
        let metadata = match metadata_map.get(&runtime_version.spec_version) {
            Some(metadata) => {
                let metadata = metadata.clone();
                drop(metadata_map);
                metadata
            }
            None => {
                drop(metadata_map);
                let mut metadata_map = self.metadata_map_lock.write().await;

                match metadata_map.get(&runtime_version.spec_version) {
                    Some(metadata) => metadata.clone(),
                    None => {
                        info!(
                            "Downloading metadata for spec version {}",
                            runtime_version.spec_version
                        );
                        let metadata: Metadata = rpc
                            .state_get_metadata(Some(block_hash))
                            .await?
                            .to_frame_metadata()?
                            .try_into()?;
                        info!(
                            "Finished downloading metadata for spec version {}",
                            runtime_version.spec_version
                        );
                        metadata_map.insert(runtime_version.spec_version, metadata.clone());
                        metadata
                    }
                }
            }
        };

        let events =
            subxt::events::new_events_from_client(metadata, block_hash, api.clone()).await?;

        for (i, event) in events.iter().enumerate() {
            match event {
                Ok(event) => {
                    let event_index = i.try_into().unwrap();
                    if self.index_variant {
                        self.index_event(
                            Key::Variant(event.pallet_index(), event.variant_index()),
                            block_number,
                            event_index,
                        )?;
                        key_count += 1;
                    }
                    if let Ok(event_key_count) =
                        R::process_event(self, block_number, event_index, event)
                    {
                        key_count += event_key_count;
                    }
                }
                Err(error) => error!("Block: {}, error: {}", block_number, error),
            }
        }

        if self.store_events {
            let key: U32<BigEndian> = block_number.into();
            let spec_version: U32<BigEndian> = runtime_version.spec_version.into();

            self.trees.block_events.insert(
                key.as_bytes(),
                [spec_version.as_bytes(), events.bytes()].concat(),
            )?;
        }

        Ok((block_number, events.len(), key_count))
    }

    pub fn notify_status_subscribers(&self) {
        let msg = process_msg_status::<R>(&self.trees.span);
        let txs = self.status_sub.lock().unwrap();
        for tx in txs.iter() {
            if tx.send(msg.clone()).is_ok() {}
        }
    }

    pub fn notify_subscribers(&self, search_key: Key<R::ChainKey>, event: Event) {
        let events_sub_map = self.events_sub_map.lock().unwrap();
        if let Some(txs) = events_sub_map.get(&search_key) {
            let msg = ResponseMessage::Events {
                key: search_key,
                events: vec![event],
            };
            for tx in txs.iter() {
                if tx.send(msg.clone()).is_ok() {}
            }
        }
    }

    pub fn index_event(
        &self,
        key: Key<R::ChainKey>,
        block_number: u32,
        event_index: u16,
    ) -> Result<(), sled::Error> {
        key.write_db_key(&self.trees, block_number, event_index)?;
        self.notify_subscribers(
            key,
            Event {
                block_number,
                event_index,
            },
        );
        Ok(())
    }
}

pub fn load_spans<R: RuntimeIndexer>(
    span_db: &Tree,
    index_variant: bool,
    store_events: bool,
) -> Result<Vec<Span>, IndexError> {
    let mut spans = vec![];
    'span: for (key, value) in span_db.into_iter().flatten() {
        let span_value = SpanDbValue::read_from(&value).unwrap();
        let start: u32 = span_value.start.into();
        let mut end: u32 = u32::from_be_bytes(key.as_ref().try_into().unwrap());
        // Check if variants are supposed to be indexed and they were not in this span.
        if index_variant && (span_value.index_variant != 1) {
            // Delete the span.
            span_db.remove(key)?;
            info!(
                "📚 Re-indexing span of blocks from #{} to #{}.",
                start.to_formatted_string(&Locale::en),
                end.to_formatted_string(&Locale::en)
            );
            info!("📚 Reason: event variants not indexed.");
            continue;
        }
        // Check if events are supposed to be stored and they were not in this span.
        if store_events && (span_value.store_events != 1) {
            // Delete the span.
            span_db.remove(key)?;
            info!(
                "📚 Re-indexing span of blocks from #{} to #{}.",
                start.to_formatted_string(&Locale::en),
                end.to_formatted_string(&Locale::en)
            );
            info!("📚 Reason: events not stored.");
            continue;
        }
        let span_version: u16 = span_value.version.into();
        // Loop through each indexer version.
        for (version, block_number) in R::get_versions().iter().enumerate() {
            if span_version < version.try_into().unwrap() && end >= *block_number {
                span_db.remove(key)?;
                if start >= *block_number {
                    info!(
                        "📚 Re-indexing span of blocks from #{} to #{}.",
                        start.to_formatted_string(&Locale::en),
                        end.to_formatted_string(&Locale::en)
                    );
                    continue 'span;
                }
                info!(
                    "📚 Re-indexing span of blocks from #{} to #{}.",
                    block_number.to_formatted_string(&Locale::en),
                    end.to_formatted_string(&Locale::en)
                );
                // Truncate the span.
                end = block_number - 1;
                span_db.insert(end.to_be_bytes(), value)?;
                break;
            }
        }
        let span = Span { start, end };
        info!(
            "📚 Previous span of indexed blocks from #{} to #{}.",
            start.to_formatted_string(&Locale::en),
            end.to_formatted_string(&Locale::en)
        );
        spans.push(span);
    }
    Ok(spans)
}

pub fn check_span(
    span_db: &Tree,
    spans: &mut Vec<Span>,
    current_span: &mut Span,
) -> Result<(), IndexError> {
    while let Some(span) = spans.last() {
        // Have we indexed all the blocks after the span?
        if current_span.start > span.start && current_span.start - 1 <= span.end {
            let skipped = span.end - span.start + 1;
            info!(
                "📚 Skipping {} blocks from #{} to #{}",
                skipped.to_formatted_string(&Locale::en),
                span.start.to_formatted_string(&Locale::en),
                span.end.to_formatted_string(&Locale::en),
            );
            current_span.start = span.start;
            // Remove the span.
            span_db.remove(span.end.to_be_bytes())?;
            spans.pop();
        } else {
            break;
        }
    }
    Ok(())
}

pub fn check_next_batch_block(spans: &[Span], next_batch_block: &mut u32) {
    // Figure out the next block to index, skipping the next span if we have reached it.
    let mut i = spans.len();
    while i != 0 {
        i -= 1;
        if *next_batch_block >= spans[i].start && *next_batch_block <= spans[i].end {
            *next_batch_block = spans[i].start - 1;
        }
    }
}

pub fn process_sub_msg<R: RuntimeIndexer>(
    indexer: &Indexer<R>,
    msg: SubscriptionMessage<R::ChainKey>,
) {
    match msg {
        SubscriptionMessage::SubscribeStatus { sub_response_tx } => {
            let mut txs = indexer.status_sub.lock().unwrap();
            txs.push(sub_response_tx);
        }
        SubscriptionMessage::UnsubscribeStatus { sub_response_tx } => {
            let mut txs = indexer.status_sub.lock().unwrap();
            txs.retain(|value| !sub_response_tx.same_channel(value));
        }
        SubscriptionMessage::SubscribeEvents {
            key,
            sub_response_tx,
        } => {
            let mut events_sub_map = indexer.events_sub_map.lock().unwrap();
            match events_sub_map.get_mut(&key) {
                Some(txs) => {
                    txs.push(sub_response_tx);
                }
                None => {
                    let txs = vec![sub_response_tx];
                    events_sub_map.insert(key, txs);
                }
            };
        }
        SubscriptionMessage::UnsubscribeEvents {
            key,
            sub_response_tx,
        } => {
            let mut events_sub_map = indexer.events_sub_map.lock().unwrap();
            if let Some(txs) = events_sub_map.get_mut(&key) {
                txs.retain(|value| !sub_response_tx.same_channel(value));
            };
        }
    };
}

pub async fn substrate_index<R: RuntimeIndexer>(
    trees: Trees<<R::ChainKey as IndexKey>::ChainTrees>,
    api: OnlineClient<R::RuntimeConfig>,
    rpc: LegacyRpcMethods<R::RuntimeConfig>,
    queue_depth: u32,
    index_variant: bool,
    store_events: bool,
    mut exit_rx: watch::Receiver<bool>,
    mut sub_rx: mpsc::UnboundedReceiver<SubscriptionMessage<R::ChainKey>>,
) -> Result<(), IndexError> {
    info!(
        "📇 Event variant indexing: {}",
        match index_variant {
            false => "disabled",
            true => "enabled",
        },
    );
    info!(
        "📇 Event storage: {}",
        match store_events {
            false => "disabled",
            true => "enabled",
        },
    );
    // Subscribe to all finalized blocks:
    let mut blocks_sub = api.blocks().subscribe_finalized().await?;
    // Determine the correct block to start batch indexing.
    let mut next_batch_block: u32 = blocks_sub
        .next()
        .await
        .ok_or(IndexError::BlockNotFound(0))??
        .number()
        .into()
        .try_into()
        .unwrap();
    info!(
        "📚 Indexing backwards from #{}",
        next_batch_block.to_formatted_string(&Locale::en)
    );
    // Load already indexed spans from the db.
    let mut spans = load_spans::<R>(&trees.span, index_variant, store_events)?;
    // If the first head block to be indexed will be touching the last span (the indexer was restarted), set the current span to the last span. Otherwise there will be no batch block indexed to connect the current span to the last span.
    let mut current_span = if let Some(span) = spans.last()
        && span.end == next_batch_block
    {
        let span = span.clone();
        let skipped = span.end - span.start + 1;
        info!(
            "📚 Skipping {} blocks from #{} to #{}",
            skipped.to_formatted_string(&Locale::en),
            span.start.to_formatted_string(&Locale::en),
            span.end.to_formatted_string(&Locale::en),
        );
        // Remove the span.
        trees.span.remove(span.end.to_be_bytes())?;
        spans.pop();
        next_batch_block = span.start - 1;
        span
    } else {
        Span {
            start: next_batch_block + 1,
            end: next_batch_block + 1,
        }
    };

    let indexer = Indexer::<R>::new(trees.clone(), api, rpc, index_variant, store_events);

    let mut head_future = Box::pin(indexer.index_head(blocks_sub.next()));

    info!("📚 Queue depth: {}", queue_depth);
    let mut futures = Vec::with_capacity(queue_depth.try_into().unwrap());

    for _ in 0..queue_depth {
        check_next_batch_block(&spans, &mut next_batch_block);
        futures.push(Box::pin(indexer.index_block(next_batch_block)));
        debug!(
            "⬆️  Block #{} queued.",
            next_batch_block.to_formatted_string(&Locale::en)
        );
        next_batch_block -= 1;
    }

    let mut orphans: AHashMap<u32, ()> = AHashMap::new();

    let mut stats_block_count = 0;
    let mut stats_event_count = 0;
    let mut stats_key_count = 0;
    let mut stats_start_time = Instant::now();

    let interval_duration = Duration::from_millis(2000);
    let mut interval = time::interval_at(Instant::now() + interval_duration, interval_duration);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut is_batching = true;

    loop {
        tokio::select! {
            biased;

            _ = exit_rx.changed() => {
                if current_span.start != current_span.end {
                    let value = SpanDbValue {
                        start: current_span.start.into(),
                        version: (R::get_versions().len() - 1).try_into().unwrap(),
                        index_variant: index_variant.into(),
                        store_events: store_events.into(),
                    };
                    trees.span.insert(current_span.end.to_be_bytes(), value.as_bytes())?;
                    info!(
                        "📚 Recording current indexed span from #{} to #{}",
                        current_span.start.to_formatted_string(&Locale::en),
                        current_span.end.to_formatted_string(&Locale::en)
                    );
                }
                return Ok(());
            }
            Some(msg) = sub_rx.recv() => process_sub_msg(&indexer, msg),
            result = &mut head_future => {
                match result {
                    Ok((block_number, event_count, key_count)) => {
                        trees.span.remove(current_span.end.to_be_bytes())?;
                        current_span.end = block_number;
                        let value = SpanDbValue {
                            start: current_span.start.into(),
                            version: (R::get_versions().len() - 1).try_into().unwrap(),
                            index_variant: index_variant.into(),
                            store_events: store_events.into(),
                        };
                        trees.span.insert(current_span.end.to_be_bytes(), value.as_bytes())?;
                        info!(
                            "✨ #{}: {} events, {} keys",
                            block_number.to_formatted_string(&Locale::en),
                            event_count.to_formatted_string(&Locale::en),
                            key_count.to_formatted_string(&Locale::en),
                        );
                        indexer.notify_status_subscribers();
                        drop(head_future);
                        head_future = Box::pin(indexer.index_head(blocks_sub.next()));
                    },
                    Err(error) => {
                        match error {
                            IndexError::BlockNotFound(block_number) => {
                                error!("✨ Block not found #{}", block_number.to_formatted_string(&Locale::en));
                            },
                            err => {
                                error!("✨ Indexing failed: {}", err);
                            },
                        }
                    },
                };
            }
            _ = interval.tick(), if is_batching => {
                let current_time = Instant::now();
                let duration = (current_time.duration_since(stats_start_time)).as_micros();
                if duration != 0 {
                    info!(
                        "📚 #{}: {} blocks/sec, {} events/sec, {} keys/sec",
                        current_span.start.to_formatted_string(&Locale::en),
                        (<u32 as Into<u128>>::into(stats_block_count) * 1_000_000 / duration).to_formatted_string(&Locale::en),
                        (<u32 as Into<u128>>::into(stats_event_count) * 1_000_000 / duration).to_formatted_string(&Locale::en),
                        (<u32 as Into<u128>>::into(stats_key_count) * 1_000_000 / duration).to_formatted_string(&Locale::en),
                    );
                }
                stats_block_count = 0;
                stats_event_count = 0;
                stats_key_count = 0;
                stats_start_time = current_time;
            }
            (result, index, _) = future::select_all(&mut futures), if is_batching => {
                match result {
                    Ok((block_number, event_count, key_count)) => {
                        // Is the new block contiguous to the current span or an orphan?
                        if block_number == current_span.start - 1 {
                            current_span.start = block_number;
                            debug!("⬇️  Block #{} indexed.", block_number.to_formatted_string(&Locale::en));
                            check_span(&trees.span, &mut spans, &mut current_span)?;
                            // Check if any orphans are now contiguous.
                            while orphans.contains_key(&(current_span.start - 1)) {
                                current_span.start -= 1;
                                orphans.remove(&current_span.start);
                                debug!("➡️  Block #{} unorphaned.", current_span.start.to_formatted_string(&Locale::en));
                                check_span(&trees.span, &mut spans, &mut current_span)?;
                            }
                        }
                        else {
                            orphans.insert(block_number, ());
                            debug!("⬇️  Block #{} indexed and orphaned.", block_number.to_formatted_string(&Locale::en));
                        }
                        stats_block_count += 1;
                        stats_event_count += event_count;
                        stats_key_count += key_count;
                    },
                    Err(error) => {
                        match error {
                            IndexError::BlockNotFound(block_number) => {
                                error!("📚 Block not found #{}", block_number.to_formatted_string(&Locale::en));
                                is_batching = false;
                            },
                            _ => {
                                error!("📚 Batch indexing failed: {:?}", error);
                                is_batching = false;
                            },
                        }
                    }
                }
                check_next_batch_block(&spans, &mut next_batch_block);
                futures[index] = Box::pin(indexer.index_block(next_batch_block));
                debug!("⬆️  Block #{} queued.", next_batch_block.to_formatted_string(&Locale::en));
                next_batch_block -= 1;
            }
        }
    }
}
