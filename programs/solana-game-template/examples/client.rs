// examples/test_client.rs

use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_client::{Client, Cluster, Program};
use anchor_client::solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    signature::Signature,
};
use anchor_lang::{Key, declare_program, system_program};
use solana_game_template::PlayerState;
use std::thread::sleep;
use std::{error::Error, rc::Rc, str::FromStr, time::Duration};

// Replace this string with your program id (must match the program you're testing)
const PROGRAM_ID_STR: &str = "3UYVFTzKfxT842KUoWrqLsZf1PJ1anCVU1rCvZcFwMSx";

// Generate a new random game id each run to avoid PDA collisions with previous runs.
use rand::rngs::OsRng;
use rand::RngCore;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse program id at runtime
    let program_id = Pubkey::from_str(PROGRAM_ID_STR)?;
    println!("Using program id: {}", program_id);

    // Create a local payer (wrap in Rc because Client expects Rc<Signer>)
    let payer = Rc::new(Keypair::new());
    
    // Create Anchor client pointed at localnet and with confirmed commitment
    let client = Client::new_with_options(Cluster::Localnet, payer.clone(), CommitmentConfig::confirmed());
    
    // Get program handle
    let program: Program<Rc<Keypair>> = client.program(program_id)?;
    
    // Create authority keypair for admin operations and also use it as player for local testing
    let game_authority = Keypair::new();
    let player_pubkey = Pubkey::from_str("HL2wxPQCv729nBtU7tQbfYP8uzYyauRAxde5QXMXL1z4")?;

    // Generate a random game id for this run to avoid PDA collisions with prior runs
    let mut rng = OsRng;
    let game_id: u64 = rng.next_u64();
    println!("Using GAME_ID: {}", game_id);
    
    // Airdrop to authority for admin operations
    println!("Airdropping SOL to authority...");
    airdrop(&program, &payer.pubkey(), 5 * LAMPORTS_PER_SOL)?;
    airdrop(&program, &game_authority.pubkey(), 5 * LAMPORTS_PER_SOL)?;
    println!("Airdrops complete.");

    // Derive PDAs for game state, treasury, and player
    let game_id_bytes = game_id.to_le_bytes();
    let (game_state_pda, _game_bump) =
        Pubkey::find_program_address(&[b"game", game_id_bytes.as_ref()], &program_id);
    let (treasury_pda, _treasury_bump) =
        Pubkey::find_program_address(&[b"treasury", game_id_bytes.as_ref()], &program_id);
    let (player_state_pda, _player_bump) = Pubkey::find_program_address(
        &[b"player", game_id_bytes.as_ref(), player_pubkey.as_ref()],
        &program_id
    );

    // Initialize Game
    println!("Testing: initialize_game...");
    program
        .request()
        .accounts(solana_game_template::accounts::InitializeGame {
            game_state: game_state_pda,
            treasury: treasury_pda,
            authority: game_authority.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_game_template::instruction::InitializeGame {
            game_id: game_id,
            player_amount: 0,
        })
        .signer(&game_authority)
        .send()?;
    println!("Game initialized.");

    // Fund Treasury
    println!("Testing: fund_treasury...");
    let fund_amount = 3 * LAMPORTS_PER_SOL;
    program
        .request()
        .accounts(solana_game_template::accounts::FundTreasury {
            treasury: treasury_pda,
            game_state: game_state_pda,
            authority: game_authority.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_game_template::instruction::FundTreasury { amount: fund_amount })
        .signer(&game_authority)
        .send()?;

    // Verify treasury balance
    let treasury_balance = program.rpc().get_balance(&treasury_pda)?;
    let rent_exempt = program.rpc().get_minimum_balance_for_rent_exemption(0)?;
    println!("Treasury funded with {} lamports (includes {} lamports rent-exempt).", treasury_balance, rent_exempt);
    assert_eq!(treasury_balance, fund_amount + rent_exempt, "Treasury balance is not the funded amount plus rent-exempt reserve");
    println!("Treasury funded with 3 SOL.");

    // Initialize Player Account
    println!("\nInitializing player account...");
    program
        .request()
        .accounts(solana_game_template::accounts::InitializePlayer {
            player_state: player_state_pda,
            game_state: game_state_pda,
            authority: game_authority.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_game_template::instruction::InitializePlayer {player: player_pubkey})
        .signer(&game_authority)
        .send()?;
    println!("Player account initialized!");

    // Add coins to player
    println!("\nAdding coins to player...");
    program
        .request()
        .accounts(solana_game_template::accounts::AddCoins {
            player_state: player_state_pda,
            game_state: game_state_pda,
            authority: game_authority.pubkey(),
        })
        .args(solana_game_template::instruction::AddCoins { amount: 10 })
        .signer(&game_authority)
        .send()?;
    println!("Added 10 coins to player!");

    // Print final account info
    println!("\nSetup Complete! Account Information:");
    println!("Authority: {}", game_authority.pubkey());
    println!("Player Pubkey: {}", player_pubkey);
    println!("Game State PDA: {}", game_state_pda);
    println!("Treasury PDA: {}", treasury_pda);
    println!("Player State PDA: {}", player_state_pda);

    println!("\n✅ Admin setup complete.");
    println!("Game ID: {} (share this with players)", game_id);
    println!("Game State PDA: {}", game_state_pda);
    println!("Treasury PDA: {}", treasury_pda);
    println!("Player State PDA: {}", player_state_pda);

    println!("\n✅ All checks done.");

    println!("player state: {:?}",  program.account::<PlayerState>(player_state_pda)?); // Just to silence unused variable warnings
    Ok(())
}

// Helper: request airdrop and wait for confirmation
fn airdrop(program: &Program<Rc<Keypair>>, to: &Pubkey, amount: u64) -> Result<(), Box<dyn Error>> {
    // request airdrop
    let sig: Signature = program.rpc().request_airdrop(to, amount)?;
    // poll for confirmation (simple loop)
    for _ in 0..30 {
        if let Ok(statuses) = program.rpc().get_signature_statuses(&[sig]) {
            if let Some(Some(_st)) = statuses.value.get(0) {
                // confirmed (or at least observed)
                return Ok(());
            }
        }
        sleep(Duration::from_millis(500));
    }
    Err("airdrop confirmation timed out".into())
}