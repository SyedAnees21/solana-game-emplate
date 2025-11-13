use std::sync::Arc;

use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use futures_util::{SinkExt, StreamExt};
use prost::{bytes::BytesMut, Message};
use proto_interface::external::handshake::{HandshakeRequest, HandshakeResponse};
use sdk::{
    alias::{Result, SessionId},
    err::Error,
};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    session::{self, Session},
    state::Global,
};

type ConnectionStream = Framed<TcpStream, LengthDelimitedCodec>;

pub async fn handle_connection(socket: tokio::net::TcpStream, state: Arc<Global>) -> Result<()> {
    let addr = socket.peer_addr()?;
    let mut framed = Framed::new(socket, LengthDelimitedCodec::new());
    let (session_id, wallet) = handshake(&mut framed, &state).await?;
    let session = Session::new(session_id, wallet, addr, state);

    tokio::spawn(session::spawn_event_loop(framed, session));

    Ok(())
}

pub async fn handshake(
    stream: &mut ConnectionStream,
    state: &Arc<Global>,
) -> Result<(SessionId, Keypair)> {
    let Some(result) = stream.next().await else {
        return Err(Error::ConnectionError("Remote connection dropped"));
    };

    let rcv = match result {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(Error::HandshakeFailed(format!(
                "Error occured while parsing handshak: {e}"
            )))
        }
    };

    let _request = HandshakeRequest::decode(rcv)?;

    let session_id = state.assign_id();
    let session_local_wallet = state.generate_local_wallet();

    let response = HandshakeResponse {
        assigned_id: session_id,
        wallet_key: Some(session_local_wallet.pubkey().to_string()),
    };

    let mut buffer = BytesMut::new();
    response.encode(&mut buffer)?;

    stream.send(buffer.freeze()).await?;

    Ok((session_id, session_local_wallet))
}
