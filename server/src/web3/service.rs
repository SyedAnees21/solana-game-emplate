use crate::web3::client::SolanaClient;
use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};

pub enum W3Commands
{
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

pub struct Web3Service {
    client: SolanaClient,
    // Implementation details...
}

impl Web3Service 
{
    pub fn new(client: SolanaClient) -> Self {
        Self { client }
    }

    pub fn start(&self) 
    {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<W3Commands>(256);

        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                match command {
                    // Handle different W3Commands...
                }
            }
        });
        // Implementation to start the web3 service...
    } 
    // Additional methods to interact with the SolanaClient...
}