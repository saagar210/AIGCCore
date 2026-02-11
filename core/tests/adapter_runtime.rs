use aigc_core::adapters::interface::{
    AdapterCapabilitiesResponse, AdapterClient, AdapterHealthResponse, AdapterModel,
    ResolveModelRequest, ResolveModelResponse,
};
use aigc_core::adapters::runtime::AdapterRuntime;
use aigc_core::error::CoreResult;

#[derive(Clone)]
struct FakeAdapter {
    endpoint: String,
    adapter_id: String,
}

impl AdapterClient for FakeAdapter {
    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn health(&self) -> CoreResult<AdapterHealthResponse> {
        Ok(AdapterHealthResponse {
            status: "ok".to_string(),
            adapter_id: self.adapter_id.clone(),
            adapter_version: "1.0.0".to_string(),
            uptime_ms: 1,
        })
    }

    fn capabilities(&self) -> CoreResult<AdapterCapabilitiesResponse> {
        Ok(AdapterCapabilitiesResponse {
            adapter_type: "LLM".to_string(),
            features: vec!["json_schema".to_string()],
            limits: serde_json::json!({"max_input_bytes": 1}),
            models: vec![AdapterModel {
                model_id: "m".to_string(),
                model_sha256: None,
                quantization: None,
                context_window: None,
                notes: None,
            }],
        })
    }

    fn resolve_model(&self, _req: ResolveModelRequest) -> CoreResult<ResolveModelResponse> {
        Ok(ResolveModelResponse {
            resolved_model: AdapterModel {
                model_id: "m".to_string(),
                model_sha256: None,
                quantization: None,
                context_window: None,
                notes: None,
            },
            rationale: "ok".to_string(),
        })
    }
}

#[test]
fn runtime_rejects_non_loopback_adapter_endpoint() {
    let rt = AdapterRuntime::new(vec![FakeAdapter {
        endpoint: "http://8.8.8.8:1234".to_string(),
        adapter_id: "a1".to_string(),
    }]);
    assert!(rt.validate_loopback_only().is_err());
}

#[test]
fn runtime_returns_health_and_capabilities() {
    let rt = AdapterRuntime::new(vec![FakeAdapter {
        endpoint: "http://127.0.0.1:1234".to_string(),
        adapter_id: "a1".to_string(),
    }]);
    assert!(rt.validate_loopback_only().is_ok());
    assert_eq!(rt.health_all().unwrap().len(), 1);
    assert_eq!(rt.capabilities_all().unwrap().len(), 1);
}
