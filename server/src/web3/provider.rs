use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{Keypair, Signature},
        signer::{EncodableKey, Signer},
    },
    Client, Cluster, Program,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};
use dashmap::{mapref::one::Ref, DashMap};
use proto_interface::{errors::AppError, web3::InlineString, PlayerId, ServerId, PDA};
use std::{
    collections::HashMap,
    error::Error,
    hash::Hash,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
use tokio::{sync::mpsc::UnboundedReceiver, time::Interval};

use crate::web3::{
    service::{self, W3Commands},
    solana_game,
};

pub type ProviderWallet = Arc<Keypair>;
pub type Contract = Program<ProviderWallet>;
pub type Programs = DashMap<String, Arc<Contract>>;
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
    pub async fn start_provider<P>(
        network: Cluster,
        wallet_path: P,
        receiver: UnboundedReceiver<W3Commands>,
        server_id: ServerId,
        server_name: InlineString,
        transaction_tick: Interval,
    ) -> Result<(), AppError>
    where
        P: AsRef<Path>,
    {
        let keypair = Keypair::read_from_file(wallet_path.as_ref())
            .map_err(|e| AppError::Custom(format!("Unable to parse Keypair from file: {e}")))?;

        let provider_wallet = Arc::new(keypair);

        let client = Client::new_with_options(
            network,
            provider_wallet.clone(),
            CommitmentConfig::confirmed(),
        );

        let game_manager_program = Arc::new(client.program(solana_game::ID)?);

        let programs = Programs::from_iter(vec![(
            "game_manager".to_string(),
            game_manager_program.clone(),
        )]);

        let state_address = service::init_game_state_on_chain(
            game_manager_program.clone(),
            provider_wallet.clone(),
            server_id,
            server_name,
        )
        .await?;

        println!(
            "State created at derived address: {:?}",
            state_address.to_string()
        );

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
        self.programs.insert(program_key, Arc::new(program_inner));
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

    pub fn get_program(
        &self,
        program_key: &str,
    ) -> Result<Ref<'_, String, Arc<Contract>>, AppError> {
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

    pub async fn interact_with_program<I, A, S>(
        &self,
        program_key: &str,
        instructions: I,
        args: A,
        signer: S,
    ) -> Result<(), AppError>
    where
        A: InstructionData + Send + 'static,
        I: ToAccountMetas + Send + 'static,
        S: Signer + Send + 'static,
    {
        let program = self.get_program(program_key)?.value().clone();
        send_transaction(program, instructions, args, signer).await?;
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
        Ok(PDA!(seeds.as_slice(), program_id))
    }
}

pub async fn send_transaction<'a, I, A, S>(
    program: Arc<Contract>,
    instructions: I,
    args: A,
    signer: S,
) -> Result<Signature, AppError>
where
    A: InstructionData + Send + 'static,
    I: ToAccountMetas + Send + 'static,
    S: Signer + Send + 'static,
{
    let sig = tokio::task::spawn_blocking(move || {
        program
            .request()
            .accounts(instructions)
            .args(args)
            .signer(signer)
            .send()
    })
    .await
    .map_err(|e| AppError::Custom(format!("Tokio Join Error occured: {e}")))??;

    Ok(sig)
}
