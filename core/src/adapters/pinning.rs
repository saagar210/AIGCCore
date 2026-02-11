use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinningLevel {
    CRYPTO_PINNED,
    VERSION_PINNED,
    NAME_ONLY,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSnapshot {
    pub adapter_id: String,
    pub adapter_version: String,
    pub adapter_endpoint: String,
    pub model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_sha256: Option<String>,
    pub pinning_level: PinningLevel,
}

pub fn classify_pinning_level(
    model_sha256: Option<&str>,
    adapter_id: &str,
    adapter_version: &str,
) -> PinningLevel {
    if model_sha256.is_some() && !adapter_id.is_empty() && !adapter_version.is_empty() {
        PinningLevel::CRYPTO_PINNED
    } else if !adapter_id.is_empty() && !adapter_version.is_empty() {
        PinningLevel::VERSION_PINNED
    } else {
        PinningLevel::NAME_ONLY
    }
}
