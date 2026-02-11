use crate::policy::allowlist::AllowlistEntry;
use crate::policy::types::{NetworkMode, ProofLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterEndpointSnapshot {
    pub endpoint: String,
    pub is_loopback: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub network_mode: NetworkMode,
    pub proof_level: ProofLevel,
    pub allowlist: Vec<AllowlistEntry>, // canonical entries; sorted
    pub ui_remote_fetch_disabled: bool,
    pub adapter_endpoints: Vec<AdapterEndpointSnapshot>,
}
