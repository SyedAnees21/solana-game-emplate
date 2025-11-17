use arrayvec::ArrayString;

use crate::{
    bytes,
    external::{
        attributes::{attribute::Value as ExtValue, Attribute as ExtAttribute, Vec3, Vec4},
        messages::Entity,
    },
    from_bytes, web3, AttributeId, EntityId, Timestamp, ValuePayload, ValueTag, MAX_STRING_LENGTH,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Value {
    #[default]
    None,
    Bool(bool),
    Vec3(f64, f64, f64),
    Vec4(f64, f64, f64, f64),
    String(ArrayString<MAX_STRING_LENGTH>),
}

impl Value {
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }

    pub fn to_payload(&self) -> (ValueTag, ValuePayload) {
        let mut payload = ValuePayload::default();

        fn fill_payload(payload: &mut ValuePayload, bytes: impl AsRef<[u8]>, delimited: bool) {
            let bytes = bytes.as_ref();
            let mut til = bytes.len();
            let mut from = 0;

            if delimited {
                from = 1;
                til += 1;
                payload.as_mut()[0] = til as u8;
            }
            payload.as_mut()[from..til].copy_from_slice(&bytes[..]);
        }

        match self {
            Value::None => (web3::tags::NONE.into(), payload),
            Value::Bool(b) => {
                if *b {
                    payload.as_mut()[0] = 1
                };

                (web3::tags::BOOL.into(), payload)
            }
            Value::Vec3(x, y, z) => {
                let bytes = bytes!(x, y, z);
                fill_payload(&mut payload, bytes, false);

                (web3::tags::VEC3.into(), payload)
            }
            Value::Vec4(x, y, z, w) => {
                let bytes = bytes!(x, y, z, w);
                fill_payload(&mut payload, bytes, false);

                (web3::tags::VEC4.into(), payload)
            }
            Value::String(s) => {
                let bytes = bytes!(s, String);
                fill_payload(&mut payload, bytes, true);

                (web3::tags::STRING.into(), payload)
            }
        }
    }

    pub fn from_payload(payload: ValuePayload, tag: ValueTag) -> Self {
        let tag = *tag.as_ref();
        match tag {
            t if t == web3::tags::NONE => Value::None,
            t if t == web3::tags::BOOL => Value::Bool(payload.as_ref()[0] == 1),
            t if t == web3::tags::VEC3 => {
                let v = from_bytes!(payload.as_ref(), 24, f64);
                Value::Vec3(v[0], v[1], v[2])
            }
            t if t == web3::tags::VEC4 => {
                let v = from_bytes!(payload.as_ref(), 32, f64);
                Value::Vec4(v[0], v[1], v[2], v[3])
            }
            t if t == web3::tags::STRING => {
                let v = from_bytes!(payload.as_ref(), String);
                Value::String(ArrayString::from(&v).unwrap_or_default())
            }
            _ => unreachable!("Should not be reached for unknown tags"),
        }
    }
}

impl From<Value> for ExtValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(b) => ExtValue::Bool(b),
            Value::Vec3(x, y, z) => ExtValue::Vec3(Vec3 { x, y, z }),
            Value::Vec4(x, y, z, w) => ExtValue::Vec4(Vec4 { x, y, z, w }),
            Value::String(s) => ExtValue::String(s.to_string()),
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

            // TODO: Handle the proper conversion in case of string received
            // larger than the allowed length instead of storing an empty value.
            ExtValue::String(s) => Value::String(ArrayString::from(&s).unwrap_or_default()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition() {
        println!("{}", 1 + 1);
        println!("{}", "Dhanno")
    }

    #[test]
    fn values_payload() {
        let b = Value::Bool(true);
        let v3 = Value::Vec3(1., 2., 3.);
        let v4 = Value::Vec4(1., 2., 3., 4.);
        let s = Value::String(ArrayString::from("ABCDefgh").unwrap());

        let len = "ABCDefgh".as_bytes().len() + 1;

        let (tag, payload) = b.to_payload();
        assert_eq!((*tag.as_ref(), payload.as_ref()[0]), (web3::tags::BOOL, 1));

        let (tag, payload) = v3.to_payload();
        assert_eq!(
            (*tag.as_ref(), payload.as_ref()[..24].into()),
            (web3::tags::VEC3, bytes!(1_f64, 2_f64, 3_f64))
        );

        let (tag, payload) = v4.to_payload();
        assert_eq!(
            (*tag.as_ref(), payload.as_ref()[..32].into()),
            (web3::tags::VEC4, bytes!(1_f64, 2_f64, 3_f64, 4_f64))
        );

        let (tag, payload) = s.to_payload();
        assert_eq!(
            (
                *tag.as_ref(),
                payload.as_ref()[1..len].into(),
                payload.as_ref()[0]
            ),
            (web3::tags::STRING, bytes!("ABCDefgh", String), len as u8)
        );
    }
}
