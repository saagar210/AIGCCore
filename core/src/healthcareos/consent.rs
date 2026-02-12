use super::parser::ConsentRecord;
use serde::{Deserialize, Serialize};
use crate::error::{CoreError, CoreResult};

/// Consent validation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsentStatus {
    Valid,       // Consent is current and valid
    Expired,     // Consent exists but older than 2 years
    Missing,     // No consent record found
    Revoked,     // Patient explicitly revoked consent
}

impl ConsentStatus {
    pub fn is_blocking(&self) -> bool {
        matches!(self, ConsentStatus::Missing | ConsentStatus::Revoked)
    }

    pub fn is_valid_or_expired(&self) -> bool {
        matches!(self, ConsentStatus::Valid | ConsentStatus::Expired)
    }
}

/// Validate consent for clinical processing
pub fn validate_consent(consent: &Option<ConsentRecord>, patient_id: &str) -> CoreResult<ConsentStatus> {
    // Missing consent is blocking
    let consent = match consent {
        Some(c) => c,
        None => return Ok(ConsentStatus::Missing),
    };

    // Patient ID mismatch
    if consent.patient_id != patient_id {
        return Err(CoreError::InvalidInput(format!(
            "Consent patient_id {} does not match transcript patient_id {}",
            consent.patient_id, patient_id
        )));
    }

    // Check for revoked status
    if consent.status == "REVOKED" {
        return Ok(ConsentStatus::Revoked);
    }

    // Check if expired (simplified: just check dates)
    if consent.date_expires < "2026-02-12".to_string() {
        // Note: In real system, use current date
        return Ok(ConsentStatus::Expired);
    }

    Ok(ConsentStatus::Valid)
}

/// Enforce consent blocking behavior
pub fn enforce_consent_block(status: &ConsentStatus) -> CoreResult<()> {
    match status {
        ConsentStatus::Missing => Err(CoreError::InvalidInput(
            "Cannot process transcript: No consent record found".to_string(),
        )),
        ConsentStatus::Revoked => Err(CoreError::InvalidInput(
            "Cannot process transcript: Patient consent has been revoked".to_string(),
        )),
        ConsentStatus::Valid | ConsentStatus::Expired => Ok(()),
    }
}

/// Get warning message for expired consent
pub fn get_consent_warning(status: &ConsentStatus) -> Option<String> {
    match status {
        ConsentStatus::Expired => Some(
            "WARNING: Consent record has expired (>2 years). Patient should renew consent.".to_string(),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_consent(status: &str, expires: &str) -> ConsentRecord {
        ConsentRecord {
            consent_id: "CONSENT_001".to_string(),
            patient_id: "PT-2026-001".to_string(),
            date_given: "2024-02-12".to_string(),
            date_expires: expires.to_string(),
            scope: "general".to_string(),
            status: status.to_string(),
        }
    }

    #[test]
    fn test_valid_consent() {
        let consent = sample_consent("VALID", "2026-12-12"); // Future expiry
        let status = validate_consent(&Some(consent), "PT-2026-001").unwrap();
        assert_eq!(status, ConsentStatus::Valid);
    }

    #[test]
    fn test_expired_consent() {
        let consent = sample_consent("VALID", "2024-01-01"); // Past expiry
        let status = validate_consent(&Some(consent), "PT-2026-001").unwrap();
        assert_eq!(status, ConsentStatus::Expired);
    }

    #[test]
    fn test_revoked_consent() {
        let consent = sample_consent("REVOKED", "2026-12-12");
        let status = validate_consent(&Some(consent), "PT-2026-001").unwrap();
        assert_eq!(status, ConsentStatus::Revoked);
    }

    #[test]
    fn test_missing_consent() {
        let status = validate_consent(&None, "PT-2026-001").unwrap();
        assert_eq!(status, ConsentStatus::Missing);
    }

    #[test]
    fn test_blocking_statuses() {
        assert!(ConsentStatus::Missing.is_blocking());
        assert!(ConsentStatus::Revoked.is_blocking());
        assert!(!ConsentStatus::Valid.is_blocking());
        assert!(!ConsentStatus::Expired.is_blocking());
    }

    #[test]
    fn test_enforce_missing_consent() {
        let result = enforce_consent_block(&ConsentStatus::Missing);
        assert!(result.is_err());
    }

    #[test]
    fn test_enforce_revoked_consent() {
        let result = enforce_consent_block(&ConsentStatus::Revoked);
        assert!(result.is_err());
    }

    #[test]
    fn test_enforce_valid_consent() {
        let result = enforce_consent_block(&ConsentStatus::Valid);
        assert!(result.is_ok());
    }

    #[test]
    fn test_consent_warning() {
        let warning = get_consent_warning(&ConsentStatus::Expired);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("expired"));

        let no_warning = get_consent_warning(&ConsentStatus::Valid);
        assert!(no_warning.is_none());
    }

    #[test]
    fn test_patient_id_mismatch() {
        let consent = sample_consent("VALID", "2026-12-12");
        let result = validate_consent(&Some(consent), "PT-DIFFERENT");
        assert!(result.is_err());
    }
}
