use crate::error::{CoreError, CoreResult};

pub fn validate_consent_present(consent_artifact_count: usize) -> CoreResult<()> {
    if consent_artifact_count == 0 {
        return Err(CoreError::ConsentMissingError(
            "at least one consent artifact is required".to_string(),
        ));
    }
    Ok(())
}
