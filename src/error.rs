use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChoadraError {
    #[error("I/O Error occurred: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("BinRead Error occurred: {0:?}")]
    BinreadError(#[from] binread::Error),
    #[error("RSA Error occurred: {0:?}")]
    RSAError(#[from] rsa::errors::Error),
    #[error("HTTP Error occurred: {0:?}")]
    HttpError(#[from] crate::mojang::error::HttpChoadraError),
    #[error("Server gave invalid info: {msg}")]
    ServerError { msg: String },
    #[error("Invalid state: {msg}")]
    InvalidState { msg: String },
}

pub type ChoadraResult<T> = std::result::Result<T, ChoadraError>;
