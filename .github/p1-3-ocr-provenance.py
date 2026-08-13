from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    p.write_text(text.replace(old, new, 1))


# 1. Browser screenshot state carries explicit raster provenance.
replace_once(
    "src-tauri/src/browser/config.rs",
    """#[derive(Debug, Clone, PartialEq)]
pub struct BrowserScreenshotState {
""",
    """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserScreenshotKind {
    Viewport,
    FullPage,
    DocumentClip,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrowserScreenshotProvenance {
    pub kind: BrowserScreenshotKind,
    pub document_origin_x: f32,
    pub document_origin_y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowserScreenshotState {
""",
    "browser screenshot provenance types",
)
replace_once(
    "src-tauri/src/browser/config.rs",
    """    pub image_bytes: Vec<u8>,
    pub bbox: Option<Rect>,
    pub width: u32,
""",
    """    pub image_bytes: Vec<u8>,
    pub bbox: Option<Rect>,
    pub provenance: BrowserScreenshotProvenance,
    pub width: u32,
""",
    "browser screenshot provenance field",
)

# 2. Capture the document-space raster origin at screenshot time.
replace_once(
    "src-tauri/src/browser/page_inspection.rs",
    """use super::{BrowserError, BrowserEvalState, BrowserHtmlState, BrowserScreenshotState};
""",
    """use super::{
    BrowserError, BrowserEvalState, BrowserHtmlState, BrowserScreenshotKind,
    BrowserScreenshotProvenance, BrowserScreenshotState,
};
""",
    "page inspection imports",
)
replace_once(
    "src-tauri/src/browser/page_inspection.rs",
    """struct DocumentScreenshotDimensions {
    width: u32,
    height: u32,
}
""",
    """struct DocumentScreenshotDimensions {
    width: u32,
    height: u32,
}

#[cfg(feature = "browser")]
#[derive(serde::Deserialize)]
struct ViewportScrollOrigin {
    x: f32,
    y: f32,
}
""",
    "viewport scroll origin type",
)
old_block = """            let screenshot_bytes = tauri::async_runtime::block_on(async {
                let mut builder = ScreenshotParams::builder().format(CaptureScreenshotFormat::Png);
                if full_page {
                    let dimensions = page
                        .evaluate(
                            \"({ width: Math.ceil(Math.max(document.documentElement?.scrollWidth || 0, document.body?.scrollWidth || 0, window.innerWidth || 0)), height: Math.ceil(Math.max(document.documentElement?.scrollHeight || 0, document.body?.scrollHeight || 0, window.innerHeight || 0)) })\",
                        )
                        .await
                        .map_err(|error| BrowserError::Screenshot(error.to_string()))?
                        .into_value::<DocumentScreenshotDimensions>()
                        .map_err(|error| BrowserError::Screenshot(error.to_string()))?;
                    validate_image_dimensions(dimensions.width, dimensions.height)
                        .map_err(image_limit_to_browser_error)?;
                    builder = builder.full_page(true);
                }
                if let Some(bbox) = bbox.as_ref() {
                    if bbox.width <= 0.0 || bbox.height <= 0.0 {
                        return Err(BrowserError::Screenshot(String::from(
                            \"bbox screenshots require positive width and height\",
                        )));
                    }
                    // CDP's Page.captureScreenshot clip.x/y are always
                    // document/page-absolute (confirmed empirically, not
                    // just from docs -- independent of full_page/
                    // captureBeyondViewport), which is exactly the
                    // coordinate space Rect::bbox is documented to be in
                    // (see page_model::Rect). Passed straight through here
                    // with no scroll correction because none is needed --
                    // the correction already happened once, at extraction
                    // (dom_extraction.rs), not here.
                    builder = builder.clip(Viewport {
                        x: f64::from(bbox.x),
                        y: f64::from(bbox.y),
                        width: f64::from(bbox.width),
                        height: f64::from(bbox.height),
                        scale: 1.0,
                    });
                }
                page.screenshot(builder.build())
                    .await
                    .map_err(|error| BrowserError::Screenshot(error.to_string()))
            })?;
"""
new_block = """            let (screenshot_bytes, provenance) = tauri::async_runtime::block_on(async {
                let mut builder = ScreenshotParams::builder().format(CaptureScreenshotFormat::Png);
                let provenance = if let Some(bbox) = bbox.as_ref() {
                    BrowserScreenshotProvenance {
                        kind: BrowserScreenshotKind::DocumentClip,
                        document_origin_x: bbox.x,
                        document_origin_y: bbox.y,
                    }
                } else if full_page {
                    BrowserScreenshotProvenance {
                        kind: BrowserScreenshotKind::FullPage,
                        document_origin_x: 0.0,
                        document_origin_y: 0.0,
                    }
                } else {
                    let origin = page
                        .evaluate(\"({ x: Number(window.scrollX) || 0, y: Number(window.scrollY) || 0 })\")
                        .await
                        .map_err(|error| BrowserError::Screenshot(error.to_string()))?
                        .into_value::<ViewportScrollOrigin>()
                        .map_err(|error| BrowserError::Screenshot(error.to_string()))?;
                    BrowserScreenshotProvenance {
                        kind: BrowserScreenshotKind::Viewport,
                        document_origin_x: origin.x,
                        document_origin_y: origin.y,
                    }
                };
                if full_page {
                    let dimensions = page
                        .evaluate(
                            \"({ width: Math.ceil(Math.max(document.documentElement?.scrollWidth || 0, document.body?.scrollWidth || 0, window.innerWidth || 0)), height: Math.ceil(Math.max(document.documentElement?.scrollHeight || 0, document.body?.scrollHeight || 0, window.innerHeight || 0)) })\",
                        )
                        .await
                        .map_err(|error| BrowserError::Screenshot(error.to_string()))?
                        .into_value::<DocumentScreenshotDimensions>()
                        .map_err(|error| BrowserError::Screenshot(error.to_string()))?;
                    validate_image_dimensions(dimensions.width, dimensions.height)
                        .map_err(image_limit_to_browser_error)?;
                    builder = builder.full_page(true);
                }
                if let Some(bbox) = bbox.as_ref() {
                    if bbox.width <= 0.0 || bbox.height <= 0.0 {
                        return Err(BrowserError::Screenshot(String::from(
                            \"bbox screenshots require positive width and height\",
                        )));
                    }
                    // CDP's Page.captureScreenshot clip.x/y are always
                    // document/page-absolute (confirmed empirically, not
                    // just from docs -- independent of full_page/
                    // captureBeyondViewport), which is exactly the
                    // coordinate space Rect::bbox is documented to be in.
                    builder = builder.clip(Viewport {
                        x: f64::from(bbox.x),
                        y: f64::from(bbox.y),
                        width: f64::from(bbox.width),
                        height: f64::from(bbox.height),
                        scale: 1.0,
                    });
                }
                let bytes = page
                    .screenshot(builder.build())
                    .await
                    .map_err(|error| BrowserError::Screenshot(error.to_string()))?;
                Ok::<_, BrowserError>((bytes, provenance))
            })?;
"""
replace_once(
    "src-tauri/src/browser/page_inspection.rs",
    old_block,
    new_block,
    "screenshot capture provenance",
)
replace_once(
    "src-tauri/src/browser/page_inspection.rs",
    """                image_bytes: screenshot_bytes,
                bbox,
                width,
""",
    """                image_bytes: screenshot_bytes,
                bbox,
                provenance,
                width,
""",
    "screenshot state provenance assignment",
)

# 3. Persist provenance with the opaque image handle and translate document bboxes fail-closed.
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """use crate::commands::{current_timestamp_ms, normalized_origin, ToolError};
""",
    """use crate::browser::{BrowserScreenshotKind, BrowserScreenshotProvenance};
use crate::commands::{current_timestamp_ms, normalized_origin, ToolError};
use crate::page_model::Rect;
""",
    "image cache imports",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """struct ImageRecord {
    path: PathBuf,
    context: ImageContext,
    created_ms: u64,
""",
    """struct ImageRecord {
    path: PathBuf,
    context: ImageContext,
    provenance: BrowserScreenshotProvenance,
    width: u32,
    height: u32,
    created_ms: u64,
""",
    "image record provenance",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """#[derive(Debug)]
pub(super) struct ImageCache {
""",
    """#[derive(Debug, Clone)]
pub(super) struct ResolvedScreenshotImage {
    path: PathBuf,
    provenance: BrowserScreenshotProvenance,
    width: u32,
    height: u32,
}

impl ResolvedScreenshotImage {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn ocr_bbox_for_document_bbox(&self, bbox: &Rect) -> Result<Rect, ToolError> {
        if !bbox.x.is_finite()
            || !bbox.y.is_finite()
            || !bbox.width.is_finite()
            || !bbox.height.is_finite()
            || bbox.width <= 0.0
            || bbox.height <= 0.0
        {
            return Err(error(
                \"invalid_ocr_bbox\",
                \"OCR document bbox must contain finite coordinates and positive dimensions\",
                false,
            ));
        }
        if !self.provenance.document_origin_x.is_finite()
            || !self.provenance.document_origin_y.is_finite()
            || self.width == 0
            || self.height == 0
        {
            return Err(error(
                \"invalid_screenshot_provenance\",
                \"cached screenshot coordinate provenance is invalid\",
                false,
            ));
        }

        let left = bbox.x - self.provenance.document_origin_x;
        let top = bbox.y - self.provenance.document_origin_y;
        let right = left + bbox.width;
        let bottom = top + bbox.height;
        let image_width = self.width as f32;
        let image_height = self.height as f32;
        const EPSILON: f32 = 0.01;

        if !left.is_finite()
            || !top.is_finite()
            || !right.is_finite()
            || !bottom.is_finite()
            || left < -EPSILON
            || top < -EPSILON
            || right > image_width + EPSILON
            || bottom > image_height + EPSILON
        {
            return Err(ToolError {
                code: String::from(\"ocr_bbox_outside_screenshot\"),
                message: String::from(
                    \"requested document bbox is not fully represented by the cached screenshot\",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    \"capture_kind\": format!(\"{:?}\", self.provenance.kind),
                    \"document_origin_x\": self.provenance.document_origin_x,
                    \"document_origin_y\": self.provenance.document_origin_y,
                    \"image_width\": self.width,
                    \"image_height\": self.height,
                    \"bbox\": {
                        \"x\": bbox.x,
                        \"y\": bbox.y,
                        \"width\": bbox.width,
                        \"height\": bbox.height,
                    },
                })),
            });
        }

        Ok(Rect {
            x: left.max(0.0),
            y: top.max(0.0),
            width: bbox.width.min(image_width - left.max(0.0)),
            height: bbox.height.min(image_height - top.max(0.0)),
        })
    }
}

#[derive(Debug)]
pub(super) struct ImageCache {
""",
    "resolved screenshot type",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """        context: ImageContext,
        bytes: &[u8],
        now_ms: u64,
""",
    """        context: ImageContext,
        provenance: BrowserScreenshotProvenance,
        width: u32,
        height: u32,
        bytes: &[u8],
        now_ms: u64,
""",
    "image persist signature",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """        if self.max_count == 0 || self.max_bytes == 0 {
""",
    """        if !provenance.document_origin_x.is_finite()
            || !provenance.document_origin_y.is_finite()
            || width == 0
            || height == 0
        {
            return Err(error(
                \"invalid_screenshot_provenance\",
                \"screenshot coordinate provenance requires a finite origin and positive dimensions\",
                false,
            ));
        }
        if self.max_count == 0 || self.max_bytes == 0 {
""",
    "image provenance validation",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """                path,
                context,
                created_ms: now_ms,
""",
    """                path,
                context,
                provenance,
                width,
                height,
                created_ms: now_ms,
""",
    "image record insert provenance",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """    ) -> Result<PathBuf, ToolError> {
""",
    """    ) -> Result<ResolvedScreenshotImage, ToolError> {
""",
    "image resolve return type",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """        Ok(path)
    }

    fn cleanup_expired""",
    """        Ok(ResolvedScreenshotImage {
            path,
            provenance: record.provenance,
            width: record.width,
            height: record.height,
        })
    }

    fn cleanup_expired""",
    "image resolve result",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """        generation: u64,
        bytes: &[u8],
""",
    """        generation: u64,
        provenance: BrowserScreenshotProvenance,
        width: u32,
        height: u32,
        bytes: &[u8],
""",
    "appcore persist screenshot signature",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """            ImageContext {
                page_id,
                origin,
                generation,
            },
            bytes,
""",
    """            ImageContext {
                page_id,
                origin,
                generation,
            },
            provenance,
            width,
            height,
            bytes,
""",
    "appcore persist screenshot args",
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    """    pub(super) fn resolve_screenshot_image(&mut self, raw: &str) -> Result<PathBuf, ToolError> {
""",
    """    pub(super) fn resolve_screenshot_image(
        &mut self,
        raw: &str,
    ) -> Result<ResolvedScreenshotImage, ToolError> {
""",
    "appcore resolve screenshot return type",
)

# Test helpers and call sites inside image_cache.rs.
p = Path("src-tauri/src/app_core/image_cache.rs")
text = p.read_text()
needle = """    fn cache(ttl_ms: u64, max_count: usize, max_bytes: u64) -> ImageCache {
        ImageCache {
            records: HashMap::new(),
            ttl_ms,
            max_count,
            max_bytes,
        }
    }
"""
replacement = needle + """

    fn provenance() -> BrowserScreenshotProvenance {
        BrowserScreenshotProvenance {
            kind: BrowserScreenshotKind::FullPage,
            document_origin_x: 0.0,
            document_origin_y: 0.0,
        }
    }

    fn persist_test_image(
        cache: &mut ImageCache,
        root: &Path,
        context: ImageContext,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<String, ToolError> {
        cache.persist(root, context, provenance(), 100, 100, bytes, now_ms)
    }
"""
if text.count(needle) != 1:
    raise SystemExit("image test helpers: unexpected cache helper count")
text = text.replace(needle, replacement, 1)
text = text.replace(".persist(dir.path(), expected.clone(), b\"png\", 10)", ".persist(dir.path(), expected.clone(), provenance(), 100, 100, b\"png\", 10)")
text = text.replace(".persist(dir.path(), expected.clone(), b\"old\", 10)", ".persist(dir.path(), expected.clone(), provenance(), 100, 100, b\"old\", 10)")
text = text.replace(".persist(dir.path(), expected.clone(), b\"original\", 30)", ".persist(dir.path(), expected.clone(), provenance(), 100, 100, b\"original\", 30)")
text = text.replace(".persist(dir.path(), expected.clone(), b\"1111\", 1)", ".persist(dir.path(), expected.clone(), provenance(), 100, 100, b\"1111\", 1)")
text = text.replace(".persist(dir.path(), expected.clone(), b\"2222\", 2)", ".persist(dir.path(), expected.clone(), provenance(), 100, 100, b\"2222\", 2)")
text = text.replace("cache.persist(dir.path(), expected, b\"3333\", 3)", "cache.persist(dir.path(), expected, provenance(), 100, 100, b\"3333\", 3)")
text = text.replace(".persist(dir.path(), ctx(\"p1\", \"https://example.com\", 1), b\"new\", 1)", ".persist(\n                dir.path(),\n                ctx(\"p1\", \"https://example.com\", 1),\n                provenance(),\n                100,\n                100,\n                b\"new\",\n                1,\n            )")
text = text.replace(".persist(dir.path(), expected.clone(), b\"private\", 1)", ".persist(dir.path(), expected.clone(), provenance(), 100, 100, b\"private\", 1)")
text = text.replace("let path = cache.resolve(dir.path(), &handle, &expected, 11).unwrap();\n        assert!(!path", "let resolved = cache.resolve(dir.path(), &handle, &expected, 11).unwrap();\n        assert!(!resolved.path")
insert_before = """    #[cfg(unix)]
    #[test]
    fn private_permissions_and_symlink_replacement_are_enforced() {
"""
new_tests = """    #[test]
    fn document_bbox_translation_uses_capture_raster_origin() {
        let document_bbox = Rect {
            x: 50.0,
            y: 1550.0,
            width: 120.0,
            height: 60.0,
        };
        let viewport = ResolvedScreenshotImage {
            path: PathBuf::from(\"unused.png\"),
            provenance: BrowserScreenshotProvenance {
                kind: BrowserScreenshotKind::Viewport,
                document_origin_x: 10.0,
                document_origin_y: 1500.0,
            },
            width: 800,
            height: 600,
        };
        assert_eq!(
            viewport.ocr_bbox_for_document_bbox(&document_bbox).unwrap(),
            Rect {
                x: 40.0,
                y: 50.0,
                width: 120.0,
                height: 60.0,
            }
        );

        let full_page = ResolvedScreenshotImage {
            path: PathBuf::from(\"unused.png\"),
            provenance: provenance(),
            width: 1200,
            height: 3000,
        };
        assert_eq!(
            full_page.ocr_bbox_for_document_bbox(&document_bbox).unwrap(),
            document_bbox
        );

        let clip = ResolvedScreenshotImage {
            path: PathBuf::from(\"unused.png\"),
            provenance: BrowserScreenshotProvenance {
                kind: BrowserScreenshotKind::DocumentClip,
                document_origin_x: 50.0,
                document_origin_y: 1550.0,
            },
            width: 120,
            height: 60,
        };
        assert_eq!(
            clip.ocr_bbox_for_document_bbox(&document_bbox).unwrap(),
            Rect {
                x: 0.0,
                y: 0.0,
                width: 120.0,
                height: 60.0,
            }
        );
    }

    #[test]
    fn document_bbox_translation_rejects_pixels_not_present_in_raster() {
        let viewport = ResolvedScreenshotImage {
            path: PathBuf::from(\"unused.png\"),
            provenance: BrowserScreenshotProvenance {
                kind: BrowserScreenshotKind::Viewport,
                document_origin_x: 0.0,
                document_origin_y: 1000.0,
            },
            width: 800,
            height: 600,
        };
        let error = viewport
            .ocr_bbox_for_document_bbox(&Rect {
                x: 10.0,
                y: 900.0,
                width: 100.0,
                height: 50.0,
            })
            .unwrap_err();
        assert_eq!(error.code, \"ocr_bbox_outside_screenshot\");
    }

""" + insert_before
if text.count(insert_before) != 1:
    raise SystemExit("image bbox tests insertion point not found")
text = text.replace(insert_before, new_tests, 1)
p.write_text(text)

# 4. Pass provenance/dimensions into the private screenshot cache.
replace_once(
    "src-tauri/src/app_core/content_tools.rs",
    """            self.state.page_generation,
            &browser_screenshot.image_bytes,
""",
    """            self.state.page_generation,
            browser_screenshot.provenance,
            browser_screenshot.width,
            browser_screenshot.height,
            &browser_screenshot.image_bytes,
""",
    "content tools persist provenance",
)

# 5. Translate document-space OCR target into cached-raster coordinates.
replace_once(
    "src-tauri/src/app_core/extraction_tools/ocr_tools.rs",
    """        let image_path = match self.resolve_screenshot_image(&image_id) {
            Ok(path) => path,
""",
    """        let image = match self.resolve_screenshot_image(&image_id) {
            Ok(image) => image,
""",
    "ocr resolve image",
)
replace_once(
    "src-tauri/src/app_core/extraction_tools/ocr_tools.rs",
    """        let ocr_result = match self.ocr.run_ocr(&image_path, ocr_bbox.as_ref()) {
""",
    """        let image_ocr_bbox = match ocr_bbox
            .as_ref()
            .map(|bbox| image.ocr_bbox_for_document_bbox(bbox))
            .transpose()
        {
            Ok(bbox) => bbox,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    error,
                    vec![String::from(
                        \"OCR target coordinates were incompatible with the cached screenshot capture area.\",
                    )],
                )
            }
        };

        let ocr_result = match self.ocr.run_ocr(image.path(), image_ocr_bbox.as_ref()) {
""",
    "ocr bbox translation",
)
replace_once(
    "src-tauri/src/app_core/extraction_tools/ocr_tools.rs",
    """                \"OCR was limited to the requested page region using its stored bounding box.\",
""",
    """                \"OCR was limited to the requested page region after translating its document-space bounding box into cached-image coordinates.\",
""",
    "ocr region observation",
)
replace_once(
    "src-tauri/src/app_core/extraction_tools/ocr_tools.rs",
    """                \"OCR was limited to the explicitly requested bounding box within the cached image.\",
""",
    """                \"OCR was limited to the explicitly requested document-space bounding box after translating it into cached-image coordinates.\",
""",
    "ocr bbox observation",
)
