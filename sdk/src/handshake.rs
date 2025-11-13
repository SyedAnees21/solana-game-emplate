use futures_util::{SinkExt, StreamExt};
use prost::{bytes::BytesMut, Message};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use proto_interface::external::handshake::{HandshakeRequest, HandshakeResponse};

use crate::err::Error as sdkError;
// use crate::interface::handshake::{HandshakeRequest, HandshakeResponse};

type ConnectionStream = Framed<TcpStream, LengthDelimitedCodec>;

pub async fn handshake(
    stream: &mut ConnectionStream,
    auth_bytes: Vec<u8>,
) -> Result<HandshakeResponse, sdkError> {
    let request = HandshakeRequest { auth: auth_bytes };
    let mut buffer = BytesMut::new();
    request.encode(&mut buffer)?;

    stream.send(buffer.freeze()).await?;

    let Some(result) = stream.next().await else {
        return Err(sdkError::ConnectionError("Connection closed by server"));
    };

    let mut buffer = match result {
        Err(e) => return Err(sdkError::HandshakeFailed(e.to_string())),
        Ok(bytes) => bytes,
    };

    let response = HandshakeResponse::decode(&mut buffer)?;
    Ok(response)
}
