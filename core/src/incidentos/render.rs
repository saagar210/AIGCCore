use super::model::IncidentOsOutputManifestV1;

pub fn output_manifest() -> IncidentOsOutputManifestV1 {
    IncidentOsOutputManifestV1 {
        schema_version: "INCIDENTOS_OUTPUT_V1".to_string(),
        deliverable_paths: vec![
            "exports/incidentos/deliverables/customer_packet.md".to_string(),
            "exports/incidentos/deliverables/internal_packet.md".to_string(),
            "exports/incidentos/deliverables/timeline.csv".to_string(),
        ],
        attachment_paths: vec![
            "exports/incidentos/attachments/redactions_map.json".to_string(),
            "exports/incidentos/attachments/citations_map.json".to_string(),
        ],
    }
}
