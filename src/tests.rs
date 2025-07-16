use crate::shared::*;
use crate::substrate::*;
use crate::websockets::*;
use crate::*;

use hex_literal::hex;
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use std::str::FromStr;
use subxt::utils::AccountId32;
use subxt::utils::H256;
use tokio::sync::mpsc::{error::TryRecvError, unbounded_channel};
use zerocopy::{AsBytes, FromBytes};

pub struct TestIndexer;

#[derive(Clone, Debug)]
pub struct ChainTrees {
    pub test_index: Tree,
    pub test_hash: Tree,
}

impl IndexTrees for ChainTrees {
    fn open(db: &Db) -> Result<Self, sled::Error> {
        Ok(ChainTrees {
            test_index: db.open_tree(b"test_index")?,
            test_hash: db.open_tree(b"candiate_hash")?,
        })
    }

    fn flush(&self) -> Result<(), sled::Error> {
        self.test_index.flush()?;
        self.test_hash.flush()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(tag = "type", content = "value")]
pub enum ChainKey {
    TestIndex(u32),
    TestHash(Bytes32),
}

impl IndexKey for ChainKey {
    type ChainTrees = ChainTrees;

    fn write_db_key(
        &self,
        trees: &ChainTrees,
        block_number: u32,
        event_index: u16,
    ) -> Result<(), sled::Error> {
        let block_number = block_number.into();
        let event_index = event_index.into();
        match self {
            ChainKey::TestIndex(test_index) => {
                let key = U32Key {
                    key: (*test_index).into(),
                    block_number,
                    event_index,
                };
                trees.test_index.insert(key.as_bytes(), &[])?
            }
            ChainKey::TestHash(test_hash) => {
                let key = Bytes32Key {
                    key: test_hash.0,
                    block_number,
                    event_index,
                };
                trees.test_hash.insert(key.as_bytes(), &[])?
            }
        };
        Ok(())
    }

    fn get_key_events(&self, trees: &ChainTrees) -> Vec<Event> {
        match self {
            ChainKey::TestIndex(test_index) => get_events_u32(&trees.test_index, *test_index),
            ChainKey::TestHash(test_hash) => get_events_bytes32(&trees.test_hash, test_hash),
        }
    }
}

impl RuntimeIndexer for TestIndexer {
    type RuntimeConfig = subxt::PolkadotConfig;
    type ChainKey = ChainKey;

    fn get_name() -> &'static str {
        "test"
    }

    fn get_genesis_hash() -> H256 {
        hex!["91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3"].into()
    }

    fn get_versions() -> &'static [u32] {
        &[0]
    }

    fn get_default_url() -> &'static str {
        ""
    }

    fn process_event(
        _indexer: &Indexer<Self>,
        _block_number: u32,
        _event_index: u16,
        _event: subxt::events::EventDetails<Self::RuntimeConfig>,
    ) -> Result<u32, IndexError> {
        Ok(0)
    }
}

pub struct TestIndexer2;

impl RuntimeIndexer for TestIndexer2 {
    type RuntimeConfig = subxt::PolkadotConfig;
    type ChainKey = ChainKey;

    fn get_name() -> &'static str {
        "test"
    }

    fn get_genesis_hash() -> H256 {
        hex!["91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3"].into()
    }

    fn get_versions() -> &'static [u32] {
        &[0, 500]
    }

    fn get_default_url() -> &'static str {
        ""
    }

    fn process_event(
        _indexer: &Indexer<Self>,
        _block_number: u32,
        _event_index: u16,
        _event: subxt::events::EventDetails<Self::RuntimeConfig>,
    ) -> Result<u32, IndexError> {
        Ok(0)
    }
}

#[tokio::test]
async fn test_process_msg_status() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();

    let value = SpanDbValue {
        start: 0_u32.try_into().unwrap(),
        version: 0_u16.try_into().unwrap(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(40_u32.to_be_bytes(), value.as_bytes())
        .unwrap();

    let value = SpanDbValue {
        start: 60_u32.try_into().unwrap(),
        version: 0_u16.try_into().unwrap(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(92_u32.to_be_bytes(), value.as_bytes())
        .unwrap();

    let value = SpanDbValue {
        start: 42_u32.try_into().unwrap(),
        version: 0_u16.try_into().unwrap(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(52_u32.to_be_bytes(), value.as_bytes())
        .unwrap();

    let response = process_msg_status::<TestIndexer>(&trees.span);

    let ResponseMessage::Status(spans) = response else {
        panic!("Wrong response message.");
    };
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 40);
    assert_eq!(spans[1].start, 42);
    assert_eq!(spans[1].end, 52);
    assert_eq!(spans[2].start, 60);
    assert_eq!(spans[2].end, 92);
}

#[tokio::test]
async fn test_process_msg_subscribe_status() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let (sub_tx, mut sub_rx) = unbounded_channel();
    let (sub_response_tx, mut sub_response_rx) = unbounded_channel();

    let value = SpanDbValue {
        start: 0_u32.try_into().unwrap(),
        version: 0_u16.try_into().unwrap(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(40_u32.to_be_bytes(), value.as_bytes())
        .unwrap();

    let response = process_msg_subscribe_status::<TestIndexer>(&sub_tx, &sub_response_tx);

    let ResponseMessage::Subscribed = response else {
        panic!("Wrong response message.");
    };

    let msg = sub_rx.recv().await.unwrap();
    process_sub_msg(&indexer, msg);
    indexer.notify_status_subscribers();

    let response_msg = sub_response_rx.recv().await.unwrap();

    let ResponseMessage::Status(spans) = response_msg else {
        panic!("Wrong response message.");
    };
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 40);

    let value = SpanDbValue {
        start: 60_u32.try_into().unwrap(),
        version: 0_u16.try_into().unwrap(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(92_u32.to_be_bytes(), value.as_bytes())
        .unwrap();

    indexer.notify_status_subscribers();

    let response_msg = sub_response_rx.recv().await.unwrap();

    let ResponseMessage::Status(spans) = response_msg else {
        panic!("Wrong response message.");
    };
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 40);
    assert_eq!(spans[1].start, 60);
    assert_eq!(spans[1].end, 92);

    let value = SpanDbValue {
        start: 42_u32.try_into().unwrap(),
        version: 0_u16.try_into().unwrap(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(52_u32.to_be_bytes(), value.as_bytes())
        .unwrap();

    indexer.notify_status_subscribers();

    let response_msg = sub_response_rx.recv().await.unwrap();

    let ResponseMessage::Status(spans) = response_msg else {
        panic!("Wrong response message.");
    };
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 40);
    assert_eq!(spans[1].start, 42);
    assert_eq!(spans[1].end, 52);
    assert_eq!(spans[2].start, 60);
    assert_eq!(spans[2].end, 92);

    let response = process_msg_unsubscribe_status::<TestIndexer>(&sub_tx, &sub_response_tx);

    let ResponseMessage::Unsubscribed = response else {
        panic!("Wrong response message.");
    };

    let msg = sub_rx.recv().await.unwrap();
    process_sub_msg(&indexer, msg);
    indexer.notify_status_subscribers();

    let response_msg = sub_response_rx.try_recv();

    let Err(TryRecvError::Empty) = response_msg else {
        panic!("Wrong response message.");
    };
}

#[test]
fn test_variant_key() {
    let key1 = VariantKey {
        pallet_index: 3,
        variant_index: 65,
        block_number: 4.into(),
        event_index: 5.into(),
    };

    let key2 = VariantKey::read_from(key1.as_bytes()).unwrap();
    assert_eq!(key1, key2);
}

#[tokio::test]
async fn test_process_msg_variant() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let key = Key::Variant(3, 65);
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[test]
fn test_bytes32_key() {
    let key1 = Bytes32Key {
        key: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .unwrap()
            .0,
        block_number: 4.into(),
        event_index: 5.into(),
    };

    let key2 = Bytes32Key::read_from(key1.as_bytes()).unwrap();
    assert_eq!(key1, key2);
}

#[tokio::test]
async fn test_process_msg_account_id() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let account_id =
        AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let key = Key::Substrate(SubstrateKey::AccountId(Bytes32(account_id.0)));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[test]
fn test_u32_key() {
    let key1 = U32Key {
        key: 8.into(),
        block_number: 4.into(),
        event_index: 5.into(),
    };

    let key2 = U32Key::read_from(key1.as_bytes()).unwrap();
    assert_eq!(key1, key2);
}

#[tokio::test]
async fn test_process_msg_account_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let account_index = 88;
    let key = Key::Substrate(SubstrateKey::AccountIndex(account_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_bounty_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let bounty_index = 88;
    let key = Key::Substrate(SubstrateKey::BountyIndex(bounty_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_era_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let era_index = 88;
    let key = Key::Substrate(SubstrateKey::EraIndex(era_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_message_id() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let message_id = Bytes32([8; 32]);
    let key = Key::Substrate(SubstrateKey::MessageId(message_id));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_pool_id() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let pool_id = 88;
    let key = Key::Substrate(SubstrateKey::PoolId(pool_id));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_preimage_hash() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let preimage_hash = Bytes32([8; 32]);
    let key = Key::Substrate(SubstrateKey::PreimageHash(preimage_hash));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_proposal_hash() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let proposal_hash = Bytes32([8; 32]);
    let key = Key::Substrate(SubstrateKey::ProposalHash(proposal_hash));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_proposal_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let proposal_index = 88;
    let key = Key::Substrate(SubstrateKey::ProposalIndex(proposal_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_ref_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let ref_index = 88;
    let key = Key::Substrate(SubstrateKey::RefIndex(ref_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_registrar_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let registrar_index = 88;
    let key = Key::Substrate(SubstrateKey::RegistrarIndex(registrar_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_session_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let session_index = 88;
    let key = Key::Substrate(SubstrateKey::SessionIndex(session_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_tip_hash() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let tip_hash = Bytes32([8; 32]);
    let key = Key::Substrate(SubstrateKey::TipHash(tip_hash));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_chain_test_index() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let test_index = 88;
    let key = Key::Chain(ChainKey::TestIndex(test_index));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_chain_test_hash() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let test_hash = Bytes32([8; 32]);
    let key = Key::Chain(ChainKey::TestHash(test_hash));
    indexer.index_event(key.clone(), 4, 5).unwrap();
    indexer.index_event(key.clone(), 8, 5).unwrap();
    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response = process_msg_get_events::<TestIndexer>(&trees, key.clone());

    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].block_number, 10);
    assert_eq!(events[1].block_number, 8);
    assert_eq!(events[2].block_number, 4);
}

#[tokio::test]
async fn test_process_msg_subscribe_events() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    let indexer = Indexer::<TestIndexer>::new_test(trees.clone());
    let (sub_tx, mut sub_rx) = unbounded_channel();
    let (sub_response_tx, mut sub_response_rx) = unbounded_channel();
    let key = Key::Variant(3, 65);

    let response =
        process_msg_subscribe_events::<TestIndexer>(key.clone(), &sub_tx, &sub_response_tx);

    let ResponseMessage::Subscribed = response else {
        panic!("Wrong response message.");
    };

    let msg = sub_rx.recv().await.unwrap();
    process_sub_msg(&indexer, msg);

    indexer.index_event(key.clone(), 4, 5).unwrap();

    let response_msg = sub_response_rx.recv().await.unwrap();
    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response_msg
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].block_number, 4);

    indexer.index_event(key.clone(), 8, 5).unwrap();

    let response_msg = sub_response_rx.recv().await.unwrap();
    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response_msg
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].block_number, 8);

    indexer.index_event(key.clone(), 10, 5).unwrap();

    let response_msg = sub_response_rx.recv().await.unwrap();
    let ResponseMessage::Events {
        key: response_key,
        events,
        block_events: _,
    } = response_msg
    else {
        panic!("Wrong response message.");
    };
    assert_eq!(key, response_key);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].block_number, 10);

    let response =
        process_msg_unsubscribe_events::<TestIndexer>(key.clone(), &sub_tx, &sub_response_tx);

    let ResponseMessage::Unsubscribed = response else {
        panic!("Wrong response message.");
    };

    let msg = sub_rx.recv().await.unwrap();
    process_sub_msg(&indexer, msg);

    indexer.index_event(key.clone(), 18, 5).unwrap();

    let response_msg = sub_response_rx.try_recv();

    let Err(TryRecvError::Empty) = response_msg else {
        panic!("Wrong response message.");
    };
}

#[test]
fn test_load_spans() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    trees.span.clear().unwrap();
    let spans = load_spans::<TestIndexer>(&trees.span, false, false).unwrap();
    assert_eq!(trees.span.len(), 0);
    assert_eq!(spans.len(), 0);
    let value = SpanDbValue {
        start: 80_u32.into(),
        version: 0_u16.into(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(100_u32.to_be_bytes(), value.as_bytes())
        .unwrap();
    let spans = load_spans::<TestIndexer>(&trees.span, false, false).unwrap();
    assert_eq!(trees.span.len(), 1);
    assert_eq!(spans.len(), 1);
    assert_eq!(
        spans[0],
        Span {
            start: 80,
            end: 100
        }
    );
    let value = SpanDbValue {
        start: 180_u32.into(),
        version: 0_u16.into(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(200_u32.to_be_bytes(), value.as_bytes())
        .unwrap();
    let spans = load_spans::<TestIndexer>(&trees.span, false, false).unwrap();
    assert_eq!(trees.span.len(), 2);
    assert_eq!(spans.len(), 2);
    assert_eq!(
        spans[0],
        Span {
            start: 80,
            end: 100
        }
    );
    assert_eq!(
        spans[1],
        Span {
            start: 180,
            end: 200
        }
    );
    let spans = load_spans::<TestIndexer2>(&trees.span, false, false).unwrap();
    assert_eq!(trees.span.len(), 2);
    assert_eq!(spans.len(), 2);
    assert_eq!(
        spans[0],
        Span {
            start: 80,
            end: 100
        }
    );
    assert_eq!(
        spans[1],
        Span {
            start: 180,
            end: 200
        }
    );
    let value = SpanDbValue {
        start: 400_u32.into(),
        version: 0_u16.into(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(600_u32.to_be_bytes(), value.as_bytes())
        .unwrap();
    let spans = load_spans::<TestIndexer2>(&trees.span, false, false).unwrap();
    assert_eq!(trees.span.len(), 3);
    assert_eq!(spans.len(), 3);
    assert_eq!(
        spans[0],
        Span {
            start: 80,
            end: 100
        }
    );
    assert_eq!(
        spans[1],
        Span {
            start: 180,
            end: 200
        }
    );
    assert_eq!(
        spans[2],
        Span {
            start: 400,
            end: 499
        }
    );
    let value = SpanDbValue {
        start: 500_u32.into(),
        version: 0_u16.into(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(600_u32.to_be_bytes(), value.as_bytes())
        .unwrap();
    let spans = load_spans::<TestIndexer2>(&trees.span, false, false).unwrap();
    assert_eq!(trees.span.len(), 3);
    assert_eq!(spans.len(), 3);
    assert_eq!(
        spans[0],
        Span {
            start: 80,
            end: 100
        }
    );
    assert_eq!(
        spans[1],
        Span {
            start: 180,
            end: 200
        }
    );
    assert_eq!(
        spans[2],
        Span {
            start: 400,
            end: 499
        }
    );
    let value = SpanDbValue {
        start: 500_u32.into(),
        version: 1_u16.into(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(600_u32.to_be_bytes(), value.as_bytes())
        .unwrap();
    let spans = load_spans::<TestIndexer2>(&trees.span, false, false).unwrap();
    assert_eq!(trees.span.len(), 4);
    assert_eq!(spans.len(), 4);
    assert_eq!(
        spans[0],
        Span {
            start: 80,
            end: 100
        }
    );
    assert_eq!(
        spans[1],
        Span {
            start: 180,
            end: 200
        }
    );
    assert_eq!(
        spans[2],
        Span {
            start: 400,
            end: 499
        }
    );
    assert_eq!(
        spans[3],
        Span {
            start: 500,
            end: 600
        }
    );
}

#[test]
fn test_check_span() {
    let db_config = sled::Config::new().temporary(true);
    let trees = open_trees::<TestIndexer>(db_config).unwrap();
    trees.span.clear().unwrap();
    let mut spans = Vec::new();
    let mut span = Span {
        start: 100,
        end: 120,
    };
    check_span(&trees.span, &mut spans, &mut span).unwrap();
    assert_eq!(trees.span.len(), 0);
    assert_eq!(spans.len(), 0);
    assert_eq!(
        span,
        Span {
            start: 100,
            end: 120
        }
    );
    let value = SpanDbValue {
        start: 10_u32.into(),
        version: 0_u16.into(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(20_u32.to_be_bytes(), value.as_bytes())
        .unwrap();
    spans.push(Span { start: 10, end: 20 });
    check_span(&trees.span, &mut spans, &mut span).unwrap();
    assert_eq!(trees.span.len(), 1);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0], Span { start: 10, end: 20 });
    assert_eq!(
        span,
        Span {
            start: 100,
            end: 120
        }
    );
    let value = SpanDbValue {
        start: 30_u32.into(),
        version: 0_u16.into(),
        index_variant: 0,
        store_events: 0,
    };
    trees
        .span
        .insert(99_u32.to_be_bytes(), value.as_bytes())
        .unwrap();
    spans.push(Span { start: 30, end: 99 });
    check_span(&trees.span, &mut spans, &mut span).unwrap();
    assert_eq!(trees.span.len(), 1);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0], Span { start: 10, end: 20 });
    assert_eq!(
        span,
        Span {
            start: 30,
            end: 120
        }
    );
}

#[test]
fn test_check_next_batch_block() {
    let mut spans = Vec::new();
    let mut next_batch_block = 50;

    check_next_batch_block(&spans, &mut next_batch_block);
    assert_eq!(next_batch_block, 50);
    spans.push(Span { start: 20, end: 30 });
    check_next_batch_block(&spans, &mut next_batch_block);
    assert_eq!(next_batch_block, 50);
    spans.push(Span { start: 45, end: 50 });
    check_next_batch_block(&spans, &mut next_batch_block);
    assert_eq!(next_batch_block, 44);
}
