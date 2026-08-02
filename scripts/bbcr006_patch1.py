from pathlib import Path


path = Path("src-tauri/src/app_core/mod.rs")
old = "use tauri::{AppHandle, Manager};"
new = "use tauri::AppHandle;"
text = path.read_text()
count = text.count(old)
if count != 1:
    raise SystemExit(
        f"{path}: expected exactly one obsolete Manager import, found {count}"
    )
path.write_text(text.replace(old, new, 1))
print("Applied BBCR-006 strict-Clippy import correction")
