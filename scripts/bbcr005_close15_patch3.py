from pathlib import Path

path = Path("src-tauri/src/commands/contracts/planner.rs")
content = path.read_text()
old = "    #[serde(skip_serializing, default)]\n    #[schemars(skip)]\n    pub runtime_state_token: String,"
new = "    #[serde(skip, default)]\n    #[schemars(skip)]\n    pub runtime_state_token: String,"
if content.count(old) != 1:
    raise SystemExit("expected exactly one pending-confirmation runtime token field")
path.write_text(content.replace(old, new, 1))

print("Made pending confirmation runtime token server-only on serialization and deserialization")
