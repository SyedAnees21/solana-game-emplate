use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use crate::{
    accumulate::Accumulator, alias::SessionId, handshake, pending::Pending, process,
    ConnectionStream, Result,
};
use futures_util::{SinkExt, StreamExt};
use proto_interface::{
    updates::{LocalUpdate, ServerUpdate},
    EntityId, Timestamp,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{interval, Interval},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct Session {
    session_id: SessionId,
    wallet_address: Option<String>,
    update_tx: UnboundedSender<LocalUpdate>,
    updates: Arc<Accumulator>,
}

impl Session {
    pub async fn start(server_address: &str, auth_bytes: &[u8]) -> Result<Arc<Self>> {
        let stream = TcpStream::connect(server_address).await?;
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

        let response = handshake::handshake(&mut framed, auth_bytes.to_vec()).await?;
        let (tx, rx) = unbounded_channel::<LocalUpdate>();

        let this = Arc::new(Self {
            session_id: response.assigned_id,
            wallet_address: response.wallet_key,
            update_tx: tx,
            updates: Arc::new(Accumulator::default()),
        });

        tokio::spawn(self::event_loop(framed, rx, this.clone()));

        Ok(this)
    }

    pub fn send(&self, tick_update: LocalUpdate) -> Result<()> {
        Ok(self.update_tx.send(tick_update)?)
    }

    pub fn time_now(&self) -> Timestamp {
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as Timestamp
    }

    pub fn updates_accumulator(&self) -> Arc<Accumulator> {
        self.updates.clone()
    }

    pub fn receive(&self) -> Option<ServerUpdate> {
        self.updates.pop()
    }

    pub fn derive_entity_id(&self, seed: EntityId) -> EntityId {
        self.session_id as EntityId ^ seed
    }
}

pub async fn event_loop(
    stream: ConnectionStream,
    mut request_rx: UnboundedReceiver<LocalUpdate>,
    session: Arc<Session>,
) {
    let mut pending = Pending::default();
    let (mut sink, mut stream) = stream.split();
    let mut tick_interval = tick_interval(10);

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                let Ok(bytes) = pending.take_updates() else {
                    eprintln!("Error occured while serializing the next update");
                    break;
                };

                if bytes.is_empty() {
                    continue;
                }

                match sink.send(bytes).await {
                    Ok(_) => continue,
                    Err(e) => {
                        eprintln!("Unable to send response to server, Error: {e}");
                        break;
                    }

                }
            }

            res = stream.next() => {
                let Some(response) = res else {
                    continue;
                };

                match response {
                    Ok(bytes) => {
                        match process::process_incoming(bytes.freeze(), session.clone()).await {
                            Ok(_) => continue,
                            Err(e) => {
                                eprintln!(
                                    "Closing session due to Error during message processing: {e}",
                                );
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Unable to listen to server due to Error: {e}");
                        break;
                    }
                }

            }

            Some(update_to_send) = request_rx.recv() => {
                pending.insert(update_to_send, session.time_now())
            }
        }
    }
}

pub fn tick_interval(ups: u8) -> Interval {
    interval(Duration::from_secs_f32(1. / ups as f32))
}
