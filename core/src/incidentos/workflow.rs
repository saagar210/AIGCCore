use super::model::IncidentOsInputV1;
use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentWorkflowStage {
    Ingested,
    Analyzed,
    Reviewed,
    Renderable,
    ExportReady,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentWorkflowState {
    pub stage: IncidentWorkflowStage,
    pub input: IncidentOsInputV1,
}

impl IncidentWorkflowState {
    pub fn ingest(input: IncidentOsInputV1) -> CoreResult<Self> {
        if input.schema_version != "INCIDENTOS_INPUT_V1" {
            return Err(CoreError::InputSchemaError(format!(
                "expected INCIDENTOS_INPUT_V1, got {}",
                input.schema_version
            )));
        }
        if input.incident_artifacts.is_empty() {
            return Err(CoreError::ArtifactMissingError(
                "at least one incident artifact is required".to_string(),
            ));
        }
        Ok(Self {
            stage: IncidentWorkflowStage::Ingested,
            input,
        })
    }

    pub fn transition(self, next: IncidentWorkflowStage) -> CoreResult<Self> {
        let allowed = matches!(
            (self.stage, next),
            (
                IncidentWorkflowStage::Ingested,
                IncidentWorkflowStage::Analyzed
            ) | (
                IncidentWorkflowStage::Analyzed,
                IncidentWorkflowStage::Reviewed
            ) | (
                IncidentWorkflowStage::Reviewed,
                IncidentWorkflowStage::Renderable
            ) | (
                IncidentWorkflowStage::Renderable,
                IncidentWorkflowStage::ExportReady
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
