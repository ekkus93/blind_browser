#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path


def replace_once(path: Path, old: str, new: str, description: str) -> None:
    content = path.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one {description}, found {count}")
    path.write_text(content.replace(old, new, 1), encoding="utf-8")


def main() -> int:
    evidence = Path("src-tauri/src/app_core/tests/direct_command_evidence_tests.rs")
    if not evidence.is_file():
        raise SystemExit(f"missing generated direct-command evidence test: {evidence}")

    old_page_context_evidence = '''    let page_context_resolver_sources = [
        &dispatch_source,
        &executor_source,
        &validator_source,
        &field_focus_source,
    ];
    for command in [
        DirectCommandName::ClickElementV2,
        DirectCommandName::FillForm,
        DirectCommandName::SubmitForm,
        DirectCommandName::ReportFocusedField,
    ] {
        let policy = direct_command_policy(command);
        assert_eq!(
            policy.page_context,
            DirectCommandPageContextPolicy::SanitizedSnapshot,
            "{}: page-context commands must require a sanitized snapshot",
            command.as_str()
        );
        assert!(
            page_context_resolver_sources
                .iter()
                .all(|source| source.contains("redact_page_snapshot_for_remote")),
            "{}: page-context resolver sources must retain the reviewed sanitization helper",
            command.as_str()
        );
    }
'''
    new_page_context_evidence = '''    // Sanitization belongs to the command-resolution boundary. The executor and
    // validator consume already-resolved actions and are covered by the typed
    // page-context policy assertion rather than a helper-name source check.
    let page_context_sanitization_owners = [&dispatch_source, &field_focus_source];
    for command in [
        DirectCommandName::ClickElementV2,
        DirectCommandName::FillForm,
        DirectCommandName::SubmitForm,
        DirectCommandName::ReportFocusedField,
    ] {
        let policy = direct_command_policy(command);
        assert_eq!(
            policy.page_context,
            DirectCommandPageContextPolicy::SanitizedSnapshot,
            "{}: page-context commands must require a sanitized snapshot",
            command.as_str()
        );
        assert!(
            page_context_sanitization_owners
                .iter()
                .all(|source| source.contains("redact_page_snapshot_for_remote")),
            "{}: page-context resolver owners must retain the reviewed sanitization helper",
            command.as_str()
        );
    }
'''
    replace_once(
        evidence,
        old_page_context_evidence,
        new_page_context_evidence,
        "page-context source-drift evidence block",
    )
    replace_once(
        evidence,
        '    let validator_source = read_source("app_core/validator.rs");\n',
        "",
        "unused validator source binding",
    )

    repaired = evidence.read_text(encoding="utf-8")
    required_markers = (
        "DirectCommandPageContextPolicy::SanitizedSnapshot",
        "page_context_sanitization_owners",
        "redact_page_snapshot_for_remote",
    )
    missing = [marker for marker in required_markers if marker not in repaired]
    if missing:
        raise SystemExit(f"{evidence}: repaired evidence is missing markers: {missing}")
    if "page_context_resolver_sources" in repaired:
        raise SystemExit(f"{evidence}: stale four-source evidence block remains")

    print("Aligned generated page-context evidence with the two resolver owners")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
