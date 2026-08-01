from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content)


def replace_exact(path: str, old: str, new: str, expected_count: int) -> None:
    content = read(path)
    count = content.count(old)
    if count != expected_count:
        raise SystemExit(
            f"{path}: expected {expected_count} matches for {old!r}, found {count}"
        )
    write(path, content.replace(old, new))


replace_exact(
    "src-tauri/src/config/tests/keyring_tests.rs",
    "resolve_secret_ref_for_endpoint(",
    "crate::config::resolve_secret_ref_for_endpoint(",
    2,
)

replace_exact(
    "src-tauri/src/app_core/tests/planner_tests.rs",
    "use super::*;\n",
    "use super::*;\nuse crate::provider_endpoint::ProviderEndpointScope;\n",
    1,
)

replace_exact(
    "src-tauri/src/app_core/tests/planner_tests.rs",
    "test_openai_api_key_connectivity(\n        &base_url,",
    "test_openai_api_key_connectivity(\n        &ProviderEndpointScope::parse(&base_url)\n            .expect(\"test server URL should be a valid provider endpoint\"),",
    2,
)

replace_exact(
    "src-tauri/src/app_core/tests/planner_tests.rs",
    "fetch_openai_compatible_models(\n        &base_url,",
    "fetch_openai_compatible_models(\n        &ProviderEndpointScope::parse(&base_url)\n            .expect(\"test server URL should be a valid provider endpoint\"),",
    2,
)

replace_exact(
    "src-tauri/src/provider_endpoint.rs",
    '''fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    host.parse::<IpAddr>()
        .map(|address| address.is_loopback())
        .unwrap_or(false)
}''',
    '''fn is_loopback_host(host: &str) -> bool {
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    host.parse::<IpAddr>()
        .map(|address| address.is_loopback())
        .unwrap_or(false)
}''',
    1,
)

replace_exact(
    "src-tauri/src/config/tests/persistence_tests.rs",
    'message.contains("valid absolute URL")',
    'message.contains("absolute URL")',
    1,
)
