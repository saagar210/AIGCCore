use aigc_core::adapters::pinning::PinningLevel;
use aigc_core::policy::export_gate::{evaluate_export_gate, ExportBlockReason, ExportGateInputs};
use aigc_core::policy::types::{NetworkMode, PolicyMode, ProofLevel};

#[test]
fn strict_blocks_name_only_pinning() {
    let r = evaluate_export_gate(&ExportGateInputs {
        policy_mode: PolicyMode::STRICT,
        pinning_level: PinningLevel::NAME_ONLY,
        citations_required_passed: true,
        redactions_required_passed: true,
        blocker_gate_failures: vec![],
        determinism_passed: true,
        network_mode: NetworkMode::OFFLINE,
        proof_level: ProofLevel::OFFLINE_STRICT,
    });
    assert_eq!(r.err(), Some(ExportBlockReason::INSUFFICIENT_PINNING));
}

#[test]
fn strict_requires_offline_strict_proof() {
    let r = evaluate_export_gate(&ExportGateInputs {
        policy_mode: PolicyMode::STRICT,
        pinning_level: PinningLevel::CRYPTO_PINNED,
        citations_required_passed: true,
        redactions_required_passed: true,
        blocker_gate_failures: vec![],
        determinism_passed: true,
        network_mode: NetworkMode::ONLINE_ALLOWLISTED,
        proof_level: ProofLevel::ONLINE_ALLOWLIST_CORE_ONLY,
    });
    assert_eq!(r.err(), Some(ExportBlockReason::OFFLINE_PROOF_INSUFFICIENT));
}

#[test]
fn balanced_passes_when_requirements_met() {
    let r = evaluate_export_gate(&ExportGateInputs {
        policy_mode: PolicyMode::BALANCED,
        pinning_level: PinningLevel::VERSION_PINNED,
        citations_required_passed: true,
        redactions_required_passed: true,
        blocker_gate_failures: vec![],
        determinism_passed: true,
        network_mode: NetworkMode::ONLINE_ALLOWLISTED,
        proof_level: ProofLevel::ONLINE_ALLOWLIST_CORE_ONLY,
    });
    assert!(r.is_ok());
}
