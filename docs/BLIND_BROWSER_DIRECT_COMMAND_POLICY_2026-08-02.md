# Blind Browser Direct Command Policy

**Date:** 2026-08-02  
**Authoritative registry:** `src-tauri/src/direct_command_policy.rs`  
**Tauri surface:** `src-tauri/src/lib.rs`

## Purpose

Planner-mediated tools already pass through deterministic action policy, confirmation, runtime snapshots, and executor preflight. Tauri commands are a separate entry surface used by the UI. This document records the required parity contract: every generated Tauri handler must appear in the typed direct-command registry, and every registry entry must state its side effects before implementation.

## Policy fields

- `class`: the existing deterministic `ActionClass`.
- `requires_user_gesture`: the command must originate from an explicit UI/user action rather than background automation.
- `mutates_runtime_state`: changes active application or browser state.
- `mutates_config`: persists or changes configuration.
- `persists_secret`: writes a credential to the OS keyring and a scoped reference to config.
- `performs_network_io`: may initiate a network request.
- `credential_bearing_network_io`: may transmit API credentials.
- `transmits_page_context`: may transmit sanitized page/OCR context to a remote planner.
- `downloads_executable_or_model_artifact`: replaces a local runtime artifact and must use verified downloads.
- `launches_external_program`: invokes an OS URL opener and requires strict URL validation.

## Command inventory

| Group | Commands | Required contract |
|---|---|---|
| Planner lifecycle | `resolve_command`, `execute_planner_output`, `submit_confirmation_response`, `transcribe_and_execute_command` | User initiated; planner transmission uses privacy enforcement; execution and confirmation remain runtime-bound. |
| Voice capture | `start_listening`, `stop_listening`, `transcribe_command` | Runtime mutation; remote ASR is credential-bearing and timeout/no-redirect protected. |
| Browser/navigation | `open_url`, `open_external_url` | User initiated; external URLs are parsed HTTPS URLs without credentials, query, fragment, or control characters. |
| State reads | `get_agent_state`, `get_model_management_settings` | Read-only; no network or persistence. |
| Audio/UI settings | `set_playback_volume`, `set_playback_speed`, `set_browser_visibility`, `set_tts_voice` | User initiated; explicit runtime/config mutation. |
| Safety/privacy/OCR settings | `set_confirmation_threshold`, `set_allow_click_without_confirmation`, `set_remote_planner_privacy_settings`, `set_ocr_thresholds` | User initiated config mutation; persistence failure is surfaced. |
| Provider settings | `set_remote_planner_connection_settings`, `reset_remote_planner_connection_settings`, `set_tts_provider_selection`, `set_asr_provider_selection`, `set_tts_model_selection` | User initiated config mutation; endpoint validation remains authoritative. |
| Credential operations | `set_remote_planner_api_key`, `set_remote_tts_api_key`, `set_remote_asr_api_key`, `test_remote_planner_api_key`, `test_remote_tts_api_key`, `test_remote_asr_api_key`, `list_remote_planner_models` | User initiated; stored secrets are endpoint scoped; credential-bearing requests use timeouts and refuse redirects; successful persistence must return a non-empty keyring reference. |
| Model management | `set_model_management_settings`, `download_active_local_tts_model`, `download_active_local_asr_model` | Config mutation or verified model-artifact download; unknown models fail closed; downloaded bytes are revision/hash/size verified before atomic replacement. |

## Enforcement

`direct_command_registry_matches_tauri_handler_surface` parses `tauri::generate_handler!` and compares it with `DirectCommandName::ALL`. Adding or removing a handler without updating the registry fails tests. Startup also validates policy invariants so production builds use the registry and cannot allow it to decay into test-only metadata.

No direct command may transmit page or OCR context except through the remote planner path that calls `sanitize_remote_planner_input`. No direct model download may write a final model path before integrity verification. No API-key setter may report success with an absent persisted reference.
