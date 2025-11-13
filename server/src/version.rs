use proto_interface::{Attribute, AttributeId, EntityId, Timestamp};
use std::collections::HashMap;

use crate::component::AttributeChange;

pub type AttributeVersions = HashMap<AttributeId, Timestamp>;
pub type EntityChanges = HashMap<EntityId, Vec<AttributeChange>>;

#[derive(Default, Debug)]
pub struct EntityVersion(HashMap<EntityId, AttributeVersions>);

impl EntityVersion {
    pub fn insert(&mut self, entity_id: EntityId) -> AttributeVersions {
        self.0.insert(entity_id, Default::default()).unwrap()
    }

    pub fn get(&self, entity_id: EntityId) -> Option<&AttributeVersions> {
        self.0.get(&entity_id)
    }

    pub fn submit_changes(&mut self, entity_id: EntityId, changes: Vec<AttributeChange>) {
        let this_version = self.0.entry(entity_id).or_default();
        for attribute_change in changes {
            let (id, timestamp) = match attribute_change {
                AttributeChange::Created(attribute_id, timestamp) => (attribute_id, timestamp),
                AttributeChange::Modified(attribute_id, timestamp) => (attribute_id, timestamp),
                _ => continue,
            };
            this_version.insert(id, timestamp);
        }
    }

    pub fn track_new(&mut self, entity_id: EntityId, attributes: &[Attribute]) {
        let this_version = self.0.entry(entity_id).or_default();

        for attribute in attributes.iter() {
            this_version.insert(attribute.id, attribute.timestamp);
        }
    }
}
