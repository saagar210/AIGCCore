use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRegistry {
    pub registry_version: String,
    pub gates: Vec<GateDef>,
    pub generated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDef {
    pub gate_id: String,
    pub category: String,
    pub severity: String,
    pub applies_to_policies: Vec<String>,
    pub pass_criteria: serde_json::Value,
    pub evidence_required: Vec<String>,
}

pub fn registry_v3() -> CoreResult<GateRegistry> {
    // Embed the authoritative JSON from Eval_Gate_Registry_v3.md.
    let json = include_str!("registry_v3.json");
    let reg: GateRegistry = serde_json::from_str(json)?;
    if reg.registry_version != "gates_registry_v3" {
        return Err(CoreError::InvalidInput(
            "embedded registry is not gates_registry_v3".to_string(),
        ));
    }
    Ok(reg)
}
