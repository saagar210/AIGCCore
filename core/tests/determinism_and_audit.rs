use aigc_core::audit::event::{finalize_event, Actor, AuditEvent};
use aigc_core::determinism::json_canonical::to_canonical_bytes;
use aigc_core::determinism::run_id::run_id_from_manifest_inputs_fingerprint_hex32;

#[test]
fn canonical_json_is_stable_for_key_order() {
    let a = serde_json::json!({"b": 1, "a": {"y": 2, "x": 3}});
    let b = serde_json::json!({"a": {"x": 3, "y": 2}, "b": 1});
    let ca = to_canonical_bytes(&a).unwrap();
    let cb = to_canonical_bytes(&b).unwrap();
    assert_eq!(ca, cb);
}

#[test]
fn event_hash_is_stable() {
    let ev = AuditEvent {
        ts_utc: "2026-02-10T00:00:00Z".to_string(),
        event_type: "RUN_CREATED".to_string(),
        run_id: "r_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        vault_id: "v_1".to_string(),
        actor: Actor::System,
        details: serde_json::json!({"pack_id":"p","pack_version":"1","policy_pack_id":"x","policy_pack_version":"1","determinism_enabled":true}),
        prev_event_hash: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        event_hash: "".to_string(),
    };
    let a = finalize_event(ev.clone()).unwrap().event_hash;
    let b = finalize_event(ev).unwrap().event_hash;
    assert_eq!(a, b);
}

#[test]
fn deterministic_run_id_rule() {
    let fp = "1234567890abcdef1234567890abcdef9999";
    let run = run_id_from_manifest_inputs_fingerprint_hex32(fp).unwrap();
    assert_eq!(run, "r_1234567890abcdef1234567890abcdef");
}
