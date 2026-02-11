use crate::determinism::run_id::sha256_hex;

use super::model::NarrativeClaimInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimLineRange {
    pub claim_id: String,
    pub start_line: u64,
    pub end_line: u64,
    pub text_sha256: String,
}

pub fn render_narrative_markdown(claims: &[NarrativeClaimInput]) -> (String, Vec<ClaimLineRange>) {
    let mut lines = vec![
        "# Evidence Narrative".to_string(),
        "".to_string(),
        "All claims below require citation support under strict mode.".to_string(),
        "".to_string(),
    ];
    let mut ranges = Vec::new();

    for claim in claims {
        lines.push(format!("<!-- CLAIM:{} -->", claim.claim_id));
        let start_line = lines.len() as u64 + 1;
        for content_line in claim.text.lines() {
            lines.push(content_line.to_string());
        }
        let end_line = lines.len() as u64;
        ranges.push(ClaimLineRange {
            claim_id: claim.claim_id.clone(),
            start_line,
            end_line,
            text_sha256: sha256_hex(claim.text.as_bytes()),
        });
        lines.push("".to_string());
    }

    (lines.join("\n"), ranges)
}
