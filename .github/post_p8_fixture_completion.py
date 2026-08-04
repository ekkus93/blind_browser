#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path


def initializer_end(lines: list[str], start: int, path: Path, struct_name: str) -> int:
    depth = 0
    opened = False
    for index in range(start, len(lines)):
        line = lines[index]
        opening = line.count("{")
        closing = line.count("}")
        if opening:
            opened = True
        depth += opening - closing
        if opened and depth == 0:
            return index
    raise SystemExit(f"{path}: unterminated {struct_name} initializer at line {start + 1}")


def is_initializer_line(line: str, struct_name: str) -> bool:
    needle = f"{struct_name} {{"
    if needle not in line:
        return False
    prefix = line.split(needle, 1)[0]
    if "->" in prefix:
        return False
    stripped_prefix = prefix.strip()
    return not stripped_prefix or "=" in prefix or ":" in prefix


def ensure_initializer_fields(
    path: Path,
    struct_name: str,
    required_fields: tuple[tuple[str, str], ...],
) -> tuple[int, int]:
    lines = path.read_text(encoding="utf-8").splitlines()
    total = 0
    modified = 0
    index = 0

    while index < len(lines):
        if not is_initializer_line(lines[index], struct_name):
            index += 1
            continue

        total += 1
        end = initializer_end(lines, index, path, struct_name)
        block = "\n".join(lines[index : end + 1])
        missing = [rendered for field, rendered in required_fields if f"{field}:" not in block]
        if missing:
            closing_indent = lines[end][: len(lines[end]) - len(lines[end].lstrip())]
            insertion = [f"{closing_indent}    {rendered}" for rendered in missing]
            lines[end:end] = insertion
            modified += 1
            end += len(insertion)
        index = end + 1

    if modified:
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    verification = path.read_text(encoding="utf-8").splitlines()
    index = 0
    verified = 0
    while index < len(verification):
        if not is_initializer_line(verification[index], struct_name):
            index += 1
            continue
        verified += 1
        end = initializer_end(verification, index, path, struct_name)
        block = "\n".join(verification[index : end + 1])
        absent = [field for field, _ in required_fields if f"{field}:" not in block]
        if absent:
            raise SystemExit(
                f"{path}: incomplete {struct_name} initializer at line {index + 1}: {absent}"
            )
        index = end + 1

    if verified != total:
        raise SystemExit(
            f"{path}: {struct_name} initializer count changed during migration: "
            f"{total} before, {verified} after"
        )
    return total, modified


def migrate_tree(root: Path) -> None:
    if not root.is_dir():
        raise SystemExit(f"missing command test root: {root}")

    specifications = (
        (
            "RemoteTtsSettings",
            (
                ("endpoint_is_loopback", "endpoint_is_loopback: None,"),
                ("availability_reason", "availability_reason: None,"),
            ),
            13,
        ),
        (
            "RemoteAsrSettings",
            (
                ("endpoint_is_loopback", "endpoint_is_loopback: None,"),
                ("availability_reason", "availability_reason: None,"),
            ),
            13,
        ),
        (
            "GetRuntimeStatusData",
            (("skill_discovery_diagnostics", "skill_discovery_diagnostics: Default::default(),"),),
            6,
        ),
    )

    totals = {name: [0, 0] for name, _, _ in specifications}
    for path in sorted(root.rglob("*.rs")):
        for struct_name, fields, _minimum in specifications:
            total, modified = ensure_initializer_fields(path, struct_name, fields)
            totals[struct_name][0] += total
            totals[struct_name][1] += modified

    for struct_name, _fields, minimum in specifications:
        total, modified = totals[struct_name]
        if total < minimum:
            raise SystemExit(
                f"fixture migration found only {total} {struct_name} initializers; "
                f"expected at least {minimum}"
            )
        print(f"Migrated {modified}/{total} {struct_name} test initializers")


def migrate_planner_redaction() -> None:
    path = Path("src-tauri/src/app_core/planner_redaction.rs")
    fields = (
        ("endpoint_is_loopback", "endpoint_is_loopback: None,"),
        ("availability_reason", "availability_reason: None,"),
    )
    for struct_name in ("RemoteTtsSettings", "RemoteAsrSettings"):
        total, modified = ensure_initializer_fields(path, struct_name, fields)
        if total < 1:
            raise SystemExit(f"{path}: expected at least one {struct_name} initializer")
        print(f"Migrated {modified}/{total} {struct_name} planner-redaction initializers")


def migrate_discovered_skills_iteration() -> None:
    path = Path("src-tauri/src/commands/tests/skill_selection.rs")
    source = path.read_text(encoding="utf-8")
    old = """    let matching_skills = loaded_skills
        .iter()
"""
    new = """    let matching_skills = loaded_skills
        .skills
        .iter()
"""
    if new not in source:
        count = source.count(old)
        if count != 1:
            raise SystemExit(
                f"{path}: expected exactly one DiscoveredSkills iteration migration, found {count}"
            )
        path.write_text(source.replace(old, new, 1), encoding="utf-8")

    repaired = path.read_text(encoding="utf-8")
    if repaired.count(new) != 1 or old in repaired:
        raise SystemExit(f"{path}: DiscoveredSkills iteration migration did not converge")
    print("Migrated DiscoveredSkills test iteration")


def main() -> int:
    migrate_planner_redaction()
    migrate_tree(Path("src-tauri/src/commands/tests"))
    migrate_discovered_skills_iteration()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
