use std::net::IpAddr;

use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEndpointScope {
    normalized_base_url: String,
    origin: String,
    path_prefix: String,
    scope_id: String,
}

impl ProviderEndpointScope {
    pub fn parse(value: &str) -> Result<Self, String> {
        let value = value.trim();
        if value.is_empty() {
            return Err(String::from("provider endpoint must not be empty"));
        }

        let mut parsed = Url::parse(value)
            .map_err(|error| format!("provider endpoint must be an absolute URL: {error}"))?;
        if parsed.username() != "" || parsed.password().is_some() {
            return Err(String::from(
                "provider endpoint must not contain embedded credentials",
            ));
        }
        if parsed.query().is_some() || parsed.fragment().is_some() {
            return Err(String::from(
                "provider endpoint must not contain a query string or fragment",
            ));
        }
        let host = parsed
            .host_str()
            .ok_or_else(|| String::from("provider endpoint must include a host"))?;
        match parsed.scheme() {
            "https" => {}
            "http" if is_loopback_host(host) => {}
            "http" => {
                return Err(String::from(
                    "unencrypted provider endpoints are permitted only for loopback hosts",
                ));
            }
            _ => {
                return Err(String::from(
                    "provider endpoint scheme must be https, or http for a loopback host",
                ));
            }
        }

        parsed.set_query(None);
        parsed.set_fragment(None);
        let path = parsed.path().trim_end_matches('/');
        let path_prefix = if path.is_empty() {
            String::from("/")
        } else {
            path.to_string()
        };
        parsed.set_path(&path_prefix);

        let origin = parsed.origin().ascii_serialization();
        let normalized_base_url = if path_prefix == "/" {
            origin.clone()
        } else {
            format!("{origin}{path_prefix}")
        };
        let scope_id = format!("{:x}", Sha256::digest(normalized_base_url.as_bytes()));

        Ok(Self {
            normalized_base_url,
            origin,
            path_prefix,
            scope_id,
        })
    }

    pub fn normalized_base_url(&self) -> &str {
        &self.normalized_base_url
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    pub fn path_prefix(&self) -> &str {
        &self.path_prefix
    }

    pub fn scope_id(&self) -> &str {
        &self.scope_id
    }

    pub fn models_url(&self) -> String {
        if self.path_prefix == "/" {
            format!("{}/models", self.origin)
        } else {
            format!("{}/models", self.normalized_base_url)
        }
    }
}

fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    host.parse::<IpAddr>()
        .map(|address| address.is_loopback())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_scheme_host_default_port_and_trailing_slash() {
        let scope = ProviderEndpointScope::parse(" HTTPS://API.EXAMPLE.COM:443/v1/ ")
            .expect("endpoint should normalize");
        assert_eq!(scope.normalized_base_url(), "https://api.example.com/v1");
        assert_eq!(scope.origin(), "https://api.example.com");
        assert_eq!(scope.path_prefix(), "/v1");
        assert_eq!(scope.models_url(), "https://api.example.com/v1/models");
    }

    #[test]
    fn scope_changes_with_scheme_host_port_or_path() {
        let baseline = ProviderEndpointScope::parse("https://api.example.com/v1").unwrap();
        for changed in [
            "https://other.example.com/v1",
            "https://api.example.com:8443/v1",
            "https://api.example.com/v2",
        ] {
            let changed = ProviderEndpointScope::parse(changed).unwrap();
            assert_ne!(baseline.scope_id(), changed.scope_id());
        }
    }

    #[test]
    fn rejects_credentials_query_fragment_and_unsafe_http() {
        for endpoint in [
            "https://user:pass@api.example.com/v1",
            "https://api.example.com/v1?token=secret",
            "https://api.example.com/v1#fragment",
            "http://api.example.com/v1",
            "file:///tmp/provider",
        ] {
            assert!(ProviderEndpointScope::parse(endpoint).is_err(), "{endpoint}");
        }
    }

    #[test]
    fn permits_http_only_for_loopback() {
        assert!(ProviderEndpointScope::parse("http://localhost:11434/v1").is_ok());
        assert!(ProviderEndpointScope::parse("http://127.0.0.1:11434/v1").is_ok());
        assert!(ProviderEndpointScope::parse("http://[::1]:11434/v1").is_ok());
    }
}
