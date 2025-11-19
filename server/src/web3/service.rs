use std::sync::Arc;

use crate::web3::{
    provider::{Contract, ProviderWallet, SolanaProvider},
    instructions,
    solana_game::client::{
        accounts::{InitializeGame, InitializePlayer},
        args,
    },
};
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::{
    prelude::{system_program, Pubkey},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use proto_interface::{bytes, errors::AppError, InlineString, PlayerId, ServerId, SessionId, PDA};
use tokio::{
    sync::{
        mpsc::{Receiver, UnboundedReceiver},
        oneshot,
    },
    time::{Interval, MissedTickBehavior},
};

pub enum W3Commands {
    InitPlayer {
        id: PlayerId,
        name: Option<InlineString>,
        sender: oneshot::Sender<Pubkey>,
    },
    // Interact{
    //     program_key: String,
    //     instructions: I,
    //     args: A,
    //     signer: S,
    // },
    // Read{
    //     program_key: String,
    //     account_pubkey: Pubkey,
    // },
}

pub fn init_game_state_on_chain(
    game_manager_program: &Contract,
    provider_wallet: ProviderWallet,
    server_id: ServerId,
    server_name: InlineString,
) -> Result<Pubkey, AppError> {
    let game_state = PDA!([server_id.to_le_bytes()], game_manager_program.id()).0;

    let inst = InitializeGame {
        game_state,
        game_owner: provider_wallet.pubkey(),
        system_program: system_program::ID,
    };

    let args = args::InitializeGame {
        game_id: server_id,
        game_name: server_name.into(),
    };

    game_manager_program
        .request()
        .accounts(inst)
        .args(args)
        .signer(provider_wallet)
        .send()?;

    Ok(game_state)
}

pub async fn provider_loop(
    provider: Arc<SolanaProvider>,
    mut receiver: UnboundedReceiver<W3Commands>,
    mut transaction_tick: Interval,
) -> Result<(), AppError> {
    transaction_tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            Some(command) = receiver.recv() => {
                if let Err(e) = handle_command(provider.clone(), command).await {
                    eprintln!("Error occured while handling command {e}");
                }

            },

            _ = transaction_tick.tick() => {

            }
        }
    }
    Ok(())
}

pub async fn handle_command(
    provider: Arc<SolanaProvider>,
    command: W3Commands,
) -> Result<(), AppError> {
    match command {
        W3Commands::InitPlayer { id, name, sender } => {
            return instructions::init_player_on_chain(
                provider,
                id,
                name.unwrap_or_default(),
                sender,
            )
        }
    };
}
