use crate::adapters::loopback::is_loopback_endpoint;
use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterErrorEnvelope {
    pub error: AdapterError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub category: String, // INVALID_INPUT|MODEL_NOT_FOUND|OUT_OF_MEMORY|TIMEOUT|RUNTIME_ERROR|SAFETY_REFUSAL|NOT_SUPPORTED
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterHealthResponse {
    pub status: String,
    pub adapter_id: String,
    pub adapter_version: String,
    pub uptime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterModel {
    pub model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapabilitiesResponse {
    pub adapter_type: String, // LLM|VLM|STT|EMB
    pub features: Vec<String>,
    pub limits: serde_json::Value,
    pub models: Vec<AdapterModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveModelRequest {
    pub preferred_model: String,
    pub constraints: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveModelResponse {
    pub resolved_model: AdapterModel,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCallMeta {
    pub call_id: String,
    pub model_id: String,
    pub adapter_version: String,
    pub input_hash: String,
    pub output_hash: String,
    pub duration_ms: u64,
    pub usage: serde_json::Value,
    pub status: String,
}

pub trait AdapterClient {
    fn endpoint(&self) -> &str;
    fn health(&self) -> CoreResult<AdapterHealthResponse>;
    fn capabilities(&self) -> CoreResult<AdapterCapabilitiesResponse>;
    fn resolve_model(&self, req: ResolveModelRequest) -> CoreResult<ResolveModelResponse>;
}

pub fn enforce_loopback_endpoint(endpoint: &str) -> CoreResult<()> {
    if !is_loopback_endpoint(endpoint)? {
        return Err(CoreError::PolicyBlocked(
            "adapter endpoint rejected: not loopback (127.0.0.1/::1)".to_string(),
        ));
    }
    Ok(())
}

pub fn classify_adapter_error(err: &str) -> AdapterErrorEnvelope {
    let (category, code, retryable) = if err.contains("timeout") {
        ("TIMEOUT", "ADAPTER_TIMEOUT", true)
    } else if err.contains("not found") {
        ("MODEL_NOT_FOUND", "MODEL_NOT_FOUND", false)
    } else if err.contains("unsupported") {
        ("NOT_SUPPORTED", "NOT_SUPPORTED", false)
    } else {
        ("RUNTIME_ERROR", "RUNTIME_ERROR", false)
    };
    AdapterErrorEnvelope {
        error: AdapterError {
            code: code.to_string(),
            message: err.to_string(),
            retryable,
            category: category.to_string(),
            details: serde_json::json!({}),
        },
    }
}
