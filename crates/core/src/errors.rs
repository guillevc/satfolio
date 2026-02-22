use crate::models::Asset;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid row at line {line}: {message}")]
    InvalidRow { line: usize, message: String },
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sql(#[from] rusqlite::Error),
}

#[derive(Debug, Error)]
pub enum PriceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Unsupported currency: {0}")]
    UnsupportedCurrency(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("asset mismatch: expected {expected}, got {got}")]
pub struct AssetMismatch {
    pub expected: Asset,
    pub got: Asset,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    AssetMismatch(#[from] AssetMismatch),
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Db(#[from] DbError),

    #[error(transparent)]
    Price(#[from] PriceError),

    #[error(transparent)]
    Engine(#[from] EngineError),
}

pub type ParseResult<T> = Result<T, ParseError>;
pub type DbResult<T> = Result<T, DbError>;
pub type PriceResult<T> = Result<T, PriceError>;
pub type EngineResult<T> = Result<T, EngineError>;
pub type CoreResult<T> = Result<T, CoreError>;
