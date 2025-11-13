use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::solana_program;
// Replace this with your own program ID after running `anchor build`
declare_id!("3UYVFTzKfxT842KUoWrqLsZf1PJ1anCVU1rCvZcFwMSx");

// 1 SOL in lamports
const ONE_SOL: u64 = 1_000_000_000;
const COIN_REQUIREMENT: u64 = 10;

#[program]
pub mod solana_game {
    use super::*;

    /// Initializes a new game state and a treasury PDA to hold rewards.
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        game_id: u64,
        player_amount: u64,
    ) -> Result<()> {
        msg!("Initializing game with ID: {}", game_id);
        ctx.accounts.game_state.game_id = game_id;
        ctx.accounts.game_state.player_amount = player_amount;
        ctx.accounts.game_state.authority = ctx.accounts.authority.key();
        ctx.accounts.game_state.bump = ctx.bumps.game_state;
        Ok(())
    }

    /// Funds the game's treasury PDA with SOL.
    pub fn fund_treasury(ctx: Context<FundTreasury>, amount: u64) -> Result<()> {
        msg!("Funding treasury with {} lamports", amount);
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;
        msg!(
            "Treasury funded. New balance: {}",
            ctx.accounts.treasury.to_account_info().lamports()
        );
        Ok(())
    }

    /// Initializes a player's state PDA. Can only be called by the game authority.
    /// Seeds: ["player", game_state_key, player_key]
    pub fn initialize_player(ctx: Context<InitializePlayer>, player: Pubkey) -> Result<()> {
        msg!("Game authority initializing player account for: {}", player);
        let player_state = &mut ctx.accounts.player_state;
        player_state.player = player;
        player_state.game_state = ctx.accounts.game_state.key();
        player_state.coin_count = 0;
        player_state.termination_count = 0;
        player_state.bump = ctx.bumps.player_state;
        Ok(())
    }

    /// (Helper instruction for testing) Adds coins to a player's state.
    /// In a real game, this logic would be more complex (e.g., only callable by the game authority).
    pub fn add_coins(ctx: Context<AddCoins>, amount: u64) -> Result<()> {
        // Basic check: Only the player can add coins to their own account.
        // In a real app, you'd want a more secure check (e.g., only game admin)
        require_keys_eq!(
            ctx.accounts.authority.key(),
            ctx.accounts.game_state.authority.key(),
            GameError::NotPlayerAuthority
        );
        
        ctx.accounts.player_state.coin_count += amount;
        msg!(
            "Added {} coins. New total: {}",
            amount,
            ctx.accounts.player_state.coin_count
        );
        Ok(())
    }

    /// Allows a player to claim 1 SOL if they have >= 10 coins.
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        msg!("Player {} attempting to claim reward...", ctx.accounts.player.key());
        let player_state = &mut ctx.accounts.player_state;

        // 1. Check if player has enough coins
        require!(
            player_state.coin_count >= COIN_REQUIREMENT,
            GameError::NotEnoughCoins
        );

        // 2. Subtract coins
        player_state.coin_count -= COIN_REQUIREMENT;
        msg!(
            "Subtracted {} coins. Remaining coins: {}",
            COIN_REQUIREMENT,
            player_state.coin_count
        );

        // 3. Transfer SOL from treasury to player
        // Build seed slices that live long enough for invoke_signed
        let game_id = ctx.accounts.game_state.game_id.to_le_bytes();
        let bump = ctx.bumps.treasury;
        let seed0: &[u8] = b"treasury";
        let seed1: &[u8] = game_id.as_ref();
        let seed2: &[u8] = &[bump];
        let signer_seeds: &[&[u8]] = &[seed0, seed1, seed2];
        let signer = &[signer_seeds];

        solana_program::program::invoke_signed(
            &solana_program::system_instruction::transfer(
                &ctx.accounts.treasury.key(),
                &ctx.accounts.player.key(),
                ONE_SOL,
            ),
            &[
                ctx.accounts.treasury.to_account_info(),
                ctx.accounts.player.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer,
        )?;

        msg!(
            "Successfully transferred 1 SOL to player {}",
            ctx.accounts.player.key()
        );
        Ok(())
    }
}

// --- Account Structures ---

#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GameState::INIT_SPACE,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: This PDA is created as a zero-data, system-owned account used only to hold lamports.
    ///         We don't deserialize any data from it.
    #[account(
        init,
        payer = authority,
        space = 0, // zero-data system-owned account to hold lamports only
        owner = system_program.key(),
        seeds = [b"treasury", game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub treasury: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FundTreasury<'info> {
    /// CHECK: This PDA is a zero-data, system-owned account used only to hold lamports.
    ///         No deserialization is required and it's safe to treat as UncheckedAccount.
    #[account(
        mut,
        seeds = [b"treasury", game_state.game_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        seeds = [b"game", game_state.game_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub game_state: Account<'info, GameState>,

    #[account(mut, address = game_state.authority)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(player: Pubkey)]
pub struct InitializePlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PlayerState::INIT_SPACE,
        seeds = [b"player", game_state.game_id.to_le_bytes().as_ref(), player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        seeds = [b"game", game_state.game_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub game_state: Account<'info, GameState>,

    #[account(mut, address = game_state.authority)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddCoins<'info> {
    #[account(
        mut,
        seeds = [b"player", game_state.game_id.to_le_bytes().as_ref(), player_state.player.key().as_ref()],
        bump,
        has_one = game_state,
    )]
    pub player_state: Account<'info, PlayerState>,
    
    pub game_state: Account<'info, GameState>, // Used for seed validation
    
    #[account(mut, address = game_state.authority)]
    pub authority: Signer<'info>, // The player whose state is being modified
}


#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(
        mut,
        seeds = [b"player", game_state.key().as_ref(), player.key().as_ref()],
        bump = player_state.bump,
        has_one = game_state,
        has_one = player,
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        seeds = [b"game", game_state.game_id.to_le_bytes().as_ref()],
        bump = game_state.bump,
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: This PDA is a zero-data, system-owned account used only to hold lamports.
    ///         No deserialization is required and it's safe to treat as UncheckedAccount.
    #[account(
        mut,
        seeds = [b"treasury", game_state.game_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub treasury: UncheckedAccount<'info>,

    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// --- Account Data ---

#[account]
#[derive(InitSpace)]
pub struct GameState {
    pub game_id: u64,
    pub player_amount: u64,
    pub authority: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace, Debug)]
pub struct PlayerState {
    pub player: Pubkey,
    pub game_state: Pubkey,
    pub coin_count: u64,
    pub termination_count: u64,
    pub bump: u8,
}




// --- Errors ---

#[error_code]
pub enum GameError {
    #[msg("Not enough coins to claim reward.")]
    NotEnoughCoins,
    #[msg("Signer is not the player associated with this state.")]
    NotPlayerAuthority,
}
