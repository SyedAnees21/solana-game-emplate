use crate::{
    component::{AttributeChange, AttributeMap, VersionAttribute},
    version::AttributeVersions,
};
use dashmap::{iter::Iter, mapref::one::Ref, DashMap};
use proto_interface::{attribute::Value, Attribute, AttributeId, Timestamp};
use sdk::{sdkError, MAX_ATTRIBUTES_PER_COMPONENT};

pub type EntityId = u32;

#[derive(Debug, Default)]
pub struct Entity {
    id: EntityId,
    attributes: AttributeMap<VersionAttribute, MAX_ATTRIBUTES_PER_COMPONENT>,
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            attributes: AttributeMap::default(),
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn new_with_attributes<A, B>(id: EntityId, attributes: A) -> Self
    where
        B: Into<VersionAttribute>,
        A: IntoIterator<Item = B>,
    {
        let attributes = attributes.into_iter().collect::<AttributeMap<_, _>>();
        Self { id, attributes }
    }

    pub fn get_attributes_by_version(
        &self,
        versions: &AttributeVersions,
    ) -> Result<Vec<Attribute>, sdkError> {
        let mut latest_attributes = vec![];
        self.attributes.iter(|attribute| {
            if attribute.id() == 0 {
                return;
            }

            if let Some(timestamp) = versions.get(&attribute.id()) {
                if *timestamp >= attribute.timestamp() {
                    return;
                }
            }

            if attribute.value().is_none() {
                return;
            }

            latest_attributes.push(Attribute {
                id: attribute.id(),
                timestamp: attribute.timestamp(),
                value: Some(attribute.value().into()),
            })
        })?;

        Ok(latest_attributes)
    }
}

impl Entity {
    pub fn update_entity<E>(&self, attributes: E) -> Result<Vec<AttributeChange>, sdkError>
    where
        E: IntoIterator<Item = Attribute>,
    {
        let mut changes = vec![];
        for attribute in attributes.into_iter() {
            let Attribute {
                id,
                timestamp,
                value,
            } = attribute;

            let Some(v) = value else {
                continue;
            };

            changes.push(self.attributes.upsert(id, v.into(), timestamp)?);
        }

        Ok(changes)
    }

    pub fn upsert_attribute(
        &self,
        id: AttributeId,
        v: Value,
        timestamp: Timestamp,
    ) -> Result<AttributeChange, sdkError> {
        self.attributes.upsert(id, v, timestamp)
    }
}

#[derive(Debug, Default)]
pub struct Entities(DashMap<EntityId, Entity>);

impl Entities {
    pub fn upsert<A>(&self, id: EntityId, attributes: A) -> Result<Vec<AttributeChange>, sdkError>
    where
        A: IntoIterator<Item = Attribute>,
    {
        let entity = self.0.entry(id).or_insert(Entity::new(id));
        entity.update_entity(attributes)
    }

    pub fn get_entity(&self, entity_id: &EntityId) -> Option<Ref<'_, EntityId, Entity>> {
        self.0.get(entity_id)
    }

    pub fn iter_entities(&self) -> Iter<'_, EntityId, Entity> {
        self.0.iter()
    }
}
