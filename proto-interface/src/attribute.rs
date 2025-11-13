use crate::{
    external::{
        attributes::{attribute::Value as ExtValue, Attribute as ExtAttribute, Vec3, Vec4},
        messages::Entity,
    },
    AttributeId, EntityId, Timestamp,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Value {
    #[default]
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

impl From<Value> for ExtValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(b) => ExtValue::Bool(b),
            Value::Vec3(x, y, z) => ExtValue::Vec3(Vec3 { x, y, z }),
            Value::Vec4(x, y, z, w) => ExtValue::Vec4(Vec4 { x, y, z, w }),
            Value::String(s) => ExtValue::String(s),
            _ => todo!("Need to handle the none case"),
        }
    }
}

impl Into<Value> for ExtValue {
    fn into(self) -> Value {
        match self {
            ExtValue::Bool(b) => Value::Bool(b),
            ExtValue::Vec3(v) => Value::Vec3(v.x, v.y, v.z),
            ExtValue::Vec4(v) => Value::Vec4(v.x, v.y, v.z, v.w),
            ExtValue::String(s) => Value::String(s),
        }
    }
}

impl From<(AttributeId, (Value, Timestamp))> for ExtAttribute {
    fn from(value: (AttributeId, (Value, Timestamp))) -> Self {
        Self {
            id: value.0 as u32,
            value: Some(value.1 .0.into()),
            timestamp: value.1 .1,
        }
    }
}

impl From<(EntityId, Vec<ExtAttribute>)> for Entity {
    fn from(value: (EntityId, Vec<ExtAttribute>)) -> Self {
        Self {
            id: value.0,
            attrs: value.1,
        }
    }
}
