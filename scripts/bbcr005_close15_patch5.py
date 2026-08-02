from pathlib import Path

path = Path("src-tauri/src/state.rs")
content = path.read_text()
old = "    pub current_page_id: Option<String>,\n    pub page_generation: u64,\n    pub last_navigation_origin: Option<String>,"
new = "    pub current_page_id: Option<String>,\n    #[serde(default)]\n    pub page_generation: u64,\n    pub last_navigation_origin: Option<String>,"
if content.count(old) != 1:
    raise SystemExit("expected exactly one AppState page_generation field")
path.write_text(content.replace(old, new, 1))

print("Made page_generation backward-compatible with legacy serialized AppState")
