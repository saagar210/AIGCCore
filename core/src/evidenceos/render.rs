use crate::error::CoreResult;
use serde::Serialize;

use super::model::EvidenceItem;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MappingReviewRow {
    pub control_id: String,
    pub capability: String,
    pub control_family: String,
    pub status: String,
    pub mapped_artifact_ids: Vec<String>,
    pub reviewer_note: String,
}

pub fn render_evidence_index_csv(items: &[EvidenceItem]) -> CoreResult<String> {
    let mut rows = items.to_vec();
    rows.sort_by(|a, b| a.artifact_id.cmp(&b.artifact_id));

    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    wtr.write_record(&[
        "artifact_id",
        "artifact_sha256",
        "title",
        "tags",
        "control_families",
    ])?;
    for row in rows {
        wtr.write_record(&[
            row.artifact_id,
            row.artifact_sha256,
            row.title,
            row.tags.join(";"),
            row.control_family_labels.join(";"),
        ])?;
    }
    let bytes = wtr.into_inner().map_err(|e| e.into_error())?;
    Ok(String::from_utf8_lossy(&bytes).replace("\r\n", "\n"))
}

pub fn render_evidence_index_markdown(items: &[EvidenceItem]) -> String {
    let mut rows = items.to_vec();
    rows.sort_by(|a, b| a.artifact_id.cmp(&b.artifact_id));

    let mut out = Vec::new();
    out.push("# Evidence Index".to_string());
    out.push("".to_string());
    out.push("| Artifact ID | SHA-256 | Title | Tags | Control Families |".to_string());
    out.push("|---|---|---|---|---|".to_string());
    for row in rows {
        out.push(format!(
            "| {} | {} | {} | {} | {} |",
            row.artifact_id,
            row.artifact_sha256,
            row.title,
            row.tags.join(", "),
            row.control_family_labels.join(", ")
        ));
    }
    out.push("".to_string());
    out.join("\n")
}

pub fn render_missing_checklist_markdown(
    missing_control_ids: &[String],
    mapping_rows: &[MappingReviewRow],
) -> String {
    let mut out = Vec::new();
    out.push("# Missing Evidence Checklist".to_string());
    out.push("".to_string());
    if missing_control_ids.is_empty() {
        out.push("- [x] No missing control evidence mappings.".to_string());
        out.push("".to_string());
        return out.join("\n");
    }

    let mut sorted = missing_control_ids.to_vec();
    sorted.sort();
    sorted.dedup();
    for control_id in sorted {
        let title = mapping_rows
            .iter()
            .find(|row| row.control_id == control_id)
            .map(|row| format!("{} ({})", row.control_id, row.capability))
            .unwrap_or(control_id);
        out.push(format!("- [ ] {}", title));
    }
    out.push("".to_string());
    out.join("\n")
}
