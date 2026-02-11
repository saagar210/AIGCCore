use crate::audit::event::{Actor, AuditEvent};
use crate::audit::log::AuditLog;
use crate::error::CoreResult;
use crate::storage::vault::VaultStorage;

pub fn emit_vault_encryption_status(
    audit: &mut AuditLog,
    run_id: &str,
    vault_id: &str,
    vault: &VaultStorage,
    ts_utc: &str,
) -> CoreResult<()> {
    audit.append(AuditEvent {
        ts_utc: ts_utc.to_string(),
        event_type: "VAULT_ENCRYPTION_STATUS".to_string(),
        run_id: run_id.to_string(),
        vault_id: vault_id.to_string(),
        actor: Actor::System,
        details: vault.encryption_status_audit_details(),
        prev_event_hash: String::new(),
        event_hash: String::new(),
    })?;
    Ok(())
}

pub fn emit_vault_key_rotated(
    audit: &mut AuditLog,
    run_id: &str,
    vault_id: &str,
    old_key_id: &str,
    new_key_id: &str,
    ts_utc: &str,
) -> CoreResult<()> {
    audit.append(AuditEvent {
        ts_utc: ts_utc.to_string(),
        event_type: "VAULT_KEY_ROTATED".to_string(),
        run_id: run_id.to_string(),
        vault_id: vault_id.to_string(),
        actor: Actor::System,
        details: serde_json::json!({
            "old_key_id": old_key_id,
            "new_key_id": new_key_id
        }),
        prev_event_hash: String::new(),
        event_hash: String::new(),
    })?;
    Ok(())
}
