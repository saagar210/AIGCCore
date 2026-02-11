use super::model::HealthcareOsOutputManifestV1;

pub fn output_manifest() -> HealthcareOsOutputManifestV1 {
    HealthcareOsOutputManifestV1 {
        schema_version: "HEALTHCAREOS_OUTPUT_V1".to_string(),
        deliverable_paths: vec![
            "exports/healthcareos/deliverables/draft_note.md".to_string(),
            "exports/healthcareos/deliverables/verification_checklist.md".to_string(),
        ],
        attachment_paths: vec![
            "exports/healthcareos/attachments/consent_record.json".to_string(),
            "exports/healthcareos/attachments/citations_map.json".to_string(),
            "exports/healthcareos/attachments/uncertainty_map.json".to_string(),
        ],
    }
}
