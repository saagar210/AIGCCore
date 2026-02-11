use super::model::FinanceOsOutputManifestV1;

pub fn output_manifest() -> FinanceOsOutputManifestV1 {
    FinanceOsOutputManifestV1 {
        schema_version: "FINANCEOS_OUTPUT_V1".to_string(),
        deliverable_paths: vec![
            "exports/financeos/deliverables/exceptions_packet.md".to_string(),
            "exports/financeos/deliverables/exceptions.csv".to_string(),
            "exports/financeos/deliverables/accounting_export.csv".to_string(),
        ],
        attachment_paths: vec![
            "exports/financeos/attachments/redactions_map.json".to_string(),
            "exports/financeos/attachments/citations_map.json".to_string(),
        ],
    }
}
