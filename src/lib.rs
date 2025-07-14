//! # Acuity Index Substrate
//!
//! A library for indexing events from Substrate blockchains.

use byte_unit::Byte;
use futures::StreamExt;
use signal_hook::{consts::TERM_SIGNALS, flag};
use signal_hook_tokio::Signals;
use std::{
    path::PathBuf,
    process::exit,
    sync::{Arc, atomic::AtomicBool},
};
use subxt::{
    OnlineClient,
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
};
use tokio::{
    join, spawn,
    sync::{mpsc, watch},
};
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

pub mod shared;
pub mod substrate;
pub mod substrate_pallets;
pub mod websockets;

use crate::shared::*;
use substrate::*;
use websockets::websockets_listen;

#[cfg(test)]
mod tests;

pub fn open_trees<R: RuntimeIndexer>(
    db_config: sled::Config,
) -> Result<Trees<<R::ChainKey as IndexKey>::ChainTrees>, sled::Error> {
    let db = db_config.open()?;
    let trees = Trees {
        root: db.clone(),
        span: db.open_tree(b"span")?,
        variant: db.open_tree(b"variant")?,
        // Each event parameter to be indexed has its own tree.
        substrate: SubstrateTrees::open(&db)?,
        chain: <R::ChainKey as IndexKey>::ChainTrees::open(&db)?,
        block_events: db.open_tree(b"block_events")?,
    };
    Ok(trees)
}

pub fn close_trees<R: RuntimeIndexer>(
    trees: Trees<<R::ChainKey as IndexKey>::ChainTrees>,
) -> Result<(), sled::Error> {
    info!("Closing db.");
    trees.root.flush()?;
    trees.span.flush()?;
    trees.variant.flush()?;
    trees.substrate.flush()?;
    trees.block_events.flush()?;
    Ok(())
}

/// Starts the indexer. Chain is defined by `R`.
#[allow(clippy::too_many_arguments)]
pub async fn start<R: RuntimeIndexer + 'static>(
    db_path: Option<String>,
    db_mode: sled::Mode,
    db_cache_capacity: u64,
    url: Option<String>,
    queue_depth: u8,
    index_variant: bool,
    store_events: bool,
    port: u16,
    log_level: LevelFilter,
) {
    tracing_subscriber::fmt().with_max_level(log_level).init();
    let name = R::get_name();
    info!("Indexing {}", name);
    let genesis_hash_config = R::get_genesis_hash().as_ref().to_vec();
    // Open database.
    let db_path = match db_path {
        Some(db_path) => PathBuf::from(db_path),
        None => match home::home_dir() {
            Some(mut db_path) => {
                db_path.push(".local/share/acuity-index/");
                db_path.push(name);
                db_path.push("db");
                db_path
            }
            None => {
                error!("No home directory.");
                exit(1);
            }
        },
    };
    info!("Database path: {}", db_path.display());
    info!("Database mode: {:?}", db_mode);
    info!(
        "Database cache capacity: {}",
        Byte::from_bytes(db_cache_capacity.into()).get_appropriate_unit(true)
    );
    let db_config = sled::Config::new()
        .path(db_path)
        .mode(db_mode)
        .cache_capacity(db_cache_capacity);
    let trees = match open_trees::<R>(db_config) {
        Ok(trees) => trees,
        Err(_) => {
            error!("Failed to open database.");
            exit(1);
        }
    };
    let genesis_hash_db = match trees.root.get("genesis_hash").unwrap() {
        Some(value) => value.to_vec(),
        //    vector_as_u8_32_array(&value),
        None => {
            trees
                .root
                .insert("genesis_hash", genesis_hash_config.clone())
                .unwrap();
            genesis_hash_config.clone()
        }
    };

    if genesis_hash_db != genesis_hash_config {
        error!("Database has wrong genesis hash.");
        error!("Correct hash:  0x{}", hex::encode(genesis_hash_config));
        error!("Database hash: 0x{}", hex::encode(genesis_hash_db));
        let _ = close_trees::<R>(trees);
        exit(1);
    }
    // Determine url of Substrate node to connect to.
    let url = match url {
        Some(url) => url,
        None => R::get_default_url().to_owned(),
    };
    info!("Connecting to: {}", url);
    let rpc_client = match RpcClient::from_url(&url).await {
        Ok(rpc_client) => rpc_client,
        Err(err) => {
            error!("Failed to connect: {}", err);
            let _ = close_trees::<R>(trees);
            exit(1);
        }
    };
    let api = match OnlineClient::<R::RuntimeConfig>::from_rpc_client(rpc_client.clone()).await {
        Ok(api) => api,
        Err(err) => {
            error!("Failed to connect: {}", err);
            let _ = close_trees::<R>(trees);
            exit(1);
        }
    };
    let rpc = LegacyRpcMethods::<R::RuntimeConfig>::new(rpc_client);

    let genesis_hash_api = api.genesis_hash().as_ref().to_vec();

    if genesis_hash_api != genesis_hash_config {
        error!("Chain has wrong genesis hash.");
        error!("Correct hash: 0x{}", hex::encode(genesis_hash_config));
        error!("Chain hash:   0x{}", hex::encode(genesis_hash_api));
        let _ = close_trees::<R>(trees);
        exit(1);
    }
    // https://docs.rs/signal-hook/0.3.17/signal_hook/#a-complex-signal-handling-with-a-background-thread
    // Make sure double CTRL+C and similar kills.
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        // When terminated by a second term signal, exit with exit code 1.
        // This will do nothing the first time (because term_now is false).
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now)).unwrap();
        // But this will "arm" the above for the second time, by setting it to true.
        // The order of registering these is important, if you put this one first, it will
        // first arm and then terminate ‒ all in the first round.
        flag::register(*sig, Arc::clone(&term_now)).unwrap();
    }
    // Create a watch channel to exit the program.
    let (exit_tx, exit_rx) = watch::channel(false);
    // Create the channel for the websockets threads to send subscribe messages to the head thread.
    let (sub_tx, sub_rx) = mpsc::unbounded_channel();
    // Start indexer thread.
    let substrate_index = spawn(substrate_index::<R>(
        trees.clone(),
        api.clone(),
        rpc.clone(),
        queue_depth.into(),
        index_variant,
        store_events,
        exit_rx.clone(),
        sub_rx,
    ));
    // Spawn websockets task.
    let websockets_task = spawn(websockets_listen::<R>(
        trees.clone(),
        rpc,
        port,
        exit_rx,
        sub_tx,
    ));
    // Wait for signal.
    let mut signals = Signals::new(TERM_SIGNALS).unwrap();
    signals.next().await;
    info!("Exiting.");
    let _ = exit_tx.send(true);
    // Wait to exit.
    let _result = join!(substrate_index, websockets_task);
    // Close db.
    let _ = close_trees::<R>(trees);
    exit(0);
}
