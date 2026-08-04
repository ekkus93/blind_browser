#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / "scripts/security-fallback-allowlist.txt"
INVENTORY = ROOT / "scripts/security-fallback-inventory.json"
DOCUMENTATION = ROOT / "docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md"
REQUIRED = {
    "path", "function", "expression", "occurrence", "context_before", "context_after",
    "justification", "user_visibility", "side_effect_impact", "test_coverage",
    "future_replacement", "disposition", "review_due", "owner_note",
}
VALID_DISPOSITIONS = {
    "permanent_accepted", "temporary_accepted", "convert_to_warning", "convert_to_error", "remove",
}
SIGNATURE = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")


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


def occurrences_from_lines(lines: list[str], expression: str) -> list[dict]:
    current = "module scope"
    per_function: dict[str, int] = defaultdict(int)
    occurrences = []
    for index, line in enumerate(lines):
        match = SIGNATURE.search(line)
        if match:
            current = match.group(1)
        if normalize(line) != expression:
            continue
        per_function[current] += 1
        before = next((normalize(value) for value in reversed(lines[:index]) if normalize(value)), "")
        after = next((normalize(value) for value in lines[index + 1:] if normalize(value)), "")
        occurrences.append({
            "function": current,
            "occurrence": per_function[current],
            "context_before": before,
            "context_after": after,
        })
    return occurrences


def source_occurrences(path: str, expression: str) -> list[dict]:
    return occurrences_from_lines(
        (ROOT / path).read_text(encoding="utf-8", errors="replace").splitlines(), expression
    )


def occurrence_key(entry: dict) -> tuple:
    return (entry.get("path"), entry.get("function"), entry.get("expression"), entry.get("occurrence"))


def metadata_problems(key: tuple, entry: dict) -> list[str]:
    problems = []
    missing = REQUIRED - set(entry)
    if missing:
        return [f"{key}: missing fields {sorted(missing)}"]
    for field in REQUIRED - {"occurrence"}:
        if not isinstance(entry[field], str) or not entry[field].strip():
            problems.append(f"{key}: empty {field}")
    if not isinstance(entry["occurrence"], int) or entry["occurrence"] < 1:
        problems.append(f"{key}: occurrence must be a positive integer")
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
            problems.append(f"accepted-fallback documentation missing current count line: {expected}")
    if "path + function + normalized expression + occurrence index" not in documentation:
        problems.append("accepted-fallback documentation is missing occurrence identity policy")
    return problems


def audit() -> list[str]:
    problems = []
    payload = json.loads(INVENTORY.read_text(encoding="utf-8"))
    if payload.get("version") != 3:
        problems.append("inventory schema version must be 3")
    entries = payload.get("entries", [])
    keys = [occurrence_key(entry) for entry in entries]
    duplicates = [key for key, count in Counter(keys).items() if count > 1]
    if duplicates:
        problems.append(f"duplicate inventory occurrence records: {duplicates}")
    expected_pairs = allowlist_keys()
    observed_pairs = {(entry.get("path"), entry.get("expression")) for entry in entries}
    if expected_pairs != observed_pairs:
        problems.append(
            f"inventory keys differ: missing={sorted(expected_pairs-observed_pairs)} extra={sorted(observed_pairs-expected_pairs)}"
        )
    records_by_pair: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for entry in entries:
        key = occurrence_key(entry)
        entry_problems = metadata_problems(key, entry)
        problems.extend(entry_problems)
        if not entry_problems:
            records_by_pair[(entry["path"], entry["expression"])].append(entry)
    for path, expression in expected_pairs:
        source_keys = {
            (path, item["function"], expression, item["occurrence"], item["context_before"], item["context_after"])
            for item in source_occurrences(path, expression)
        }
        inventory_keys = {
            (entry["path"], entry["function"], entry["expression"], entry["occurrence"], entry["context_before"], entry["context_after"])
            for entry in records_by_pair.get((path, expression), [])
        }
        if source_keys != inventory_keys:
            problems.append(
                f"{path}|{expression}: occurrence mismatch missing={sorted(source_keys-inventory_keys)} stale={sorted(inventory_keys-source_keys)}"
            )
    problems.extend(documentation_problems(entries, DOCUMENTATION.read_text(encoding="utf-8")))
    return problems


def self_test() -> None:
    broad = occurrences_from_lines(["fn example() {", "  .ok()", "  .ok()", "}"], ".ok()")
    assert len(broad) == 2 and [item["occurrence"] for item in broad] == [1, 2]
    defaults = occurrences_from_lines(
        ["fn example() {", "  .unwrap_or_default()", "  .unwrap_or_default()", "}"],
        ".unwrap_or_default()",
    )
    assert len(defaults) == 2
    base = {
        "path": "src/example.rs", "function": "example", "expression": ".ok()", "occurrence": 1,
        "context_before": "let value = input", "context_after": "return value",
        "justification": "safe", "user_visibility": "visible", "side_effect_impact": "none",
        "test_coverage": "unit", "future_replacement": "none", "disposition": "permanent_accepted",
        "review_due": "not_applicable", "owner_note": "Permanent exact fallback with no authority impact.",
    }
    missing = dict(base); missing.pop("occurrence")
    assert "missing fields" in metadata_problems(occurrence_key(missing), missing)[0]
    temporary = dict(base, disposition="temporary_accepted", review_due="not_applicable", owner_note="short")
    issues = metadata_problems(occurrence_key(temporary), temporary)
    assert any("review boundary" in issue for issue in issues)
    assert any("actionable owner_note" in issue for issue in issues)
    assert len({occurrence_key(base), occurrence_key(dict(base))}) == 1
    stale = dict(base, context_after="changed")
    assert stale["context_after"] != base["context_after"]
    print("Security fallback inventory self-test passed")


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        self_test(); return 0
    if sys.argv[1:]:
        print("usage: check-security-fallback-inventory.py [--self-test]", file=sys.stderr); return 2
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
