//! High-risk page-context detection: sites/pages where sanitized data should
//! never leave the device for network remote planning, regardless of consent
//! settings (see `high_risk_origin_policy` handling in [`super::super::remote_data_consent`]).

use crate::commands::PlannerInput;
use crate::commands::PlannerToolHistoryEntry;
use crate::page_model::PageModel;

use super::sensitive::{contains_high_risk_page_text, is_sensitive_element};

const HIGH_RISK_PATH_SEGMENTS: &[&str] = &[
    "admin", "auth", "billing", "checkout", "identity", "login", "password", "patient", "security",
    "signin", "sign-in", "wallet",
];

pub(crate) fn high_risk_context_reason(input: &PlannerInput) -> Option<&'static str> {
    high_risk_page_context_reason(
        input.agent_state.url.as_deref(),
        input.page_model.as_ref(),
        input.page_snapshot.as_ref(),
        &input.recent_tool_results,
    )
}

pub(crate) fn high_risk_page_context_reason(
    agent_url: Option<&str>,
    page_model: Option<&PageModel>,
    page_snapshot: Option<&crate::commands::PageSnapshotData>,
    recent_tool_results: &[PlannerToolHistoryEntry],
) -> Option<&'static str> {
    let has_sensitive_element = page_model
        .iter()
        .flat_map(|page| &page.interactive_elements)
        .chain(
            page_snapshot
                .iter()
                .flat_map(|snapshot| &snapshot.interactive_elements),
        )
        .any(is_sensitive_element);
    if has_sensitive_element {
        return Some("sensitive_form_control");
    }

    let has_high_risk_page_text = page_model
        .iter()
        .flat_map(|page| {
            page.regions.iter().flat_map(|region| {
                std::iter::once(region.text.as_str()).chain(region.label.as_deref())
            })
        })
        .chain(
            page_snapshot
                .iter()
                .map(|snapshot| snapshot.visible_text_excerpt.as_str()),
        )
        .chain(
            recent_tool_results
                .iter()
                .flat_map(|result| result.observation_summary.iter().map(String::as_str)),
        )
        .any(contains_high_risk_page_text);
    if has_high_risk_page_text {
        return Some("high_risk_page_text");
    }

    let urls = [
        agent_url,
        page_model.and_then(|page| page.url.as_deref()),
        page_snapshot.map(|snapshot| snapshot.url.as_str()),
    ];
    if urls.into_iter().flatten().any(is_high_risk_url_path) {
        return Some("high_risk_url_path");
    }

    None
}

fn is_high_risk_url_path(raw: &str) -> bool {
    let Ok(parsed) = url::Url::parse(raw) else {
        return false;
    };

    let host_is_high_risk = parsed.host_str().is_some_and(|host| {
        let host = host.to_ascii_lowercase();
        [
            "bank", "coinbase", "health", "identity", "login", "patient", "paypal", "stripe",
            "wallet",
        ]
        .iter()
        .any(|marker| host.split(['.', '-']).any(|part| part == *marker))
    });
    host_is_high_risk
        || parsed.path_segments().is_some_and(|segments| {
            segments
                .map(|segment| segment.to_ascii_lowercase())
                .any(|segment| HIGH_RISK_PATH_SEGMENTS.contains(&segment.as_str()))
        })
}
