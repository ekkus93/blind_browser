from pathlib import Path


def replace_exact(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"expected exactly one match in {path}, found {count}")
    target.write_text(content.replace(old, new, 1), encoding="utf-8")


replace_exact(
    "src-tauri/src/app_core/tests/mod.rs",
    "use crate::state::AppState;\n\nmod helpers;\n",
    "use crate::state::AppState;\n\n"
    "static WRY_TEST_RUNTIME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());\n\n"
    "fn lock_wry_test_runtime() -> std::sync::MutexGuard<'static, ()> {\n"
    "    WRY_TEST_RUNTIME_LOCK\n"
    "        .lock()\n"
    "        .expect(\"Wry test runtime lock must not be poisoned\")\n"
    "}\n\n"
    "mod helpers;\n",
)

replace_exact(
    "src-tauri/src/app_core/tests/confirmation_replay_tests.rs",
    "fn app_core_confirmation_replay_and_runtime_state_binding_are_enforced() {\n"
    "    let builder = tauri::Builder::<tauri::Wry>::default();\n",
    "fn app_core_confirmation_replay_and_runtime_state_binding_are_enforced() {\n"
    "    let _wry_test_guard = lock_wry_test_runtime();\n"
    "    let builder = tauri::Builder::<tauri::Wry>::default();\n",
)

replace_exact(
    "src-tauri/src/app_core/tests/remote_privacy_api_tests.rs",
    '        "http://api.example.com/v1",\n',
    "",
)

replace_exact(
    "src-tauri/src/app_core/tests/remote_privacy_api_tests.rs",
    "fn remote_privacy_status_reports_every_non_ephemeral_decision_and_stale_rules() {\n"
    "    let builder = tauri::Builder::<tauri::Wry>::default();\n",
    "fn remote_privacy_status_reports_every_non_ephemeral_decision_and_stale_rules() {\n"
    "    let _wry_test_guard = lock_wry_test_runtime();\n"
    "    let builder = tauri::Builder::<tauri::Wry>::default();\n",
)

replace_exact(
    "src-tauri/src/app_core/tests/remote_privacy_api_tests.rs",
    "fn remote_privacy_operations_fail_closed_without_unnecessary_persistence() {\n"
    "    let builder = tauri::Builder::<tauri::Wry>::default();\n",
    "fn remote_privacy_operations_fail_closed_without_unnecessary_persistence() {\n"
    "    let _wry_test_guard = lock_wry_test_runtime();\n"
    "    let builder = tauri::Builder::<tauri::Wry>::default();\n",
)
