from pathlib import Path

path = Path("src-tauri/src/state.rs")
content = path.read_text()
old = "    pub current_page: Option<PageModel>,\n    pub page_generation: u64,"
new = "    pub current_page: Option<PageModel>,\n    #[serde(default)]\n    pub page_generation: u64,"
count = content.count(old)
if count != 1:
    raise SystemExit(f"expected one page_generation field anchor, found {count}")
path.write_text(content.replace(old, new, 1))
print("Applied legacy page_generation deserialization default")
