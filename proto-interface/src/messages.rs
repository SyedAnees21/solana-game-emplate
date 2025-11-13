use prost::{
    bytes::{Bytes, BytesMut},
    DecodeError, EncodeError, Message,
};

use crate::{external, TickUpdate};

pub enum ToServer {
    None,
    Tick(TickUpdate),
}

impl ToServer {
    pub fn into_bytes(self) -> Result<Bytes, EncodeError> {
        let update = match self {
            Self::Tick(inner) => {
                let this = external::messages::to_server::Msg::Tick(inner);
                external::messages::ToServer { msg: Some(this) }
            }
            Self::None => external::messages::ToServer::default(),
        };

        let mut buffer = BytesMut::new();
        update.encode(&mut buffer)?;
        Ok(buffer.freeze())
    }

    pub fn from_bytes(bytes: Bytes) -> Result<Self, DecodeError> {
        let update = external::messages::ToServer::decode(bytes)?;

        let Some(this) = update.msg else {
            return Ok(Self::None);
        };

        let update = match this {
            external::messages::to_server::Msg::Tick(inner) => ToServer::Tick(inner),
        };

        Ok(update)
    }
}

impl From<TickUpdate> for ToServer {
    fn from(value: TickUpdate) -> Self {
        Self::Tick(value)
    }
}

pub enum ToClient {
    None,
    Tick(TickUpdate),
}

impl ToClient {
    pub fn into_bytes(self) -> Result<Bytes, EncodeError> {
        let update = match self {
            Self::Tick(inner) => {
                let this = external::messages::to_client::Msg::Tick(inner);
                external::messages::ToClient { msg: Some(this) }
            }
            Self::None => external::messages::ToClient::default(),
        };

        let mut buffer = BytesMut::new();
        update.encode(&mut buffer)?;
        Ok(buffer.freeze())
    }

    pub fn from_bytes(bytes: Bytes) -> Result<Self, DecodeError> {
        let update = external::messages::ToClient::decode(bytes)?;

        let Some(this) = update.msg else {
            return Ok(Self::None);
        };

        let update = match this {
            external::messages::to_client::Msg::Tick(inner) => ToClient::Tick(inner),
        };

        Ok(update)
    }
}
