use super::*;

#[test]
fn post_p8_url_sanitization_reconstructs_approved_components() {
    let mut metadata = SanitizationMetadata::default();
    let safe = sanitize_url(
        "https://user:pass@example.com:8443/safe/path?token=secret#fragment",
        &mut metadata,
    );
    assert_eq!(safe.0, "https://example.com:8443/safe/path");
    assert_eq!(metadata.query_values_removed, 1);
    assert!(!safe.0.contains("user"));
    assert!(!safe.0.contains("pass"));
    assert!(!safe.0.contains("token"));
    assert!(!safe.0.contains("fragment"));

    let malformed = sanitize_url("https://[invalid?token=secret", &mut metadata);
    assert_eq!(malformed.0, "[REDACTED INVALID URL]");
}
