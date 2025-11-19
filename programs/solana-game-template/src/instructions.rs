use crate::state::{EntityState, GameState, PlayerState};
use anchor_lang::prelude::*;

const ANCHOR_DISCRIMINATOR: usize = 8;

#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = game_owner,
        space = ANCHOR_DISCRIMINATOR + GameState::INIT_SPACE,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub game_state: AccountLoader<'info, GameState>,

    #[account(mut)]
    pub game_owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(game_id: u64, player_id: u64)]
pub struct InitializePlayer<'info> {
    #[account(
        init,
        payer = game_owner,
        space = ANCHOR_DISCRIMINATOR + PlayerState::INIT_SPACE,
        seeds = [b"player", player_id.to_le_bytes().as_ref()],
        bump
    )]
    pub player_state: AccountLoader<'info, PlayerState>,

    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub game_state: AccountLoader<'info, GameState>,

    #[account(mut)]
    pub game_owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(player_id: u64, entity_id: u32)]
pub struct InitializeEntity<'info> {
    #[account(
        init,
        payer = player,
        space = ANCHOR_DISCRIMINATOR + EntityState::INIT_SPACE,
        seeds = [b"entity", player_id.to_le_bytes().as_ref(), entity_id.to_le_bytes().as_ref()],
        bump
    )]
    pub entity: AccountLoader<'info, EntityState>,

    #[account(
        mut,
        seeds = [b"player", player_id.to_le_bytes().as_ref()],
        bump
    )]
    pub player_state: AccountLoader<'info, PlayerState>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(player_id: u64, entity_id: u32)]
pub struct UpdateEntity<'info> {
    #[account(
        mut,
        seeds = [b"entity", player_id.to_le_bytes().as_ref(), entity_id.to_le_bytes().as_ref()],
        bump
    )]
    pub entity: AccountLoader<'info, EntityState>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,
}
