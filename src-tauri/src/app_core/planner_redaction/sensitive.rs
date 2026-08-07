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

/// Folds a narrow, hand-picked set of Unicode compatibility/confusable
/// characters to their canonical ASCII form before marker matching, so a
/// sensitive-content marker (e.g. "password=") isn't defeated by rendering
/// it in fullwidth form or substituting a handful of common homoglyphs from
/// other scripts.
///
/// This deliberately is *not* general-purpose Unicode confusable detection
/// (Unicode TR39's full skeleton algorithm -- e.g. the `unicode-security`
/// crate -- would be needed for that, and adding a dependency needs
/// sign-off this pass doesn't have). It's narrow and dependency-free
/// instead, and for good reason, verified empirically: NFKC normalization
/// (this item's original literal instruction) does *not* fold the Cyrillic
/// homoglyph in the motivating example ("pаssword=" with a Cyrillic а,
/// U+0430) to its Latin equivalent -- NFKC only unifies compatibility
/// variants of the *same* character, like fullwidth forms, not lookalikes
/// across different scripts. It would not have closed the gap it was
/// proposed to close. NFKC *does* correctly fold fullwidth Latin letters
/// and fullwidth digits, which this function also covers (the fixed
/// -0xFEE0 offset over the Halfwidth and Fullwidth Forms block is exactly
/// what NFKC does for that block, confirmed against Python's
/// `unicodedata.normalize`).
fn fold_confusable_ascii(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if ('\u{FF01}'..='\u{FF5E}').contains(&character) {
                char::from_u32(character as u32 - 0xFEE0).unwrap_or(character)
            } else {
                fold_common_homoglyph(character)
            }
        })
        .collect()
}

/// Folds the small set of Cyrillic and Greek letters that are visually
/// indistinguishable from a Latin letter appearing in this module's marker
/// vocabulary (a-z). Not exhaustive of Unicode's confusables table --
/// scoped to what our own marker words are built from.
fn fold_common_homoglyph(character: char) -> char {
    match character {
        'а' => 'a', // U+0430 CYRILLIC SMALL LETTER A
        'е' => 'e', // U+0435 CYRILLIC SMALL LETTER IE
        'о' => 'o', // U+043E CYRILLIC SMALL LETTER O
        'р' => 'p', // U+0440 CYRILLIC SMALL LETTER ER
        'с' => 'c', // U+0441 CYRILLIC SMALL LETTER ES
        'у' => 'y', // U+0443 CYRILLIC SMALL LETTER U
        'х' => 'x', // U+0445 CYRILLIC SMALL LETTER HA
        'А' => 'A', // U+0410 CYRILLIC CAPITAL LETTER A
        'В' => 'B', // U+0412 CYRILLIC CAPITAL LETTER VE
        'Е' => 'E', // U+0415 CYRILLIC CAPITAL LETTER IE
        'К' => 'K', // U+041A CYRILLIC CAPITAL LETTER KA
        'М' => 'M', // U+041C CYRILLIC CAPITAL LETTER EM
        'Н' => 'H', // U+041D CYRILLIC CAPITAL LETTER EN
        'О' => 'O', // U+041E CYRILLIC CAPITAL LETTER O
        'Р' => 'P', // U+0420 CYRILLIC CAPITAL LETTER ER
        'С' => 'C', // U+0421 CYRILLIC CAPITAL LETTER ES
        'Т' => 'T', // U+0422 CYRILLIC CAPITAL LETTER TE
        'Х' => 'X', // U+0425 CYRILLIC CAPITAL LETTER HA
        'α' => 'a', // U+03B1 GREEK SMALL LETTER ALPHA
        'ο' => 'o', // U+03BF GREEK SMALL LETTER OMICRON
        _ => character,
    }
}

/// Case-folds and confusable-folds a value before marker matching. `str::to_lowercase`
/// (full Unicode case folding), not `to_ascii_lowercase` (A-Z only), so
/// non-ASCII scripts with case (e.g. Cyrillic, Greek) still fold correctly.
fn normalize_for_marker_matching(value: &str) -> String {
    fold_confusable_ascii(value).to_lowercase()
}

pub(super) fn contains_sensitive_material(value: &str) -> bool {
    let normalized = normalize_for_marker_matching(value);
    if SENSITIVE_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return true;
    }

    // Split on any boundary that isn't alphanumeric or one of the
    // characters a credential token is itself built from (-_.), rather
    // than only on whitespace: a credential-shaped token glued directly to
    // surrounding text with no whitespace at all (e.g. `token=sk-...`, a
    // URL query string, a JSON `"key":"sk-..."` pair) must still be
    // isolated as its own candidate token, not missed because
    // split_whitespace() left it fused to what came before it.
    normalized
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.')
        })
        .any(is_credential_shaped_token)
        || contains_long_digit_sequence(value)
        || contains_ssn_shape(value)
}

fn is_credential_shaped_token(token: &str) -> bool {
    let trimmed = token.trim_matches(|character: char| matches!(character, '-' | '_' | '.'));

    if ["sk-", "ghp_", "github_pat_", "xoxb-", "xoxp-", "akia"]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
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

/// Counts a run of Unicode decimal digits, not just ASCII ones -- a card
/// number rendered with fullwidth (U+FF10-FF19) or Arabic-Indic
/// (U+0660-0669) digits must count the same as one rendered in ASCII.
/// `char::is_numeric()` recognizes decimal digits across scripts directly;
/// unlike the marker-matching path above, no folding step is needed first,
/// since we're testing digit-ness, not comparing against a fixed marker
/// string.
fn contains_long_digit_sequence(value: &str) -> bool {
    let mut run = 0usize;
    for character in value.chars() {
        if character.is_numeric() {
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

/// Scans for an SSN-shaped run (3 digits, `-`, 2 digits, `-`, 4
/// digits) over Unicode decimal digits, not just ASCII bytes -- the
/// previous byte-windowed implementation could never match a fullwidth or
/// Arabic-Indic digit (their UTF-8 encodings never collide with ASCII
/// digit byte values, so it wasn't unsound, just blind to them).
fn contains_ssn_shape(value: &str) -> bool {
    let characters = value.chars().collect::<Vec<_>>();
    if characters.len() < 11 {
        return false;
    }
    characters.windows(11).any(|window| {
        window[0..3].iter().all(|character| character.is_numeric())
            && window[3] == '-'
            && window[4..6].iter().all(|character| character.is_numeric())
            && window[6] == '-'
            && window[7..11].iter().all(|character| character.is_numeric())
    })
}

pub(super) fn is_sensitive_element(element: &InteractiveElement) -> bool {
    let mut descriptors = vec![normalize_for_marker_matching(&element.tag_name)];
    descriptors.extend(
        ["type", "name", "id", "autocomplete"]
            .into_iter()
            .filter_map(|name| element.attributes.get(name))
            .map(|value| normalize_for_marker_matching(value)),
    );
    if let Some(placeholder) = &element.placeholder {
        descriptors.push(normalize_for_marker_matching(placeholder));
    }
    if let Some(accessible_name) = &element.accessible_name {
        descriptors.push(normalize_for_marker_matching(accessible_name));
    }

    let combined = descriptors.join(" ");
    SENSITIVE_ELEMENT_MARKERS
        .iter()
        .any(|marker| combined.contains(marker))
        || element.attributes.get("type").is_some_and(|kind| {
            matches!(
                normalize_for_marker_matching(kind).as_str(),
                "password" | "hidden"
            )
        })
}

pub(super) fn contains_high_risk_page_text(value: &str) -> bool {
    let normalized = normalize_for_marker_matching(value);
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
        .any(|marker| normalized.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    // CR3 P2.4: contains_long_digit_sequence gates high-risk-page
    // classification and redaction both; a card number rendered in
    // fullwidth or Arabic-Indic digits must count the same as ASCII.
    #[test]
    fn contains_long_digit_sequence_recognizes_fullwidth_digits() {
        // Fullwidth 16-digit card number (U+FF10-FF19).
        let fullwidth = "４５３２１５２３４５３４５３４５";
        assert!(contains_long_digit_sequence(fullwidth));
        assert!(contains_high_risk_page_text(&format!(
            "card number {fullwidth}"
        )));
    }

    #[test]
    fn contains_long_digit_sequence_recognizes_arabic_indic_digits() {
        // Arabic-Indic 16-digit card number (U+0660-0669).
        let arabic_indic = "٤٥٣٢١٥٢٣٤٥٣٤٥٣٤٥";
        assert!(contains_long_digit_sequence(arabic_indic));
    }

    #[test]
    fn contains_long_digit_sequence_still_ignores_short_ascii_runs() {
        assert!(!contains_long_digit_sequence("order #12345"));
    }

    // A Cyrillic homoglyph substituted into a marker word must still match
    // -- verified this is NOT solved by NFKC normalization alone (see the
    // fold_confusable_ascii doc comment), so this specifically exercises
    // the hand-rolled homoglyph table.
    #[test]
    fn contains_sensitive_material_detects_cyrillic_homoglyph_markers() {
        // "pаssword=secret123" with a Cyrillic а (U+0430) in place of Latin a.
        assert!(contains_sensitive_material("pаssword=secret123"));
        // "sеcret:" with a Cyrillic е (U+0435, CYRILLIC SMALL LETTER IE) in
        // place of Latin e.
        assert!(contains_sensitive_material("sеcret: hunter2"));
    }

    #[test]
    fn contains_sensitive_material_detects_fullwidth_marker() {
        // Fullwidth Latin rendering of "token=" (U+FF41 etc.).
        assert!(contains_sensitive_material("ｔｏｋｅｎ＝abc123xyz789"));
    }

    // A credential-shaped token with no surrounding whitespace at all --
    // fused directly to adjacent text via `=`, `:`, or quotes -- must still
    // be isolated as its own candidate token instead of being missed
    // because split_whitespace() left it glued to what came before it.
    #[test]
    fn contains_sensitive_material_detects_credential_token_without_whitespace() {
        assert!(contains_sensitive_material(
            "config:key=sk-abc123def456ghi789;timeout=30"
        ));
        assert!(contains_sensitive_material(
            r#"{"authorization":"ghp_abc123def456ghi789"}"#
        ));
    }

    #[test]
    fn contains_sensitive_material_still_requires_marker_or_shape() {
        assert!(!contains_sensitive_material(
            "this page has no secrets on it"
        ));
    }

    #[test]
    fn contains_ssn_shape_recognizes_arabic_indic_digits() {
        assert!(contains_ssn_shape("SSN: ٥٥٥-١٢-١٢٣٤ on file"));
        assert!(!contains_ssn_shape("no ssn shape here"));
    }

    #[test]
    fn is_sensitive_element_detects_fullwidth_and_homoglyph_descriptors() {
        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert(String::from("id"), String::from("ｐａｓｓｗｏｒｄ-field"));
        let element = InteractiveElement {
            element_id: String::from("field-1"),
            dom_locator: Some(String::from("#field-1")),
            role: crate::page_model::ElementRole::Input,
            tag_name: String::from("input"),
            text: None,
            accessible_name: None,
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes,
        };
        assert!(is_sensitive_element(&element));
    }
}
