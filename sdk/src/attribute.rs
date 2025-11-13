use std::sync::RwLock;

use arrayvec::ArrayVec;

use proto_interface::{Attribute as ExtAttribute, external::attributes::attribute::Value as ExtValue};
// use proto_interface::attributes::{attribute::Value as ExtValue, Attribute as ExtAttribute, Vec3, Vec4};

use crate::{
    sdkError, Result,
};

pub type AttributeId = u8;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Vec3(f64, f64, f64),
    Vec4(f64, f64, f64, f64),
    String(String),
}

impl Value {
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }
}

pub struct Attribute {
    pub id: AttributeId,
    pub value: Value,
}

impl From<ExtAttribute> for Attribute {
    fn from(attr: ExtAttribute) -> Self {
        let value = match attr.value {
            Some(ExtValue::Bool(b)) => Value::Bool(b),
            Some(ExtValue::Vec3(v)) => Value::Vec3(v.x, v.y, v.z),
            Some(ExtValue::Vec4(v)) => Value::Vec4(v.x, v.y, v.z, v.w),
            Some(ExtValue::String(s)) => Value::String(s),
            None => Value::None,
        };
        Attribute {
            id: attr.id as u8,
            value,
        }
    }
}

// impl From<Attribute> for ExtAttribute {
//     fn from(attr: Attribute) -> Self {
//         if attr.value.is_none() {
//             return ExtAttribute {
//                 id: attr.id as u32,
//                 value: None,
//             };
//         }

//         let value = match attr.value {
//             Value::Bool(b) => ExtValue::Bool(b),
//             Value::String(s) => ExtValue::String(s),
//             Value::Vec3(x, y, z) => {
//                 let vec3 = Vec3 { x, y, z };
//                 ExtValue::Vec3(vec3)
//             }
//             Value::Vec4(x, y, z, w) => {
//                 let vec4 = Vec4 { x, y, z, w };
//                 ExtValue::Vec4(vec4)
//             }
//             Value::None => unreachable!("Already checked for None value"),
//         };

//         ExtAttribute {
//             id: attr.id as u32,
//             value: Some(value),
//         }
//     }
// }

pub enum AttributeChange {
    Created,
    Modified,
}

pub struct AttributeMap<const CAP: usize>(RwLock<ArrayVec<Attribute, CAP>>);

impl<const CAP: usize> AttributeMap<CAP> {
    pub fn new() -> Self {
        AttributeMap(RwLock::new(ArrayVec::new()))
    }

    pub fn upsert(&self, attr_id: AttributeId, value: Value) -> Result<AttributeChange> {
        let mut inner_write = self
            .0
            .write()
            .map_err(|_| sdkError::AttributesError("Attribute map is poisoned"))?;

        if inner_write.is_full() {
            return Err(sdkError::AttributesError(
                "Attributes storage is at capacity",
            ));
        };

        let change = match inner_write.binary_search_by(|attribute| attribute.id.cmp(&attr_id)) {
            Ok(index) => {
                let attribute = inner_write.get_mut(index).unwrap();
                attribute.value = value;

                AttributeChange::Modified
            }
            Err(index) => {
                inner_write.insert(index, Attribute { id: attr_id, value });
                inner_write.sort_by(|a, b| a.id.cmp(&b.id));
                AttributeChange::Created
            }
        };

        Ok(change)
    }

    pub fn get(self, attr_id: AttributeId) -> Result<Value> {
        let inner_read = self
            .0
            .read()
            .map_err(|_| sdkError::AttributesError("Attribute map is poisoned"))?;

        let attribute = match inner_read.binary_search_by(|attribute| attribute.id.cmp(&attr_id)) {
            Ok(index) => inner_read.get(index).unwrap(),
            Err(_) => return Err(sdkError::AttributesError("Attribute map is poisoned")),
        };

        Ok(attribute.value.clone())
    }
}

/// These attributes are reserved by the server and should not be used
/// by the client for custom attributes.
pub enum ReservedAttributeId {
    Name = 1,
    Position = 2,
    Velocity = 3,
    Rotation = 4,
}
