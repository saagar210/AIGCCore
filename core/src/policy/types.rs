use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyMode {
    STRICT,
    BALANCED,
    DRAFT_ONLY,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkMode {
    OFFLINE,
    ONLINE_ALLOWLISTED,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofLevel {
    OFFLINE_STRICT,
    ONLINE_ALLOWLIST_CORE_ONLY,
    ONLINE_ALLOWLIST_WITH_OS_FIREWALL_PROFILE,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputExportProfile {
    HASH_ONLY,
    INCLUDE_INPUT_BYTES,
}
