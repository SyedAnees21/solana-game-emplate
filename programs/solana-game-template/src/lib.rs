use anchor_lang::prelude::*;
use proto_interface::web3::*;

use crate::instructions::*;

pub mod instructions;
pub mod state;

declare_id!("ABXBgpeKKeRu9eKLtCVF7MKnc6E4ABkx8nJdLSsFdK2M");

#[program]
pub mod solana_game {
    use super::*;

    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        game_id: u64,
        game_name: InlineString,
    ) -> Result<()> {
        let mut game_state = ctx.accounts.game_state.load_init()?;

        game_state.id = game_id;
        game_state.game_name = game_name;

        msg!("Initializing game with ID: {}", game_id);
        Ok(())
    }

    pub fn initialize_player(
        ctx: Context<InitializePlayer>,
        _game_id: u64,
        player_id: u64,
        player_name: InlineString,
    ) -> Result<()> {
        let mut player_state = ctx.accounts.player_state.load_init()?;
        let mut game_state = ctx.accounts.game_state.load_mut()?;

        player_state.player_id = player_id;
        player_state.player_name = player_name;

        game_state.total_players += 1;

        msg!(
            "Game authority initializing player account for: {:?}",
            player_state
        );
        Ok(())
    }

    pub fn initialize_entity(
        ctx: Context<InitializeEntity>,
        _player_id: u64,
        entity_id: u32,
    ) -> Result<()> {
        msg!("Initializing entity with ID: {}", entity_id);
        let mut entity = ctx.accounts.entity.load_init()?;
        let mut player_state = ctx.accounts.player_state.load_mut()?;

        entity.header.entity_id = entity_id;
        entity.header.owner = ctx.accounts.player.key().to_bytes();

        player_state.total_entities += 1;

        Ok(())
    }

    pub fn update_entity(
        ctx: Context<UpdateEntity>,
        _player_id: u64,
        _entity_id: u32,
        id: u32,
        tag: ValueTag,
        payload: ValuePayload,
    ) -> Result<()> {
        let mut entity = ctx.accounts.entity.load_mut()?;

        require_keys_eq!(
            Pubkey::from(entity.header.owner),
            ctx.accounts.player.key(),
            GameError::EntityOwnerMismatch
        );

        let mut free_slot = None;

        for slot_index in 0..MAX_ATTR_PER_ENTITY {
            let slot = entity.attributes[slot_index];
            if slot.in_use() {
                if slot.tag() == tag && slot.id() == id {
                    entity.attributes[slot_index]
                        .payload
                        .as_mut()
                        .copy_from_slice(payload.as_ref());
                    return Ok(());
                }
            }

            if free_slot.is_none() {
                free_slot = Some(slot_index);
            }
        }

        let Some(free_slot) = free_slot else {
            return Err(error!(GameError::EntityStorageFull));
        };

        entity.attributes[free_slot].id = id;
        entity.attributes[free_slot].used = 1;
        entity.attributes[free_slot].tag = tag;
        entity.attributes[free_slot]
            .payload
            .as_mut()
            .copy_from_slice(payload.as_ref());

        entity.header.len = entity.header.len.saturating_add(1);

        Ok(())
    }
}

// #[account]
// #[derive(InitSpace)]
// pub struct GameState {
//     pub game_id: u64,
//     pub player_amount: u64,
//     pub authority: Pubkey,
//     pub bump: u8,
// }

// #[account]
// #[derive(InitSpace, Debug)]
// pub struct PlayerState {
//     pub player: Pubkey,
//     pub game_state: Pubkey,
//     pub coin_count: u64,
//     pub termination_count: u64,
//     pub bump: u8,
// }

// --- Errors ---

#[error_code]
pub enum GameError {
    #[msg("Not enough coins to claim reward.")]
    NotEnoughCoins,
    #[msg("Signer is not the player associated with this state.")]
    NotPlayerAuthority,
    #[msg("Signer is not the player (owner) associated with this entity.")]
    EntityOwnerMismatch,
    #[msg("Unable to upsert the new attribute due to entity out-of-slots.")]
    EntityStorageFull,
}
