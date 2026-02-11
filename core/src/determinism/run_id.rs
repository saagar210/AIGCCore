use crate::error::{CoreError, CoreResult};
use sha2::{Digest, Sha256};
use ulid::Ulid;

// Phase_2_5_Lock_Addendum_v2.5-lock-4.md §3.2
pub fn run_id_from_manifest_inputs_fingerprint_hex32(
    manifest_inputs_fingerprint_hex: &str,
) -> CoreResult<String> {
    let hex = manifest_inputs_fingerprint_hex.trim();
    if hex.len() < 32 || !hex.chars().take(32).all(|c| c.is_ascii_hexdigit()) {
        return Err(CoreError::InvalidInput(
            "manifest_inputs_fingerprint must be hex with length >= 32".to_string(),
        ));
    }
    Ok(format!("r_{}", hex[..32].to_ascii_lowercase()))
}

pub fn run_id_ulid() -> String {
    format!("r_{}", Ulid::new().to_string())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}
