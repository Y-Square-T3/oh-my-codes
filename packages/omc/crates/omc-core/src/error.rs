use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmcError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, OmcError>;
