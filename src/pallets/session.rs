use subxt::PolkadotConfig;
use crate::shared::*;
use crate::substrate::*;

pub fn session_index_event(indexer: &Indexer, block_number: u32, event_index: u32, event: subxt::events::EventDetails<PolkadotConfig>) -> Result<(), subxt::Error> {
    match event.variant_name() {
        "NewSession" => {
            let event = event.as_event::<polkadot::session::events::NewSession>()?.unwrap();
            indexer.index_event_session_index(event.session_index, block_number, event_index);
            Ok(())
        },
        _ => Ok(()),
    }
}