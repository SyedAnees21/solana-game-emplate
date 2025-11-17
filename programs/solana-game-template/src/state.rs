use anchor_lang::prelude::*;
use proto_interface::web3::*;

#[account(zero_copy(unsafe))]
#[derive(InitSpace, Debug)]
pub struct EntityState {
    pub header: EntityHeader,
    pub attributes: [AttributeSlot; MAX_ATTR_PER_ENTITY],
}

impl EntityState {
    pub fn next_available_slot(&self) -> Option<usize> {
        if self.header.len as usize == MAX_ATTR_PER_ENTITY {
            return None;
        }
        Some(self.header.len as usize)
    }
}

#[account(zero_copy(unsafe))]
#[derive(InitSpace, Debug)]
pub struct PlayerState {
    pub player_id: u64,
    pub player_name: InlineString,
    pub total_entities: u32,
}

#[account(zero_copy(unsafe))]
#[derive(InitSpace, Debug)]
pub struct GameState {
    pub id: u64,
    pub game_name: InlineString,
    pub total_players: u32,
}
