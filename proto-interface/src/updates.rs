use crate::{AttributeId, EntityId, Value};

#[derive(Debug)]
pub enum LocalUpdate {
    Entity(EntityId, AttributeId, Value)
}

#[derive(Debug)]
pub enum ServerUpdate {
    Entity(EntityId, AttributeId, Value)
}