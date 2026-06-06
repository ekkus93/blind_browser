use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::page_model::Rect;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum LoadState {
    DomContentLoaded,
    Load,
    NetworkIdle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum BrowserVisibilityMode {
    Visible,
    Headless,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ScrollTarget {
    Top,
    Bottom,
    NextSection,
    PreviousSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BrowserSessionConfig {
    pub visibility: BrowserVisibilityMode,
    pub user_agent: Option<String>,
}

impl Default for BrowserSessionConfig {
    fn default() -> Self {
        Self {
            visibility: BrowserVisibilityMode::Visible,
            user_agent: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserPageState {
    pub url: String,
    pub title: Option<String>,
    pub history: crate::state::BrowserHistoryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserClickState {
    pub url: String,
    pub title: Option<String>,
    pub page_changed: bool,
    pub history: crate::state::BrowserHistoryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserFocusState {
    pub url: String,
    pub title: Option<String>,
    pub focused: bool,
    pub history: crate::state::BrowserHistoryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTypeState {
    pub url: String,
    pub title: Option<String>,
    pub page_changed: bool,
    pub accepted_input: bool,
    pub value_after: Option<String>,
    pub history: crate::state::BrowserHistoryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserSubmitState {
    pub url: String,
    pub title: Option<String>,
    pub page_changed: bool,
    pub submitted: bool,
    pub history: crate::state::BrowserHistoryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserNavigationState {
    pub navigated: bool,
    pub url: Option<String>,
    pub title: Option<String>,
    pub history: crate::state::BrowserHistoryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserHtmlState {
    pub url: String,
    pub title: Option<String>,
    pub history: crate::state::BrowserHistoryState,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowserEvalState {
    pub url: String,
    pub title: Option<String>,
    pub history: crate::state::BrowserHistoryState,
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowserScrollState {
    pub previous_scroll_y: f32,
    pub current_scroll_y: f32,
    pub reached_boundary: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowserScreenshotState {
    pub url: String,
    pub title: Option<String>,
    pub history: crate::state::BrowserHistoryState,
    pub image_bytes: Vec<u8>,
    pub bbox: Option<Rect>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowserPageMetrics {
    pub scroll_y: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub document_height: f32,
}

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("browser support is not enabled in this build")]
    FeatureDisabled,
    #[error("failed to launch chromium backend: {0}")]
    Launch(String),
    #[error("failed to create browser page: {0}")]
    CreatePage(String),
    #[error("failed to navigate browser page: {0}")]
    Navigate(String),
    #[error("failed to inspect browser page state: {0}")]
    Inspect(String),
    #[error("click_element requires an active browser page")]
    NoActivePage,
    #[error("click_element requires a stable dom_locator for element_id={element_id}")]
    MissingDomLocator { element_id: String },
    #[error("failed to resolve the requested DOM element: {0}")]
    Resolve(String),
    #[error("stored dom_locator did not match a live DOM element for element_id={element_id}: {locator}")]
    ElementNotFound { element_id: String, locator: String },
    #[error("failed to click the resolved DOM element: {0}")]
    Click(String),
    #[error("failed to focus the resolved DOM element: {0}")]
    Focus(String),
    #[error("failed to type into the resolved DOM element: {0}")]
    Type(String),
    #[error("failed to submit the resolved DOM form: {0}")]
    Submit(String),
    #[error("failed to read browser navigation history: {0}")]
    History(String),
    #[error("failed to reload the active page: {0}")]
    Reload(String),
    #[error("failed to evaluate the requested JavaScript expression: {0}")]
    Eval(String),
    #[error("failed to scroll the active page: {0}")]
    Scroll(String),
    #[error("failed to capture the screenshot: {0}")]
    Screenshot(String),
}
