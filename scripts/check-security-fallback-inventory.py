#!/usr/bin/env python3
"""Verify exact metadata and human-readable parity for accepted fallbacks."""
from __future__ import annotations

import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / "scripts/security-fallback-allowlist.txt"
INVENTORY = ROOT / "scripts/security-fallback-inventory.json"
DOCUMENTATION = ROOT / "docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md"
REQUIRED = {
    "path",
    "functions",
    "expression",
    "justification",
    "user_visibility",
    "side_effect_impact",
    "test_coverage",
    "future_replacement",
    "disposition",
    "review_due",
    "owner_note",
}
VALID_DISPOSITIONS = {
    "permanent_accepted",
    "temporary_accepted",
    "convert_to_warning",
    "convert_to_error",
    "remove",
}


def normalize(line: str) -> str:
    return " ".join(line.strip().split())


def allowlist_keys() -> set[tuple[str, str]]:
    keys = set()
    for raw in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#"):
            continue
        path, expression = raw.split("|", 1)
        keys.add((path, expression))
    return keys


def source_functions(path: str, expression: str) -> list[str]:
    current = "module scope"
    functions = []
    signature = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
    for line in (ROOT / path).read_text(encoding="utf-8", errors="replace").splitlines():
        found = signature.search(line)
        if found:
            current = found.group(1)
        if normalize(line) == expression:
            functions.append(current)
    return sorted(set(functions))


def metadata_problems(key: tuple[str, str], entry: dict) -> list[str]:
    problems = []
    missing = REQUIRED - set(entry)
    if missing:
        return [f"{key}: missing fields {sorted(missing)}"]
    for field in REQUIRED - {"functions"}:
        if not isinstance(entry[field], str) or not entry[field].strip():
            problems.append(f"{key}: empty {field}")
    if not isinstance(entry["functions"], list) or not entry["functions"]:
        problems.append(f"{key}: functions must be a non-empty list")
    if entry["disposition"] not in VALID_DISPOSITIONS:
        problems.append(f"{key}: invalid disposition {entry['disposition']!r}")
    if entry["disposition"] == "temporary_accepted":
        if entry["review_due"] == "not_applicable":
            problems.append(f"{key}: temporary fallback requires a review boundary")
        if len(entry["owner_note"].strip()) < 20:
            problems.append(f"{key}: temporary fallback requires an actionable owner_note")
    return problems


def documentation_problems(entries: list[dict], documentation: str) -> list[str]:
    problems = []
    counts = Counter(entry.get("disposition") for entry in entries)
    for disposition in ("permanent_accepted", "temporary_accepted"):
        expected = f"- `{disposition}`: **{counts[disposition]}**"
        if expected not in documentation:
            problems.append(
                f"accepted-fallback documentation missing current count line: {expected}"
            )

    temporary = [
        entry for entry in entries if entry.get("disposition") == "temporary_accepted"
    ]
    for entry in temporary:
        path_marker = f"`{entry['path']}`"
        expression_marker = f"`{entry['expression']}`"
        if path_marker not in documentation or expression_marker not in documentation:
            problems.append(
                "accepted-fallback documentation is missing temporary entry "
                f"{entry['path']}|{entry['expression']}"
            )

    stale_table_markers = (
        "<!-- BEGIN GENERATED SECURITY FALLBACK INVENTORY -->",
        "| File | Function(s) | Exact expression |",
    )
    for marker in stale_table_markers:
        if marker in documentation:
            problems.append(
                "accepted-fallback documentation still contains the deprecated duplicated "
                f"per-expression table marker: {marker}"
            )
    return problems


def audit() -> list[str]:
    problems = []
    payload = json.loads(INVENTORY.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    indexed = {(entry.get("path"), entry.get("expression")): entry for entry in entries}
    expected = allowlist_keys()
    observed = set(indexed)
    if expected != observed:
        problems.append(
            f"inventory keys differ: missing={sorted(expected-observed)} extra={sorted(observed-expected)}"
        )
    for key, entry in indexed.items():
        entry_problems = metadata_problems(key, entry)
        problems.extend(entry_problems)
        if entry_problems:
            continue
        actual_functions = source_functions(*key)
        if actual_functions != entry["functions"]:
            problems.append(f"{key}: functions {entry['functions']} != source {actual_functions}")

    documentation = DOCUMENTATION.read_text(encoding="utf-8")
    problems.extend(documentation_problems(entries, documentation))
    return problems


def self_test() -> None:
    assert normalize("  let   x = 1; ") == "let x = 1;"
    base = {
        "path": "src-tauri/src/app_core/click_authorization.rs",
        "functions": ["example"],
        "expression": "example()",
        "justification": "safe",
        "user_visibility": "visible",
        "side_effect_impact": "none",
        "test_coverage": "unit",
        "future_replacement": "none",
        "disposition": "permanent_accepted",
        "review_due": "not_applicable",
        "owner_note": "Permanent exact fallback with no authority impact.",
    }
    missing = dict(base)
    missing.pop("disposition")
    assert "missing fields" in metadata_problems(("p", "e"), missing)[0]

    invalid = dict(base, disposition="maybe")
    assert any(
        "invalid disposition" in problem
        for problem in metadata_problems(("p", "e"), invalid)
    )

    temporary = dict(
        base,
        disposition="temporary_accepted",
        review_due="not_applicable",
        owner_note="short",
    )
    temporary_problems = metadata_problems(("p", "e"), temporary)
    assert any("review boundary" in problem for problem in temporary_problems)
    assert any("actionable owner_note" in problem for problem in temporary_problems)

    documented_temporary = dict(
        base,
        path="src/example.rs",
        expression="value.ok()",
        functions=["example"],
        disposition="temporary_accepted",
        review_due="before_release_candidate",
        owner_note="Replace with a typed reason before the release candidate.",
    )
    valid_documentation = (
        "- `permanent_accepted`: **1**\n"
        "- `temporary_accepted`: **1**\n"
        "- `src/example.rs` — `value.ok()`\n"
    )
    assert not documentation_problems([base, documented_temporary], valid_documentation)
    assert documentation_problems(
        [base, documented_temporary], "- `permanent_accepted`: **1**\n"
    )
    assert documentation_problems(
        [base, documented_temporary],
        valid_documentation + "<!-- BEGIN GENERATED SECURITY FALLBACK INVENTORY -->",
    )

    assert source_functions(
        "scripts/check-security-fallback-inventory.py", "definitely stale expression"
    ) == []
    print("Security fallback inventory self-test passed")


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return 0
    if sys.argv[1:]:
        print("usage: check-security-fallback-inventory.py [--self-test]", file=sys.stderr)
        return 2
    problems = audit()
    if problems:
        print("Security fallback inventory audit failed:", file=sys.stderr)
        for problem in problems:
            print(f"- {problem}", file=sys.stderr)
        return 1
    print("Security fallback inventory audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
