#!/usr/bin/env python3
"""Apply or verify the exact integration repairs exposed by strict Rust builds.

This branch-only script is state-aware: each repair must be either present in
its known stale form or already present in an accepted corrected form. Unknown
source shapes abort the worker. The workflow deletes this file before committing
validated product changes.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, source: str) -> None:
    (ROOT / path).write_text(source, encoding="utf-8")


def replace_or_verify(
    path: str,
    old: str,
    new: str,
    *,
    expected_old: int = 1,
    expected_new: int = 1,
) -> None:
    source = read(path)
    old_count = source.count(old)
    new_count = source.count(new)
    if old_count == expected_old and new_count == 0:
        write(path, source.replace(old, new))
        return
    if old_count == 0 and new_count >= expected_new:
        return
    raise SystemExit(
        f"{path}: unexpected repair state; old={old_count}, new={new_count}, "
        f"expected old={expected_old} or new>={expected_new}"
    )


def repair_confirmation_summary() -> None:
    path = "src-tauri/src/commands/confirmation_manifest.rs"
    source = read(path)
    stale = re.compile(
        r"append_confirmation_warnings\(\s*&mut summary,\s*"
        r"string_array_argument\(step, RUNTIME_CONFIRMATION_WARNINGS_ARG\),\s*\);",
        re.MULTILINE,
    )
    matches = list(stale.finditer(source))
    if matches:
        if len(matches) != 2:
            raise SystemExit(f"{path}: expected two stale warning calls, found {len(matches)}")
        source = stale.sub(
            "summary = append_confirmation_warnings(\n"
            "            summary,\n"
            "            string_array_argument(step, RUNTIME_CONFIRMATION_WARNINGS_ARG),\n"
            "        );",
            source,
        )
        write(path, source)
        return
    corrected_markers = [
        "summary = append_confirmation_warnings(",
        "let warnings = string_array_argument(step, RUNTIME_CONFIRMATION_WARNINGS_ARG);",
        "with_confirmation_warnings(",
    ]
    if any(marker in source for marker in corrected_markers):
        return
    raise SystemExit(f"{path}: neither stale nor corrected warning-summary shape was found")


def repair_model_downloads() -> None:
    path = "src-tauri/src/app_core/model_management.rs"
    replace_or_verify(
        path,
        'return attempt.error("model download redirect limit exceeded");',
        "return attempt.stop();",
    )
    source = read(path)
    old_parameter = "    response: reqwest::blocking::Response,"
    new_parameter = "    mut response: reqwest::blocking::Response,"
    if old_parameter in source:
        source = source.replace(old_parameter, new_parameter, 1)
        write(path, source)
    elif new_parameter not in source and "fn write_verified_reader_atomically<R: Read>(" not in source:
        raise SystemExit(f"{path}: no accepted mutable response/reader implementation found")

    source = read(path)
    old_path = "                path: temp_path,"
    new_path = "                path: temp_path.clone(),"
    if old_path in source:
        source = source.replace(old_path, new_path, 1)
        write(path, source)
    elif new_path not in source and "reason: error.to_string()," not in source:
        raise SystemExit(f"{path}: no accepted non-moving temporary-path error shape found")


def repair_planner_metadata_move() -> None:
    path = "src-tauri/src/app_core/planner_redaction.rs"
    source = read(path)
    old = "            sanitization: metadata,"
    new = "            sanitization: metadata.clone(),"
    if old in source:
        if source.count(old) != 1:
            raise SystemExit(f"{path}: expected one sanitization metadata move")
        write(path, source.replace(old, new, 1))
    elif new not in source:
        raise SystemExit(f"{path}: sanitization metadata field was not found")


def main() -> None:
    repair_confirmation_summary()
    repair_model_downloads()
    repair_planner_metadata_move()
    print("Deterministic post-Batch-8 compile repairs applied or verified")


if __name__ == "__main__":
    main()
