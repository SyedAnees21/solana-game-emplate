use std::sync::{PoisonError, mpsc::SendError};

use anchor_client::ClientError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Prost encode error: {0}")]
    EncodeError(#[from] prost::EncodeError),

    #[error("Prost decode error: {0}")]
    DecodeError(#[from] prost::DecodeError),

    #[error("System IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Handshake failed due to: {0}")]
    HandshakeFailed(String),

    #[error("Remote connection error: {0}")]
    ConnectionError(&'static str),

    #[error("Error in attributes map: {0}")]
    AttributesError(&'static str),

    #[error("lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("Unable to parse keypair from file {0}")]
    KeyPairParse(String),

    #[error("Error in solana client: {0}")]
    SolanaError(#[from] ClientError),

    #[error("Error occured: {0}")]
    Custom(String),
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(value: PoisonError<T>) -> Self {
       AppError::LockPoisoned(value.to_string()) 
    }
}

impl<T> From<SendError<T>> for AppError {
    fn from(value: SendError<T>) -> Self {
       AppError::Custom(value.to_string()) 
    }
}