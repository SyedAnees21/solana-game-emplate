use crate::{bytes, AttributeId, EntityId, Value, MAX_ATTRIBUTES_PER_COMPONENT, MAX_STRING_LENGTH};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

pub const PAYLOAD_BYTES: usize = 128;
pub const ANCHOR_DISCRIMINATOR: usize = 8;
pub const WEB3_VALUE_PAYLOAD_BYTES: usize = 64;
pub const MAX_ATTR_PER_ENTITY: usize = MAX_ATTRIBUTES_PER_COMPONENT;

pub type ValueTag = u8;
pub type ValuePayload = [u8; WEB3_VALUE_PAYLOAD_BYTES];

pub mod tags {
    type ValueTag = u8;

    pub const NONE: ValueTag = 0;
    pub const BOOL: ValueTag = 1;
    pub const VEC3: ValueTag = 2;
    pub const VEC4: ValueTag = 3;
    pub const STRING: ValueTag = 4;
}

// #[repr(C)]
// #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, AnchorSerialize, AnchorDeserialize)]
// pub struct ValueTag(u8);

// impl From<u8> for ValueTag {
//     fn from(value: u8) -> Self {
//         Self(value)
//     }
// }

// impl AsRef<u8> for ValueTag {
//     fn as_ref(&self) -> &u8 {
//         &self.0
//     }
// }

// #[repr(C)]
// #[derive(Clone, Copy, Pod, Zeroable, AnchorSerialize, AnchorDeserialize, Debug)]
// pub struct ValuePayload([u8; WEB3_VALUE_PAYLOAD_BYTES]);

// impl Default for ValuePayload {
//     fn default() -> Self {
//         Self([0; WEB3_VALUE_PAYLOAD_BYTES])
//     }
// }

// impl AsRef<[u8; WEB3_VALUE_PAYLOAD_BYTES]> for ValuePayload {
//     fn as_ref(&self) -> &[u8; WEB3_VALUE_PAYLOAD_BYTES] {
//         &self.0
//     }
// }

// impl AsMut<[u8; WEB3_VALUE_PAYLOAD_BYTES]> for ValuePayload {
//     fn as_mut(&mut self) -> &mut [u8; WEB3_VALUE_PAYLOAD_BYTES] {
//         &mut self.0
//     }
// }

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Pod, Zeroable, Debug, Default)]
pub struct InlineString {
    pub len: u8,
    pub buffer: [u8; MAX_STRING_LENGTH],
}

impl anchor_lang::Space for InlineString {
    const INIT_SPACE: usize = size_of::<Self>();
}

impl AsMut<[u8; MAX_STRING_LENGTH]> for InlineString {
    fn as_mut(&mut self) -> &mut [u8; MAX_STRING_LENGTH] {
        &mut self.buffer
    }
}

impl AsRef<[u8; MAX_STRING_LENGTH]> for InlineString {
    fn as_ref(&self) -> &[u8; MAX_STRING_LENGTH] {
        &self.buffer
    }
}

impl<T: AsRef<[u8]>> From<T> for InlineString {
    fn from(value: T) -> Self {
        let mut iniline_string = InlineString::default();
        let bytes = value.as_ref();
        iniline_string.set_len(bytes.len());
        iniline_string.as_mut()[..bytes.len()].copy_from_slice(bytes);

        iniline_string
    }
}

impl From<Value> for InlineString {
    fn from(value: Value) -> Self {
        let Value::String(s) = value else {
            return Default::default();
        };
        InlineString::from(s.as_bytes())
    }
}

impl InlineString {
    pub fn set_len(&mut self, len: usize) {
        self.len += len as u8;
    }

    pub fn get_len(&self) -> usize {
        self.len as usize
    }

    pub fn from_string(mut value: String) -> Self {
        let mut inline = Self::zeroed();

        value.truncate(MAX_STRING_LENGTH);
        let len = value.len();

        inline.set_len(len);
        inline.as_mut()[..len].copy_from_slice(&bytes!(value, String));

        inline
    }

    pub fn as_string(&self) -> String {
        String::from_utf8(self.as_ref()[..self.get_len()].into()).unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct EntityHeader {
    pub entity_id: EntityId,
    pub owner: [u8; 32],
    pub len: u32,
    pub bump: u8,
    pub _pad: [u8; 3],
}

impl anchor_lang::Space for EntityHeader {
    const INIT_SPACE: usize = size_of::<Self>();
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct AttributeSlot {
    pub id: AttributeId,
    pub tag: ValueTag,
    pub used: u8,
    pub _pad: [u8; 2],
    pub payload: ValuePayload,
}

impl AttributeSlot {
    pub fn id(&self) -> AttributeId {
        self.id
    }

    pub fn in_use(&self) -> bool {
        self.used != 0
    }

    pub fn value(&self) -> Value {
        Value::from_payload(self.payload, self.tag)
    }

    pub fn tag(&self) -> ValueTag {
        self.tag
    }
}

impl anchor_lang::Space for AttributeSlot {
    const INIT_SPACE: usize = size_of::<Self>();
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn inline_string_ops() {
        let name = String::from("Anees");
        let inline_name = InlineString::from_string(name.clone());

        assert!(!inline_name.is_empty());
        assert_eq!(name, inline_name.as_string());
        assert_eq!(inline_name.get_len(), name.len());
    }

    #[test]
    fn inline_string_truncation() {
        let address = String::from("3UYVFTzKfxT842KUoWrqLsZf1PJ1anCVU1rCvZcFwMSx");
        let inline_string = InlineString::from_string(address.clone());

        assert!(!inline_string.is_empty());
        assert_eq!(
            inline_string.as_string(),
            String::from_str(&address[..MAX_STRING_LENGTH]).unwrap()
        )
    }
}
