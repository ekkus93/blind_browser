#!/usr/bin/env python3
"""Fail closed on remote-planner consent data leaking into ambient state or logs."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

REPO_ROOT = Path(__file__).resolve().parents[1]

REQUIRED_PATHS = (
    "src/planner-orchestration.ts",
    "src/remote-planner-privacy-state.ts",
    "src/remote-planner-privacy-controller.ts",
    "src/ui-store.ts",
    "src/store.ts",
    "src-tauri/src/app_core/remote_privacy_api.rs",
    "src-tauri/src/app_core/remote_data_consent.rs",
    "src-tauri/src/commands/contracts/planner.rs",
    "src-tauri/src/commands/contracts/providers.rs",
)

FRONTEND_STATE_PATHS = (
    "src/planner-orchestration.ts",
    "src/remote-planner-privacy-state.ts",
    "src/ui-store.ts",
    "src/store.ts",
)

PROHIBITED_STATE_IDENTIFIERS = (
    "sanitized_input",
    "sanitizedInput",
    "planner_input",
    "plannerInput",
    "request_payload",
    "requestPayload",
    "raw_transcript",
    "rawTranscript",
    "page_model",
    "pageModel",
    "ocr_text",
    "ocrText",
    "tool_results",
    "toolResults",
    "skill_summaries",
    "skillSummaries",
)

LOG_SENSITIVE_IDENTIFIERS = (
    "challenge_digest",
    "challengeDigest",
    "sanitized_input",
    "sanitizedInput",
    "planner_input",
    "plannerInput",
    "request_payload",
    "requestPayload",
    "raw_transcript",
    "rawTranscript",
    "page_model",
    "pageModel",
    "ocr_text",
    "ocrText",
    "recent_tool_results",
    "recentToolResults",
    "relevant_skill_summaries",
    "relevantSkillSummaries",
)

CRITICAL_FALLBACK_PATTERNS = (
    re.compile(r"remote_data_consent[^\n]{0,160}(?:unwrap_or|unwrap_or_else|unwrap_or_default)"),
    re.compile(r"pending_remote_planner_consent[^\n]{0,160}\.ok\(\)"),
    re.compile(r"remote_planner_privacy[^\n]{0,160}unwrap_or_default"),
)

CONSOLE_CALL_RE = re.compile(
    r"\bconsole\.(?:log|debug|info|warn|error)\s*\((.*?)\)\s*;",
    re.DOTALL,
)


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    message: str

    def render(self) -> str:
        return f"{self.path}:{self.line}: {self.message}"


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def require_readable(path: Path) -> str:
    try:
        data = path.read_bytes()
    except OSError as exc:
        raise RuntimeError(f"required source is unreadable: {path}: {exc}") from exc
    if b"\x00" in data:
        raise RuntimeError(f"required source contains NUL bytes: {path}")
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise RuntimeError(f"required source is not UTF-8: {path}: {exc}") from exc


def extract_named_struct(text: str, struct_name: str) -> tuple[int, str] | None:
    match = re.search(rf"\bstruct\s+{re.escape(struct_name)}\s*\{{", text)
    if not match:
        return None
    brace = text.find("{", match.start())
    depth = 0
    for index in range(brace, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return match.start(), text[match.start() : index + 1]
    return None


def audit_frontend_state(path: str, text: str) -> list[Finding]:
    findings: list[Finding] = []
    for identifier in PROHIBITED_STATE_IDENTIFIERS:
        for match in re.finditer(rf"\b{re.escape(identifier)}\b\s*[?:]", text):
            findings.append(
                Finding(
                    path,
                    line_number(text, match.start()),
                    f"ambient frontend privacy/consent state declares prohibited field {identifier!r}",
                )
            )
    return findings


def audit_logging(path: str, text: str) -> list[Finding]:
    findings: list[Finding] = []
    for call in CONSOLE_CALL_RE.finditer(text):
        body = call.group(1)
        for identifier in LOG_SENSITIVE_IDENTIFIERS:
            if re.search(rf"\b{re.escape(identifier)}\b", body):
                findings.append(
                    Finding(
                        path,
                        line_number(text, call.start()),
                        f"production console call references sensitive consent value {identifier!r}",
                    )
                )
    return findings


def audit_backend_status(path: str, text: str) -> list[Finding]:
    findings: list[Finding] = []
    for name in ("RemotePlannerConsentChallengeSummary", "RemotePlannerPrivacyStatus"):
        extracted = extract_named_struct(text, name)
        if extracted is None:
            findings.append(Finding(path, 1, f"required ambient status struct {name} is missing"))
            continue
        offset, block = extracted
        for identifier in (
            "challenge_digest",
            "sanitized_input",
            "planner_input",
            "payload_digest",
            "transcript",
            "page_model",
            "ocr_text",
            "tool_results",
            "skill_summaries",
        ):
            match = re.search(rf"\b{re.escape(identifier)}\b", block)
            if match:
                findings.append(
                    Finding(
                        path,
                        line_number(text, offset + match.start()),
                        f"ambient backend status exposes prohibited field {identifier!r}",
                    )
                )
    return findings


def audit_critical_fallbacks(path: str, text: str) -> list[Finding]:
    findings: list[Finding] = []
    for pattern in CRITICAL_FALLBACK_PATTERNS:
        for match in pattern.finditer(text):
            findings.append(
                Finding(
                    path,
                    line_number(text, match.start()),
                    "critical consent/privacy path contains a defaulting or swallowed-error pattern",
                )
            )
    return findings


def audit_sources(root: Path) -> list[Finding]:
    source: dict[str, str] = {}
    for relative in REQUIRED_PATHS:
        path = root / relative
        if not path.is_file():
            raise RuntimeError(f"required source path is missing: {relative}")
        source[relative] = require_readable(path)

    findings: list[Finding] = []
    for relative in FRONTEND_STATE_PATHS:
        findings.extend(audit_frontend_state(relative, source[relative]))

    for path in sorted((root / "src").rglob("*")):
        if not path.is_file() or path.suffix not in {".ts", ".tsx", ".mjs"}:
            continue
        if path.name.endswith(".test.mjs"):
            continue
        relative = path.relative_to(root).as_posix()
        findings.extend(audit_logging(relative, require_readable(path)))

    backend_status = "src-tauri/src/commands/contracts/providers.rs"
    findings.extend(audit_backend_status(backend_status, source[backend_status]))
    for relative in (
        "src-tauri/src/app_core/remote_data_consent.rs",
        "src-tauri/src/app_core/remote_privacy_api.rs",
        "src/remote-planner-privacy-controller.ts",
    ):
        findings.extend(audit_critical_fallbacks(relative, source[relative]))

    # Explicit challenge digest is required for response binding. Fail closed if
    # that contract silently disappears, because broad rules must never replace it.
    contract = source["src-tauri/src/commands/contracts/planner.rs"]
    challenge = extract_named_struct(contract, "RemotePlannerConsentChallenge")
    if challenge is None or "challenge_digest" not in challenge[1]:
        findings.append(
            Finding(
                "src-tauri/src/commands/contracts/planner.rs",
                1,
                "explicit consent challenge no longer carries response-binding challenge_digest",
            )
        )

    controller = source["src/remote-planner-privacy-controller.ts"]
    if "challengeDigest: consentState.challenge.challenge_digest" not in controller:
        findings.append(
            Finding(
                "src/remote-planner-privacy-controller.ts",
                1,
                "frontend response path no longer binds the explicit challenge digest",
            )
        )
    return findings


def expect_finding(findings: Iterable[Finding], text: str) -> None:
    if not any(text in finding.message for finding in findings):
        rendered = "\n".join(finding.render() for finding in findings)
        raise AssertionError(f"expected finding containing {text!r}; got:\n{rendered}")


def self_test() -> None:
    unsafe_state = "interface State { sanitizedInput?: string; }"
    expect_finding(
        audit_frontend_state("unsafe.ts", unsafe_state),
        "prohibited field",
    )

    unsafe_log = "console.warn('bad', { challengeDigest });"
    expect_finding(audit_logging("unsafe.ts", unsafe_log), "console call")

    unsafe_status = """
    struct RemotePlannerConsentChallengeSummary {
        challenge_digest: String,
    }
    struct RemotePlannerPrivacyStatus {
        sanitized_input: String,
    }
    """
    status_findings = audit_backend_status("unsafe.rs", unsafe_status)
    expect_finding(status_findings, "challenge_digest")
    expect_finding(status_findings, "sanitized_input")

    unsafe_fallback = "remote_data_consent.unwrap_or_default()"
    expect_finding(
        audit_critical_fallbacks("unsafe.rs", unsafe_fallback),
        "defaulting or swallowed-error",
    )

    safe_state = """
    type RemoteDataConsentUiState = {
      challenge: RemotePlannerConsentChallenge;
      submissionError: RemoteDataConsentSubmissionFailure | null;
    };
    """
    assert not audit_frontend_state("safe.ts", safe_state)

    safe_log = "console.warn('stale consent', { challengeId, decision });"
    assert not audit_logging("safe.ts", safe_log)

    safe_status = """
    struct RemotePlannerConsentChallengeSummary {
        challenge_id: String,
        page_origin: String,
        endpoint_display: String,
    }
    struct RemotePlannerPrivacyStatus {
        pending_challenge: Option<RemotePlannerConsentChallengeSummary>,
    }
    """
    assert not audit_backend_status("safe.rs", safe_status)

    explicit_binding = "challengeDigest: consentState.challenge.challenge_digest"
    assert "challenge_digest" in explicit_binding


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        print("remote planner privacy state scanner self-test: PASS")
        return 0

    try:
        findings = audit_sources(REPO_ROOT)
    except RuntimeError as exc:
        print(f"remote planner privacy state scanner: ERROR: {exc}", file=sys.stderr)
        return 2

    if findings:
        print("remote planner privacy state scanner: FAIL", file=sys.stderr)
        for finding in findings:
            print(f"  {finding.render()}", file=sys.stderr)
        return 1

    print("remote planner privacy state scanner: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
