from pathlib import Path
import base64, zlib


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))


image_cache = zlib.decompress(base64.b64decode("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")).decode()
image_cache_path = Path("src-tauri/src/app_core/image_cache.rs")
if image_cache_path.exists():
    raise SystemExit("image_cache.rs already exists")
image_cache_path.write_text(image_cache)

replace_once(
    "src-tauri/Cargo.toml",
    'tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }\nzeroize = "1"\n',
    'tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }\nuuid = { version = "1", features = ["v4"] }\nzeroize = "1"\n',
)

replace_once(
    "src-tauri/src/app_core/mod.rs",
    "use std::fs;\nuse std::path::PathBuf;\nuse std::sync::atomic::{AtomicU64, Ordering};\n",
    "use std::sync::atomic::{AtomicU64, Ordering};\n",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    "    recent_field_context: Option<RecentFieldContext>,\n    ocr: OcrController,\n",
    "    recent_field_context: Option<RecentFieldContext>,\n    image_cache: ImageCache,\n    ocr: OcrController,\n",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    "mod content_tools;\n\nmod extraction_tools;\n",
    "mod content_tools;\n\nmod image_cache;\nuse image_cache::ImageCache;\n\nmod extraction_tools;\n",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    "            browser,\n            recent_field_context: None,\n            ocr: OcrController::new(),\n",
    "            browser,\n            recent_field_context: None,\n            image_cache: ImageCache::default(),\n            ocr: OcrController::new(),\n",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    '''    fn next_image_id(&self, request_id: &str) -> String {
        self.next_id("image", request_id)
    }

''',
    "",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    '''    fn cached_image_dir(&self) -> Result<PathBuf, ToolError> {
        let cache_dir = self
            .app_handle
            .path()
            .app_cache_dir()
            .map_err(|error| ToolError {
                code: String::from("resolve_app_cache_dir_failed"),
                message: String::from(
                    "capture_screenshot could not resolve the app cache directory",
                ),
                retryable: true,
                details: Some(serde_json::json!({ "reason": error.to_string() })),
            })?;
        let image_dir = cache_dir.join("screenshots");
        fs::create_dir_all(&image_dir).map_err(|error| ToolError {
            code: String::from("create_screenshot_dir_failed"),
            message: String::from(
                "capture_screenshot could not create the screenshot cache directory",
            ),
            retryable: true,
            details: Some(serde_json::json!({
                "path": image_dir.display().to_string(),
                "reason": error.to_string(),
            })),
        })?;
        Ok(image_dir)
    }

    fn screenshot_output_path(&self, image_id: &str) -> Result<PathBuf, ToolError> {
        Ok(self.cached_image_dir()?.join(format!("{image_id}.png")))
    }

    fn cached_image_path(&self, image_id: &str) -> Result<PathBuf, ToolError> {
        Ok(self.cached_image_dir()?.join(format!("{image_id}.png")))
    }
''',
    "",
)

replace_once(
    "src-tauri/src/commands/contracts/tools.rs",
    "    pub image_id: String,\n    pub path: String,\n    pub bbox: Option<Rect>,\n",
    "    pub image_id: String,\n    pub bbox: Option<Rect>,\n",
)
replace_once(
    "src-tauri/src/commands/tests/fixtures/mock_executor_impl/media.rs",
    '''        CaptureScreenshotData {
            image_id: String::from("image-1"),
            path: String::from("/tmp/image-1.png"),
            bbox: input.bbox,
''',
    '''        CaptureScreenshotData {
            image_id: String::from("img_00000000000040008000000000000001"),
            bbox: input.bbox,
''',
)

replace_once(
    "src-tauri/src/app_core/content_tools.rs",
    "use std::fs;\n\nuse crate::commands::{\n",
    "use crate::commands::{\n    normalized_origin,\n",
)
replace_once(
    "src-tauri/src/app_core/content_tools.rs",
    '''        let image_id = self.next_image_id(&input.request_id);
        let screenshot_path = match self.screenshot_output_path(&image_id) {
            Ok(path) => path,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::CaptureScreenshot,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Screenshot capture completed, but the image could not be persisted to app storage.",
                    )],
                )
            }
        };
        if let Err(error) = fs::write(&screenshot_path, &browser_screenshot.image_bytes) {
            return ToolResult::failure(
                ToolName::CaptureScreenshot,
                input.request_id,
                ToolError {
                    code: String::from("screenshot_write_failed"),
                    message: String::from(
                        "capture_screenshot could not write the PNG file to app storage",
                    ),
                    retryable: true,
                    details: Some(serde_json::json!({
                        "path": screenshot_path.display().to_string(),
                        "reason": error.to_string(),
                    })),
                },
                vec![String::from(
                    "Screenshot capture completed, but writing the PNG file to disk failed.",
                )],
            );
        }

        let mut observations = vec![format!(
            "Captured a deterministic browser screenshot and persisted it as {image_id}.png."
        )];
''',
    '''        let page_id = self
            .state
            .current_page_id
            .clone()
            .expect("capture_screenshot checked that an active page exists");
        let origin = normalized_origin(Some(browser_screenshot.url.as_str()));
        let image_id = match self.persist_screenshot_image(
            page_id,
            origin,
            self.state.page_generation,
            &browser_screenshot.image_bytes,
        ) {
            Ok(handle) => handle,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::CaptureScreenshot,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Screenshot capture completed, but the image could not be registered in private app storage.",
                    )],
                )
            }
        };

        let mut observations = vec![String::from(
            "Captured a deterministic browser screenshot and persisted it behind an opaque application-owned image handle.",
        )];
''',
)
replace_once(
    "src-tauri/src/app_core/content_tools.rs",
    "                image_id,\n                path: screenshot_path.display().to_string(),\n                bbox: browser_screenshot.bbox,\n",
    "                image_id,\n                bbox: browser_screenshot.bbox,\n",
)

replace_once(
    "src-tauri/src/app_core/extraction_tools/ocr_tools.rs",
    '''        let image_id = input
            .image_id
            .as_deref()
            .map(str::trim)
            .filter(|image_id| !image_id.is_empty())
            .map(ToOwned::to_owned);
''',
    "        let image_id = input.image_id;\n",
)
replace_once(
    "src-tauri/src/app_core/extraction_tools/ocr_tools.rs",
    '''        let image_path = match self.cached_image_path(&image_id) {
            Ok(path) => path,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR could not resolve the cached screenshot path.",
                    )],
                )
            }
        };

        if !image_path.is_file() {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("ocr_image_not_found"),
                    message: String::from(
                        "run_ocr could not find the cached screenshot for the requested image_id",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "image_id": image_id,
                        "path": image_path.display().to_string(),
                    })),
                },
                vec![String::from(
                    "OCR could not start because the requested cached screenshot does not exist.",
                )],
            );
        }
''',
    '''        let image_path = match self.resolve_screenshot_image(&image_id) {
            Ok(path) => path,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR could not resolve the opaque screenshot handle for the current page state.",
                    )],
                )
            }
        };
''',
)

print("Applied BBCR-006 opaque screenshot handle transformation")
