//! Shared web-only URL policy for browser navigation.
//!
//! Planner validation (`commands::validators::navigation`) and runtime execution
//! (`app_core::navigation_tools`) both go through [`normalize_browser_navigation_url`]
//! so the two paths cannot drift. Internal browser navigation is `http`/`https`
//! only and fails closed on every other scheme (`file:`, `javascript:`, `data:`,
//! `chrome:`, `about:`, scheme-relative `//host`, authority-less `https:///path`,
//! and `http:host` without `//`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlPolicyError {
    Empty,
    MissingScheme { url: String },
    InvalidScheme { url: String, scheme: String },
    UnsupportedScheme { url: String, scheme: String },
    MissingAuthority { url: String, scheme: String },
}

impl UrlPolicyError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "empty_url",
            Self::MissingScheme { .. } => "missing_scheme",
            Self::InvalidScheme { .. } => "invalid_scheme",
            Self::UnsupportedScheme { .. } => "unsupported_scheme",
            Self::MissingAuthority { .. } => "missing_authority",
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

    let Some(separator_index) = trimmed.find(':') else {
        return Err(UrlPolicyError::MissingScheme {
            url: trimmed.to_string(),
        });
    };

    let scheme = trimmed[..separator_index].to_ascii_lowercase();
    let valid_scheme = scheme.chars().enumerate().all(|(index, ch)| match index {
        0 => ch.is_ascii_alphabetic(),
        _ => ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'),
    });
    if !valid_scheme {
        return Err(UrlPolicyError::InvalidScheme {
            url: trimmed.to_string(),
            scheme,
        });
    }

    if !is_allowed_browser_navigation_scheme(&scheme) {
        return Err(UrlPolicyError::UnsupportedScheme {
            url: trimmed.to_string(),
            scheme,
        });
    }

    let after_scheme = &trimmed[separator_index + 1..];
    if !after_scheme.starts_with("//") {
        return Err(UrlPolicyError::MissingAuthority {
            url: trimmed.to_string(),
            scheme,
        });
    }

    let authority = after_scheme[2..]
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim();
    if authority.is_empty() {
        return Err(UrlPolicyError::MissingAuthority {
            url: trimmed.to_string(),
            scheme,
        });
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_urls_with_hosts() {
        assert_eq!(
            normalize_browser_navigation_url("  https://example.com/path  ").unwrap(),
            "https://example.com/path"
        );
        assert_eq!(
            normalize_browser_navigation_url("http://localhost:3000").unwrap(),
            "http://localhost:3000"
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
}
