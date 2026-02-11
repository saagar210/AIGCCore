use super::model::FinanceOsInputV1;
use super::policies::validate_retention_profile;
use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FinanceWorkflowStage {
    Ingested,
    Analyzed,
    Reviewed,
    Renderable,
    ExportReady,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceWorkflowState {
    pub stage: FinanceWorkflowStage,
    pub input: FinanceOsInputV1,
}

impl FinanceWorkflowState {
    pub fn ingest(input: FinanceOsInputV1) -> CoreResult<Self> {
        if input.schema_version != "FINANCEOS_INPUT_V1" {
            return Err(CoreError::InputSchemaError(format!(
                "expected FINANCEOS_INPUT_V1, got {}",
                input.schema_version
            )));
        }
        if input.finance_artifacts.is_empty() {
            return Err(CoreError::ArtifactMissingError(
                "at least one finance artifact is required".to_string(),
            ));
        }
        validate_retention_profile(&input.retention_profile)?;
        Ok(Self {
            stage: FinanceWorkflowStage::Ingested,
            input,
        })
    }

    pub fn transition(self, next: FinanceWorkflowStage) -> CoreResult<Self> {
        let allowed = matches!(
            (self.stage, next),
            (
                FinanceWorkflowStage::Ingested,
                FinanceWorkflowStage::Analyzed
            ) | (
                FinanceWorkflowStage::Analyzed,
                FinanceWorkflowStage::Reviewed
            ) | (
                FinanceWorkflowStage::Reviewed,
                FinanceWorkflowStage::Renderable
            ) | (
                FinanceWorkflowStage::Renderable,
                FinanceWorkflowStage::ExportReady
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
