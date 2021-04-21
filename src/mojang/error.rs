use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpChoadraError {
    #[error("I/O Error occurred: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("HTTP Error occurred: {0:?}")]
    AttoHttpClientError(#[from] attohttpc::Error),
    #[error("HTTP Error occurred: {0}")]
    ExtractedHttpError(String),
}

pub type HttpChoadraResult<T> = std::result::Result<T, HttpChoadraError>;
