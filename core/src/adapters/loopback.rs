use crate::error::{CoreError, CoreResult};
use std::net::IpAddr;

pub fn is_loopback_endpoint(endpoint: &str) -> CoreResult<bool> {
    // Expect URL like http://127.0.0.1:port
    let url = url::Url::parse(endpoint)
        .map_err(|_| CoreError::InvalidInput("invalid adapter endpoint URL".to_string()))?;
    let host = url
        .host_str()
        .ok_or_else(|| CoreError::InvalidInput("adapter endpoint missing host".to_string()))?;
    let ip: IpAddr = host.parse().map_err(|_| {
        CoreError::InvalidInput("adapter endpoint host must be an IP address".to_string())
    })?;
    Ok(ip.is_loopback())
}
