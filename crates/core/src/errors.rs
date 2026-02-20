use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum EngineError {
    // future: computation errors
}

#[derive(Error, Debug)]
pub enum CoreError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Engine(#[from] EngineError),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
pub type EngineResult<T> = std::result::Result<T, EngineError>;
pub type CoreResult<T> = std::result::Result<T, CoreError>;
