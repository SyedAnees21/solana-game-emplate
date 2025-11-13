use crate::err::Error as sdkError;

pub type AttributeId = u8;
pub type SessionId = u64;
pub type PlayerId = SessionId;
pub type Timestamp = u64;

pub type Result<T> = std::result::Result<T, sdkError>;

pub const MAX_ATTRIBUTES_PER_COMPONENT: usize = 128;
pub const MAX_STRING_LENGTH: usize = 64;