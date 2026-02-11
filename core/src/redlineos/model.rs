use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractArtifactRef {
    pub artifact_id: String,
    pub sha256: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedlineOsInputV1 {
    pub schema_version: String,
    pub contract_artifacts: Vec<ContractArtifactRef>,
    pub extraction_mode: String,
    pub jurisdiction_hint: Option<String>,
    pub review_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedlineOsOutputManifestV1 {
    pub schema_version: String,
    pub deliverable_paths: Vec<String>,
    pub attachment_paths: Vec<String>,
}
