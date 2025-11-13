use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair, signer::Signer},
    Client, Cluster, Program,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};
use std::{collections::HashMap, error::Error, hash::Hash, rc::Rc};

pub struct ProgramKey<T: Hash>(T);
pub type Programs = HashMap<String, Program<Rc<Keypair>>>;

pub struct SolanaClient {
    rpc_client: Client<Rc<Keypair>>,
    programs: Programs,
}

impl SolanaClient {
    pub fn new(network: Cluster, payer: Rc<Keypair>) -> Self {
        let client = Client::new_with_options(network, payer, CommitmentConfig::confirmed());

        Self {
            rpc_client: client,
            programs: HashMap::new(),
        }
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

    pub fn get_program(&self, program_key: &str) -> Result<&Program<Rc<Keypair>>, Box<dyn Error>> {
        match self.programs.get(program_key) {
            Some(program) => Ok(program),
            None => Err(format!("Program with key '{}' not found", program_key).into()),
        }
    }

    pub fn interact_with_program<'a, I, A, S>(
        &self,
        program_key: &str,
        instructions: I,
        args: A,
        signer: S,
    ) -> Result<(), Box<dyn Error>>
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
    ) -> Result<T, Box<dyn Error>>
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
