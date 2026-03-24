use std::collections::BTreeMap;
use std::time::Duration;

#[cfg(feature = "browser")]
use chromiumoxide::{Browser, BrowserConfig, Page};
#[cfg(feature = "browser")]
use chromiumoxide::types::ClickOptions;
#[cfg(feature = "browser")]
use futures::StreamExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::page_model::{
    ElementRole, InteractiveElement, PageModel, PageRegion, Rect, RegionSource,
};


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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserClickState {
    pub url: String,
    pub title: Option<String>,
    pub page_changed: bool,
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
}

pub struct BrowserController {
    config: BrowserSessionConfig,
    #[cfg(feature = "browser")]
    session: Option<LiveBrowserSession>,
}

impl BrowserController {
    pub fn new(config: BrowserSessionConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "browser")]
            session: None,
        }
    }

    pub fn open_url(
        &mut self,
        url: &str,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserPageState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let _ = load_state;
            let _ = timeout_ms;
            let user_agent = self.config.user_agent.clone();
            let session = self.ensure_session()?;
            let page = session.ensure_page(user_agent.as_deref())?;

            tauri::async_runtime::block_on(async {
                page.goto(url)
                    .await
                    .map_err(|error| BrowserError::Navigate(error.to_string()))?;

                snapshot_page_state(&page).await
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = url;
            let _ = load_state;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn click_element(
        &mut self,
        element: &InteractiveElement,
        double_click: bool,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserClickState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            let before = tauri::async_runtime::block_on(snapshot_page_state(&page))?;
            let selector = element
                .dom_locator
                .as_deref()
                .map(str::trim)
                .filter(|locator| !locator.is_empty())
                .ok_or_else(|| BrowserError::MissingDomLocator {
                    element_id: element.element_id.clone(),
                })?;
            let live_element = tauri::async_runtime::block_on(async {
                page.find_element(selector)
                    .await
                    .map_err(|_| BrowserError::ElementNotFound {
                        element_id: element.element_id.clone(),
                        locator: selector.to_string(),
                    })
            })?;

            tauri::async_runtime::block_on(async {
                if double_click {
                    let options = ClickOptions::builder().click_count(2).build();
                    live_element
                        .click_with(options)
                        .await
                        .map_err(|error| BrowserError::Click(error.to_string()))?;
                } else {
                    live_element
                        .click()
                        .await
                        .map_err(|error| BrowserError::Click(error.to_string()))?;
                }

                Ok::<(), BrowserError>(())
            })?;

            wait_for_page_settle(timeout_ms);
            let after = tauri::async_runtime::block_on(snapshot_page_state(&page))?;

            Ok(BrowserClickState {
                page_changed: after.url != before.url,
                url: after.url,
                title: after.title,
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = element;
            let _ = double_click;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn extract_page_model(&mut self) -> Result<PageModel, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            tauri::async_runtime::block_on(extract_live_page_model(&page))
        }

        #[cfg(not(feature = "browser"))]
        {
            Err(BrowserError::FeatureDisabled)
        }
    }

    #[cfg(feature = "browser")]
    fn ensure_session(&mut self) -> Result<&mut LiveBrowserSession, BrowserError> {
        if self.session.is_none() {
            self.session = Some(LiveBrowserSession::launch(&self.config)?);
        }

        self.session
            .as_mut()
            .ok_or_else(|| BrowserError::Launch(String::from("chromium session was not initialized")))
    }
}

#[cfg(feature = "browser")]
struct LiveBrowserSession {
    browser: Browser,
    page: Option<Page>,
}

#[cfg(feature = "browser")]
impl LiveBrowserSession {
    fn launch(config: &BrowserSessionConfig) -> Result<Self, BrowserError> {
        let browser_config = build_browser_config(config)?;
        let (browser, mut handler) = tauri::async_runtime::block_on(async {
            Browser::launch(browser_config)
                .await
                .map_err(|error| BrowserError::Launch(error.to_string()))
        })?;

        tauri::async_runtime::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(error) = event {
                    tracing::error!(error = %error, "chromium handler stopped");
                    break;
                }
            }
        });

        Ok(Self {
            browser,
            page: None,
        })
    }

    fn ensure_page(&mut self, user_agent: Option<&str>) -> Result<Page, BrowserError> {
        if let Some(page) = self.page.clone() {
            return Ok(page);
        }

        let page = tauri::async_runtime::block_on(async {
            let page = self
                .browser
                .new_page("about:blank")
                .await
                .map_err(|error| BrowserError::CreatePage(error.to_string()))?;

            if let Some(user_agent) = user_agent.filter(|value| !value.trim().is_empty()) {
                page.set_user_agent(user_agent)
                    .await
                    .map_err(|error| BrowserError::CreatePage(error.to_string()))?;
            }

            Ok::<Page, BrowserError>(page)
        })?;

        self.page = Some(page.clone());
        Ok(page)
    }
}

#[cfg(feature = "browser")]
fn build_browser_config(config: &BrowserSessionConfig) -> Result<BrowserConfig, BrowserError> {
    let mut builder = BrowserConfig::builder();
    if matches!(config.visibility, BrowserVisibilityMode::Visible) {
        builder = builder.with_head();
    }

    builder
        .build()
        .map_err(BrowserError::Launch)
}

#[cfg(feature = "browser")]
async fn snapshot_page_state(page: &Page) -> Result<BrowserPageState, BrowserError> {
    let url = page
        .url()
        .await
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .unwrap_or_else(|| String::from("about:blank"));
    let title = page
        .evaluate("document.title || null")
        .await
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .into_value::<Option<String>>()
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

    Ok(BrowserPageState { url, title })
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveExtractedPage {
    title: Option<String>,
    url: Option<String>,
    regions: Vec<LiveExtractedRegion>,
    interactive_elements: Vec<LiveExtractedInteractiveElement>,
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveExtractedRegion {
    region_id: String,
    label: Option<String>,
    text: String,
    source: String,
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveExtractedInteractiveElement {
    element_id: String,
    dom_locator: Option<String>,
    role: String,
    tag_name: String,
    text: Option<String>,
    accessible_name: Option<String>,
    placeholder: Option<String>,
    href: Option<String>,
    value: Option<String>,
    bbox: Option<Rect>,
    visible: bool,
    enabled: bool,
    attributes: BTreeMap<String, String>,
}

#[cfg(feature = "browser")]
async fn extract_live_page_model(page: &Page) -> Result<PageModel, BrowserError> {
    let evaluation = r#"(() => {
        const normalizeText = (value) => {
            const text = String(value ?? '').replace(/\s+/g, ' ').trim();
            return text.length > 0 ? text : null;
        };

        const isVisible = (node) => {
            if (!(node instanceof Element)) {
                return false;
            }
            const style = window.getComputedStyle(node);
            if (style.display === 'none' || style.visibility === 'hidden' || style.visibility === 'collapse') {
                return false;
            }
            const rect = node.getBoundingClientRect();
            return rect.width > 0 && rect.height > 0;
        };

        const isEnabled = (node) => {
            if (!(node instanceof Element)) {
                return false;
            }
            return !node.hasAttribute('disabled') && node.getAttribute('aria-disabled') !== 'true';
        };

        const collectLabelText = (node) => {
            if (!(node instanceof Element) || !('labels' in node) || !node.labels) {
                return null;
            }
            return normalizeText(Array.from(node.labels).map((label) => label.textContent ?? '').join(' '));
        };

        const accessibleNameFor = (node) => {
            if (!(node instanceof Element)) {
                return null;
            }
            const ariaLabel = normalizeText(node.getAttribute('aria-label'));
            if (ariaLabel) {
                return ariaLabel;
            }
            const labelledBy = normalizeText(node.getAttribute('aria-labelledby'));
            if (labelledBy) {
                const referenced = labelledBy
                    .split(' ')
                    .map((id) => document.getElementById(id))
                    .filter(Boolean)
                    .map((element) => element.textContent ?? '')
                    .join(' ');
                const normalized = normalizeText(referenced);
                if (normalized) {
                    return normalized;
                }
            }
            const labelText = collectLabelText(node);
            if (labelText) {
                return labelText;
            }
            return (
                normalizeText(node.getAttribute('title')) ||
                normalizeText(node.getAttribute('alt')) ||
                normalizeText(node.innerText) ||
                normalizeText(node.textContent)
            );
        };

        const uniqueSelector = (node) => {
            if (!(node instanceof Element)) {
                return null;
            }
            if (node.id) {
                const idSelector = `#${CSS.escape(node.id)}`;
                if (document.querySelectorAll(idSelector).length === 1) {
                    return idSelector;
                }
            }

            const parts = [];
            let current = node;
            while (current && current.nodeType === Node.ELEMENT_NODE) {
                const tagName = current.tagName.toLowerCase();
                let part = tagName;
                if (current.parentElement) {
                    const sameTagSiblings = Array.from(current.parentElement.children)
                        .filter((sibling) => sibling.tagName === current.tagName);
                    if (sameTagSiblings.length > 1) {
                        part += `:nth-of-type(${sameTagSiblings.indexOf(current) + 1})`;
                    }
                }
                parts.unshift(part);
                const selector = parts.join(' > ');
                if (document.querySelectorAll(selector).length === 1) {
                    return selector;
                }
                current = current.parentElement;
            }

            return parts.length > 0 ? parts.join(' > ') : null;
        };

        const roleFor = (node) => {
            if (!(node instanceof Element)) {
                return 'Other';
            }
            const explicitRole = normalizeText(node.getAttribute('role'));
            switch (explicitRole) {
                case 'link': return "Link";
                case 'button': return "Button";
                case 'textbox': return "Input";
                case 'combobox': return "Select";
                case 'checkbox': return "Checkbox";
                case 'radio': return "Radio";
                case 'form': return "Form";
                case 'navigation':
                case 'main':
                case 'banner':
                case 'contentinfo': return "Landmark";
                default: break;
            }

            const tagName = node.tagName.toLowerCase();
            if (tagName === 'a' && node.hasAttribute('href')) return "Link";
            if (tagName === 'button') return "Button";
            if (tagName === 'textarea') return "TextArea";
            if (tagName === 'select') return "Select";
            if (tagName === 'form') return "Form";
            if (tagName === 'input') {
                const type = normalizeText(node.getAttribute('type'));
                if (type === 'checkbox') return "Checkbox";
                if (type === 'radio') return "Radio";
                return "Input";
            }
            return 'Other';
        };

        const interactiveSelector = [
            'a[href]',
            'button',
            'input',
            'textarea',
            'select',
            'form',
            '[role="button"]',
            '[role="link"]',
            '[role="textbox"]',
            '[role="combobox"]',
            '[role="checkbox"]',
            '[role="radio"]',
            '[role="form"]',
            '[role="navigation"]',
            '[role="main"]',
            '[role="banner"]',
            '[role="contentinfo"]'
        ].join(',');

        const interactive_elements = Array.from(document.querySelectorAll(interactiveSelector)).map((node, index) => {
            const rect = node.getBoundingClientRect();
            const text = normalizeText(node.innerText || node.textContent);
            const placeholder = normalizeText(node.getAttribute('placeholder'));
            const href = 'href' in node ? normalizeText(node.href) : normalizeText(node.getAttribute('href'));
            const value = 'value' in node ? normalizeText(node.value) : null;
            return {
                element_id: `element-${index + 1}`,
                dom_locator: uniqueSelector(node),
                role: roleFor(node),
                tag_name: node.tagName.toLowerCase(),
                text,
                accessible_name: accessibleNameFor(node),
                placeholder,
                href,
                value,
                bbox: isVisible(node) ? {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                } : null,
                visible: isVisible(node),
                enabled: isEnabled(node),
                attributes: Object.fromEntries(Array.from(node.attributes).map((attribute) => [attribute.name, attribute.value]))
            };
        });

        const regionCandidates = Array.from(document.querySelectorAll('main, article, section, nav, aside, p, li, blockquote, pre, h1, h2, h3, h4, h5, h6'));
        const seenTexts = new Set();
        const regions = [];
        for (const node of regionCandidates) {
            if (!isVisible(node)) {
                continue;
            }
            const text = normalizeText(node.innerText || node.textContent);
            if (!text || seenTexts.has(text)) {
                continue;
            }
            seenTexts.add(text);
            regions.push({
                region_id: `dom-region-${regions.length + 1}`,
                label: normalizeText(node.getAttribute('aria-label')),
                text,
                source: 'Dom'
            });
        }

        return {
            title: normalizeText(document.title),
            url: normalizeText(window.location.href),
            regions,
            interactive_elements,
        };
    })()"#;

    let extracted = page
        .evaluate(evaluation)
        .await
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .into_value::<LiveExtractedPage>()
        .map_err(|error| BrowserError::Inspect(error.to_string()))?;

    Ok(PageModel {
        title: extracted.title,
        url: extracted.url,
        regions: extracted
            .regions
            .into_iter()
            .map(|region| PageRegion {
                region_id: region.region_id,
                label: region.label,
                text: region.text,
                source: match region.source.as_str() {
                    "Mixed" => RegionSource::Mixed,
                    "Ocr" => RegionSource::Ocr,
                    _ => RegionSource::Dom,
                },
            })
            .collect(),
        interactive_elements: extracted
            .interactive_elements
            .into_iter()
            .map(|element| InteractiveElement {
                element_id: element.element_id,
                dom_locator: element.dom_locator.and_then(|locator| {
                    let trimmed = locator.trim();
                    (!trimmed.is_empty()).then(|| trimmed.to_string())
                }),
                role: match element.role.as_str() {
                    "Link" => ElementRole::Link,
                    "Button" => ElementRole::Button,
                    "Input" => ElementRole::Input,
                    "TextArea" => ElementRole::TextArea,
                    "Select" => ElementRole::Select,
                    "Checkbox" => ElementRole::Checkbox,
                    "Radio" => ElementRole::Radio,
                    "Form" => ElementRole::Form,
                    "Landmark" => ElementRole::Landmark,
                    _ => ElementRole::Other,
                },
                tag_name: element.tag_name,
                text: element.text,
                accessible_name: element.accessible_name,
                placeholder: element.placeholder,
                href: element.href,
                value: element.value,
                bbox: element.bbox,
                visible: element.visible,
                enabled: element.enabled,
                attributes: element.attributes,
            })
            .collect(),
    })
}

#[cfg(feature = "browser")]
fn wait_for_page_settle(timeout_ms: Option<u64>) {
    let wait_ms = timeout_ms.unwrap_or(400).clamp(150, 2_000);
    std::thread::sleep(Duration::from_millis(wait_ms));
}
