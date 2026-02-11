use crate::adapters::interface::{
    enforce_loopback_endpoint, AdapterCapabilitiesResponse, AdapterClient, AdapterHealthResponse,
    ResolveModelRequest, ResolveModelResponse,
};
use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredAdapter {
    pub adapter_id: String,
    pub endpoint: String,
    pub adapter_type: String,
}

pub struct AdapterRuntime<C: AdapterClient> {
    clients: Vec<C>,
}

impl<C: AdapterClient> AdapterRuntime<C> {
    pub fn new(clients: Vec<C>) -> Self {
        Self { clients }
    }

    pub fn validate_loopback_only(&self) -> CoreResult<()> {
        for c in &self.clients {
            enforce_loopback_endpoint(c.endpoint())?;
        }
        Ok(())
    }

    pub fn health_all(&self) -> CoreResult<Vec<AdapterHealthResponse>> {
        let mut out = Vec::new();
        for c in &self.clients {
            out.push(c.health()?);
        }
        Ok(out)
    }

    pub fn capabilities_all(&self) -> CoreResult<Vec<AdapterCapabilitiesResponse>> {
        let mut out = Vec::new();
        for c in &self.clients {
            out.push(c.capabilities()?);
        }
        Ok(out)
    }

    pub fn resolve_model_for(
        &self,
        adapter_id: &str,
        req: ResolveModelRequest,
    ) -> CoreResult<ResolveModelResponse> {
        for c in &self.clients {
            let h = c.health()?;
            if h.adapter_id == adapter_id {
                return c.resolve_model(req);
            }
        }
        Err(CoreError::InvalidInput(format!(
            "adapter not found: {}",
            adapter_id
        )))
    }
}
