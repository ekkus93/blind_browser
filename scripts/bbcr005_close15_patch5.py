from pathlib import Path

path = Path("src-tauri/src/state.rs")
content = path.read_text()

app_state_start = content.index("pub struct AppState {")
app_state_end = content.index("\n}\n", app_state_start)
app_state = content[app_state_start:app_state_end]
field = "    pub page_generation: u64,"
defaulted_field = "    #[serde(default)]\n    pub page_generation: u64,"

if defaulted_field in app_state:
    print("AppState page_generation already preserves legacy deserialization")
elif app_state.count(field) == 1:
    updated_app_state = app_state.replace(field, defaulted_field, 1)
    path.write_text(
        content[:app_state_start] + updated_app_state + content[app_state_end:]
    )
    print("Made AppState page_generation backward-compatible with legacy serialized state")
else:
    raise SystemExit("expected exactly one AppState page_generation field")
