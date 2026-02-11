use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("determinism violation: {0}")]
    DeterminismViolation(String),

    #[error("policy blocked: {0}")]
    PolicyBlocked(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("zip error: {0}")]
    Zip(String),

    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
}

pub type CoreResult<T> = Result<T, CoreError>;
