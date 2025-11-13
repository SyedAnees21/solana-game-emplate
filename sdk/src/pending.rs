use std::collections::HashMap;
use prost::bytes::Bytes;
use proto_interface::{
    attribute::Value,
    external::{attributes::Attribute, messages::Entity},
    messages::ToServer,
    updates::LocalUpdate,
    AttributeId, EntityId, Timestamp,
};

use crate::sdkError;

#[derive(Default)]
pub struct Pending {
    pending_updates: HashMap<EntityId, HashMap<AttributeId, (Value, Timestamp)>>,
}

impl Pending {
    pub fn insert(&mut self, update: LocalUpdate, timestamp: Timestamp) {
        match update {
            LocalUpdate::Entity(entity_id, attribute_id, value) => {
                let attributes = self.pending_updates.entry(entity_id).or_default();
                attributes.insert(attribute_id, (value, timestamp));
            }
        }
    }

    pub fn take_updates(&mut self) -> Result<Bytes, sdkError> {
        let pending = std::mem::take(&mut self.pending_updates);

        if pending.is_empty() {
            return Ok(Bytes::default())
        }

        let mut tick_inbound = proto_interface::TickUpdate::default();

        for (entitiy_id, attributes) in pending.into_iter() {
            let attrs = attributes
                .into_iter()
                .map(|attribute_info| Attribute::from(attribute_info))
                .collect::<Vec<_>>();

            tick_inbound
                .entities
                .push(Entity::from((entitiy_id, attrs)));
        }

        let update = ToServer::from(tick_inbound);
        Ok(update.into_bytes()?)
    }
}
