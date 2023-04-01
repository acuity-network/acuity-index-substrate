use subxt::{
    utils::AccountId32,
};

use parity_scale_codec::{Encode, Decode};
use serde::{Serialize, Deserialize};

use crate::shared::*;
use crate::substrate::*;

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Identity {
    #[serde(rename_all = "camelCase")]
    IdentitySet {
        who: AccountId32,
    },
    #[serde(rename_all = "camelCase")]
    IdentityCleared {
        who: AccountId32,
        deposit: u128,
    },
    #[serde(rename_all = "camelCase")]
    IdentityKilled {
        who: AccountId32,
        deposit: u128,
    },
    #[serde(rename_all = "camelCase")]
    JudgementRequested {
        who: AccountId32,
        registrar_index: u32,
    },
    #[serde(rename_all = "camelCase")]
    JudgementUnrequested {
        who: AccountId32,
        registrar_index: u32,
    },
    #[serde(rename_all = "camelCase")]
    JudgementGiven {
        target: AccountId32,
        registrar_index: u32,
    },
    #[serde(rename_all = "camelCase")]
	RegistrarAdded {
	    registrar_index: u32,
	},
    #[serde(rename_all = "camelCase")]
	SubIdentityAdded {
	    sub: AccountId32,
	    main: AccountId32,
	    deposit: u128,
	},
    #[serde(rename_all = "camelCase")]
	SubIdentityRemoved {
	    sub: AccountId32,
	    main: AccountId32,
	    deposit: u128,
	},
    #[serde(rename_all = "camelCase")]
	SubIdentityRevoked {
	    sub: AccountId32,
	    main: AccountId32,
	    deposit: u128,
	},
}

pub fn identity_index_event(trees: Trees, block_number: u32, event_index: u32, event: subxt::events::EventDetails) {
    match event.variant_name() {
        "IdentitySet" => {
            let event = event.as_event::<polkadot::identity::events::IdentitySet>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::IdentitySet {
                    who: event.who.clone(),
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.who.clone(), block_number, event_index, &value);
        },
        "IdentityCleared" => {
            let event = event.as_event::<polkadot::identity::events::IdentityCleared>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::IdentityCleared {
                    who: event.who.clone(),
                    deposit: event.deposit,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.who, block_number, event_index, &value);
        },
        "IdentityKilled" => {
            let event = event.as_event::<polkadot::identity::events::IdentityKilled>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::IdentityKilled {
                    who: event.who.clone(),
                    deposit: event.deposit,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.who, block_number, event_index, &value);
        },
        "JudgementRequested" => {
            let event = event.as_event::<polkadot::identity::events::JudgementRequested>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::JudgementRequested {
                    who: event.who.clone(),
                    registrar_index: event.registrar_index,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.who, block_number, event_index, &value);
            index_event_registrar_index(trees.clone(), event.registrar_index, block_number, event_index, &value);
        },
        "JudgementUnrequested" => {
            let event = event.as_event::<polkadot::identity::events::JudgementUnrequested>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::JudgementUnrequested {
                    who: event.who.clone(),
                    registrar_index: event.registrar_index,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.who, block_number, event_index, &value);
            index_event_registrar_index(trees.clone(), event.registrar_index, block_number, event_index, &value);
        },
        "JudgementGiven" => {
            let event = event.as_event::<polkadot::identity::events::JudgementGiven>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::JudgementGiven {
                    target: event.target.clone(),
                    registrar_index: event.registrar_index,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.target, block_number, event_index, &value);
            index_event_registrar_index(trees.clone(), event.registrar_index, block_number, event_index, &value);
        },
        "RegistrarAdded" => {
            let event = event.as_event::<polkadot::identity::events::RegistrarAdded>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::RegistrarAdded {
                    registrar_index: event.registrar_index,
                }
            );
            let value = Event::encode(&event_db);
            index_event_registrar_index(trees.clone(), event.registrar_index, block_number, event_index, &value);
        },
        "SubIdentityAdded" => {
            let event = event.as_event::<polkadot::identity::events::SubIdentityAdded>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::SubIdentityAdded {
                    sub: event.sub.clone(),
                    main: event.main.clone(),
                    deposit: event.deposit,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.sub, block_number, event_index, &value);
            index_event_account_id(trees.clone(), event.main, block_number, event_index, &value);
        },
        "SubIdentityRemoved" => {
            let event = event.as_event::<polkadot::identity::events::SubIdentityRemoved>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::SubIdentityRemoved {
                    sub: event.sub.clone(),
                    main: event.main.clone(),
                    deposit: event.deposit,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.sub, block_number, event_index, &value);
            index_event_account_id(trees.clone(), event.main, block_number, event_index, &value);
        },
        "SubIdentityRevoked" => {
            let event = event.as_event::<polkadot::identity::events::SubIdentityRevoked>().unwrap().unwrap();
            let event_db = Event::Identity(
                Identity::SubIdentityRevoked {
                    sub: event.sub.clone(),
                    main: event.main.clone(),
                    deposit: event.deposit,
                }
            );
            let value = Event::encode(&event_db);
            index_event_account_id(trees.clone(), event.sub, block_number, event_index, &value);
            index_event_account_id(trees.clone(), event.main, block_number, event_index, &value);
        },
        &_ => {},
    };
}