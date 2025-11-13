use std::sync::PoisonError;

use thiserror::Error;
use tokio::sync::mpsc::error::SendError;


#[derive(Debug, Error)]
pub enum Error {
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

    #[error("Error occured: {0}")]
    Custom(String),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
       Error::LockPoisoned(value.to_string()) 
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(value: SendError<T>) -> Self {
       Error::Custom(value.to_string()) 
    }
}
