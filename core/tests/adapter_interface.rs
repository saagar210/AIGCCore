use aigc_core::adapters::interface::{classify_adapter_error, enforce_loopback_endpoint};

#[test]
fn loopback_endpoint_is_enforced() {
    assert!(enforce_loopback_endpoint("http://127.0.0.1:11434").is_ok());
    assert!(enforce_loopback_endpoint("http://192.168.1.8:11434").is_err());
}

#[test]
fn adapter_error_envelope_categories_are_stable() {
    let t = classify_adapter_error("timeout while waiting");
    assert_eq!(t.error.category, "TIMEOUT");
    let nf = classify_adapter_error("model not found");
    assert_eq!(nf.error.category, "MODEL_NOT_FOUND");
    let ns = classify_adapter_error("unsupported feature");
    assert_eq!(ns.error.category, "NOT_SUPPORTED");
}
