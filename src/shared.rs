use byteorder::BigEndian;
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use std::fmt;
use std::hash::Hash;
use subxt::utils::H256;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite;
use zerocopy::{
    AsBytes,
    byteorder::{U16, U32},
};
use zerocopy_derive::*;

/// Errors this crate can return
#[derive(thiserror::Error, Debug)]
pub enum IndexError {
    #[error("database error")]
    Sled(#[from] sled::Error),
    #[error("connection error")]
    Subxt(#[from] subxt::Error),
    #[error("connection error")]
    Tungstenite(#[from] tungstenite::Error),
    #[error("parse error")]
    Hex(#[from] hex::FromHexError),
    #[error("parse error")]
    ParseError,
    #[error("connection error")]
    BlockNotFound(u32),
    #[error("RPC error")]
    RpcError(#[from] subxt::ext::subxt_rpcs::Error),
    #[error("codec error")]
    CodecError(#[from] subxt::ext::codec::Error),
    #[error("metadata error")]
    MetadataError(#[from] subxt::error::MetadataTryFromError),
}

/// Indexer for a specific chain
pub trait RuntimeIndexer {
    type RuntimeConfig: subxt::Config;
    type ChainKey: IndexKey
        + Serialize
        + for<'a> Deserialize<'a>
        + Clone
        + Eq
        + PartialEq
        + Hash
        + Send;

    fn get_name() -> &'static str;

    fn get_genesis_hash() -> H256;

    fn get_versions() -> &'static [u32];

    fn get_default_url() -> &'static str;

    fn process_event(
        indexer: &crate::Indexer<Self>,
        block_number: u32,
        event_index: u16,
        event: subxt::events::EventDetails<Self::RuntimeConfig>,
    ) -> Result<u32, IndexError>;
}

pub trait IndexTrees {
    fn open(db: &Db) -> Result<Self, sled::Error>
    where
        Self: Sized;
    fn flush(&self) -> Result<(), sled::Error>;
}

/// Database trees for built-in Substrate keys
#[derive(Clone)]
pub struct SubstrateTrees {
    pub account_id: Tree,
    pub account_index: Tree,
    pub bounty_index: Tree,
    pub era_index: Tree,
    pub message_id: Tree,
    pub pool_id: Tree,
    pub preimage_hash: Tree,
    pub proposal_hash: Tree,
    pub proposal_index: Tree,
    pub ref_index: Tree,
    pub registrar_index: Tree,
    pub session_index: Tree,
    pub tip_hash: Tree,
    pub spend_index: Tree,
}

impl SubstrateTrees {
    pub fn open(db: &Db) -> Result<Self, sled::Error> {
        Ok(SubstrateTrees {
            account_id: db.open_tree(b"account_id")?,
            account_index: db.open_tree(b"account_index")?,
            bounty_index: db.open_tree(b"bounty_index")?,
            era_index: db.open_tree(b"era_index")?,
            message_id: db.open_tree(b"message_id")?,
            pool_id: db.open_tree(b"pool_id")?,
            preimage_hash: db.open_tree(b"preimage_hash")?,
            proposal_hash: db.open_tree(b"proposal_hash")?,
            proposal_index: db.open_tree(b"proposal_index")?,
            ref_index: db.open_tree(b"ref_index")?,
            registrar_index: db.open_tree(b"registrar_index")?,
            session_index: db.open_tree(b"session_index")?,
            tip_hash: db.open_tree(b"tip_hash")?,
            spend_index: db.open_tree(b"spend_index")?,
        })
    }

    pub fn flush(&self) -> Result<(), sled::Error> {
        self.account_id.flush()?;
        self.account_index.flush()?;
        self.bounty_index.flush()?;
        self.era_index.flush()?;
        self.message_id.flush()?;
        self.pool_id.flush()?;
        self.preimage_hash.flush()?;
        self.proposal_hash.flush()?;
        self.proposal_index.flush()?;
        self.ref_index.flush()?;
        self.registrar_index.flush()?;
        self.session_index.flush()?;
        self.tip_hash.flush()?;
        Ok(())
    }
}

/// Database trees for the indexer
#[derive(Clone)]
pub struct Trees<CT> {
    pub root: sled::Db,
    pub span: Tree,
    pub variant: Tree,
    pub substrate: SubstrateTrees,
    pub chain: CT,
    pub block_events: Tree,
}

/// On-disk format for variant keys
#[derive(FromZeroes, FromBytes, AsBytes, Unaligned, PartialEq, Debug)]
#[repr(C)]
pub struct VariantKey {
    pub pallet_index: u8,
    pub variant_index: u8,
    pub block_number: U32<BigEndian>,
    pub event_index: U16<BigEndian>,
}

/// On-disk format for 32-byte keys
#[derive(FromZeroes, FromBytes, AsBytes, Unaligned, PartialEq, Debug)]
#[repr(C)]
pub struct Bytes32Key {
    pub key: [u8; 32],
    pub block_number: U32<BigEndian>,
    pub event_index: U16<BigEndian>,
}

/// On-disk format for u32 keys
#[derive(FromZeroes, FromBytes, AsBytes, Unaligned, PartialEq, Debug)]
#[repr(C)]
pub struct U32Key {
    pub key: U32<BigEndian>,
    pub block_number: U32<BigEndian>,
    pub event_index: U16<BigEndian>,
}

/// Datatype to hold 32-byte keys
#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub struct Bytes32(pub [u8; 32]);

impl AsRef<[u8; 32]> for Bytes32 {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for Bytes32 {
    fn from(x: [u8; 32]) -> Self {
        Bytes32(x)
    }
}

impl AsRef<[u8]> for Bytes32 {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl Serialize for Bytes32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut hex_string = "0x".to_owned();
        hex_string.push_str(&hex::encode(self.0));
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for Bytes32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.get(2..66) {
            Some(message_id) => match hex::decode(message_id) {
                Ok(message_id) => Ok(Bytes32(message_id.try_into().unwrap())),
                Err(_error) => Err(serde::de::Error::custom("error")),
            },
            None => Err(serde::de::Error::custom("error")),
        }
    }
}

impl std::str::FromStr for Bytes32 {
    type Err = IndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Bytes32(
            hex::decode(s)?
                .try_into()
                .map_err(|_| IndexError::ParseError)?,
        ))
    }
}

/// All the key types that are built-in to Substrate
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(tag = "type", content = "value")]
pub enum SubstrateKey {
    AccountId(Bytes32),
    AccountIndex(u32),
    BountyIndex(u32),
    EraIndex(u32),
    MessageId(Bytes32),
    PoolId(u32),
    PreimageHash(Bytes32),
    ProposalHash(Bytes32),
    ProposalIndex(u32),
    RefIndex(u32),
    RegistrarIndex(u32),
    SessionIndex(u32),
    TipHash(Bytes32),
    SpendIndex(u32),
}

impl SubstrateKey {
    pub fn write_db_key(
        &self,
        trees: &SubstrateTrees,
        block_number: u32,
        event_index: u16,
    ) -> Result<(), sled::Error> {
        let block_number = block_number.into();
        let event_index = event_index.into();
        match self {
            SubstrateKey::AccountId(account_id) => {
                let key = Bytes32Key {
                    key: account_id.0,
                    block_number,
                    event_index,
                };
                trees.account_id.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::AccountIndex(account_index) => {
                let key = U32Key {
                    key: (*account_index).into(),
                    block_number,
                    event_index,
                };
                trees.account_index.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::BountyIndex(bounty_index) => {
                let key = U32Key {
                    key: (*bounty_index).into(),
                    block_number,
                    event_index,
                };
                trees.bounty_index.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::EraIndex(era_index) => {
                let key = U32Key {
                    key: (*era_index).into(),
                    block_number,
                    event_index,
                };
                trees.era_index.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::MessageId(message_id) => {
                let key = Bytes32Key {
                    key: message_id.0,
                    block_number,
                    event_index,
                };
                trees.message_id.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::PoolId(pool_id) => {
                let key = U32Key {
                    key: (*pool_id).into(),
                    block_number,
                    event_index,
                };
                trees.pool_id.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::PreimageHash(preimage_hash) => {
                let key = Bytes32Key {
                    key: preimage_hash.0,
                    block_number,
                    event_index,
                };
                trees.preimage_hash.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::ProposalHash(proposal_hash) => {
                let key = Bytes32Key {
                    key: proposal_hash.0,
                    block_number,
                    event_index,
                };
                trees.proposal_hash.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::ProposalIndex(proposal_index) => {
                let key = U32Key {
                    key: (*proposal_index).into(),
                    block_number,
                    event_index,
                };
                trees.proposal_index.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::RefIndex(ref_index) => {
                let key = U32Key {
                    key: (*ref_index).into(),
                    block_number,
                    event_index,
                };
                trees.ref_index.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::RegistrarIndex(registrar_index) => {
                let key = U32Key {
                    key: (*registrar_index).into(),
                    block_number,
                    event_index,
                };
                trees.registrar_index.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::SessionIndex(session_index) => {
                let key = U32Key {
                    key: (*session_index).into(),
                    block_number,
                    event_index,
                };
                trees.session_index.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::TipHash(tip_hash) => {
                let key = Bytes32Key {
                    key: tip_hash.0,
                    block_number,
                    event_index,
                };
                trees.tip_hash.insert(key.as_bytes(), &[])?
            }
            SubstrateKey::SpendIndex(spend_index) => {
                let key = U32Key {
                    key: (*spend_index).into(),
                    block_number,
                    event_index,
                };
                trees.spend_index.insert(key.as_bytes(), &[])?
            }
        };
        Ok(())
    }
}

pub trait IndexKey {
    type ChainTrees: IndexTrees + Send + Sync + Clone;

    fn write_db_key(
        &self,
        trees: &Self::ChainTrees,
        block_number: u32,
        event_index: u16,
    ) -> Result<(), sled::Error>;

    fn get_key_events(&self, trees: &Self::ChainTrees) -> Vec<Event>;
}

/// All the key types for the chain
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(tag = "type", content = "value")]
pub enum Key<CK: IndexKey> {
    Variant(u8, u8),
    Substrate(SubstrateKey),
    Chain(CK),
}

impl<CK: IndexKey> Key<CK> {
    pub fn write_db_key(
        &self,
        trees: &Trees<CK::ChainTrees>,
        block_number: u32,
        event_index: u16,
    ) -> Result<(), sled::Error> {
        match self {
            Key::Variant(pallet_index, variant_index) => {
                let key = VariantKey {
                    pallet_index: *pallet_index,
                    variant_index: *variant_index,
                    block_number: block_number.into(),
                    event_index: event_index.into(),
                };
                trees.variant.insert(key.as_bytes(), &[])?;
            }
            Key::Substrate(substrate_key) => {
                substrate_key.write_db_key(&trees.substrate, block_number, event_index)?;
            }
            Key::Chain(chain_key) => {
                chain_key.write_db_key(&trees.chain, block_number, event_index)?;
            }
        };
        Ok(())
    }
}

/// JSON request messages
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum RequestMessage<CK: IndexKey> {
    Status,
    SubscribeStatus,
    UnsubscribeStatus,
    Variants,
    GetEvents { key: Key<CK> },
    SubscribeEvents { key: Key<CK> },
    UnsubscribeEvents { key: Key<CK> },
    SizeOnDisk,
}

/// Identifies an event by block number and event index
#[derive(Serialize, Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub block_number: u32,
    pub event_index: u16,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "block number: {}, event index: {}",
            self.block_number, self.event_index
        )
    }
}

/// Index and name of an event type
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct EventMeta {
    pub index: u8,
    pub name: String,
}

/// Index, name and list of event types for a pallet
#[derive(Serialize, Debug, Clone, Deserialize, PartialEq)]
pub struct PalletMeta {
    pub index: u8,
    pub name: String,
    pub events: Vec<EventMeta>,
}

/// On-disk format for span value
#[derive(FromZeroes, FromBytes, AsBytes, Unaligned, PartialEq, Debug)]
#[repr(C)]
pub struct SpanDbValue {
    pub start: U32<BigEndian>,
    pub version: U16<BigEndian>,
    pub index_variant: u8,
    pub store_events: u8,
}

/// Start and end block number for a span of blocks
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "start: {}, end: {}", self.start, self.end)
    }
}

/// JSON response messages
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum ResponseMessage<CK: IndexKey> {
    Status(Vec<Span>),
    Variants(Vec<PalletMeta>),
    Events { key: Key<CK>, events: Vec<Event> },
    Subscribed,
    Unsubscribed,
    SizeOnDisk(u64),
    //    Error,
}

/// Subscription message sent from a WebSocket connection thread to the indexer thread
#[derive(Debug)]
pub enum SubscriptionMessage<CK: IndexKey> {
    SubscribeStatus {
        sub_response_tx: UnboundedSender<ResponseMessage<CK>>,
    },
    UnsubscribeStatus {
        sub_response_tx: UnboundedSender<ResponseMessage<CK>>,
    },
    SubscribeEvents {
        key: Key<CK>,
        sub_response_tx: UnboundedSender<ResponseMessage<CK>>,
    },
    UnsubscribeEvents {
        key: Key<CK>,
        sub_response_tx: UnboundedSender<ResponseMessage<CK>>,
    },
}
