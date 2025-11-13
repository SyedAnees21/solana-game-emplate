use prost::{
    bytes::{Bytes, BytesMut},
    DecodeError, EncodeError, Message,
};

use crate::{external, TickUpdate};

pub enum ClientOutGoing {
    None,
    Tick(TickUpdate),
}

impl ClientOutGoing {
    pub fn into_bytes(self) -> Result<Bytes, EncodeError> {
        let update = match self {
            Self::Tick(inner) => {
                let this = external::outgoing::client_out_going::ClientOutGoing::Tick(inner);
                external::outgoing::ClientOutGoing {
                    client_out_going: Some(this),
                }
            }
            Self::None => external::outgoing::ClientOutGoing::default(),
        };

        let mut buffer = BytesMut::new();
        update.encode(&mut buffer)?;
        Ok(buffer.freeze())
    }

    pub fn from_bytes(bytes: Bytes) -> Result<Self, DecodeError> {
        let update = external::outgoing::ClientOutGoing::decode(bytes)?;
        
        let Some(this) = update.client_out_going else {
            return Ok(Self::None);
        };

        let update = match this {
            external::outgoing::client_out_going::ClientOutGoing::Tick(inner) => {
                ClientOutGoing::Tick(inner)
            }
        };

        Ok(update)
    }
}

impl From<TickUpdate> for ClientOutGoing {
    fn from(value: TickUpdate) -> Self {
        Self::Tick(value)
    }
}