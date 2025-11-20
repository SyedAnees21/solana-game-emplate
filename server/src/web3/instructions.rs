use std::sync::Arc;

use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::prelude::{system_program, Pubkey};
use proto_interface::{bytes, errors::AppError, web3::InlineString, PlayerId, ServerId, PDA};
use tokio::sync::oneshot;

use crate::web3::{
    provider::SolanaProvider,
    solana_game::client::{accounts::InitializePlayer, args},
};

pub async fn init_player_on_chain(
    provider: Arc<SolanaProvider>,
    player_id: PlayerId,
    player_name: InlineString,
    sender: oneshot::Sender<Pubkey>,
) -> Result<(), AppError> {
    let program = provider.get_program("game_manager")?;

    let player_wallet = Keypair::new();
    let player_public_key = player_wallet.pubkey();
    let player_state = PDA!([bytes!("player", String), bytes!(player_id)], program.id()).0;

    let provider_wallet = provider.provider_wallet();

    let inst = InitializePlayer {
        player_state,
        game_state: provider.state_address(),
        game_owner: provider_wallet.pubkey(),
        system_program: system_program::ID,
    };

    let args = args::InitializePlayer {
        _game_id: provider.server_id(),
        player_id,
        player_name: player_name.into(),
    };

    provider
        .interact_with_program("game_manager", inst, args, provider_wallet)
        .await?;
    
    provider.track_player(player_id, player_state, player_wallet);

    if let Err(_) = sender.send(player_public_key) {
        return Err(AppError::Custom(format!(
            "Receiver dropped from session {player_id}"
        )));
    };

    Ok(())
}
