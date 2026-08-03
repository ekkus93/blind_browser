#!/usr/bin/env python3
"""Verify exact per-expression metadata for every accepted security fallback."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / "scripts/security-fallback-allowlist.txt"
INVENTORY = ROOT / "scripts/security-fallback-inventory.json"
REQUIRED = {
    "path",
    "functions",
    "expression",
    "justification",
    "user_visibility",
    "side_effect_impact",
    "test_coverage",
    "future_replacement",
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


def audit() -> list[str]:
    problems = []
    payload = json.loads(INVENTORY.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    indexed = {(entry.get("path"), entry.get("expression")): entry for entry in entries}
    expected = allowlist_keys()
    observed = set(indexed)
    if expected != observed:
        problems.append(f"inventory keys differ: missing={sorted(expected-observed)} extra={sorted(observed-expected)}")
    for key, entry in indexed.items():
        missing = REQUIRED - set(entry)
        if missing:
            problems.append(f"{key}: missing fields {sorted(missing)}")
            continue
        for field in REQUIRED - {"functions"}:
            if not isinstance(entry[field], str) or not entry[field].strip():
                problems.append(f"{key}: empty {field}")
        if not isinstance(entry["functions"], list) or not entry["functions"]:
            problems.append(f"{key}: functions must be a non-empty list")
        actual_functions = source_functions(*key)
        if actual_functions != entry["functions"]:
            problems.append(f"{key}: functions {entry['functions']} != source {actual_functions}")
    return problems


def self_test() -> None:
    assert normalize("  let   x = 1; ") == "let x = 1;"
    assert REQUIRED.issuperset({"path", "expression", "functions"})
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
