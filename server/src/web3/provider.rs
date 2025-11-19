use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::Keypair,
        signer::{EncodableKey, Signer},
    },
    Client, Cluster, Program,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};
use dashmap::{mapref::one::Ref, DashMap};
use proto_interface::{errors::AppError, InlineString, PlayerId, ServerId};
use std::{collections::HashMap, error::Error, hash::Hash, path::PathBuf, rc::Rc, sync::Arc};
use tokio::{sync::mpsc::UnboundedReceiver, time::Interval};

use crate::web3::{
    service::{self, W3Commands},
    solana_game,
};

pub type ProviderWallet = Arc<Keypair>;
pub type Contract = Program<ProviderWallet>;
pub type Programs = DashMap<String, Contract>;
pub type PlayerWallets = DashMap<PlayerId, Keypair>;
pub type DerivedAddresses = DashMap<PlayerId, Pubkey>;

pub struct SolanaProvider {
    server_id: ServerId,
    wallet: ProviderWallet,
    state_address: Pubkey,
    rpc_client: Client<ProviderWallet>,
    programs: Programs,
    player_wallets: PlayerWallets,
    derived_addresses: DerivedAddresses,
}

impl SolanaProvider {
    pub fn new<P>(
        network: Cluster,
        wallet_path: P,
        receiver: UnboundedReceiver<W3Commands>,
        server_id: ServerId,
        server_name: InlineString,
        transaction_tick: Interval,
    ) -> Result<(), AppError>
    where
        P: AsRef<PathBuf>,
    {
        let keypair = Keypair::read_from_file(wallet_path.as_ref())
            .map_err(|e| AppError::Custom(format!("Unable to parse Keypair from file: {e}")))?;

        let provider_wallet = Arc::new(keypair);

        let client = Client::new_with_options(
            network,
            provider_wallet.clone(),
            CommitmentConfig::confirmed(),
        );

        let programs = Programs::from_iter(vec![(
            "game_manager".to_string(),
            client.program(solana_game::ID)?,
        )]);

        let state_address = service::init_game_state_on_chain(
            programs.get("game_manager").unwrap().value(),
            provider_wallet.clone(),
            server_id,
            server_name,
        )?;

        let provider = Arc::new(Self {
            server_id,
            state_address,
            programs,
            wallet: provider_wallet,
            rpc_client: client,
            player_wallets: PlayerWallets::default(),
            derived_addresses: DerivedAddresses::default(),
        });

        tokio::spawn(service::provider_loop(provider, receiver, transaction_tick));

        Ok(())
    }

    pub fn new_program(
        &mut self,
        program_id: Pubkey,
        program_key: String,
    ) -> Result<(), Box<dyn Error>> {
        let program_inner = self.rpc_client.program(program_id)?;
        self.programs.insert(program_key, program_inner);
        Ok(())
    }

    pub fn provider_wallet(&self) -> ProviderWallet {
        self.wallet.clone()
    }

    pub fn programs(&self) -> &Programs {
        &self.programs
    }

    pub fn state_address(&self) -> Pubkey {
        self.state_address
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn get_program(&self, program_key: &str) -> Result<Ref<'_, String, Contract>, AppError> {
        match self.programs.get(program_key) {
            Some(program) => Ok(program),
            None => Err(AppError::Custom(format!(
                "Program with key '{}' not found",
                program_key
            ))),
        }
    }

    pub fn track_player(&self, id: PlayerId, custodial_state: Pubkey, custodial_wallet: Keypair) {
        self.player_wallets.insert(id, custodial_wallet);
        self.derived_addresses.insert(id, custodial_state);
    }

    pub fn interact_with_program<'a, I, A, S>(
        &self,
        program_key: &str,
        instructions: I,
        args: A,
        signer: S,
    ) -> Result<(), AppError>
    where
        A: InstructionData,
        I: ToAccountMetas,
        S: Signer + 'a,
    {
        let program = self.get_program(program_key)?;

        program
            .request()
            .accounts(instructions)
            .args(args)
            .signer(signer)
            .send()?;
        Ok(())
    }

    pub fn read_account_data<T>(
        &self,
        program_key: &str,
        account_pubkey: &Pubkey,
    ) -> Result<T, AppError>
    where
        T: AccountDeserialize,
    {
        let program = self.get_program(program_key)?;
        let account_data = program.account::<T>(*account_pubkey)?;
        Ok(account_data)
    }

    pub fn get_pda<T>(&self, seeds: Vec<T>, program_key: String) -> Result<Pubkey, Box<dyn Error>>
    where
        T: AsRef<[u8]>,
    {
        Ok(self.get_pda_and_bump(seeds, program_key)?.0)
    }

    pub fn get_pda_and_bump<T>(
        &self,
        seeds: Vec<T>,
        program_key: String,
    ) -> Result<(Pubkey, u8), Box<dyn Error>>
    where
        T: AsRef<[u8]>,
    {
        let program_id = self.get_program(&program_key)?.id();
        let seeds = seeds.iter().map(|s| s.as_ref()).collect::<Vec<&[u8]>>();
        Ok(crate::PDA!(program_id => seeds.as_slice()))
    }
}

#[macro_export]
macro_rules! PDA {
    ( $($seed:expr),* $(,)? ) => {{
        let __pda_seeds: &[&[u8]] = &[$($seed),*];
        anchor_client::solana_sdk::pubkey::Pubkey::find_program_address(__pda_seeds, &crate::PROGRAM_ID)
    }};
    ( $programID:expr => $seeds:expr ) => {{
        anchor_client::solana_sdk::pubkey::Pubkey::find_program_address($seeds, &$programID)
    }};
}
