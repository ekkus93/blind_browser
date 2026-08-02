#!/usr/bin/env python3
"""Fail CI when a new quiet fallback appears in security-sensitive production code."""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / "scripts/security-fallback-allowlist.txt"
ROOTS = [
    ROOT / "src-tauri/src/app_core",
    ROOT / "src-tauri/src/commands",
    ROOT / "src-tauri/src/config",
    ROOT / "src-tauri/src/asr",
    ROOT / "src-tauri/src/tts",
    ROOT / "src-tauri/src/ocr",
    ROOT / "src/api",
]
TOP_LEVEL_FRONTEND = [ROOT / "src/privacy-redaction.ts"]
SUSPICIOUS = re.compile(
    r"\.ok\(\)|\.unwrap_or_default\(\)|\bfilter_map\(Result::ok\)|\blet\s+_\s*=",
)
DIRECT_MODEL_WRITE = re.compile(
    r"(?:fs::)?File::create\(\s*&?target_path\s*\)|fs::write\(\s*&?target_path",
)


def normalize_line(line: str) -> str:
    return " ".join(line.strip().split())


def load_allowlist(path: Path = ALLOWLIST) -> set[str]:
    entries: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#"):
            continue
        if "|" not in raw:
            raise ValueError(f"invalid fallback allowlist entry: {raw}")
        entries.add(raw)
    return entries


def production_lines(path: Path) -> list[tuple[int, str]]:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    cutoff = len(lines)
    for index, line in enumerate(lines):
        if line.strip() == "#[cfg(test)]":
            cutoff = index
            break
    return [(index + 1, line) for index, line in enumerate(lines[:cutoff])]


def candidate_files() -> list[Path]:
    paths: list[Path] = []
    for root in ROOTS:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path.suffix not in {".rs", ".ts", ".tsx", ".mjs"}:
                continue
            if "tests" in path.parts or path.name.endswith("_tests.rs"):
                continue
            paths.append(path)
    paths.extend(path for path in TOP_LEVEL_FRONTEND if path.exists())
    return sorted(set(paths))


def analyze_line(relative: str, line: str, allowlist: set[str]) -> str | None:
    normalized = normalize_line(line)
    if not normalized:
        return None
    if not (SUSPICIOUS.search(line) or DIRECT_MODEL_WRITE.search(line)):
        return None
    key = f"{relative}|{normalized}"
    if key in allowlist:
        return None
    return key


def audit(allowlist: set[str]) -> list[str]:
    violations: list[str] = []
    for path in candidate_files():
        relative = path.relative_to(ROOT).as_posix()
        for line_number, line in production_lines(path):
            violation = analyze_line(relative, line, allowlist)
            if violation is not None:
                violations.append(f"{relative}:{line_number}: {normalize_line(line)}")
    return violations


def self_test() -> None:
    allow = {"src-tauri/src/example.rs|let _ = cleanup();"}
    assert analyze_line("src-tauri/src/example.rs", "let value = parse().ok();", allow)
    assert analyze_line(
        "src-tauri/src/app_core/model_management.rs",
        "let file = fs::File::create(target_path)?;",
        allow,
    )
    assert analyze_line(
        "src-tauri/src/command_handlers/api_key_handlers.rs",
        "api_key_reference.unwrap_or_default()",
        allow,
    )
    assert analyze_line("src-tauri/src/example.rs", "let _ = cleanup();", allow) is None
    print("Security fallback scanner self-test passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    try:
        allowlist = load_allowlist()
    except (OSError, ValueError) as error:
        print(f"Security fallback audit could not load its allowlist: {error}", file=sys.stderr)
        return 1
    violations = audit(allowlist)
    if violations:
        print("Security fallback audit failed; review or remove these quiet fallbacks:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        print(
            "Document accepted behavior in docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md "
            "and add an exact scripts/security-fallback-allowlist.txt entry.",
            file=sys.stderr,
        )
        return 1
    print("Security fallback audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
