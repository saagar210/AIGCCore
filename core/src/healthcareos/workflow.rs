use super::consent::{validate_consent, enforce_consent_block, get_consent_warning};
use super::model::HealthcareOsInputV1;
use super::parser::{parse_transcript, parse_consent};
use super::render::{render_draft_note, render_verification_checklist, render_uncertainty_map};
use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthcareWorkflowStage {
    Ingested,
    Analyzed,
    Reviewed,
    Renderable,
    ExportReady,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareWorkflowState {
    pub stage: HealthcareWorkflowStage,
    pub input: HealthcareOsInputV1,
}

impl HealthcareWorkflowState {
    pub fn ingest(input: HealthcareOsInputV1) -> CoreResult<Self> {
        if input.schema_version != "HEALTHCAREOS_INPUT_V1" {
            return Err(CoreError::InputSchemaError(format!(
                "expected HEALTHCAREOS_INPUT_V1, got {}",
                input.schema_version
            )));
        }
        if input.consent_artifacts.is_empty() {
            return Err(CoreError::ArtifactMissingError(
                "at least one consent artifact is required".to_string(),
            ));
        }
        if input.transcript_artifacts.is_empty() {
            return Err(CoreError::ArtifactMissingError(
                "at least one transcript artifact is required".to_string(),
            ));
        }
        Ok(Self {
            stage: HealthcareWorkflowStage::Ingested,
            input,
        })
    }

    pub fn transition(self, next: HealthcareWorkflowStage) -> CoreResult<Self> {
        let allowed = matches!(
            (self.stage, next),
            (
                HealthcareWorkflowStage::Ingested,
                HealthcareWorkflowStage::Analyzed
            ) | (
                HealthcareWorkflowStage::Analyzed,
                HealthcareWorkflowStage::Reviewed
            ) | (
                HealthcareWorkflowStage::Reviewed,
                HealthcareWorkflowStage::Renderable
            ) | (
                HealthcareWorkflowStage::Renderable,
                HealthcareWorkflowStage::ExportReady
            )
        );
        if !allowed {
            return Err(CoreError::WorkflowTransitionError(format!(
                "invalid transition {:?} -> {:?}",
                self.stage, next
            )));
        }
        Ok(Self {
            stage: next,
            input: self.input,
        })
    }
}

/// Output of HealthcareOS workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareWorkflowOutput {
    pub stage: HealthcareWorkflowStage,
    pub draft_note: String,
    pub verification_checklist: String,
    pub uncertainty_map: String,
    pub consent_status: String,
    pub consent_warning: Option<String>,
}

/// Execute complete HealthcareOS workflow: parse → validate consent → render
pub fn execute_healthcareos_workflow(
    input: HealthcareOsInputV1,
    transcript_content: &str,
    consent_content: Option<&str>,
) -> CoreResult<HealthcareWorkflowOutput> {
    // Step 1: Ingest and validate input
    let mut state = HealthcareWorkflowState::ingest(input)?;

    // Step 2: Parse transcript
    let transcript = parse_transcript(transcript_content)?;
    state = state.transition(HealthcareWorkflowStage::Analyzed)?;

    // Step 3: Parse and validate consent
    let consent_record = if let Some(consent_str) = consent_content {
        Some(parse_consent(consent_str)?)
    } else {
        None
    };

    let consent_status = validate_consent(&consent_record, &transcript.patient_id)?;

    // Enforce blocking: Missing or Revoked consent blocks export
    enforce_consent_block(&consent_status)?;

    state = state.transition(HealthcareWorkflowStage::Reviewed)?;

    // Step 4: Render deliverables
    let draft_note = render_draft_note(&transcript, &consent_status)?;
    let verification_checklist = render_verification_checklist(&transcript)?;
    let uncertainty_map = render_uncertainty_map(&transcript)?;

    state = state.transition(HealthcareWorkflowStage::Renderable)?;
    state = state.transition(HealthcareWorkflowStage::ExportReady)?;

    Ok(HealthcareWorkflowOutput {
        stage: state.stage,
        draft_note,
        verification_checklist,
        uncertainty_map,
        consent_status: format!("{:?}", consent_status),
        consent_warning: get_consent_warning(&consent_status),
    })
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn sample_transcript() -> &'static str {
        r#"{
            "patient_id": "PT-2026-001",
            "date": "2026-02-12",
            "provider": "Dr. Smith",
            "specialty": "Cardiology",
            "content": "Patient with chest pain. Possible myocardial infarction. EKG shows ST elevation. Recommend troponin levels.",
            "confidence": 0.96
        }"#
    }

    fn sample_consent() -> &'static str {
        r#"{
            "patient_id": "PT-2026-001",
            "date_given": "2024-06-12",
            "scope": "general",
            "status": "VALID"
        }"#
    }

    #[test]
    fn test_full_workflow_execution() {
        let input = HealthcareOsInputV1 {
            schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
            transcript_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "tx_001".to_string(),
                    sha256: "abc123".to_string(),
                    artifact_kind: "transcript".to_string(),
                },
            ],
            consent_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "consent_001".to_string(),
                    sha256: "def456".to_string(),
                    artifact_kind: "consent".to_string(),
                },
            ],
            draft_template_profile: "standard".to_string(),
            verifier_identity: "Dr. Reviewer".to_string(),
        };

        let output = execute_healthcareos_workflow(input, sample_transcript(), Some(sample_consent()));
        assert!(output.is_ok());

        let output = output.unwrap();
        assert_eq!(output.stage, HealthcareWorkflowStage::ExportReady);
        assert!(output.draft_note.contains("Draft Note"));
        assert!(output.verification_checklist.contains("Verification"));
        assert!(output.consent_status.contains("Valid"));
    }

    #[test]
    fn test_workflow_missing_consent_blocks() {
        let input = HealthcareOsInputV1 {
            schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
            transcript_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "tx_001".to_string(),
                    sha256: "abc123".to_string(),
                    artifact_kind: "transcript".to_string(),
                },
            ],
            consent_artifacts: vec![],
            draft_template_profile: "standard".to_string(),
            verifier_identity: "Dr. Reviewer".to_string(),
        };

        let result = execute_healthcareos_workflow(input, sample_transcript(), None);
        // Should fail on ingest due to missing consent artifacts
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_revoked_consent_blocks() {
        let revoked_consent = r#"{
            "patient_id": "PT-2026-001",
            "date_given": "2024-06-12",
            "scope": "general",
            "status": "REVOKED"
        }"#;

        let input = HealthcareOsInputV1 {
            schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
            transcript_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "tx_001".to_string(),
                    sha256: "abc123".to_string(),
                    artifact_kind: "transcript".to_string(),
                },
            ],
            consent_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "consent_001".to_string(),
                    sha256: "def456".to_string(),
                    artifact_kind: "consent".to_string(),
                },
            ],
            draft_template_profile: "standard".to_string(),
            verifier_identity: "Dr. Reviewer".to_string(),
        };

        let result = execute_healthcareos_workflow(input, sample_transcript(), Some(revoked_consent));
        // Should fail due to revoked consent
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_expired_consent_warns() {
        let expired_consent = r#"{
            "patient_id": "PT-2026-001",
            "date_given": "2024-01-01",
            "scope": "general",
            "status": "VALID"
        }"#;

        let input = HealthcareOsInputV1 {
            schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
            transcript_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "tx_001".to_string(),
                    sha256: "abc123".to_string(),
                    artifact_kind: "transcript".to_string(),
                },
            ],
            consent_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "consent_001".to_string(),
                    sha256: "def456".to_string(),
                    artifact_kind: "consent".to_string(),
                },
            ],
            draft_template_profile: "standard".to_string(),
            verifier_identity: "Dr. Reviewer".to_string(),
        };

        let output = execute_healthcareos_workflow(input, sample_transcript(), Some(expired_consent));
        assert!(output.is_ok());

        let output = output.unwrap();
        // Should have warning about expired consent
        assert!(output.consent_warning.is_some());
        assert!(output.consent_warning.unwrap().contains("expired"));
    }

    #[test]
    fn test_workflow_citation_enforcement() {
        let input = HealthcareOsInputV1 {
            schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
            transcript_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "tx_001".to_string(),
                    sha256: "abc123".to_string(),
                    artifact_kind: "transcript".to_string(),
                },
            ],
            consent_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "consent_001".to_string(),
                    sha256: "def456".to_string(),
                    artifact_kind: "consent".to_string(),
                },
            ],
            draft_template_profile: "standard".to_string(),
            verifier_identity: "Dr. Reviewer".to_string(),
        };

        let output = execute_healthcareos_workflow(input, sample_transcript(), Some(sample_consent())).unwrap();

        // Draft note must have citation markers
        assert!(output.draft_note.contains("<!-- CLAIM:C"));
    }

    #[test]
    fn test_workflow_state_transitions() {
        let input = HealthcareOsInputV1 {
            schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
            transcript_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "tx_001".to_string(),
                    sha256: "abc123".to_string(),
                    artifact_kind: "transcript".to_string(),
                },
            ],
            consent_artifacts: vec![
                crate::healthcareos::model::HealthcareArtifactRef {
                    artifact_id: "consent_001".to_string(),
                    sha256: "def456".to_string(),
                    artifact_kind: "consent".to_string(),
                },
            ],
            draft_template_profile: "standard".to_string(),
            verifier_identity: "Dr. Reviewer".to_string(),
        };

        let state = HealthcareWorkflowState::ingest(input).unwrap();
        assert_eq!(state.stage, HealthcareWorkflowStage::Ingested);

        let state = state.transition(HealthcareWorkflowStage::Analyzed).unwrap();
        assert_eq!(state.stage, HealthcareWorkflowStage::Analyzed);

        let state = state.transition(HealthcareWorkflowStage::Reviewed).unwrap();
        assert_eq!(state.stage, HealthcareWorkflowStage::Reviewed);

        let state = state.transition(HealthcareWorkflowStage::Renderable).unwrap();
        assert_eq!(state.stage, HealthcareWorkflowStage::Renderable);

        let state = state.transition(HealthcareWorkflowStage::ExportReady).unwrap();
        assert_eq!(state.stage, HealthcareWorkflowStage::ExportReady);

        // Invalid transition should fail
        let invalid = state.transition(HealthcareWorkflowStage::Ingested);
        assert!(invalid.is_err());
    }
}
