use dashmap::DashMap;
use proto_interface::{updates::ServerUpdate, AttributeId, EntityId, Value};

#[derive(Default)]
pub struct Accumulator {
    items: DashMap<EntityId, DashMap<AttributeId, Value>>,
}

impl Accumulator {
    pub fn insert(&self, entity_id: EntityId, attribute_id: AttributeId, value: Value) {
        let entity = self.items.entry(entity_id).or_default();
        entity.insert(attribute_id, value);
    }

    pub fn pop(&self) -> Option<ServerUpdate> {
        if self.items.is_empty() {
            return None;
        }

        let entity = self.items.iter().next()?;
        let attribute = entity.value().iter().next()?;

        let attribute_id = *attribute.key();
        drop(attribute);

        // SAFETY: unwrapig here since we know we have the write lock over
        // accumulator and no other thread can pop at this time.
        let (_, value) = entity.remove(&attribute_id).unwrap();

        let entity_id = *entity.key();
        let should_remove = entity.is_empty();
        drop(entity);

        if should_remove {
            self.items.remove(&entity_id);
        }

        Some(ServerUpdate::Entity(entity_id, attribute_id, value))
    }
}
