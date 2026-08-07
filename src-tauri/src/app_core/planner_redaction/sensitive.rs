//! Marker-based detection of credential-shaped, PII-shaped, or otherwise
//! sensitive content, used to decide what [`super::sanitize`] must redact
//! outright rather than merely truncate.

use crate::page_model::InteractiveElement;

const SENSITIVE_MARKERS: &[&str] = &[
    "password=",
    "password:",
    "password is ",
    "passwd=",
    "passwd:",
    "secret=",
    "secret:",
    "token=",
    "token:",
    "access_token=",
    "id_token=",
    "api_key=",
    "apikey=",
    "authorization:",
    "bearer ",
    "one-time code",
    "one time code",
    "otp=",
    "otp:",
    "security answer",
];

const SENSITIVE_ELEMENT_MARKERS: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "csrf",
    "authorization",
    "api_key",
    "apikey",
    "one-time-code",
    "current-password",
    "new-password",
    "credit-card",
    "cc-number",
    "cc-csc",
    "security-answer",
    "social-security",
    "ssn",
];

pub(super) fn contains_sensitive_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if SENSITIVE_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return true;
    }

    value.split_whitespace().any(is_credential_shaped_token)
        || contains_long_digit_sequence(value)
        || contains_ssn_shape(value)
}

fn is_credential_shaped_token(token: &str) -> bool {
    let trimmed = token.trim_matches(|character: char| {
        character.is_ascii_punctuation() && !matches!(character, '-' | '_' | '.')
    });
    let lower = trimmed.to_ascii_lowercase();

    if ["sk-", "ghp_", "github_pat_", "xoxb-", "xoxp-", "akia"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
        && trimmed.len() >= 16
    {
        return true;
    }

    let jwt_parts = trimmed.split('.').collect::<Vec<_>>();
    jwt_parts.len() == 3
        && jwt_parts.iter().all(|part| part.len() >= 8)
        && jwt_parts.iter().all(|part| {
            part.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
        })
}

fn contains_long_digit_sequence(value: &str) -> bool {
    let mut run = 0usize;
    for character in value.chars() {
        if character.is_ascii_digit() {
            run += 1;
            if (13..=19).contains(&run) {
                return true;
            }
        } else if !matches!(character, ' ' | '-') {
            run = 0;
        }
    }
    false
}

fn contains_ssn_shape(value: &str) -> bool {
    value.as_bytes().windows(11).any(|window| {
        window[0..3].iter().all(u8::is_ascii_digit)
            && window[3] == b'-'
            && window[4..6].iter().all(u8::is_ascii_digit)
            && window[6] == b'-'
            && window[7..11].iter().all(u8::is_ascii_digit)
    })
}

pub(super) fn is_sensitive_element(element: &InteractiveElement) -> bool {
    let mut descriptors = vec![element.tag_name.to_ascii_lowercase()];
    descriptors.extend(
        ["type", "name", "id", "autocomplete"]
            .into_iter()
            .filter_map(|name| element.attributes.get(name))
            .map(|value| value.to_ascii_lowercase()),
    );
    if let Some(placeholder) = &element.placeholder {
        descriptors.push(placeholder.to_ascii_lowercase());
    }
    if let Some(accessible_name) = &element.accessible_name {
        descriptors.push(accessible_name.to_ascii_lowercase());
    }

    let combined = descriptors.join(" ");
    SENSITIVE_ELEMENT_MARKERS
        .iter()
        .any(|marker| combined.contains(marker))
        || element
            .attributes
            .get("type")
            .is_some_and(|kind| matches!(kind.to_ascii_lowercase().as_str(), "password" | "hidden"))
}

pub(super) fn contains_high_risk_page_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    contains_long_digit_sequence(value)
        || contains_ssn_shape(value)
        || [
            "payment receipt",
            "card number",
            "credit card",
            "security code",
            "cvv",
            "cvc",
            "social security",
            "medical record",
            "patient record",
            "wallet seed",
            "seed phrase",
            "recovery phrase",
            "one-time code",
            "one time code",
            "otp code",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
}
