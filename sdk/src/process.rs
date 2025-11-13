use std::sync::Arc;

use crate::{sdkError, session::Session};
use prost::bytes::Bytes;
use proto_interface::messages::ToClient as FromServer;

pub async fn process_incoming(buffer: Bytes, session: Arc<Session>) -> Result<(), sdkError> {
    if buffer.is_empty() {
        return Ok(());
    }

    let tick = match FromServer::from_bytes(buffer)? {
        FromServer::Tick(tick_update) => tick_update,
        FromServer::None => return Ok(()),
    };

    // TODO: Use a proper debug tracing subscriber for debug
    // logs.
    // println!("Received server update: {:?}", tick);

    for entity in tick.entities {
        for attribute in entity.attrs {
            if let Some(value) = attribute.value {
                session
                    .updates_accumulator()
                    .insert(entity.id, attribute.id, value.into());
            }
        }
    }

    Ok(())
}
