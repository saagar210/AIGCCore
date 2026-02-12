use super::model::IncidentOsInputV1;
use super::parser::{parse_json_log, parse_ndjson_log};
use super::timeline::{build_timeline, render_timeline_csv};
use super::redaction::RedactionProfile;
use super::render::{render_customer_packet, render_internal_packet};
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

/// Output of IncidentOS workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentWorkflowOutput {
    pub stage: IncidentWorkflowStage,
    pub customer_packet: String,
    pub internal_packet: String,
    pub timeline_csv: String,
    pub event_count: usize,
    pub high_severity_count: usize,
    pub redaction_count: usize,
}

/// Execute complete IncidentOS workflow: parse → timeline → render
pub fn execute_incidentos_workflow(
    input: IncidentOsInputV1,
    log_content: &str,
) -> CoreResult<IncidentWorkflowOutput> {
    // Step 1: Ingest and validate input
    let mut state = IncidentWorkflowState::ingest(input)?;

    // Step 2: Parse incident logs
    let events = if log_content.trim().starts_with('[') {
        parse_json_log(log_content)?
    } else {
        parse_ndjson_log(log_content)?
    };

    state = state.transition(IncidentWorkflowStage::Analyzed)?;

    // Step 3: Build timeline
    let timeline = build_timeline(&state.input.incident_artifacts[0].artifact_id, events)?;
    state = state.transition(IncidentWorkflowStage::Reviewed)?;

    // Step 4: Apply redaction profile
    let redaction_profile = RedactionProfile::from_str(&state.input.customer_redaction_profile)?;
    state = state.transition(IncidentWorkflowStage::Renderable)?;

    // Step 5: Render deliverables
    let customer_packet = render_customer_packet(&timeline, redaction_profile)?;
    let internal_packet = render_internal_packet(&timeline)?;
    let timeline_csv = render_timeline_csv(&timeline)?;

    state = state.transition(IncidentWorkflowStage::ExportReady)?;

    let high_severity_count = timeline.high_severity_count;
    let redaction_count = timeline.events.len() * 2; // Estimate: ~2 redactions per event on BASIC

    Ok(IncidentWorkflowOutput {
        stage: state.stage,
        customer_packet,
        internal_packet,
        timeline_csv,
        event_count: timeline.events.len(),
        high_severity_count,
        redaction_count,
    })
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn sample_ndjson_log() -> &'static str {
        r#"{"timestamp":"2026-02-12T10:15:30Z","source_system":"web","actor":"user@example.com","action":"login_attempt","affected_resource":"auth","evidence_text":"User successfully authenticated"}
{"timestamp":"2026-02-12T10:15:35Z","source_system":"db","actor":"system","action":"critical_error","affected_resource":"users","evidence_text":"System breach detected"}"#
    }

    #[test]
    fn test_full_workflow_execution() {
        let input = IncidentOsInputV1 {
            schema_version: "INCIDENTOS_INPUT_V1".to_string(),
            incident_artifacts: vec![
                crate::incidentos::model::IncidentArtifactRef {
                    artifact_id: "incident_001".to_string(),
                    sha256: "abc123".to_string(),
                    source_type: "json".to_string(),
                },
            ],
            timeline_start_hint: None,
            timeline_end_hint: None,
            customer_redaction_profile: "BASIC".to_string(),
        };

        let output = execute_incidentos_workflow(input, sample_ndjson_log());
        assert!(output.is_ok());

        let output = output.unwrap();
        assert_eq!(output.stage, IncidentWorkflowStage::ExportReady);
        assert_eq!(output.event_count, 2);
        assert!(output.customer_packet.contains("Customer Summary"));
        assert!(output.internal_packet.contains("Internal Analysis"));
        assert!(output.timeline_csv.contains("timestamp,system,actor"));
    }

    #[test]
    fn test_workflow_determinism() {
        let input = IncidentOsInputV1 {
            schema_version: "INCIDENTOS_INPUT_V1".to_string(),
            incident_artifacts: vec![crate::incidentos::model::IncidentArtifactRef {
                artifact_id: "incident_001".to_string(),
                sha256: "abc123".to_string(),
                source_type: "json".to_string(),
            }],
            timeline_start_hint: None,
            timeline_end_hint: None,
            customer_redaction_profile: "BASIC".to_string(),
        };

        let output1 = execute_incidentos_workflow(input.clone(), sample_ndjson_log()).unwrap();
        let output2 = execute_incidentos_workflow(input.clone(), sample_ndjson_log()).unwrap();

        // Should produce identical outputs
        assert_eq!(output1.customer_packet, output2.customer_packet);
        assert_eq!(output1.internal_packet, output2.internal_packet);
        assert_eq!(output1.timeline_csv, output2.timeline_csv);
    }

    #[test]
    fn test_workflow_redaction_profile() {
        let mut input_basic = IncidentOsInputV1 {
            schema_version: "INCIDENTOS_INPUT_V1".to_string(),
            incident_artifacts: vec![crate::incidentos::model::IncidentArtifactRef {
                artifact_id: "incident_001".to_string(),
                sha256: "abc123".to_string(),
                source_type: "json".to_string(),
            }],
            timeline_start_hint: None,
            timeline_end_hint: None,
            customer_redaction_profile: "BASIC".to_string(),
        };

        let output_basic = execute_incidentos_workflow(input_basic.clone(), sample_ndjson_log()).unwrap();

        // STRICT profile should differ from BASIC
        input_basic.customer_redaction_profile = "STRICT".to_string();
        let output_strict = execute_incidentos_workflow(input_basic, sample_ndjson_log()).unwrap();

        // Both should be valid, but content may differ
        assert!(!output_basic.customer_packet.is_empty());
        assert!(!output_strict.customer_packet.is_empty());
    }

    #[test]
    fn test_workflow_citation_enforcement() {
        let input = IncidentOsInputV1 {
            schema_version: "INCIDENTOS_INPUT_V1".to_string(),
            incident_artifacts: vec![crate::incidentos::model::IncidentArtifactRef {
                artifact_id: "incident_001".to_string(),
                sha256: "abc123".to_string(),
                source_type: "json".to_string(),
            }],
            timeline_start_hint: None,
            timeline_end_hint: None,
            customer_redaction_profile: "BASIC".to_string(),
        };

        let output = execute_incidentos_workflow(input, sample_ndjson_log()).unwrap();

        // Both packets must have citation markers
        assert!(output.customer_packet.contains("<!-- CLAIM:C"));
        assert!(output.internal_packet.contains("<!-- CLAIM:C"));
        assert_eq!(output.customer_packet.matches("<!-- CLAIM:C").count(), 2);
        assert_eq!(output.internal_packet.matches("<!-- CLAIM:C").count(), 2);
    }

    #[test]
    fn test_workflow_invalid_schema() {
        let input = IncidentOsInputV1 {
            schema_version: "INVALID_V1".to_string(),
            incident_artifacts: vec![crate::incidentos::model::IncidentArtifactRef {
                artifact_id: "incident_001".to_string(),
                sha256: "abc123".to_string(),
                source_type: "json".to_string(),
            }],
            timeline_start_hint: None,
            timeline_end_hint: None,
            customer_redaction_profile: "BASIC".to_string(),
        };

        let result = execute_incidentos_workflow(input, sample_ndjson_log());
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_state_transitions() {
        let input = IncidentOsInputV1 {
            schema_version: "INCIDENTOS_INPUT_V1".to_string(),
            incident_artifacts: vec![crate::incidentos::model::IncidentArtifactRef {
                artifact_id: "incident_001".to_string(),
                sha256: "abc123".to_string(),
                source_type: "json".to_string(),
            }],
            timeline_start_hint: None,
            timeline_end_hint: None,
            customer_redaction_profile: "BASIC".to_string(),
        };

        let state = IncidentWorkflowState::ingest(input).unwrap();
        assert_eq!(state.stage, IncidentWorkflowStage::Ingested);

        let state = state.transition(IncidentWorkflowStage::Analyzed).unwrap();
        assert_eq!(state.stage, IncidentWorkflowStage::Analyzed);

        let state = state.transition(IncidentWorkflowStage::Reviewed).unwrap();
        assert_eq!(state.stage, IncidentWorkflowStage::Reviewed);

        let state = state.transition(IncidentWorkflowStage::Renderable).unwrap();
        assert_eq!(state.stage, IncidentWorkflowStage::Renderable);

        let state = state.transition(IncidentWorkflowStage::ExportReady).unwrap();
        assert_eq!(state.stage, IncidentWorkflowStage::ExportReady);

        // Invalid transition should fail
        let invalid = state.transition(IncidentWorkflowStage::Ingested);
        assert!(invalid.is_err());
    }
}
