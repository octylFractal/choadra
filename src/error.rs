use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChoadraError {
    #[error("I/O Error occurred: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("BinRead Error occurred: {0:?}")]
    BinreadError(#[from] binread::Error),
    #[error("x.509 Error occurred: {0:?}")]
    X509Error(#[from] x509_parser::error::X509Error),
    #[error("RSA Error occurred: {0:?}")]
    RSAError(#[from] rsa::errors::Error),
    #[error("Server gave invalid info: {msg}")]
    ServerError { msg: String },
}

pub type ChoadraResult<T> = std::result::Result<T, ChoadraError>;
