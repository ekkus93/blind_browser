from pathlib import Path

path = Path("src-tauri/src/app_core/image_cache.rs")
text = path.read_text()
old_import = "use crate::browser::{BrowserScreenshotKind, BrowserScreenshotProvenance};\n"
new_import = "use crate::browser::BrowserScreenshotProvenance;\n#[cfg(test)]\nuse crate::browser::BrowserScreenshotKind;\n"
if text.count(old_import) != 1:
    raise SystemExit(f"expected one browser import, found {text.count(old_import)}")
text = text.replace(old_import, new_import, 1)

unused_helper = '''\n    fn persist_test_image(\n        cache: &mut ImageCache,\n        root: &Path,\n        context: ImageContext,\n        bytes: &[u8],\n        now_ms: u64,\n    ) -> Result<String, ToolError> {\n        cache.persist(root, context, provenance(), 100, 100, bytes, now_ms)\n    }\n'''
if text.count(unused_helper) != 1:
    raise SystemExit(f"expected one unused helper, found {text.count(unused_helper)}")
text = text.replace(unused_helper, "", 1)
path.write_text(text)
