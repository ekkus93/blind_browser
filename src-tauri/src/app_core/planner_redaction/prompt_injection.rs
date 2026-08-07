//! Best-effort prompt-injection detection over page text handed to a remote
//! planner. Detection is advisory only — [`super::types::PromptInjectionIndicators`]
//! is always `caution_only`, since it must never itself authorize or block an action.

use crate::commands::PlannerInput;

use super::types::PromptInjectionIndicators;

const INJECTION_OVERRIDE_MARKERS: &[&str] = &[
    "ignore previous instructions",
    "ignore all previous",
    "disregard previous instructions",
    "override the system",
    "new instructions:",
];

const INJECTION_AUTHORITY_MARKERS: &[&str] = &[
    "system message",
    "developer message",
    "assistant message",
    "you are chatgpt",
    "trusted instruction",
];

const INJECTION_SECRET_MARKERS: &[&str] = &[
    "reveal the password",
    "show the password",
    "send the token",
    "exfiltrate",
    "api key",
    "authorization header",
    "session cookie",
];

const INJECTION_CONFIRMATION_MARKERS: &[&str] = &[
    "skip confirmation",
    "without confirmation",
    "do not ask for confirmation",
    "confirmation is not required",
    "auto approve",
];

const INJECTION_SCRIPT_MARKERS: &[&str] = &[
    "execute javascript",
    "run javascript",
    "eval(",
    "evaljs",
    "document.cookie",
];

pub(super) fn detect_prompt_injection(input: &PlannerInput) -> PromptInjectionIndicators {
    use std::collections::BTreeSet;

    let mut codes = BTreeSet::new();

    let mut inspect = |value: &str| {
        let lower = value.to_ascii_lowercase();
        if INJECTION_OVERRIDE_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        {
            codes.insert(String::from("instruction_override"));
        }
        if INJECTION_AUTHORITY_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        {
            codes.insert(String::from("authority_impersonation"));
        }
        if INJECTION_SECRET_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        {
            codes.insert(String::from("secret_exfiltration"));
        }
        if INJECTION_CONFIRMATION_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        {
            codes.insert(String::from("confirmation_bypass"));
        }
        if INJECTION_SCRIPT_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        {
            codes.insert(String::from("script_execution"));
        }
    };

    if let Some(page) = &input.page_model {
        for region in &page.regions {
            inspect(&region.text);
            if let Some(label) = &region.label {
                inspect(label);
            }
        }
        for element in &page.interactive_elements {
            for value in [
                element.text.as_deref(),
                element.accessible_name.as_deref(),
                element.placeholder.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                inspect(value);
            }
            for value in element.attributes.values() {
                inspect(value);
            }
        }
    }

    if let Some(snapshot) = &input.page_snapshot {
        inspect(&snapshot.visible_text_excerpt);
        for element in &snapshot.interactive_elements {
            for value in [
                element.text.as_deref(),
                element.accessible_name.as_deref(),
                element.placeholder.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                inspect(value);
            }
            for value in element.attributes.values() {
                inspect(value);
            }
        }
    }

    for result in &input.recent_tool_results {
        for observation in &result.observation_summary {
            inspect(observation);
        }
    }

    for skill in &input.relevant_skill_summaries {
        inspect(&skill.description);
        for tag in &skill.intent_tags {
            inspect(tag);
        }
    }

    let reason_codes = codes.into_iter().collect::<Vec<_>>();
    PromptInjectionIndicators {
        caution_only: true,
        detected: !reason_codes.is_empty(),
        reason_codes,
    }
}
