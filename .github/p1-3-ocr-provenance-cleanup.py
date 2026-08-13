from pathlib import Path


def replace_once(path: Path, old: str, new: str, label: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    path.write_text(text.replace(old, new, 1))


image_cache = Path("src-tauri/src/app_core/image_cache.rs")
content_tools = Path("src-tauri/src/app_core/content_tools.rs")

# Keep capture-kind debug detail test-only while binding persistence to a complete
# BrowserScreenshotState at the AppCore boundary.
replace_once(
    image_cache,
    "use crate::browser::{BrowserScreenshotKind, BrowserScreenshotProvenance};\n",
    "use crate::browser::{BrowserScreenshotProvenance, BrowserScreenshotState};\n#[cfg(test)]\nuse crate::browser::BrowserScreenshotKind;\n",
    "browser imports",
)

replace_once(
    image_cache,
    """#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageContext {
    page_id: String,
    origin: Option<String>,
    generation: u64,
}
""",
    """#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageContext {
    page_id: String,
    origin: Option<String>,
    generation: u64,
}

#[derive(Debug, Clone, Copy)]
struct ScreenshotRasterMetadata {
    provenance: BrowserScreenshotProvenance,
    width: u32,
    height: u32,
}
""",
    "raster metadata type",
)

replace_once(
    image_cache,
    """        context: ImageContext,
        provenance: BrowserScreenshotProvenance,
        width: u32,
        height: u32,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<String, ToolError> {
        if !provenance.document_origin_x.is_finite()
""",
    """        context: ImageContext,
        raster: ScreenshotRasterMetadata,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<String, ToolError> {
        let ScreenshotRasterMetadata {
            provenance,
            width,
            height,
        } = raster;
        if !provenance.document_origin_x.is_finite()
""",
    "cache persist argument grouping",
)

replace_once(
    image_cache,
    """        generation: u64,
        provenance: BrowserScreenshotProvenance,
        width: u32,
        height: u32,
        bytes: &[u8],
    ) -> Result<String, ToolError> {
        let root = self.screenshot_cache_root()?;
        self.image_cache.persist(
            &root,
            ImageContext {
                page_id,
                origin,
                generation,
            },
            provenance,
            width,
            height,
            bytes,
            current_timestamp_ms(),
        )
""",
    """        generation: u64,
        screenshot: &BrowserScreenshotState,
    ) -> Result<String, ToolError> {
        let root = self.screenshot_cache_root()?;
        self.image_cache.persist(
            &root,
            ImageContext {
                page_id,
                origin,
                generation,
            },
            ScreenshotRasterMetadata {
                provenance: screenshot.provenance,
                width: screenshot.width,
                height: screenshot.height,
            },
            &screenshot.image_bytes,
            current_timestamp_ms(),
        )
""",
    "appcore screenshot persistence binding",
)

# Remove the temporary helper introduced by the first-stage patch.
text = image_cache.read_text()
unused_helper = '''\n    fn persist_test_image(\n        cache: &mut ImageCache,\n        root: &Path,\n        context: ImageContext,\n        bytes: &[u8],\n        now_ms: u64,\n    ) -> Result<String, ToolError> {\n        cache.persist(root, context, provenance(), 100, 100, bytes, now_ms)\n    }\n'''
if text.count(unused_helper) != 1:
    raise SystemExit(f"unused test helper: expected one match, found {text.count(unused_helper)}")
text = text.replace(unused_helper, "", 1)

# The first-stage patch updates existing cache tests with separate provenance and
# dimensions. Collapse those repeated values into the same typed raster metadata
# used by production code. Some call sites are compact and others are multiline.
old_test_args = "provenance(), 100, 100,"
new_test_args = "ScreenshotRasterMetadata { provenance: provenance(), width: 100, height: 100 },"
if text.count(old_test_args) < 1:
    raise SystemExit("compact test raster arguments: expected at least one match")
text = text.replace(old_test_args, new_test_args)

old_multiline_test_args = """                provenance(),
                100,
                100,
"""
new_multiline_test_args = """                ScreenshotRasterMetadata {
                    provenance: provenance(),
                    width: 100,
                    height: 100,
                },
"""
if text.count(old_multiline_test_args) < 1:
    raise SystemExit("multiline test raster arguments: expected at least one match")
text = text.replace(old_multiline_test_args, new_multiline_test_args)
image_cache.write_text(text)

replace_once(
    content_tools,
    """            self.state.page_generation,
            browser_screenshot.provenance,
            browser_screenshot.width,
            browser_screenshot.height,
            &browser_screenshot.image_bytes,
""",
    """            self.state.page_generation,
            &browser_screenshot,
""",
    "content tools screenshot persistence binding",
)
