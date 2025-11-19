use dashmap::DashMap;
use sdk::alias::{PlayerId, SessionId};
use tokio::sync::mpsc::UnboundedSender;

pub enum S2SCommand {
    Stream
}

pub struct WorldUpdate {
    
}

pub enum Server2Session {
    World(WorldUpdate)
}

pub type IntChannels = DashMap<PlayerId, UnboundedSender<Server2Session>>; 

#[derive(Default)]
pub struct Channels(IntChannels);

impl AsRef<IntChannels> for Channels {
    fn as_ref(&self) -> &IntChannels {
        &self.0
    }
}

impl AsMut<IntChannels> for Channels {
    fn as_mut(&mut self) -> &mut IntChannels {
        &mut self.0
    }
}

impl Channels {
    pub fn register_channel(&self, id: SessionId, channel: UnboundedSender<Server2Session>) {
        self.as_ref().insert(id, channel);
    }
}