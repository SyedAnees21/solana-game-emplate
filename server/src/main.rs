use sdk::alias::Result;
use std::sync::Arc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::state::Global;

pub mod commands;
pub mod component;
pub mod entity;
pub mod handshake;
pub mod process;
pub mod session;
pub mod state;
pub mod version;
pub mod web3;

type ConnectionStream = Framed<tokio::net::TcpStream, LengthDelimitedCodec>;

#[tokio::main]
async fn main() -> Result<()> {
    let global = Global::new();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let peer_addr = socket.peer_addr().unwrap();
        let global = Arc::clone(&global);
        tokio::spawn(async move {
            let result = handshake::handle_connection(socket, global)
                .await
                .map_err(|e| {
                    format!("Unable to establish connection with {peer_addr:?} due to : {e}")
                });

            let Err(err) = result else {
                return;
            };

            eprintln!("{err}");
        });
    }
}
