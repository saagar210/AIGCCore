use aigc_core::financeos::model::{FinanceArtifactRef, FinanceOsInputV1};
use aigc_core::financeos::workflow::{FinanceWorkflowStage, FinanceWorkflowState};
use aigc_core::healthcareos::model::{HealthcareArtifactRef, HealthcareOsInputV1};
use aigc_core::healthcareos::workflow::{HealthcareWorkflowStage, HealthcareWorkflowState};
use aigc_core::incidentos::model::{IncidentArtifactRef, IncidentOsInputV1};
use aigc_core::incidentos::sanitize::sanitize_untrusted_log;
use aigc_core::incidentos::workflow::{IncidentWorkflowStage, IncidentWorkflowState};
use aigc_core::redlineos::anchors::stable_clause_anchor;
use aigc_core::redlineos::model::{ContractArtifactRef, RedlineOsInputV1};
use aigc_core::redlineos::workflow::{RedlineWorkflowStage, RedlineWorkflowState};

#[test]
fn redline_workflow_requires_artifact_and_valid_transition() {
    let bad = RedlineOsInputV1 {
        schema_version: "REDLINEOS_INPUT_V1".to_string(),
        contract_artifacts: vec![],
        extraction_mode: "OCR".to_string(),
        jurisdiction_hint: None,
        review_profile: "default".to_string(),
    };
    assert!(RedlineWorkflowState::ingest(bad).is_err());

    let ok = RedlineOsInputV1 {
        schema_version: "REDLINEOS_INPUT_V1".to_string(),
        contract_artifacts: vec![ContractArtifactRef {
            artifact_id: "a1".to_string(),
            sha256: "s1".to_string(),
            filename: "contract.pdf".to_string(),
        }],
        extraction_mode: "OCR".to_string(),
        jurisdiction_hint: Some("US-CA".to_string()),
        review_profile: "default".to_string(),
    };
    let state = RedlineWorkflowState::ingest(ok).expect("ingest");
    assert!(state.transition(RedlineWorkflowStage::ExportReady).is_err());
    assert!(
        state
            .transition(RedlineWorkflowStage::Analyzed)
            .expect("transition")
            .stage
            == RedlineWorkflowStage::Analyzed
    );
}

#[test]
fn incident_workflow_and_sanitizer_behave_as_expected() {
    let sanitized = sanitize_untrusted_log("hello\0world");
    assert_eq!(sanitized.content, "helloworld");

    let input = IncidentOsInputV1 {
        schema_version: "INCIDENTOS_INPUT_V1".to_string(),
        incident_artifacts: vec![IncidentArtifactRef {
            artifact_id: "i1".to_string(),
            sha256: "s1".to_string(),
            source_type: "syslog".to_string(),
        }],
        timeline_start_hint: None,
        timeline_end_hint: None,
        customer_redaction_profile: "strict".to_string(),
    };
    let state = IncidentWorkflowState::ingest(input).expect("ingest");
    assert_eq!(state.stage, IncidentWorkflowStage::Ingested);
}

#[test]
fn finance_and_healthcare_ingest_require_policy_and_consent() {
    let finance_input = FinanceOsInputV1 {
        schema_version: "FINANCEOS_INPUT_V1".to_string(),
        finance_artifacts: vec![FinanceArtifactRef {
            artifact_id: "f1".to_string(),
            sha256: "s1".to_string(),
            artifact_kind: "invoice".to_string(),
        }],
        period: "2026-01".to_string(),
        exception_rules_profile: "default".to_string(),
        retention_profile: "ret_min".to_string(),
    };
    let finance_state = FinanceWorkflowState::ingest(finance_input).expect("ingest");
    assert_eq!(finance_state.stage, FinanceWorkflowStage::Ingested);

    let healthcare_bad = HealthcareOsInputV1 {
        schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
        consent_artifacts: vec![],
        transcript_artifacts: vec![HealthcareArtifactRef {
            artifact_id: "t1".to_string(),
            sha256: "s1".to_string(),
            artifact_kind: "transcript".to_string(),
        }],
        draft_template_profile: "soap".to_string(),
        verifier_identity: "clinician_1".to_string(),
    };
    assert!(HealthcareWorkflowState::ingest(healthcare_bad).is_err());

    let healthcare_ok = HealthcareOsInputV1 {
        schema_version: "HEALTHCAREOS_INPUT_V1".to_string(),
        consent_artifacts: vec![HealthcareArtifactRef {
            artifact_id: "c1".to_string(),
            sha256: "s1".to_string(),
            artifact_kind: "consent".to_string(),
        }],
        transcript_artifacts: vec![HealthcareArtifactRef {
            artifact_id: "t1".to_string(),
            sha256: "s2".to_string(),
            artifact_kind: "transcript".to_string(),
        }],
        draft_template_profile: "soap".to_string(),
        verifier_identity: "clinician_1".to_string(),
    };
    let state = HealthcareWorkflowState::ingest(healthcare_ok).expect("ingest");
    assert_eq!(state.stage, HealthcareWorkflowStage::Ingested);
}

#[test]
fn stable_anchor_is_deterministic() {
    let a = stable_clause_anchor("Payment must be made in 30 days.");
    let b = stable_clause_anchor("Payment   must be made in 30 days.");
    assert_eq!(a, b);
}
