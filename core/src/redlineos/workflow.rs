use super::model::{RedlineOsInputV1, RiskAssessment};
use super::extraction;
use super::anchors;
use super::risk_analysis;
use super::render;
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

/// Execute complete RedlineOS workflow: extract → segment → assess → render
pub fn execute_redlineos_workflow(
    input: RedlineOsInputV1,
    contract_bytes: &[u8],
) -> CoreResult<RedlineWorkflowOutput> {
    // Step 1: Ingest and validate input
    let mut state = RedlineWorkflowState::ingest(input)?;

    // Step 2: Extract text from contract
    let extracted = extraction::extract_contract_text(contract_bytes, &state.input.extraction_mode)?;
    state = state.transition(RedlineWorkflowStage::Analyzed)?;

    // Step 3: Segment into clauses
    let clauses = anchors::segment_clauses(&extracted.extracted_text, &extracted.artifact_id)?;
    let anchors = anchors::generate_anchors(&clauses, &extracted.artifact_id)?;
    state = state.transition(RedlineWorkflowStage::Reviewed)?;

    // Step 4: Assess risks
    let assessments: Vec<RiskAssessment> = clauses
        .iter()
        .zip(anchors.iter())
        .map(|(clause, anchor)| risk_analysis::assess_clause_risk(clause, anchor))
        .collect();
    state = state.transition(RedlineWorkflowStage::Renderable)?;

    // Step 5: Render deliverables
    let risk_memo = render::render_risk_memo(&assessments, &clauses)?;
    let clause_map = render::render_clause_map_csv(&assessments)?;
    let suggestions = render::render_redline_suggestions(&assessments)?;
    state = state.transition(RedlineWorkflowStage::ExportReady)?;

    Ok(RedlineWorkflowOutput {
        stage: state.stage,
        risk_memo,
        clause_map,
        suggestions,
        assessment_count: assessments.len(),
        high_risk_count: assessments.iter().filter(|a| a.risk_level == "HIGH").count(),
        extraction_confidence: extracted.extraction_confidence,
    })
}

/// Output of RedlineOS workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedlineWorkflowOutput {
    pub stage: RedlineWorkflowStage,
    pub risk_memo: String,
    pub clause_map: String,
    pub suggestions: String,
    pub assessment_count: usize,
    pub high_risk_count: usize,
    pub extraction_confidence: f32,
}
