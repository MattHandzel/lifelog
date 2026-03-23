use std::collections::HashSet;
use std::sync::Arc;

use lifelog_core::PrivacyLevel;

#[derive(Debug, Clone)]
pub struct EgressGuard {
    allowed_hosts: Arc<HashSet<String>>,
}

impl EgressGuard {
    pub fn new(allowed_hosts: Vec<String>) -> Self {
        Self {
            allowed_hosts: Arc::new(allowed_hosts.into_iter().collect()),
        }
    }

    pub fn check(&self, endpoint: &str, privacy_level: PrivacyLevel) -> Result<(), EgressError> {
        if privacy_level == PrivacyLevel::LocalOnly {
            if !is_local_endpoint(endpoint) {
                return Err(EgressError::LocalOnlyViolation {
                    endpoint: endpoint.to_string(),
                });
            }
            return Ok(());
        }

        if is_local_endpoint(endpoint) {
            return Ok(());
        }

        let host = extract_host(endpoint).ok_or_else(|| EgressError::InvalidEndpoint {
            endpoint: endpoint.to_string(),
        })?;

        if self.allowed_hosts.is_empty() {
            return Err(EgressError::NoAllowedHosts {
                host,
                endpoint: endpoint.to_string(),
            });
        }

        if !self.allowed_hosts.contains(&host) {
            return Err(EgressError::HostNotAllowed {
                host,
                endpoint: endpoint.to_string(),
            });
        }

        Ok(())
    }
}

fn extract_host(endpoint: &str) -> Option<String> {
    let without_scheme = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"))
        .unwrap_or(endpoint);
    let host_port = without_scheme.split('/').next()?;
    let host = if host_port.starts_with('[') {
        host_port.split(']').next().map(|h| format!("{}]", h))
    } else {
        host_port.split(':').next().map(|h| h.to_string())
    };
    host.filter(|h| !h.is_empty())
}

fn is_local_endpoint(endpoint: &str) -> bool {
    let host = match extract_host(endpoint) {
        Some(h) => h,
        None => return false,
    };
    host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host == "[::1]"
        || host.ends_with(".local")
        || host.starts_with("192.168.")
        || host.starts_with("10.")
        || host.starts_with("172.16.")
}

#[derive(Debug, thiserror::Error)]
pub enum EgressError {
    #[error("local_only transform attempted non-local endpoint: {endpoint}")]
    LocalOnlyViolation { endpoint: String },
    #[error("cannot parse host from endpoint: {endpoint}")]
    InvalidEndpoint { endpoint: String },
    #[error(
        "no allowed_hosts configured but transform targets external host {host} at {endpoint}"
    )]
    NoAllowedHosts { host: String, endpoint: String },
    #[error("host {host} not in allowed_hosts for endpoint {endpoint}")]
    HostNotAllowed { host: String, endpoint: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_endpoints_always_allowed() {
        let guard = EgressGuard::new(vec![]);
        assert!(guard
            .check("http://localhost:47770", PrivacyLevel::Standard)
            .is_ok());
        assert!(guard
            .check("http://127.0.0.1:11434", PrivacyLevel::Standard)
            .is_ok());
        assert!(guard
            .check("http://192.168.1.100:8080", PrivacyLevel::Zdr)
            .is_ok());
    }

    #[test]
    fn external_blocked_when_no_allowlist() {
        let guard = EgressGuard::new(vec![]);
        assert!(guard
            .check("https://api.openai.com/v1/audio", PrivacyLevel::Standard)
            .is_err());
    }

    #[test]
    fn external_allowed_when_in_allowlist() {
        let guard = EgressGuard::new(vec!["api.openai.com".to_string()]);
        assert!(guard
            .check("https://api.openai.com/v1/audio", PrivacyLevel::Standard)
            .is_ok());
    }

    #[test]
    fn external_blocked_when_not_in_allowlist() {
        let guard = EgressGuard::new(vec!["api.openai.com".to_string()]);
        assert!(guard
            .check("https://evil.com/steal", PrivacyLevel::Standard)
            .is_err());
    }

    #[test]
    fn local_only_rejects_non_local() {
        let guard = EgressGuard::new(vec!["api.openai.com".to_string()]);
        assert!(guard
            .check("https://api.openai.com/v1/audio", PrivacyLevel::LocalOnly)
            .is_err());
    }

    #[test]
    fn local_only_accepts_local() {
        let guard = EgressGuard::new(vec![]);
        assert!(guard
            .check("http://localhost:47770/v1/audio", PrivacyLevel::LocalOnly)
            .is_ok());
    }
}
