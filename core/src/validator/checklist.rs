use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checklist {
    pub checklist_version: String,
    pub checks: Vec<ChecklistCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistCheck {
    pub check_id: String,
    pub severity: String,
    pub description: String,
    pub validate: serde_json::Value,
}

pub fn checklist_v3() -> Checklist {
    // Embedded for reference/documentation; validator logic is implemented in code in mod.rs.
    // This ensures we still "ship" the checklist contract and can surface versions.
    let json = include_str!("checklist_v3.json");
    serde_json::from_str(json).expect("embedded checklist v3 JSON must parse")
}
