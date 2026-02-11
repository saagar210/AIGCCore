use super::consent::validate_consent_present;
use super::model::HealthcareOsInputV1;
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
        validate_consent_present(input.consent_artifacts.len())?;
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
