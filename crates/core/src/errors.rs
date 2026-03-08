use crate::models::Asset;
use thiserror::Error;

/// CSV parsing failures (malformed rows, IO, unrecognized format).
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid row at line {line}: {message}")]
    InvalidRow { line: usize, message: String },

    #[error("Unrecognized CSV format. Headers: {0}")]
    UnrecognizedFormat(String),

    #[error("{0} import is not yet supported")]
    NotYetSupported(String),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sql(#[from] rusqlite::Error),
}

/// Price fetching: bundled CSV loading + Kraken OHLC API.
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

/// Currency type safety guard — raised by `AssetAmount::checked_add/sub` when
/// operands have different assets (e.g. adding EUR to BTC).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("asset mismatch: expected {expected}, got {got}")]
pub struct AssetMismatch {
    pub expected: Asset,
    pub got: Asset,
}

/// Position tracking and BEP/P&L calculation errors.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    AssetMismatch(#[from] AssetMismatch),
}

/// Top-level error unifying all modules. Returned by public `api::` functions.
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

    #[error("This file has already been imported (SHA-256 match)")]
    DuplicateFile,

    #[error("All {0} trades in this file already exist in the database")]
    AllTradesDuplicate(usize),
}

pub type ParseResult<T> = Result<T, ParseError>;
pub type DbResult<T> = Result<T, DbError>;
pub type PriceResult<T> = Result<T, PriceError>;
pub type EngineResult<T> = Result<T, EngineError>;
pub type CoreResult<T> = Result<T, CoreError>;
