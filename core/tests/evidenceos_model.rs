use aigc_core::evidenceos::model::{CitationInput, EvidenceItem, NarrativeClaimInput};
use aigc_core::evidenceos::workflow::{generate_evidenceos_artifacts, EvidenceOsRequest};
use aigc_core::policy::types::PolicyMode;
use serde_json::json;

#[test]
fn evidence_item_normalization_sorts_and_dedups_labels() {
    let item = EvidenceItem {
        artifact_id: "a_1".to_string(),
        artifact_sha256: "abc".to_string(),
        title: "Artifact".to_string(),
        tags: vec!["PII".to_string(), "PII".to_string(), "DOC".to_string()],
        control_family_labels: vec![
            "Traceability".to_string(),
            "Auditability".to_string(),
            "Traceability".to_string(),
        ],
    }
    .normalized();

    assert_eq!(item.tags, vec!["DOC".to_string(), "PII".to_string()]);
    assert_eq!(
        item.control_family_labels,
        vec!["Auditability".to_string(), "Traceability".to_string()]
    );
}

#[test]
fn strict_mode_blocks_claim_without_citation() {
    let req = EvidenceOsRequest {
        pack_id: "evidenceos".to_string(),
        pack_version: "1.0.0".to_string(),
        run_id: "r_test".to_string(),
        policy_mode: PolicyMode::STRICT,
        enabled_capabilities: vec![],
        evidence_items: vec![EvidenceItem {
            artifact_id: "a_1".to_string(),
            artifact_sha256: "abc".to_string(),
            title: "Artifact".to_string(),
            tags: vec!["DOC".to_string()],
            control_family_labels: vec!["Traceability".to_string()],
        }],
        narrative_claims: vec![NarrativeClaimInput {
            claim_id: "C0001".to_string(),
            text: "Claim text".to_string(),
            citations: vec![],
        }],
    };

    let err = generate_evidenceos_artifacts(&req).unwrap_err();
    assert!(err.to_string().contains("strict mode requires citations"));
}

#[test]
fn mapping_workflow_marks_missing_controls() {
    let req = EvidenceOsRequest {
        pack_id: "evidenceos".to_string(),
        pack_version: "1.0.0".to_string(),
        run_id: "r_test".to_string(),
        policy_mode: PolicyMode::STRICT,
        enabled_capabilities: vec![],
        evidence_items: vec![EvidenceItem {
            artifact_id: "a_1".to_string(),
            artifact_sha256: "abc".to_string(),
            title: "Artifact".to_string(),
            tags: vec!["DOC".to_string()],
            control_family_labels: vec!["Traceability".to_string()],
        }],
        narrative_claims: vec![NarrativeClaimInput {
            claim_id: "C0001".to_string(),
            text: "Claim text".to_string(),
            citations: vec![CitationInput {
                artifact_id: "a_1".to_string(),
                locator_type: "PDF_TEXT_SPAN_V1".to_string(),
                locator: json!({"page_index": 0, "start_char": 0, "end_char": 5, "text_sha256": "abc"}),
            }],
        }],
    };

    let out = generate_evidenceos_artifacts(&req).unwrap();
    assert!(!out.missing_control_ids.is_empty());
    assert!(out
        .mapping_review_rows
        .iter()
        .any(|row| row.status == "MISSING"));
}
