use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
