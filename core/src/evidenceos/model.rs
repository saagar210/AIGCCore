use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceItem {
    pub artifact_id: String,
    pub artifact_sha256: String,
    pub title: String,
    pub tags: Vec<String>,
    pub control_family_labels: Vec<String>,
}

impl EvidenceItem {
    pub fn normalized(mut self) -> Self {
        self.tags.sort();
        self.tags.dedup();
        self.control_family_labels.sort();
        self.control_family_labels.dedup();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitationInput {
    pub artifact_id: String,
    pub locator_type: String,
    pub locator: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NarrativeClaimInput {
    pub claim_id: String,
    pub text: String,
    pub citations: Vec<CitationInput>,
}
