use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlDefinition {
    pub control_id: String,
    pub title: String,
    pub capability: String,
    pub control_family: String,
    pub description: String,
}

pub fn default_control_library() -> Vec<ControlDefinition> {
    let mut controls = vec![
        ControlDefinition {
            control_id: "CTRL-AC-001".to_string(),
            title: "Access control boundary".to_string(),
            capability: "Access Control".to_string(),
            control_family: "AccessControl".to_string(),
            description: "Evidence that access boundaries are defined and enforced.".to_string(),
        },
        ControlDefinition {
            control_id: "CTRL-AU-001".to_string(),
            title: "Audit event integrity".to_string(),
            capability: "Auditability".to_string(),
            control_family: "Auditability".to_string(),
            description: "Evidence that audit events are complete and tamper-evident.".to_string(),
        },
        ControlDefinition {
            control_id: "CTRL-DP-001".to_string(),
            title: "Data protection in exports".to_string(),
            capability: "Data Protection".to_string(),
            control_family: "DataProtection".to_string(),
            description: "Evidence that restricted data is handled with policy controls."
                .to_string(),
        },
        ControlDefinition {
            control_id: "CTRL-NW-001".to_string(),
            title: "Offline and egress posture".to_string(),
            capability: "Network Governance".to_string(),
            control_family: "NetworkGovernance".to_string(),
            description: "Evidence that network mode and allowlist controls are enforced."
                .to_string(),
        },
        ControlDefinition {
            control_id: "CTRL-MG-001".to_string(),
            title: "Model identity governance".to_string(),
            capability: "Model Governance".to_string(),
            control_family: "ModelGovernance".to_string(),
            description: "Evidence that model identity and pinning are recorded.".to_string(),
        },
        ControlDefinition {
            control_id: "CTRL-CI-001".to_string(),
            title: "Citation traceability".to_string(),
            capability: "Traceability".to_string(),
            control_family: "Traceability".to_string(),
            description: "Evidence that narrative claims are citation-backed.".to_string(),
        },
    ];
    controls.sort_by(|a, b| a.control_id.cmp(&b.control_id));
    controls
}

pub fn controls_for_capabilities(enabled_capabilities: &[String]) -> Vec<ControlDefinition> {
    if enabled_capabilities.is_empty() {
        return default_control_library();
    }
    let mut normalized = enabled_capabilities.to_vec();
    normalized.sort();
    normalized.dedup();
    let mut controls: Vec<ControlDefinition> = default_control_library()
        .into_iter()
        .filter(|c| normalized.iter().any(|cap| cap == &c.capability))
        .collect();
    controls.sort_by(|a, b| a.control_id.cmp(&b.control_id));
    controls
}
