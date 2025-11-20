pub use crate::attribute::Value;
pub use external::attributes::Attribute;
pub use external::messages::TickUpdate;

pub mod attribute;
pub mod external;
pub mod macros;
pub mod messages;
pub mod outgoing;
pub mod updates;

#[cfg(feature = "proto")]
pub mod errors;

#[cfg(feature = "web3")]
pub mod web3;

pub type AttributeId = u32;
pub type SessionId = u64;
pub type Timestamp = u64;
pub type EntityId = u32;
pub type PlayerId = u64;
pub type ServerId = u64;

pub const MAX_ATTRIBUTES_PER_COMPONENT: usize = 128;
pub const MAX_STRING_LENGTH: usize = 30;
pub const WEB3_VALUE_PAYLOAD_BYTES: usize = 64;

pub fn fill_payload<B>(payload: &mut web3::ValuePayload, bytes: B)
where
    B: AsRef<[u8]>,
{
    let bytes = bytes.as_ref();
    let til = bytes.len();

    payload.as_mut()[..til].copy_from_slice(&bytes[..]);
}

#[cfg(test)]
mod tests {
    use crate::{web3::ValuePayload, bytes, fill_payload, from_bytes, web3::WEB3_VALUE_PAYLOAD_BYTES};

    #[test]
    fn payload_operations() {
        let x: f64 = 1.;
        let y: f64 = 2.;
        let z: f64 = 3.;
        let w: f64 = 4.;
        let string = String::from("ABCDFEGH");
        let mut payload: ValuePayload = [0; WEB3_VALUE_PAYLOAD_BYTES];

        let bytes = bytes!(string, String);
        fill_payload(&mut payload, bytes);

        assert_eq!(&payload[..bytes.len()], bytes);
        assert_eq!(from_bytes!(&payload[..bytes.len()], String), string);

        payload = [0; WEB3_VALUE_PAYLOAD_BYTES];

        fill_payload(&mut payload, bytes!(x, y, z, w));
        assert_eq!(from_bytes!(payload, 32, f64), [x, y, z, w]);
    }
}
