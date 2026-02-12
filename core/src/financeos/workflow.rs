use super::model::FinanceOsInputV1;
use super::parser::parse_financial_statement;
use super::exceptions::ExceptionDetector;
use super::render::{render_exceptions_audit, render_compliance_internal, render_exceptions_csv};
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

/// Output of FinanceOS workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceWorkflowOutput {
    pub stage: FinanceWorkflowStage,
    pub exceptions_audit: String,
    pub compliance_internal: String,
    pub exceptions_csv: String,
    pub transaction_count: usize,
    pub exception_count: usize,
    pub high_severity_count: usize,
}

/// Execute complete FinanceOS workflow: parse → analyze → render
pub fn execute_financeos_workflow(
    input: FinanceOsInputV1,
    statement_content: &str,
) -> CoreResult<FinanceWorkflowOutput> {
    // Step 1: Ingest and validate input
    let mut state = FinanceWorkflowState::ingest(input)?;

    // Step 2: Parse financial statement
    let statement = parse_financial_statement(statement_content)?;
    state = state.transition(FinanceWorkflowStage::Analyzed)?;

    // Step 3: Detect exceptions
    let detector = ExceptionDetector::new();
    let exceptions = detector.detect_exceptions(&statement)?;
    state = state.transition(FinanceWorkflowStage::Reviewed)?;

    // Step 4: Render deliverables
    let exceptions_audit = render_exceptions_audit(&statement, &exceptions)?;
    let compliance_internal = render_compliance_internal(&statement, &exceptions)?;
    let exceptions_csv = render_exceptions_csv(&exceptions)?;
    state = state.transition(FinanceWorkflowStage::Renderable)?;

    state = state.transition(FinanceWorkflowStage::ExportReady)?;

    let high_severity_count = exceptions.iter().filter(|e| e.severity == "HIGH").count();

    Ok(FinanceWorkflowOutput {
        stage: state.stage,
        exceptions_audit,
        compliance_internal,
        exceptions_csv,
        transaction_count: statement.summary.transaction_count,
        exception_count: exceptions.len(),
        high_severity_count,
    })
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn sample_statement() -> &'static str {
        r#"{
            "statement_id": "STMT_2026_01",
            "period_start": "2026-01-01",
            "period_end": "2026-01-31",
            "transactions": [
                {
                    "date": "2026-01-05",
                    "amount": 1000.00,
                    "account": "checking",
                    "category": "salary",
                    "description": "Monthly salary"
                },
                {
                    "date": "2026-01-10",
                    "amount": 15000.00,
                    "account": "checking",
                    "category": "transfer",
                    "description": "Large transfer"
                },
                {
                    "date": "2026-01-15",
                    "amount": 5000.00,
                    "account": "savings",
                    "category": "transfer",
                    "description": "Savings deposit"
                }
            ]
        }"#
    }

    #[test]
    fn test_full_workflow_execution() {
        let input = FinanceOsInputV1 {
            schema_version: "FINANCEOS_INPUT_V1".to_string(),
            finance_artifacts: vec![
                super::super::model::FinanceArtifactRef {
                    artifact_id: "stmt_001".to_string(),
                    sha256: "abc123".to_string(),
                    artifact_kind: "statement".to_string(),
                },
            ],
            period: "2026-01".to_string(),
            exception_rules_profile: "standard".to_string(),
            retention_profile: "standard".to_string(),
        };

        let output = execute_financeos_workflow(input, sample_statement());
        assert!(output.is_ok());

        let output = output.unwrap();
        assert_eq!(output.stage, FinanceWorkflowStage::ExportReady);
        assert_eq!(output.transaction_count, 3);
        assert!(output.exceptions_audit.contains("Audit"));
        assert!(output.compliance_internal.contains("Compliance"));
        assert!(output.exceptions_csv.contains("transaction_id"));
    }

    #[test]
    fn test_workflow_exception_detection() {
        let input = FinanceOsInputV1 {
            schema_version: "FINANCEOS_INPUT_V1".to_string(),
            finance_artifacts: vec![super::super::model::FinanceArtifactRef {
                artifact_id: "stmt_001".to_string(),
                sha256: "abc123".to_string(),
                artifact_kind: "json".to_string(),
            }],
            period: "2026-01".to_string(),
            exception_rules_profile: "standard".to_string(),
            retention_profile: "standard".to_string(),
        };

        let output = execute_financeos_workflow(input, sample_statement()).unwrap();
        // 15000 exceeds 10000 threshold
        assert!(output.high_severity_count > 0);
        assert!(output.exception_count > 0);
    }

    #[test]
    fn test_workflow_citation_enforcement() {
        let input = FinanceOsInputV1 {
            schema_version: "FINANCEOS_INPUT_V1".to_string(),
            finance_artifacts: vec![super::super::model::FinanceArtifactRef {
                artifact_id: "stmt_001".to_string(),
                sha256: "abc123".to_string(),
                artifact_kind: "json".to_string(),
            }],
            period: "2026-01".to_string(),
            exception_rules_profile: "standard".to_string(),
            retention_profile: "standard".to_string(),
        };

        let output = execute_financeos_workflow(input, sample_statement()).unwrap();

        // Both reports must have citation markers
        assert!(output.exceptions_audit.contains("<!-- CLAIM:C"));
        assert!(output.compliance_internal.contains("<!-- CLAIM:C"));
    }

    #[test]
    fn test_workflow_invalid_schema() {
        let input = FinanceOsInputV1 {
            schema_version: "INVALID_V1".to_string(),
            finance_artifacts: vec![super::super::model::FinanceArtifactRef {
                artifact_id: "stmt_001".to_string(),
                sha256: "abc123".to_string(),
                artifact_kind: "json".to_string(),
            }],
            period: "2026-01".to_string(),
            exception_rules_profile: "standard".to_string(),
            retention_profile: "standard".to_string(),
        };

        let result = execute_financeos_workflow(input, sample_statement());
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_state_transitions() {
        let input = FinanceOsInputV1 {
            schema_version: "FINANCEOS_INPUT_V1".to_string(),
            finance_artifacts: vec![super::super::model::FinanceArtifactRef {
                artifact_id: "stmt_001".to_string(),
                sha256: "abc123".to_string(),
                artifact_kind: "json".to_string(),
            }],
            period: "2026-01".to_string(),
            exception_rules_profile: "standard".to_string(),
            retention_profile: "standard".to_string(),
        };

        let state = FinanceWorkflowState::ingest(input).unwrap();
        assert_eq!(state.stage, FinanceWorkflowStage::Ingested);

        let state = state.transition(FinanceWorkflowStage::Analyzed).unwrap();
        assert_eq!(state.stage, FinanceWorkflowStage::Analyzed);

        let state = state.transition(FinanceWorkflowStage::Reviewed).unwrap();
        assert_eq!(state.stage, FinanceWorkflowStage::Reviewed);

        let state = state.transition(FinanceWorkflowStage::Renderable).unwrap();
        assert_eq!(state.stage, FinanceWorkflowStage::Renderable);

        let state = state.transition(FinanceWorkflowStage::ExportReady).unwrap();
        assert_eq!(state.stage, FinanceWorkflowStage::ExportReady);

        // Invalid transition should fail
        let invalid = state.transition(FinanceWorkflowStage::Ingested);
        assert!(invalid.is_err());
    }
}
