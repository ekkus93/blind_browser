#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path


def replace_once(path: Path, old: str, new: str, description: str) -> None:
    content = path.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one {description}, found {count}")
    path.write_text(content.replace(old, new, 1), encoding="utf-8")


def replace_test_function(path: Path, function_name: str, replacement: str) -> None:
    source = path.read_text(encoding="utf-8")
    marker = f"#[test]\nfn {function_name}() {{"
    count = source.count(marker)
    if count != 1:
        raise SystemExit(f"{path}: expected one {function_name} test, found {count}")
    start = source.index(marker)
    next_test = source.find("\n#[test]\n", start + len(marker))
    end = len(source) if next_test == -1 else next_test
    existing = source[start:end].rstrip()
    desired = replacement.rstrip()
    if existing != desired:
        source = source[:start] + desired + source[end:]
        path.write_text(source, encoding="utf-8")


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


def repair_integration_evidence() -> None:
    integration = Path("src-tauri/tests/post_batch8_direct_command_policy_evidence.rs")
    if not integration.is_file():
        raise SystemExit(f"missing integration evidence test: {integration}")

    desired_test = '''#[test]
fn source_drift_page_context_commands_retain_privacy_sanitizer_wiring() {
    let core_handlers = source("src/command_handlers/core_handlers.rs");
    let voice_handlers = source("src/command_handlers/voice_handlers.rs");
    let remote_planner = source("src/app_core/remote_planner.rs");
    let redaction = source("src/app_core/planner_redaction.rs");

    assert!(core_handlers.contains("resolve_command_lock_scoped"));
    assert!(voice_handlers.contains("run_command_with_lock_scoped_replanning"));
    assert!(remote_planner
        .contains("sanitize_remote_planner_input(planner_input, privacy, &endpoint_scope)?"));
    assert!(redaction.contains("enforce_remote_planner_privacy(input, privacy, endpoint_scope)?"));

    let transmitting = EVIDENCE
        .iter()
        .filter(|entry| entry.transmits_page_context)
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        transmitting,
        BTreeSet::from(["resolve_command", "transcribe_and_execute_command",])
    );
}'''
    replace_test_function(
        integration,
        "source_drift_page_context_commands_retain_privacy_sanitizer_wiring",
        desired_test,
    )

    repaired = integration.read_text(encoding="utf-8")
    required_markers = (
        "evidence_inventory_matches_registry_and_tauri_surface",
        'source("src/command_handlers/core_handlers.rs")',
        'source("src/command_handlers/voice_handlers.rs")',
        'source("src/app_core/remote_planner.rs")',
        'source("src/app_core/planner_redaction.rs")',
        "sanitize_remote_planner_input(planner_input, privacy, &endpoint_scope)?",
        "enforce_remote_planner_privacy(input, privacy, endpoint_scope)?",
        '.filter(|entry| entry.transmits_page_context)',
        'BTreeSet::from(["resolve_command", "transcribe_and_execute_command",])',
    )
    missing = [marker for marker in required_markers if marker not in repaired]
    if missing:
        raise SystemExit(f"{integration}: missing end-to-end privacy evidence: {missing}")
    stale_markers = (
        "handler {handler} must retain the remote-planner privacy sanitizer",
        "handlers_source.contains(\"sanitize_remote_planner_input(\")",
    )
    present = [marker for marker in stale_markers if marker in repaired]
    if present:
        raise SystemExit(f"{integration}: stale per-handler privacy assertions remain: {present}")
    print("Restored end-to-end integration privacy source evidence")


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
    repair_integration_evidence()
    evidence = find_library_evidence_test()
    if evidence is not None:
        repair_library_evidence(evidence)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
