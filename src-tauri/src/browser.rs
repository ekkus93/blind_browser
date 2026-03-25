use std::collections::BTreeMap;
use std::time::Duration;

#[cfg(feature = "browser")]
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, GetNavigationHistoryParams, NavigateToHistoryEntryParams,
    NavigationEntry, ReloadParams, Viewport,
};
#[cfg(feature = "browser")]
use chromiumoxide::types::ClickOptions;
#[cfg(feature = "browser")]
use chromiumoxide::{Browser, BrowserConfig, Page};
#[cfg(feature = "browser")]
use chromiumoxide::page::ScreenshotParams;
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
    #[error("failed to scroll the active page: {0}")]
    Scroll(String),
    #[error("failed to capture the screenshot: {0}")]
    Screenshot(String),
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
                history: after.history,
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

    pub fn focus_element(
        &mut self,
        element: &InteractiveElement,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserFocusState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            let selector = stable_dom_selector(element)?;
            ensure_live_element(&page, element, selector)?;
            let selector_literal =
                serde_json::to_string(selector).map_err(|error| BrowserError::Resolve(error.to_string()))?;

            let focus_result = tauri::async_runtime::block_on(async {
                page.evaluate(format!(
                    r#"(() => {{
                        const selector = {selector_literal};
                        const element = document.querySelector(selector);
                        if (!element) {{
                            return {{ found: false, focused: false }};
                        }}
                        if (typeof element.focus === 'function') {{
                            element.focus();
                        }}
                        return {{
                            found: true,
                            focused: document.activeElement === element
                        }};
                    }})()"#
                ))
                .await
                .map_err(|error| BrowserError::Focus(error.to_string()))?
                .into_value::<LiveFocusResult>()
                .map_err(|error| BrowserError::Focus(error.to_string()))
            })?;

            if !focus_result.found {
                return Err(BrowserError::ElementNotFound {
                    element_id: element.element_id.clone(),
                    locator: selector.to_string(),
                });
            }

            wait_for_page_settle(timeout_ms);
            let after = tauri::async_runtime::block_on(snapshot_page_state(&page))?;

            Ok(BrowserFocusState {
                url: after.url,
                title: after.title,
                focused: focus_result.focused,
                history: after.history,
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = element;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn type_into_element(
        &mut self,
        element: &InteractiveElement,
        text: &str,
        clear_first: bool,
        submit_after: bool,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserTypeState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            let before = tauri::async_runtime::block_on(snapshot_page_state(&page))?;
            let selector = stable_dom_selector(element)?;
            ensure_live_element(&page, element, selector)?;
            let selector_literal =
                serde_json::to_string(selector).map_err(|error| BrowserError::Resolve(error.to_string()))?;
            let text_literal =
                serde_json::to_string(text).map_err(|error| BrowserError::Type(error.to_string()))?;

            let type_result = tauri::async_runtime::block_on(async {
                page.evaluate(format!(
                    r#"(() => {{
                        const selector = {selector_literal};
                        const text = {text_literal};
                        const clearFirst = {clear_first};
                        const submitAfter = {submit_after};
                        const element = document.querySelector(selector);
                        if (!element) {{
                            return {{ found: false, accepted_input: false, value_after: null }};
                        }}

                        const dispatchInputEvents = (target) => {{
                            target.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            target.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }};

                        const setNativeValue = (target, value) => {{
                            const prototype = Object.getPrototypeOf(target);
                            const descriptor = prototype
                                ? Object.getOwnPropertyDescriptor(prototype, 'value')
                                : null;
                            if (descriptor && typeof descriptor.set === 'function') {{
                                descriptor.set.call(target, value);
                            }} else {{
                                target.value = value;
                            }}
                        }};

                        if (typeof element.focus === 'function') {{
                            element.focus();
                        }}

                        if (element instanceof HTMLSelectElement) {{
                            const desired = text.trim().toLowerCase();
                            const matched = Array.from(element.options).find((option) => {{
                                const optionValue = String(option.value ?? '').trim().toLowerCase();
                                const optionLabel = String(option.textContent ?? '').trim().toLowerCase();
                                return optionValue === desired || optionLabel === desired;
                            }});
                            if (!matched) {{
                                return {{
                                    found: true,
                                    accepted_input: false,
                                    value_after: String(element.value ?? '')
                                }};
                            }}
                            element.value = matched.value;
                            dispatchInputEvents(element);
                        }} else if ('value' in element) {{
                            const nextValue = clearFirst
                                ? text
                                : `${{String(element.value ?? '')}}${{text}}`;
                            setNativeValue(element, nextValue);
                            dispatchInputEvents(element);
                        }} else {{
                            return {{ found: true, accepted_input: false, value_after: null }};
                        }}

                        if (submitAfter && element.form) {{
                            if (typeof element.form.requestSubmit === 'function') {{
                                element.form.requestSubmit();
                            }} else {{
                                element.form.submit();
                            }}
                        }}

                        return {{
                            found: true,
                            accepted_input: true,
                            value_after: 'value' in element ? String(element.value ?? '') : null
                        }};
                    }})()"#
                ))
                .await
                .map_err(|error| BrowserError::Type(error.to_string()))?
                .into_value::<LiveTypeResult>()
                .map_err(|error| BrowserError::Type(error.to_string()))
            })?;

            if !type_result.found {
                return Err(BrowserError::ElementNotFound {
                    element_id: element.element_id.clone(),
                    locator: selector.to_string(),
                });
            }

            wait_for_page_settle(timeout_ms);
            let after = tauri::async_runtime::block_on(snapshot_page_state(&page))?;
            let value_after = type_result
                .value_after
                .as_deref()
                .and_then(normalize_optional_text);

            Ok(BrowserTypeState {
                page_changed: after.url != before.url,
                url: after.url,
                title: after.title,
                accepted_input: type_result.accepted_input,
                value_after,
                history: after.history,
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = element;
            let _ = text;
            let _ = clear_first;
            let _ = submit_after;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn submit_active_form(
        &mut self,
        form: Option<&InteractiveElement>,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserSubmitState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            let before = tauri::async_runtime::block_on(snapshot_page_state(&page))?;

            let submit_result = if let Some(form) = form {
                let selector = stable_dom_selector(form)?;
                ensure_live_element(&page, form, selector)?;
                let selector_literal = serde_json::to_string(selector)
                    .map_err(|error| BrowserError::Resolve(error.to_string()))?;

                tauri::async_runtime::block_on(async {
                    page.evaluate(format!(
                        r#"(() => {{
                            const selector = {selector_literal};
                            const form = document.querySelector(selector);
                            if (!(form instanceof HTMLFormElement)) {{
                                return {{ found: false, ambiguous: false, submitted: false }};
                            }}
                            if (typeof form.requestSubmit === 'function') {{
                                form.requestSubmit();
                            }} else {{
                                form.submit();
                            }}
                            return {{ found: true, ambiguous: false, submitted: true }};
                        }})()"#
                    ))
                    .await
                    .map_err(|error| BrowserError::Submit(error.to_string()))?
                    .into_value::<LiveSubmitResult>()
                    .map_err(|error| BrowserError::Submit(error.to_string()))
                })?
            } else {
                tauri::async_runtime::block_on(async {
                    page.evaluate(
                        r#"(() => {
                            const isVisible = (element) => {
                                if (!(element instanceof Element)) {
                                    return false;
                                }
                                const style = window.getComputedStyle(element);
                                if (style.display === 'none' || style.visibility === 'hidden') {
                                    return false;
                                }
                                const rect = element.getBoundingClientRect();
                                return rect.width > 0 && rect.height > 0;
                            };

                            const activeElement = document.activeElement;
                            const activeForm = activeElement instanceof HTMLElement ? activeElement.form : null;
                            if (activeForm instanceof HTMLFormElement) {
                                if (typeof activeForm.requestSubmit === 'function') {
                                    activeForm.requestSubmit();
                                } else {
                                    activeForm.submit();
                                }
                                return { found: true, ambiguous: false, submitted: true };
                            }

                            const visibleForms = Array.from(document.forms).filter((form) => isVisible(form));
                            if (visibleForms.length !== 1) {
                                return {
                                    found: false,
                                    ambiguous: visibleForms.length > 1,
                                    submitted: false
                                };
                            }

                            const resolvedForm = visibleForms[0];
                            if (typeof resolvedForm.requestSubmit === 'function') {
                                resolvedForm.requestSubmit();
                            } else {
                                resolvedForm.submit();
                            }
                            return { found: true, ambiguous: false, submitted: true };
                        })()"#,
                    )
                    .await
                    .map_err(|error| BrowserError::Submit(error.to_string()))?
                    .into_value::<LiveSubmitResult>()
                    .map_err(|error| BrowserError::Submit(error.to_string()))
                })?
            };

            if !submit_result.found {
                if submit_result.ambiguous {
                    return Err(BrowserError::Resolve(String::from(
                        "multiple visible forms are present and no active form could be determined",
                    )));
                }
                return Err(BrowserError::Resolve(String::from(
                    "no active or uniquely visible form could be determined",
                )));
            }

            wait_for_page_settle(timeout_ms);
            let after = tauri::async_runtime::block_on(snapshot_page_state(&page))?;

            Ok(BrowserSubmitState {
                page_changed: after.url != before.url,
                url: after.url,
                title: after.title,
                submitted: submit_result.submitted,
                history: after.history,
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = form;
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

    pub fn capture_screenshot(
        &mut self,
        full_page: bool,
        bbox: Option<Rect>,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserScreenshotState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            let screenshot_bytes = tauri::async_runtime::block_on(async {
                let mut builder =
                    ScreenshotParams::builder().format(CaptureScreenshotFormat::Png);
                if full_page {
                    builder = builder.full_page(true);
                }
                if let Some(bbox) = bbox.as_ref() {
                    if bbox.width <= 0.0 || bbox.height <= 0.0 {
                        return Err(BrowserError::Screenshot(String::from(
                            "bbox screenshots require positive width and height",
                        )));
                    }
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

            wait_for_page_settle(timeout_ms);
            let after = tauri::async_runtime::block_on(snapshot_page_state(&page))?;
            let (width, height) = png_dimensions(&screenshot_bytes)?;

            Ok(BrowserScreenshotState {
                url: after.url,
                title: after.title,
                history: after.history,
                image_bytes: screenshot_bytes,
                bbox,
                width,
                height,
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = full_page;
            let _ = bbox;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn go_back(
        &mut self,
        steps: u8,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserNavigationState, BrowserError> {
        self.navigate_history(-(steps as isize), load_state, timeout_ms)
    }

    pub fn go_forward(
        &mut self,
        steps: u8,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserNavigationState, BrowserError> {
        self.navigate_history(steps as isize, load_state, timeout_ms)
    }

    pub fn reload_page(
        &mut self,
        hard_reload: bool,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserPageState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            tauri::async_runtime::block_on(async {
                if hard_reload {
                    page.execute(ReloadParams::builder().ignore_cache(true).build())
                        .await
                        .map_err(|error| BrowserError::Reload(error.to_string()))?;
                    page.wait_for_navigation()
                        .await
                        .map_err(|error| BrowserError::Reload(error.to_string()))?;
                } else {
                    page.reload()
                        .await
                        .map_err(|error| BrowserError::Reload(error.to_string()))?;
                }

                let _ = load_state;
                let _ = timeout_ms;
                snapshot_page_state(&page).await
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = hard_reload;
            let _ = load_state;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn scroll_page(
        &mut self,
        direction: ScrollDirection,
        amount_px: Option<f32>,
        target: Option<ScrollTarget>,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserScrollState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            tauri::async_runtime::block_on(async {
                let previous_scroll_y = current_scroll_y(&page).await?;
                let scroll_instruction =
                    build_scroll_instruction(direction, amount_px, target, &page).await?;
                let result = page
                    .evaluate_expression(scroll_instruction)
                    .await
                    .map_err(|error| BrowserError::Scroll(error.to_string()))?
                    .into_value::<LiveScrollResult>()
                    .map_err(|error| BrowserError::Scroll(error.to_string()))?;
                wait_for_page_settle(timeout_ms);

                Ok(BrowserScrollState {
                    previous_scroll_y,
                    current_scroll_y: result.current_scroll_y,
                    reached_boundary: result.reached_boundary,
                })
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = direction;
            let _ = amount_px;
            let _ = target;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    #[cfg(feature = "browser")]
    fn ensure_session(&mut self) -> Result<&mut LiveBrowserSession, BrowserError> {
        if self.session.is_none() {
            self.session = Some(LiveBrowserSession::launch(&self.config)?);
        }

        self.session.as_mut().ok_or_else(|| {
            BrowserError::Launch(String::from("chromium session was not initialized"))
        })
    }

    fn navigate_history(
        &mut self,
        delta: isize,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserNavigationState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            tauri::async_runtime::block_on(async {
                let history_snapshot = read_navigation_history_snapshot(&page).await?;
                let Some(current_index) = history_snapshot.current_index else {
                    return Ok(BrowserNavigationState {
                        navigated: false,
                        url: None,
                        title: None,
                        history: history_snapshot.history,
                    });
                };

                let target_index = current_index + delta;
                if target_index < 0 || target_index >= history_snapshot.entries.len() as isize {
                    let current_entry = history_snapshot.entries.get(current_index as usize);
                    return Ok(BrowserNavigationState {
                        navigated: false,
                        url: current_entry.map(|entry| entry.url.clone()),
                        title: current_entry
                            .and_then(|entry| normalize_optional_text(&entry.title)),
                        history: history_snapshot.history,
                    });
                }

                let target_entry = &history_snapshot.entries[target_index as usize];
                page.execute(NavigateToHistoryEntryParams::new(target_entry.id))
                    .await
                    .map_err(|error| BrowserError::Navigate(error.to_string()))?;
                page.wait_for_navigation()
                    .await
                    .map_err(|error| BrowserError::Navigate(error.to_string()))?;
                let _ = load_state;
                let _ = timeout_ms;
                let current_page = snapshot_page_state(&page).await?;

                Ok(BrowserNavigationState {
                    navigated: true,
                    url: Some(current_page.url),
                    title: current_page.title,
                    history: current_page.history,
                })
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = delta;
            let _ = load_state;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
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

    builder.build().map_err(BrowserError::Launch)
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
        .and_then(|value| normalize_optional_text(&value));

    Ok(BrowserPageState {
        url,
        title,
        history: read_browser_history(page).await?,
    })
}

#[cfg(feature = "browser")]
fn stable_dom_selector(element: &InteractiveElement) -> Result<&str, BrowserError> {
    element
        .dom_locator
        .as_deref()
        .map(str::trim)
        .filter(|locator| !locator.is_empty())
        .ok_or_else(|| BrowserError::MissingDomLocator {
            element_id: element.element_id.clone(),
        })
}

#[cfg(feature = "browser")]
fn ensure_live_element(page: &Page, element: &InteractiveElement, selector: &str) -> Result<(), BrowserError> {
    tauri::async_runtime::block_on(async {
        page.find_element(selector)
            .await
            .map(|_| ())
            .map_err(|_| BrowserError::ElementNotFound {
                element_id: element.element_id.clone(),
                locator: selector.to_string(),
            })
    })
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveFocusResult {
    found: bool,
    focused: bool,
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveTypeResult {
    found: bool,
    accepted_input: bool,
    value_after: Option<String>,
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveSubmitResult {
    found: bool,
    ambiguous: bool,
    submitted: bool,
}

fn png_dimensions(image_bytes: &[u8]) -> Result<(u32, u32), BrowserError> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if image_bytes.len() < 24 || &image_bytes[..8] != PNG_SIGNATURE {
        return Err(BrowserError::Screenshot(String::from(
            "captured screenshot was not a valid PNG image",
        )));
    }

    let width = u32::from_be_bytes(
        image_bytes[16..20]
            .try_into()
            .map_err(|_| BrowserError::Screenshot(String::from("failed to read PNG width")))?,
    );
    let height = u32::from_be_bytes(
        image_bytes[20..24]
            .try_into()
            .map_err(|_| BrowserError::Screenshot(String::from("failed to read PNG height")))?,
    );
    Ok((width, height))
}

#[cfg(feature = "browser")]
async fn read_browser_history(
    page: &Page,
) -> Result<crate::state::BrowserHistoryState, BrowserError> {
    Ok(read_navigation_history_snapshot(page).await?.history)
}

#[cfg(feature = "browser")]
async fn read_navigation_history_snapshot(
    page: &Page,
) -> Result<LiveNavigationHistorySnapshot, BrowserError> {
    let history = page
        .execute(GetNavigationHistoryParams::default())
        .await
        .map_err(|error| BrowserError::History(error.to_string()))?
        .result;
    let current_index = isize::try_from(history.current_index).ok();
    let entry_count = history.entries.len();

    Ok(LiveNavigationHistorySnapshot {
        current_index,
        history: crate::state::BrowserHistoryState {
            can_go_back: current_index.is_some_and(|index| index > 0),
            can_go_forward: current_index.is_some_and(|index| (index as usize) + 1 < entry_count),
            current_entry_index: current_index.and_then(|index| usize::try_from(index).ok()),
            entry_count,
        },
        entries: history.entries,
    })
}

#[cfg(feature = "browser")]
async fn current_scroll_y(page: &Page) -> Result<f32, BrowserError> {
    let scroll_y = page
        .evaluate_expression("window.scrollY")
        .await
        .map_err(|error| BrowserError::Scroll(error.to_string()))?
        .into_value::<f64>()
        .map_err(|error| BrowserError::Scroll(error.to_string()))?;
    Ok(scroll_y as f32)
}

#[cfg(feature = "browser")]
async fn build_scroll_instruction(
    direction: ScrollDirection,
    amount_px: Option<f32>,
    target: Option<ScrollTarget>,
    page: &Page,
) -> Result<String, BrowserError> {
    let metrics = page
        .layout_metrics()
        .await
        .map_err(|error| BrowserError::Scroll(error.to_string()))?;
    let viewport_height = metrics.css_layout_viewport.client_height as f32;
    let default_amount = (viewport_height * 0.85).max(200.0);
    let requested_amount = amount_px.unwrap_or(default_amount).max(0.0);
    let axis = match direction {
        ScrollDirection::Up | ScrollDirection::Down => "y",
        ScrollDirection::Left | ScrollDirection::Right => "x",
    };
    let signed_amount = match direction {
        ScrollDirection::Up | ScrollDirection::Left => -requested_amount,
        ScrollDirection::Down | ScrollDirection::Right => requested_amount,
    };
    let target_expression = match target {
        Some(ScrollTarget::Top) => "window.scrollTo({ top: 0, behavior: 'auto' });".to_string(),
        Some(ScrollTarget::Bottom) => {
            "window.scrollTo({ top: document.documentElement.scrollHeight, behavior: 'auto' });"
                .to_string()
        }
        Some(ScrollTarget::NextSection) => {
            "window.scrollBy({ top: window.innerHeight * 0.9, behavior: 'auto' });".to_string()
        }
        Some(ScrollTarget::PreviousSection) => {
            "window.scrollBy({ top: -window.innerHeight * 0.9, behavior: 'auto' });".to_string()
        }
        None => format!(
            "window.scrollBy({{ {}: {}, behavior: 'auto' }});",
            if axis == "y" { "top" } else { "left" },
            signed_amount
        ),
    };

    Ok(format!(
        "(() => {{\
            const previousY = window.scrollY;\
            {target_expression}\
            const maxY = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);\
            return {{\
                current_scroll_y: window.scrollY,\
                reached_boundary: window.scrollY <= 0 || window.scrollY >= maxY || window.scrollY === previousY\
            }};\
        }})()"
    ))
}

fn normalize_optional_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
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
struct LiveScrollResult {
    current_scroll_y: f32,
    reached_boundary: bool,
}

#[cfg(feature = "browser")]
struct LiveNavigationHistorySnapshot {
    current_index: Option<isize>,
    history: crate::state::BrowserHistoryState,
    entries: Vec<NavigationEntry>,
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
