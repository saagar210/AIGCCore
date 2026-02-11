use crate::error::{CoreError, CoreResult};
use idna::domain_to_ascii;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowlistEntry {
    pub scheme: String, // http|https
    pub host: String,   // ASCII; punycode normalized
    pub port: u16,      // explicit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    pub purpose: String,
    pub policy_pack_id: String,
    pub policy_pack_version: String,
}

impl AllowlistEntry {
    pub fn canonicalize(mut self) -> CoreResult<Self> {
        let scheme = self.scheme.to_ascii_lowercase();
        if scheme != "https" && scheme != "http" {
            return Err(CoreError::InvalidInput(
                "allowlist scheme must be http or https".to_string(),
            ));
        }
        self.scheme = scheme;

        let host_ascii = domain_to_ascii(&self.host)
            .map_err(|_| CoreError::InvalidInput("invalid allowlist host".to_string()))?;
        self.host = host_ascii.to_ascii_lowercase();

        if self.port == 0 {
            self.port = if self.scheme == "https" { 443 } else { 80 };
        }

        if let Some(pp) = &self.path_prefix {
            let mut p = pp.replace('\\', "/");
            if !p.starts_with('/') {
                p = format!("/{}", p);
            }
            if p.contains("..") {
                return Err(CoreError::InvalidInput(
                    "allowlist path_prefix must not contain ..".to_string(),
                ));
            }
            self.path_prefix = Some(p);
        }

        Ok(self)
    }

    pub fn matches_url(&self, url: &Url) -> bool {
        let scheme = url.scheme().to_ascii_lowercase();
        let host = match url.host_str() {
            Some(h) => match domain_to_ascii(h) {
                Ok(x) => x.to_ascii_lowercase(),
                Err(_) => return false,
            },
            None => return false,
        };
        let port = url
            .port_or_known_default()
            .unwrap_or(if scheme == "https" { 443 } else { 80 });
        if scheme != self.scheme || host != self.host || port != self.port {
            return false;
        }
        if let Some(pp) = &self.path_prefix {
            return url.path().starts_with(pp);
        }
        true
    }
}
