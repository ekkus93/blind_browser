#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path


def replace_once(path: Path, old: str, new: str, description: str) -> None:
    content = path.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one {description}, found {count}")
    path.write_text(content.replace(old, new, 1), encoding="utf-8")


def find_library_evidence_test() -> Path | None:
    function_marker = "direct_command_policy_evidence_is_complete"
    stale_marker = "page_context_resolver_sources"
    candidates: list[Path] = []
    marker_hits: list[tuple[Path, bool, bool]] = []

    for path in sorted(Path("src-tauri").rglob("*.rs")):
        source = path.read_text(encoding="utf-8")
        has_function = function_marker in source
        has_stale_block = stale_marker in source
        if has_function or has_stale_block:
            marker_hits.append((path, has_function, has_stale_block))
        if has_function and has_stale_block:
            candidates.append(path)

    if len(candidates) > 1:
        rendered = ", ".join(str(path) for path in candidates)
        raise SystemExit(f"multiple generated library evidence tests found: {rendered}")
    if len(candidates) == 1:
        return candidates[0]

    if marker_hits:
        rendered = "; ".join(
            f"{path} function={has_function} stale_block={has_stale_block}"
            for path, has_function, has_stale_block in marker_hits
        )
        raise SystemExit(
            "incomplete or split generated library evidence markers; " + rendered
        )
    return None


def verify_integration_evidence() -> None:
    integration = Path("src-tauri/tests/post_batch8_direct_command_policy_evidence.rs")
    if not integration.is_file():
        raise SystemExit(
            "no generated library evidence test and missing integration evidence test: "
            f"{integration}"
        )
    source = integration.read_text(encoding="utf-8")
    required_markers = (
        "DirectCommandPageContextPolicy::SanitizedSnapshot",
        "direct_command_policy",
        "redact_page_snapshot_for_remote",
    )
    missing = [marker for marker in required_markers if marker not in source]
    if missing:
        raise SystemExit(f"{integration}: missing authoritative evidence markers: {missing}")
    print(
        "No generated library source-drift test was present; verified the integration "
        "semantic evidence gate instead"
    )


def repair_library_evidence(evidence: Path) -> None:
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
    print(f"Aligned generated page-context evidence in {evidence}")


def main() -> int:
    evidence = find_library_evidence_test()
    if evidence is None:
        verify_integration_evidence()
    else:
        repair_library_evidence(evidence)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
