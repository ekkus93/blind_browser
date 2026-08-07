use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A bounding box in **document/page-absolute** CSS pixels — i.e. relative
/// to the top-left of the full document, not the current viewport/scroll
/// position. This is the coordinate space CDP's `Page.captureScreenshot`
/// `clip` parameter expects (confirmed empirically: `clip`'s x/y are always
/// document-absolute, independent of `captureBeyondViewport`) and the space
/// a full-page screenshot raster's pixel origin corresponds to.
///
/// Live DOM extraction (`browser::dom_extraction`) computes this via
/// `getBoundingClientRect()` (viewport-relative) plus `window.scrollX`/
/// `window.scrollY`, once, at the source — every consumer (region/element
/// screenshot capture, OCR region cropping) can therefore treat a `Rect` as
/// already scroll-corrected and must not add or assume any further scroll
/// offset itself. `dom_smoothie`-derived regions (`extractor.rs`, no live
/// DOM to measure) set `bbox: None` rather than fabricate one.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ElementRole {
    Link,
    Button,
    Input,
    TextArea,
    Select,
    Checkbox,
    Radio,
    Form,
    Landmark,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RegionSource {
    Dom,
    Ocr,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub enum RegionRole {
    Title,
    Heading,
    Paragraph,
    Section,
    #[default]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ExtractionSource {
    DomSmoothie,
    DomFallback,
    Ocr,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct InteractiveElement {
    pub element_id: String,
    pub dom_locator: Option<String>,
    pub role: ElementRole,
    pub tag_name: String,
    pub text: Option<String>,
    pub accessible_name: Option<String>,
    pub placeholder: Option<String>,
    pub href: Option<String>,
    pub value: Option<String>,
    pub bbox: Option<Rect>,
    pub visible: bool,
    pub enabled: bool,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PageRegion {
    pub region_id: String,
    #[serde(default)]
    pub role: RegionRole,
    pub label: Option<String>,
    pub text: String,
    #[serde(default)]
    pub bbox: Option<Rect>,
    pub source: RegionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub struct PageModel {
    pub title: Option<String>,
    pub url: Option<String>,
    pub regions: Vec<PageRegion>,
    pub interactive_elements: Vec<InteractiveElement>,
}
