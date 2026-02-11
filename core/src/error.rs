use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("input schema error: {0}")]
    InputSchemaError(String),

    #[error("artifact missing: {0}")]
    ArtifactMissingError(String),

    #[error("policy violation: {0}")]
    PolicyViolationError(String),

    #[error("determinism violation: {0}")]
    DeterminismViolation(String),

    #[error("determinism violation: {0}")]
    DeterminismViolationError(String),

    #[error("citation violation: {0}")]
    CitationViolationError(String),

    #[error("redaction violation: {0}")]
    RedactionViolationError(String),

    #[error("consent missing: {0}")]
    ConsentMissingError(String),

    #[error("workflow transition error: {0}")]
    WorkflowTransitionError(String),

    #[error("policy blocked: {0}")]
    PolicyBlocked(String),

    #[error("evidenceos validation failed: {0}")]
    EvidenceOsValidation(String),

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
