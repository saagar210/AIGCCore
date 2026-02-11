use crate::error::{CoreError, CoreResult};

pub fn validate_retention_profile(profile: &str) -> CoreResult<()> {
    if profile.trim().is_empty() {
        return Err(CoreError::PolicyViolationError(
            "retention profile is required".to_string(),
        ));
    }
    Ok(())
}
