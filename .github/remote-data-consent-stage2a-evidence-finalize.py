from __future__ import annotations

from pathlib import Path

PATH = Path("src-tauri/tests/post_batch8_direct_command_policy_evidence.rs")


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(
            f"Stage 2A evidence finalizer: expected one {label}, found {count}"
        )
    return source.replace(old, new, 1)


def replace_exact_count(
    source: str,
    old: str,
    new: str,
    expected_count: int,
    label: str,
) -> str:
    count = source.count(old)
    if count != expected_count:
        raise SystemExit(
            "Stage 2A evidence finalizer: expected "
            f"{expected_count} {label} occurrences, found {count}"
        )
    return source.replace(old, new, expected_count)


text = PATH.read_text()

text = replace_once(
    text,
    '''    Evidence {
        name: "submit_confirmation_response",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
''',
    '''    Evidence {
        name: "submit_confirmation_response",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "submit_remote_planner_consent_response",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: true,
    },
''',
    "remote planner consent evidence entry",
)

# The same adjacent pair appears once in the networked inventory and once in the
# credential-bearing inventory. Require and replace both occurrences atomically.
text = replace_exact_count(
    text,
    '''            "resolve_command",
            "transcribe_command",
''',
    '''            "resolve_command",
            "submit_remote_planner_consent_response",
            "transcribe_command",
''',
    2,
    "networked/credential-bearing remote planner consent evidence",
)

text = replace_once(
    text,
    '''    let remote_planner = source("src/app_core/remote_planner.rs");
    let redaction = source("src/app_core/planner_redaction.rs");
''',
    '''    let replanning = source("src/app_core/replanning_orchestrator.rs");
    let consent = source("src/app_core/remote_data_consent.rs");
    let redaction = source("src/app_core/planner_redaction.rs");
''',
    "privacy source inventory",
)

text = replace_once(
    text,
    '''    assert!(remote_planner
        .contains("sanitize_remote_planner_input(planner_input, privacy, &endpoint_scope)?"));
    assert!(redaction.contains("enforce_remote_planner_privacy(input, privacy, endpoint_scope)?"));
''',
    '''    assert!(replanning.contains("guard.prepare_remote_planner_request("));
    assert!(consent.contains("match evaluate_remote_planner_policy("));
    assert!(consent
        .contains("sanitize_remote_planner_input_authorized(&planner_input, mode)?"));
    assert!(redaction.contains("pub(crate) fn sanitize_remote_planner_input_authorized("));
''',
    "privacy preparation wiring assertions",
)

text = replace_once(
    text,
    '''        BTreeSet::from(["resolve_command", "transcribe_and_execute_command",])
''',
    '''        BTreeSet::from([
            "resolve_command",
            "submit_remote_planner_consent_response",
            "transcribe_and_execute_command",
        ])
''',
    "page-context remote planner consent evidence",
)

PATH.write_text(text)
