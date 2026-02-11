use super::model::RedlineOsOutputManifestV1;

pub fn output_manifest() -> RedlineOsOutputManifestV1 {
    RedlineOsOutputManifestV1 {
        schema_version: "REDLINEOS_OUTPUT_V1".to_string(),
        deliverable_paths: vec![
            "exports/redlineos/deliverables/risk_memo.md".to_string(),
            "exports/redlineos/deliverables/clause_map.csv".to_string(),
            "exports/redlineos/deliverables/redline_suggestions.md".to_string(),
        ],
        attachment_paths: vec![
            "exports/redlineos/attachments/citations_map.json".to_string(),
            "exports/redlineos/attachments/anchor_index.json".to_string(),
        ],
    }
}
