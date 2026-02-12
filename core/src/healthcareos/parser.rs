use serde::{Deserialize, Serialize};
use crate::error::{CoreError, CoreResult};
use sha2::{Digest, Sha256};

/// Clinical transcript from speech-to-text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalTranscript {
    pub transcript_id: String,
    pub patient_id: String,
    pub date: String,
    pub provider: String,
    pub specialty: String,
    pub content: String,
    pub confidence: f32,
}

/// Patient consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub consent_id: String,
    pub patient_id: String,
    pub date_given: String,
    pub date_expires: String,
    pub scope: String,           // "general", "research", "limited"
    pub status: String,          // "VALID", "EXPIRED", "REVOKED"
}

/// Parse JSON clinical transcript
pub fn parse_transcript(json_str: &str) -> CoreResult<ClinicalTranscript> {
    let raw: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| CoreError::InvalidInput(format!("Failed to parse transcript: {}", e)))?;

    let patient_id = raw
        .get("patient_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing patient_id".to_string()))?
        .to_string();

    let date = raw
        .get("date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing date".to_string()))?
        .to_string();

    let provider = raw
        .get("provider")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing provider".to_string()))?
        .to_string();

    let specialty = raw
        .get("specialty")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing specialty".to_string()))?
        .to_string();

    let content = raw
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing content".to_string()))?
        .to_string();

    let confidence: f32 = raw
        .get("confidence")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| CoreError::InvalidInput("Missing/invalid confidence".to_string()))?
        as f32;

    // Generate deterministic transcript ID
    let transcript_id = generate_transcript_id(&patient_id, &date, &content);

    if content.is_empty() {
        return Err(CoreError::InvalidInput("Transcript content cannot be empty".to_string()));
    }

    Ok(ClinicalTranscript {
        transcript_id,
        patient_id,
        date,
        provider,
        specialty,
        content,
        confidence,
    })
}

/// Parse JSON consent record
pub fn parse_consent(json_str: &str) -> CoreResult<ConsentRecord> {
    let raw: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| CoreError::InvalidInput(format!("Failed to parse consent: {}", e)))?;

    let patient_id = raw
        .get("patient_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing patient_id".to_string()))?
        .to_string();

    let date_given = raw
        .get("date_given")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing date_given".to_string()))?
        .to_string();

    let scope = raw
        .get("scope")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing scope".to_string()))?
        .to_string();

    let status = raw
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("VALID")
        .to_string();

    // Calculate expiry (2 years from date_given)
    let date_expires = add_years(&date_given, 2);

    let consent_id = generate_consent_id(&patient_id, &date_given);

    Ok(ConsentRecord {
        consent_id,
        patient_id,
        date_given,
        date_expires,
        scope,
        status,
    })
}

/// Generate deterministic transcript ID
fn generate_transcript_id(patient_id: &str, date: &str, content: &str) -> String {
    let combined = format!("{}{}{}", patient_id, date, content);
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(&hash_bytes[0..8]);

    format!("CLINICAL_{}", hash_hex)
}

/// Generate deterministic consent ID
fn generate_consent_id(patient_id: &str, date: &str) -> String {
    let combined = format!("CONSENT_{}{}", patient_id, date);
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(&hash_bytes[0..8]);

    format!("CONSENT_{}", hash_hex)
}

/// Add years to a date (simplified)
fn add_years(date: &str, years: u32) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        if let Ok(year) = parts[0].parse::<u32>() {
            return format!("{}-{}-{}", year + years, parts[1], parts[2]);
        }
    }
    date.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_transcript() -> &'static str {
        r#"{
            "patient_id": "PT-2026-001",
            "date": "2026-02-12",
            "provider": "Dr. Smith",
            "specialty": "Cardiology",
            "content": "Patient presents with chest pain. EKG normal. Recommend stress test.",
            "confidence": 0.95
        }"#
    }

    fn sample_consent() -> &'static str {
        r#"{
            "patient_id": "PT-2026-001",
            "date_given": "2024-02-12",
            "scope": "general",
            "status": "VALID"
        }"#
    }

    #[test]
    fn test_parse_transcript() {
        let tx = parse_transcript(sample_transcript()).unwrap();
        assert_eq!(tx.patient_id, "PT-2026-001");
        assert_eq!(tx.provider, "Dr. Smith");
        assert!(tx.confidence > 0.9);
    }

    #[test]
    fn test_parse_consent() {
        let consent = parse_consent(sample_consent()).unwrap();
        assert_eq!(consent.patient_id, "PT-2026-001");
        assert_eq!(consent.scope, "general");
    }

    #[test]
    fn test_transcript_id_determinism() {
        let tx1 = parse_transcript(sample_transcript()).unwrap();
        let tx2 = parse_transcript(sample_transcript()).unwrap();

        assert_eq!(tx1.transcript_id, tx2.transcript_id);
    }

    #[test]
    fn test_consent_expiry_calculation() {
        let consent = parse_consent(sample_consent()).unwrap();
        // Consent from 2024-02-12 should expire 2026-02-12
        assert_eq!(consent.date_expires, "2026-02-12");
    }

    #[test]
    fn test_invalid_transcript() {
        let invalid = r#"{ invalid json }"#;
        let result = parse_transcript(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_consent_status() {
        let minimal = r#"{
            "patient_id": "PT-001",
            "date_given": "2024-01-01",
            "scope": "general"
        }"#;
        let consent = parse_consent(minimal).unwrap();
        assert_eq!(consent.status, "VALID"); // Default
    }
}
