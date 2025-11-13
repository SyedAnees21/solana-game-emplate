pub use external::messages::TickUpdate;
pub use external::attributes::Attribute;
pub use crate::attribute::Value;

pub mod external;
pub mod attribute;
pub mod outgoing;
pub mod messages;
pub mod updates;

pub type AttributeId = u32;
pub type SessionId = u64;
pub type Timestamp = u64;
pub type EntityId = u32;
pub type PlayerId = SessionId;

pub const MAX_ATTRIBUTES_PER_COMPONENT: usize = 128;
pub const MAX_STRING_LENGTH: usize = 64;

