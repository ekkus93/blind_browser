//! Shared web-only URL policy for browser navigation.
//!
//! Planner validation (`commands::validators::navigation`) and runtime execution
//! (`app_core::navigation_tools`) both go through [`normalize_browser_navigation_url`]
//! so the two paths cannot drift. Internal browser navigation is `http`/`https`
//! only and fails closed on every other scheme (`file:`, `javascript:`, `data:`,
//! `chrome:`, `about:`, scheme-relative `//host`, authority-less `https:///path`,
//! `http:host` without `//`, and URLs with embedded whitespace or control characters).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlPolicyError {
    Empty,
    MissingScheme { url: String },
    InvalidScheme { url: String, scheme: String },
    UnsupportedScheme { url: String, scheme: String },
    MissingAuthority { url: String, scheme: String },
    InvalidUrl { url: String, reason: String },
}

impl UrlPolicyError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "empty_url",
            Self::MissingScheme { .. } => "missing_scheme",
            Self::InvalidScheme { .. } => "invalid_scheme",
            Self::UnsupportedScheme { .. } => "unsupported_scheme",
            Self::MissingAuthority { .. } => "missing_authority",
            Self::InvalidUrl { .. } => "invalid_url",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Empty => "open_url requires a non-empty URL",
            Self::MissingScheme { .. } => "open_url requires an absolute http or https URL",
            Self::InvalidScheme { .. } => "open_url requires a URL with a valid scheme",
            Self::UnsupportedScheme { .. } => "open_url only supports http and https URLs",
            Self::MissingAuthority { .. } => {
                "open_url requires http/https URLs to include // and a host"
            }
            Self::InvalidUrl { .. } => "open_url requires a valid absolute http or https URL",
        }
    }

    pub fn details(&self) -> serde_json::Value {
        match self {
            Self::Empty => serde_json::json!({}),
            Self::MissingScheme { url } => serde_json::json!({ "url": url }),
            Self::InvalidScheme { url, scheme }
            | Self::UnsupportedScheme { url, scheme }
            | Self::MissingAuthority { url, scheme } => {
                serde_json::json!({ "url": url, "scheme": scheme })
            }
            Self::InvalidUrl { url, reason } => serde_json::json!({ "url": url, "reason": reason }),
        }
    }
}

pub fn is_allowed_browser_navigation_scheme(scheme: &str) -> bool {
    matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https")
}

pub fn normalize_browser_navigation_url(raw: &str) -> Result<String, UrlPolicyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(UrlPolicyError::Empty);
    }

    // Reject embedded whitespace or control characters before handing to the parser,
    // because some parsers silently strip or reinterpret such characters.
    if trimmed.chars().any(|ch| ch.is_control() || ch.is_whitespace()) {
        return Err(UrlPolicyError::InvalidUrl {
            url: trimmed.to_string(),
            reason: String::from("URL contains whitespace or control characters"),
        });
    }

    // Require "://" in the original string. The url crate (WHATWG URL Standard)
    // accepts "http:example.com" for special schemes, treating the trailing token
    // as the host even without the authority prefix. Reject all such inputs before
    // calling the parser so the scheme and host checks below are not bypassed.
    if !trimmed.contains("://") {
        return Err(UrlPolicyError::MissingScheme {
            url: trimmed.to_string(),
        });
    }

    // Pre-parse: extract the raw authority (between "://" and the next "/", "?", "#", or
    // end-of-string) and verify the host portion is non-empty. This is done before calling
    // url::Url::parse because the url crate (WHATWG URL Standard) normalizes some forms
    // (e.g. "https:///path" → authority="" but host="path") in ways that our host_str()
    // check alone cannot reliably catch.
    {
        let after_sep = &trimmed[trimmed.find("://").unwrap() + 3..];
        let authority =
            &after_sep[..after_sep.find(['/', '?', '#']).unwrap_or(after_sep.len())];
        // Strip userinfo ("user:pass@") and port (":N") to isolate the raw host.
        let host_part = authority.rsplit('@').next().unwrap_or(authority);
        let host_only = host_part.split(':').next().unwrap_or(host_part);
        if host_only.is_empty() {
            let raw_scheme = trimmed[..trimmed.find("://").unwrap()].to_ascii_lowercase();
            return Err(UrlPolicyError::MissingAuthority {
                url: trimmed.to_string(),
                scheme: raw_scheme,
            });
        }
    }

    let parsed = url::Url::parse(trimmed).map_err(|error| UrlPolicyError::InvalidUrl {
        url: trimmed.to_string(),
        reason: error.to_string(),
    })?;

    let scheme = parsed.scheme().to_ascii_lowercase();
    if !is_allowed_browser_navigation_scheme(&scheme) {
        return Err(UrlPolicyError::UnsupportedScheme {
            url: trimmed.to_string(),
            scheme,
        });
    }

    if parsed.host_str().is_none() {
        return Err(UrlPolicyError::MissingAuthority {
            url: trimmed.to_string(),
            scheme,
        });
    }

    Ok(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_urls_with_hosts() {
        // The url crate normalizes bare-host URLs by appending a trailing slash.
        assert_eq!(
            normalize_browser_navigation_url("  https://example.com/path  ").unwrap(),
            "https://example.com/path"
        );
        assert_eq!(
            normalize_browser_navigation_url("http://localhost:3000/").unwrap(),
            "http://localhost:3000/"
        );
        assert_eq!(
            normalize_browser_navigation_url("https://example.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn rejects_non_web_and_malformed_urls() {
        for raw in [
            "",
            "   ",
            "/relative/path",
            "//example.com",
            "about:blank",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html,<h1>x</h1>",
            "chrome://version",
            "http:example.com",
            "https:///missing-host",
        ] {
            assert!(
                normalize_browser_navigation_url(raw).is_err(),
                "expected URL to be rejected: {raw}"
            );
        }
    }

    #[test]
    fn rejects_malformed_http_authorities() {
        for raw in [
            "http://:80",
            "https://",
            "https:///missing-host",
            "https://exa mple.com",
            "https://\nexample.com",
            "https://\texample.com",
        ] {
            assert!(
                normalize_browser_navigation_url(raw).is_err(),
                "expected malformed URL to be rejected: {raw:?}"
            );
        }
    }
}
