#!/usr/bin/env python3
"""Apply the exact integration repairs exposed by the first strict Rust build.

This script is branch-only validation machinery. It refuses to run if the
expected stale source shapes are absent or duplicated, and the workflow removes
it before committing validated product changes.
"""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_exact(path: str, old: str, new: str, expected_count: int = 1) -> None:
    target = ROOT / path
    source = target.read_text(encoding="utf-8")
    observed = source.count(old)
    if observed != expected_count:
        raise SystemExit(
            f"{path}: expected {expected_count} occurrence(s), found {observed}: {old!r}"
        )
    target.write_text(source.replace(old, new), encoding="utf-8")


def main() -> None:
    replace_exact(
        "src-tauri/src/commands/confirmation_manifest.rs",
        """        append_confirmation_warnings(
            &mut summary,
            string_array_argument(step, RUNTIME_CONFIRMATION_WARNINGS_ARG),
        );""",
        """        summary = append_confirmation_warnings(
            summary,
            string_array_argument(step, RUNTIME_CONFIRMATION_WARNINGS_ARG),
        );""",
        expected_count=2,
    )
    replace_exact(
        "src-tauri/src/app_core/model_management.rs",
        'return attempt.error("model download redirect limit exceeded");',
        "return attempt.stop();",
    )
    replace_exact(
        "src-tauri/src/app_core/model_management.rs",
        "    response: reqwest::blocking::Response,",
        "    mut response: reqwest::blocking::Response,",
    )
    replace_exact(
        "src-tauri/src/app_core/model_management.rs",
        "                path: temp_path,",
        "                path: temp_path.clone(),",
    )
    replace_exact(
        "src-tauri/src/app_core/planner_redaction.rs",
        "            sanitization: metadata,",
        "            sanitization: metadata.clone(),",
    )
    print("Deterministic post-Batch-8 compile repairs applied")


if __name__ == "__main__":
    main()
