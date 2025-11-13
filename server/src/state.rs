use anchor_client::solana_sdk::signature::Keypair;
use dashmap::DashSet;
use proto_interface::{PlayerId, SessionId};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{interval, Interval},
};

use crate::{
    commands::{Channels, Server2Session},
    entity::{Entities, EntityId},
};

type Players = DashSet<PlayerId>;
type PlayerEntities = Entities;

// TODO: This is just a mock relevance snapshot structure. In reality
// its nothing more than just global container for all the entities inside
// the server.
// type RelevanceSnapshot = DashMap<PlayerId, DashSet<EntityId>>;
type RelevanceSnapshot = DashSet<EntityId>;

type UPS = u8;

pub struct Global {
    global_id: AtomicU64,
    players: Arc<Players>,
    entities: Arc<PlayerEntities>,
    relevance_snapshot: Arc<RelevanceSnapshot>,
    channels: Arc<Channels>,
    tick_rate: UPS,
}

impl Global {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn assign_id(&self) -> SessionId {
        self.global_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn generate_local_wallet(&self) -> Keypair {
        Keypair::new()
    }

    pub fn register_session(
        &self,
        session_id: SessionId,
        channel: UnboundedSender<Server2Session>,
    ) {
        self.players.insert(session_id);
        self.channels.register_channel(session_id, channel);

        // TODO: Once we have the working relevance manager. We will need
        // this to register the slot for each new session to get its own
        // relevance snapshot.
        // self.relevance_snapshot.insert(session_id, Default::default());
    }

    pub fn init_global_id() -> AtomicU64 {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        AtomicU64::new(seed)
    }

    pub fn get_tick_interval(&self) -> Interval {
        let ups = 1. / self.tick_rate as f32;
        interval(Duration::from_secs_f32(ups))
    }

    pub fn entities(&self) -> Arc<PlayerEntities> {
        self.entities.clone()
    }

    pub fn relevance_snap(&self) -> Arc<RelevanceSnapshot> {
        self.relevance_snapshot.clone()
    }
}

impl Default for Global {
    fn default() -> Self {
        let global_id = Self::init_global_id();
        Self {
            global_id,
            players: Arc::new(Players::default()),
            channels: Arc::new(Channels::default()),
            entities: Arc::new(PlayerEntities::default()),
            relevance_snapshot: Arc::new(RelevanceSnapshot::default()),
            tick_rate: 10,
        }
    }
}
