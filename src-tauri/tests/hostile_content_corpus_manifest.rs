use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

const CORPUS: &str = include_str!("fixtures/post_batch8_hostile_content_corpus.json");

fn string_set(value: &Value, key: &str) -> BTreeSet<String> {
    value[key]
        .as_array()
        .unwrap_or_else(|| panic!("{key} must be an array"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("{key} entries must be strings"))
                .to_string()
        })
        .collect()
}

fn case_map<'a>(root: &'a Value, key: &str) -> BTreeMap<String, &'a Value> {
    root[key]
        .as_array()
        .unwrap_or_else(|| panic!("{key} must be an array"))
        .iter()
        .map(|case| {
            let id = case["id"]
                .as_str()
                .unwrap_or_else(|| panic!("{key} case id must be a string"));
            (id.to_string(), case)
        })
        .collect()
}

fn assert_case_contract(cases: &BTreeMap<String, &Value>) {
    let mut contents = BTreeSet::new();
    let allowed_reason_codes = BTreeSet::from([
        "authority_impersonation",
        "confirmation_bypass",
        "instruction_override",
        "script_execution",
        "secret_exfiltration",
    ]);

    for (id, case) in cases {
        let source = case["source"]
            .as_str()
            .unwrap_or_else(|| panic!("{id} source must be a string"));
        let content = case["content"]
            .as_str()
            .unwrap_or_else(|| panic!("{id} content must be a string"));
        let reason_codes = string_set(case, "expected_reason_codes");

        assert!(!source.trim().is_empty(), "{id} source must not be empty");
        assert!(!content.trim().is_empty(), "{id} content must not be empty");
        assert!(contents.insert(content), "{id} duplicates another fixture");
        assert!(!reason_codes.is_empty(), "{id} must expect an indicator");
        assert!(
            reason_codes
                .iter()
                .all(|code| allowed_reason_codes.contains(code.as_str())),
            "{id} contains an unsupported reason code"
        );
    }
}

#[test]
fn hostile_dom_corpus_contains_every_required_attack_shape() {
    let root: Value = serde_json::from_str(CORPUS).expect("hostile corpus must be valid JSON");
    assert_eq!(root["version"], "post-batch8-hostile-content-v1");
    let cases = case_map(&root, "dom_cases");
    let expected = BTreeSet::from([
        "aria_label_prompt_injection",
        "confirmation_bypass_near_button",
        "credential_exfiltration_near_input",
        "data_attribute_prompt_injection",
        "hidden_input_prompt_injection",
        "invisible_overlay_prompt_injection",
        "malicious_form_label",
        "offscreen_css_prompt_injection",
        "script_style_comment_prompt_injection",
        "title_alt_prompt_injection",
    ]);

    assert_eq!(
        cases.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected
    );
    assert_case_contract(&cases);
}

#[test]
fn hostile_ocr_corpus_contains_every_required_attack_shape() {
    let root: Value = serde_json::from_str(CORPUS).expect("hostile corpus must be valid JSON");
    let cases = case_map(&root, "ocr_cases");
    let expected = BTreeSet::from([
        "ocr_attempts_to_authorize_action",
        "ocr_authority_impersonation",
        "ocr_ignore_previous_instructions",
        "ocr_mixed_benign_and_hostile_regions",
        "ocr_payment_receipt_context",
        "ocr_reveal_credentials",
        "ocr_skip_confirmation",
    ]);

    assert_eq!(
        cases.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected
    );
    assert_case_contract(&cases);
    assert_eq!(
        cases["ocr_payment_receipt_context"]["high_risk_context"],
        true
    );
}

#[test]
fn hostile_corpus_declares_monotonic_security_invariants() {
    let root: Value = serde_json::from_str(CORPUS).expect("hostile corpus must be valid JSON");
    assert_eq!(
        string_set(&root, "required_invariants"),
        BTreeSet::from([
            "cannot_bypass_high_risk_origin_policy".to_string(),
            "cannot_create_click_authorization".to_string(),
            "cannot_enter_trusted_runtime_prompt_sections".to_string(),
            "cannot_lower_confirmation".to_string(),
            "cannot_mark_destructive_click_safe".to_string(),
            "caution_only".to_string(),
        ])
    );
    assert_eq!(
        string_set(&root, "allowed_security_effects"),
        BTreeSet::from([
            "abort".to_string(),
            "block".to_string(),
            "increase_caution".to_string(),
            "redact".to_string(),
            "replan".to_string(),
            "require_confirmation".to_string(),
        ])
    );
}

#[test]
fn hostile_corpus_keeps_a_real_ocr_image_fixture() {
    let root: Value = serde_json::from_str(CORPUS).expect("hostile corpus must be valid JSON");
    let relative = root["real_image_fixture"]
        .as_str()
        .expect("real_image_fixture must be a string");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let metadata = std::fs::metadata(&path)
        .unwrap_or_else(|error| panic!("real OCR fixture {} is missing: {error}", path.display()));
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);
}
