use super::model::RedlineOsInputV1;
use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RedlineWorkflowStage {
    Ingested,
    Analyzed,
    Reviewed,
    Renderable,
    ExportReady,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedlineWorkflowState {
    pub stage: RedlineWorkflowStage,
    pub input: RedlineOsInputV1,
}

impl RedlineWorkflowState {
    pub fn ingest(input: RedlineOsInputV1) -> CoreResult<Self> {
        if input.schema_version != "REDLINEOS_INPUT_V1" {
            return Err(CoreError::InputSchemaError(format!(
                "expected REDLINEOS_INPUT_V1, got {}",
                input.schema_version
            )));
        }
        if input.contract_artifacts.is_empty() {
            return Err(CoreError::ArtifactMissingError(
                "at least one contract artifact is required".to_string(),
            ));
        }
        Ok(Self {
            stage: RedlineWorkflowStage::Ingested,
            input,
        })
    }

    pub fn transition(self, next: RedlineWorkflowStage) -> CoreResult<Self> {
        let allowed = matches!(
            (self.stage, next),
            (
                RedlineWorkflowStage::Ingested,
                RedlineWorkflowStage::Analyzed
            ) | (
                RedlineWorkflowStage::Analyzed,
                RedlineWorkflowStage::Reviewed
            ) | (
                RedlineWorkflowStage::Reviewed,
                RedlineWorkflowStage::Renderable
            ) | (
                RedlineWorkflowStage::Renderable,
                RedlineWorkflowStage::ExportReady
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
