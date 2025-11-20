use proto_interface::errors::AppError;
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc::unbounded_channel, time::interval};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{state::Global, web3::service::W3Commands};

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
async fn main() -> Result<(), AppError> {
    let global = Global::new();
    let (tx, receiver) = unbounded_channel::<W3Commands>();

    let server_id = global.global_id();
    let server_name = global.server_name();
    let wallet_path = global.provider_wallet_path();
    let transaction_tick = interval(Duration::from_millis(500));

    web3::provider::SolanaProvider::start_provider(
        anchor_client::Cluster::Localnet,
        wallet_path,
        receiver,
        server_id,
        server_name,
        transaction_tick,
    )
    .await?;

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
