pub use crate::attribute::Value;
pub use external::attributes::Attribute;
pub use external::messages::TickUpdate;
pub use web3::*;

pub mod attribute;
pub mod external;
pub mod macros;
pub mod messages;
pub mod outgoing;
pub mod updates;
pub mod web3;

pub type AttributeId = u32;
pub type SessionId = u64;
pub type Timestamp = u64;
pub type EntityId = u32;
pub type PlayerId = u64;

pub const MAX_ATTRIBUTES_PER_COMPONENT: usize = 128;
pub const MAX_STRING_LENGTH: usize = 30;
pub const WEB3_VALUE_PAYLOAD_BYTES: usize = 64;

pub fn fill_payload<B>(payload: &mut ValuePayload, bytes: B)
where
    B: AsRef<[u8]>,
{
    let bytes = bytes.as_ref();
    let til = bytes.len();

    payload.as_mut()[..til].copy_from_slice(&bytes[..]);
}

#[cfg(test)]
mod tests {
    use crate::{bytes, fill_payload, from_bytes, ValuePayload};

    #[test]
    fn payload_operations() {
        let x: f64 = 1.;
        let y: f64 = 2.;
        let z: f64 = 3.;
        let w: f64 = 4.;
        let string = String::from("ABCDFEGH");
        let mut payload: ValuePayload = ValuePayload::default();

        let bytes = bytes!(string, String);
        fill_payload(&mut payload, bytes);

        assert_eq!(&payload.as_ref()[..bytes.len()], bytes);
        assert_eq!(from_bytes!(&payload.as_ref()[..bytes.len()], String), string);

        payload = ValuePayload::default();

        fill_payload(&mut payload, bytes!(x, y, z, w));
        assert_eq!(from_bytes!(payload.as_ref(), 32, f64), [x, y, z, w]);
    }
}
