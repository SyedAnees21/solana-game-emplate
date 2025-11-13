use std::{net::SocketAddr, ops::ControlFlow, sync::Arc};

use anchor_client::solana_sdk::signature::Keypair;
use futures_util::{SinkExt, StreamExt};
use sdk::alias::SessionId;
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    commands::Server2Session, process, state::Global, version::EntityVersion, ConnectionStream,
};

pub struct Session {
    id: SessionId,
    address: SocketAddr,
    wallet: Keypair,
    global: Arc<Global>,
    local_versions: EntityVersion,
}

impl Session {
    pub fn new(
        assigned_id: SessionId,
        wallet: Keypair,
        socket_addr: SocketAddr,
        global: Arc<Global>,
    ) -> Self {
        let (channel_tx, _channel_rx) = unbounded_channel::<Server2Session>();
        global.register_session(assigned_id, channel_tx);

        Self {
            address: socket_addr,
            wallet,
            id: assigned_id,
            global,
            local_versions: EntityVersion::default(),
        }
    }

    pub fn global(&self) -> Arc<Global> {
        self.global.clone()
    }

    pub fn my_id(&self) -> SessionId {
        self.id
    }

    pub fn my_address(&self) -> SocketAddr {
        self.address
    }

    pub fn version_map(&self) -> &EntityVersion {
        &self.local_versions
    }

    pub fn version_map_mut(&mut self) -> &mut EntityVersion {
        &mut self.local_versions
    }
}

pub async fn spawn_event_loop(stream: ConnectionStream, mut session: Session) {
    let (mut sink, mut stream) = stream.split();
    let mut next_tick = session.global.get_tick_interval();

    println!("Event loop started for session: {:?}", session.address);

    loop {
        let flow = tokio::select! {
            res = stream.next() => {
                let Some(response) = res else {
                    return
                };

                match response {
                    Ok(bytes) => {
                        match process::process_message(bytes.freeze(), &mut session).await {
                            Ok(_) => ControlFlow::Continue(()),
                            Err(e) => {
                                eprintln!(
                                    "Closing session {}, due to Error during message processing: {e}",
                                    session.address
                                );
                                ControlFlow::Break(())
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Closing session {}, due to Error: {e}", session.address);
                        ControlFlow::Break(())
                    }
                }
            },

            _ = next_tick.tick() => {
                let bytes = match process::create_updates(&mut session).await {
                    Ok(to_send) => to_send,
                    Err(e) => {
                        eprintln!("Error while processing the response for session{}, {e}", session.my_id());
                        continue;
                    }
                };

                if bytes.is_empty() {
                    continue;
                }

                let res = sink.send(bytes).await;

                if res.is_err() {
                    eprintln!("Error while sending updates for session{}, {res:?}", session.my_id());
                    break;
                }

                ControlFlow::Continue(())
            }
        };

        if flow.is_break() {
            break;
        }
    }
}
