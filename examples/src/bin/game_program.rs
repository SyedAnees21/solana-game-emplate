use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        native_token::LAMPORTS_PER_SOL,
        signature::{Keypair, Signature},
        signer::{EncodableKey, Signer},
    },
    Client, Cluster, Program,
};
use anchor_lang::{declare_program, prelude::system_program, InstructionData, ToAccountMetas};
use anyhow;
use proto_interface::{bytes, EntityId, InlineString, PlayerId, PDA};
use rand::rngs::OsRng;
use rand::RngCore;
use std::{ops::Deref, rc::Rc, thread::sleep, time::Duration};

use crate::solana_game::{
    client::{
        accounts::{InitializeEntity, InitializeGame, InitializePlayer},
        args,
    },
    types::{self},
};

declare_program!(solana_game);

fn main() -> anyhow::Result<()> {
    let mut rng = OsRng;

    let program_id = solana_game::ID;
    println!("Using program id: {}", program_id);

    let provider_keypair = Keypair::read_from_file("./assets/provider_wallet.json").unwrap();
    let payer = Rc::new(provider_keypair);
    let player = Rc::new(Keypair::new());

    let game_id = rng.next_u64();
    let player_id: PlayerId = rng.next_u64();
    let entity_id: EntityId = rng.next_u32();

    let client = Client::new_with_options(
        Cluster::Localnet,
        payer.clone(),
        CommitmentConfig::confirmed(),
    );

    let program = client.program(program_id)?;

    program
        .rpc()
        .request_airdrop(&player.pubkey(), 5 * LAMPORTS_PER_SOL)?;

    sleep(Duration::from_millis(500));

    let entity_pda = PDA!(
        ["entity".as_bytes(), bytes!(player_id), bytes!(entity_id)],
        program_id
    )
    .0;

    let player_pda = PDA!(["player".as_bytes(), bytes!(player_id)], program_id).0;
    let game_state_pda = PDA!(["game".as_bytes(), bytes!(game_id)], program_id).0;

    let inst = InitializeGame {
        game_state: game_state_pda,
        game_owner: payer.pubkey(),
        system_program: system_program::ID,
    };

    let args = args::InitializeGame {
        game_id,
        game_name: InlineString::from("TPP Demo").into(),
    };

    send_transaction(&program, inst, args, &payer)?;

    let inst = InitializePlayer {
        player_state: player_pda,
        game_state: game_state_pda,
        game_owner: payer.pubkey(),
        system_program: system_program::ID,
    };

    let args = args::InitializePlayer {
        _game_id: game_id,
        player_id,
        player_name: InlineString::from("Gatsby").into(),
    };

    send_transaction(&program, inst, args, &payer)?;

    // let inst = InitializeEntity {
    //     entity: entity_pda,
    //     player: player.pubkey(),
    //     system_program: system_program::ID,
    // };

    // let args = solana_game::client::args::InitializeEntity {
    //     _player_id: player_id,
    //     entity_id: entity_id,
    // };

    // send_transaction(&program, inst, args, &player)?;
    let entity_state = program.account::<solana_game::accounts::GameState>(game_state_pda);

    println!("{:?}", entity_state);

    Ok(())
}

fn send_transaction<C, I, A, S>(
    program: &Program<C>,
    instructions: I,
    arguments: A,
    signer: &S,
) -> anyhow::Result<Signature>
where
    I: ToAccountMetas,
    A: InstructionData,
    S: Signer,
    C: Deref<Target = S> + Clone,
{
    let sig = program
        .request()
        .accounts(instructions)
        .args(arguments)
        .signer(signer)
        .send()?;

    Ok(sig)
}

impl From<InlineString> for types::InlineString {
    fn from(value: InlineString) -> Self {
        Self {
            len: value.get_len() as u8,
            buffer: value.as_ref().to_owned(),
        }
    }
}
