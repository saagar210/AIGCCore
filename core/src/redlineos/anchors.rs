use crate::determinism::run_id::sha256_hex;

pub fn stable_clause_anchor(clause_text: &str) -> String {
    let normalized = clause_text.split_whitespace().collect::<Vec<_>>().join(" ");
    let digest = sha256_hex(normalized.as_bytes());
    format!("clause_{}", &digest[..16])
}
