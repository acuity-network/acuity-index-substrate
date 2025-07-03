use crate::shared::*;
use futures::{SinkExt, StreamExt};
use sled::Tree;
use std::net::SocketAddr;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::metadata::types::Metadata;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{
    mpsc::{UnboundedSender, unbounded_channel},
    watch::Receiver,
};
use tokio_tungstenite::tungstenite;
use tracing::{error, info};
use zerocopy::FromBytes;

pub fn process_msg_status<R: RuntimeIndexer>(span_db: &Tree) -> ResponseMessage<R::ChainKey> {
    let mut spans = vec![];
    for (key, value) in span_db.into_iter().flatten() {
        let span_value = SpanDbValue::read_from(&value).unwrap();
        let start: u32 = span_value.start.into();
        let end: u32 = u32::from_be_bytes(key.as_ref().try_into().unwrap());
        let span = Span { start, end };
        spans.push(span);
    }
    ResponseMessage::Status(spans)
}

pub fn process_msg_subscribe_status<R: RuntimeIndexer>(
    sub_tx: &UnboundedSender<SubscriptionMessage<R::ChainKey>>,
    sub_response_tx: &UnboundedSender<ResponseMessage<R::ChainKey>>,
) -> ResponseMessage<R::ChainKey> {
    let msg = SubscriptionMessage::SubscribeStatus {
        sub_response_tx: sub_response_tx.clone(),
    };
    sub_tx.send(msg).unwrap();
    ResponseMessage::Subscribed
}

pub fn process_msg_unsubscribe_status<R: RuntimeIndexer>(
    sub_tx: &UnboundedSender<SubscriptionMessage<R::ChainKey>>,
    sub_response_tx: &UnboundedSender<ResponseMessage<R::ChainKey>>,
) -> ResponseMessage<R::ChainKey> {
    let msg = SubscriptionMessage::UnsubscribeStatus {
        sub_response_tx: sub_response_tx.clone(),
    };
    sub_tx.send(msg).unwrap();
    ResponseMessage::Unsubscribed
}

pub async fn process_msg_variants<R: RuntimeIndexer>(
    rpc: &LegacyRpcMethods<R::RuntimeConfig>,
) -> Result<ResponseMessage<R::ChainKey>, IndexError> {
    let metadata: Metadata = rpc
        .state_get_metadata(None)
        .await?
        .to_frame_metadata()?
        .try_into()?;
    let mut pallets = Vec::new();

    for pallet in metadata.pallets() {
        let mut pallet_meta = PalletMeta {
            index: pallet.index(),
            name: pallet.name().to_owned(),
            events: Vec::new(),
        };

        if let Some(variants) = pallet.event_variants() {
            for variant in variants {
                pallet_meta.events.push(EventMeta {
                    index: variant.index,
                    name: variant.name.clone(),
                })
            }
            pallets.push(pallet_meta);
        }
    }
    Ok(ResponseMessage::Variants(pallets))
}

pub fn get_events_variant(tree: &Tree, pallet_id: u8, variant_id: u8) -> Vec<Event> {
    let mut events = Vec::new();
    let mut iter = tree.scan_prefix([pallet_id, variant_id]).keys();

    while let Some(Ok(key)) = iter.next_back() {
        let key = VariantKey::read_from(&key).unwrap();

        events.push(Event {
            block_number: key.block_number.into(),
            event_index: key.event_index.into(),
        });

        if events.len() == 100 {
            break;
        }
    }
    events
}

pub fn get_events_bytes32(tree: &Tree, key: &Bytes32) -> Vec<Event> {
    let mut events = Vec::new();
    let mut iter = tree.scan_prefix(key).keys();

    while let Some(Ok(key)) = iter.next_back() {
        let key = Bytes32Key::read_from(&key).unwrap();

        events.push(Event {
            block_number: key.block_number.into(),
            event_index: key.event_index.into(),
        });

        if events.len() == 100 {
            break;
        }
    }
    events
}

pub fn get_events_u32(tree: &Tree, key: u32) -> Vec<Event> {
    let mut events = Vec::new();
    let mut iter = tree.scan_prefix(key.to_be_bytes()).keys();

    while let Some(Ok(key)) = iter.next_back() {
        let key = U32Key::read_from(&key).unwrap();

        events.push(Event {
            block_number: key.block_number.into(),
            event_index: key.event_index.into(),
        });

        if events.len() == 100 {
            break;
        }
    }
    events
}

pub fn process_msg_get_events_substrate<R: RuntimeIndexer>(
    trees: &Trees<<R::ChainKey as IndexKey>::ChainTrees>,
    key: &SubstrateKey,
) -> Vec<Event> {
    match key {
        SubstrateKey::AccountId(account_id) => {
            get_events_bytes32(&trees.substrate.account_id, account_id)
        }
        SubstrateKey::AccountIndex(account_index) => {
            get_events_u32(&trees.substrate.account_index, *account_index)
        }
        SubstrateKey::BountyIndex(bounty_index) => {
            get_events_u32(&trees.substrate.bounty_index, *bounty_index)
        }
        SubstrateKey::EraIndex(era_index) => get_events_u32(&trees.substrate.era_index, *era_index),
        SubstrateKey::MessageId(message_id) => {
            get_events_bytes32(&trees.substrate.message_id, message_id)
        }
        SubstrateKey::PoolId(pool_id) => get_events_u32(&trees.substrate.pool_id, *pool_id),
        SubstrateKey::PreimageHash(preimage_hash) => {
            get_events_bytes32(&trees.substrate.preimage_hash, preimage_hash)
        }
        SubstrateKey::ProposalHash(proposal_hash) => {
            get_events_bytes32(&trees.substrate.proposal_hash, proposal_hash)
        }
        SubstrateKey::ProposalIndex(proposal_index) => {
            get_events_u32(&trees.substrate.proposal_index, *proposal_index)
        }
        SubstrateKey::RefIndex(ref_index) => get_events_u32(&trees.substrate.ref_index, *ref_index),
        SubstrateKey::RegistrarIndex(registrar_index) => {
            get_events_u32(&trees.substrate.registrar_index, *registrar_index)
        }
        SubstrateKey::SessionIndex(session_index) => {
            get_events_u32(&trees.substrate.session_index, *session_index)
        }
        SubstrateKey::TipHash(tip_hash) => get_events_bytes32(&trees.substrate.tip_hash, tip_hash),
        SubstrateKey::SpendIndex(spend_index) => {
            get_events_u32(&trees.substrate.spend_index, *spend_index)
        }
    }
}

pub fn process_msg_get_events<R: RuntimeIndexer>(
    trees: &Trees<<R::ChainKey as IndexKey>::ChainTrees>,
    key: Key<R::ChainKey>,
) -> ResponseMessage<R::ChainKey> {
    let events = match key {
        Key::Variant(pallet_id, variant_id) => {
            get_events_variant(&trees.variant, pallet_id, variant_id)
        }
        Key::Substrate(ref key) => process_msg_get_events_substrate::<R>(trees, key),
        Key::Chain(ref key) => key.get_key_events(&trees.chain),
    };
    ResponseMessage::Events { key, events }
}

pub fn process_msg_subscribe_events<R: RuntimeIndexer>(
    key: Key<R::ChainKey>,
    sub_tx: &UnboundedSender<SubscriptionMessage<R::ChainKey>>,
    sub_response_tx: &UnboundedSender<ResponseMessage<R::ChainKey>>,
) -> ResponseMessage<R::ChainKey> {
    let msg = SubscriptionMessage::SubscribeEvents {
        key,
        sub_response_tx: sub_response_tx.clone(),
    };
    sub_tx.send(msg).unwrap();
    ResponseMessage::Subscribed
}

pub fn process_msg_unsubscribe_events<R: RuntimeIndexer>(
    key: Key<R::ChainKey>,
    sub_tx: &UnboundedSender<SubscriptionMessage<R::ChainKey>>,
    sub_response_tx: &UnboundedSender<ResponseMessage<R::ChainKey>>,
) -> ResponseMessage<R::ChainKey> {
    let msg = SubscriptionMessage::UnsubscribeEvents {
        key,
        sub_response_tx: sub_response_tx.clone(),
    };
    sub_tx.send(msg).unwrap();
    ResponseMessage::Unsubscribed
}

pub async fn process_msg<R: RuntimeIndexer>(
    rpc: &LegacyRpcMethods<R::RuntimeConfig>,
    trees: &Trees<<R::ChainKey as IndexKey>::ChainTrees>,
    msg: RequestMessage<R::ChainKey>,
    sub_tx: &UnboundedSender<SubscriptionMessage<R::ChainKey>>,
    sub_response_tx: &UnboundedSender<ResponseMessage<R::ChainKey>>,
) -> Result<ResponseMessage<R::ChainKey>, IndexError> {
    Ok(match msg {
        RequestMessage::Status => process_msg_status::<R>(&trees.span),
        RequestMessage::SubscribeStatus => {
            process_msg_subscribe_status::<R>(sub_tx, sub_response_tx)
        }
        RequestMessage::UnsubscribeStatus => {
            process_msg_unsubscribe_status::<R>(sub_tx, sub_response_tx)
        }
        RequestMessage::Variants => process_msg_variants::<R>(rpc).await?,
        RequestMessage::GetEvents { key } => process_msg_get_events::<R>(trees, key),
        RequestMessage::SubscribeEvents { key } => {
            process_msg_subscribe_events::<R>(key, sub_tx, sub_response_tx)
        }
        RequestMessage::UnsubscribeEvents { key } => {
            process_msg_unsubscribe_events::<R>(key, sub_tx, sub_response_tx)
        }
        RequestMessage::SizeOnDisk => ResponseMessage::SizeOnDisk(trees.root.size_on_disk()?),
    })
}

async fn handle_connection<R: RuntimeIndexer>(
    rpc: LegacyRpcMethods<R::RuntimeConfig>,
    raw_stream: TcpStream,
    addr: SocketAddr,
    trees: Trees<<R::ChainKey as IndexKey>::ChainTrees>,
    sub_tx: UnboundedSender<SubscriptionMessage<R::ChainKey>>,
) -> Result<(), IndexError> {
    info!("Incoming TCP connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream).await?;
    info!("WebSocket connection established: {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    // Create the channel for the substrate thread to send event messages to this thread.
    let (sub_events_tx, mut sub_events_rx) = unbounded_channel();

    loop {
        tokio::select! {
            Some(Ok(msg)) = ws_receiver.next() => {
                if msg.is_text() || msg.is_binary() {
                    match serde_json::from_str(msg.to_text()?) {
                        Ok(request_json) => {
                            let response_msg = process_msg::<R>(&rpc, &trees, request_json, &sub_tx, &sub_events_tx).await?;
                            let response_json = serde_json::to_string(&response_msg).unwrap();
                            ws_sender.send(tungstenite::Message::Text(response_json)).await?;
                        },
                        Err(error) => error!("{}", error),
                    }
                }
            },
            Some(msg) = sub_events_rx.recv() => {
                let response_json = serde_json::to_string(&msg).unwrap();
                ws_sender.send(tungstenite::Message::Text(response_json)).await?;
            },
        }
    }
}

pub async fn websockets_listen<R: RuntimeIndexer + 'static>(
    trees: Trees<<R::ChainKey as IndexKey>::ChainTrees>,
    rpc: LegacyRpcMethods<R::RuntimeConfig>,
    port: u16,
    mut exit_rx: Receiver<bool>,
    sub_tx: UnboundedSender<SubscriptionMessage<R::ChainKey>>,
) {
    let mut addr = "0.0.0.0:".to_string();
    addr.push_str(&port.to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    loop {
        tokio::select! {
            biased;

            _ = exit_rx.changed() => {
                break;
            }
            Ok((stream, addr)) = listener.accept() => {
                tokio::spawn(handle_connection::<R>(
                    rpc.clone(),
                    stream,
                    addr,
                    trees.clone(),
                    sub_tx.clone(),
                ));
            }
        }
    }
}
