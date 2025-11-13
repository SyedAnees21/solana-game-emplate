use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub mod accumulate;
pub mod alias;
pub mod attribute;
pub mod err;
pub mod handshake;
pub mod pending;
pub mod process;
pub mod session;
pub mod transceiver;

pub use err::Error as sdkError;
pub use handshake::handshake;

pub type Result<T> = std::result::Result<T, sdkError>;
pub type ConnectionStream = Framed<TcpStream, LengthDelimitedCodec>;

pub type AttributeId = u8;
pub type SessionId = u64;
pub type PlayerId = SessionId;
pub type Timestamp = u64;

pub const MAX_ATTRIBUTES_PER_COMPONENT: usize = 128;
pub const MAX_STRING_LENGTH: usize = 64;