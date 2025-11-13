use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use arrayvec::ArrayVec;
use proto_interface::{Attribute, AttributeId, Value};
use sdk::{
    alias::{Result, Timestamp},
};
pub type RawAtrributeMap<T, const CAP: usize> = RwLock<ArrayVec<T, CAP>>;

pub enum AttributeChange {
    None,
    Created(AttributeId, Timestamp),
    Modified(AttributeId, Timestamp),
}

#[derive(Debug)]
pub struct AttributeMap<T, const CAP: usize>(RawAtrributeMap<T, CAP>);

impl<T, const CAP: usize> AttributeMap<T, CAP> {
    pub fn iter<F>(&self, mut op: F) -> Result<()>
    where
        F: FnMut(&T),
    {
        let inner_read = self.0.read()?;
        for internal in inner_read.iter() {
            op(internal)
        }
        Ok(())
    }
}

impl<const CAP: usize> AttributeMap<VersionAttribute, CAP> {
    pub fn new(attributes: ArrayVec<VersionAttribute, CAP>) -> Self {
        Self(RwLock::new(attributes))
    }

    pub fn get(&self, attr_id: AttributeId) -> Result<Option<Value>> {
        let inner_read = self.0.read()?;

        match inner_read.binary_search_by(|attribute| attribute.id.cmp(&attr_id)) {
            Ok(index) => return Ok(Some(inner_read.get(index).unwrap().value())),
            Err(_) => return Ok(None),
        }
    }

    pub fn get_timestamped_value(
        &self,
        attr_id: AttributeId,
    ) -> Result<Option<(Value, Timestamp)>> {
        let inner = self.0.read()?;

        match inner.binary_search_by(|attribute| attribute.id.cmp(&attr_id)) {
            Ok(index) => {
                let attribute = inner.get(index).unwrap();
                return Ok(Some(attribute.get()));
            }
            Err(_) => return Ok(None),
        }
    }

    pub fn upsert(
        &self,
        attr_id: AttributeId,
        value: Value,
        timestamp: u64,
    ) -> Result<AttributeChange> {
        let inner_read = self.0.read()?;

        match inner_read.binary_search_by(|attribute| attribute.id.cmp(&attr_id)) {
            Ok(index) => {
                let attribute = inner_read.get(index).unwrap();

                if !attribute.set(value, timestamp) {
                    return Ok(AttributeChange::None);
                }

                return Ok(AttributeChange::Modified(attr_id, timestamp));
            }
            Err(index) => {
                drop(inner_read);
                let mut inner_write = self.0.write()?;

                if inner_write.is_full() {
                    return Ok(AttributeChange::None);
                }

                inner_write.insert(index, VersionAttribute::new(attr_id, value, timestamp));
                inner_write.sort_by(|a, b| a.id.cmp(&b.id));
                return Ok(AttributeChange::Created(attr_id, timestamp));
            }
        };
    }
}

impl<const CAP: usize> Default for AttributeMap<VersionAttribute, CAP> {
    fn default() -> Self {
        AttributeMap(RawAtrributeMap::new(ArrayVec::new()))
    }
}

impl<A, const CAP: usize> FromIterator<A> for AttributeMap<VersionAttribute, CAP>
where
    A: Into<VersionAttribute>,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut inner: ArrayVec<VersionAttribute, CAP> = ArrayVec::new();
        for item in iter {
            if inner.is_full() {
                break;
            }
            inner.push(item.into());
        }
        Self(RawAtrributeMap::new(inner))
    }
}

#[derive(Default, Debug)]
pub struct VersionAttribute {
    id: AttributeId,
    value: ArcSwap<(Value, Timestamp)>,
}

impl VersionAttribute {
    pub fn new(id: AttributeId, value: Value, timestamp: Timestamp) -> Self {
        Self {
            id,
            value: ArcSwap::new(Arc::new((value, timestamp))),
        }
    }

    pub fn id(&self) -> AttributeId {
        self.id
    }

    pub fn timestamp(&self) -> u64 {
        self.value.load().1
    }

    pub fn value(&self) -> Value {
        self.value.load().0.clone()
    }

    pub fn get(&self) -> (Value, u64) {
        let v = self.value.load();
        (v.0.clone(), v.1)
    }

    pub fn set(&self, value: Value, timestamp: u64) -> bool {
        let v = self.value.load();

        if v.1 >= timestamp || v.0 == value {
            return false;
        }

        self.value.store(Arc::new((value, timestamp)));
        true
    }
}

impl Into<VersionAttribute> for Attribute {
    fn into(self) -> VersionAttribute {
        let Attribute {
            id,
            timestamp,
            value,
        } = self;

        let Some(v) = value else {
            return VersionAttribute::default();
        };

        VersionAttribute::new(id, v.into(), timestamp)
    }
}
