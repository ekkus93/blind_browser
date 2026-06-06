pub mod config;
pub use config::*;

mod session;
mod navigation;
mod page_metrics;
mod dom_extraction;

#[cfg(feature = "browser")]
use session::{
    ensure_live_element, snapshot_page_state, stable_dom_selector, LiveBrowserSession,
    LiveFocusResult, LiveSubmitResult, LiveTypeResult,
};
#[cfg(any(feature = "browser", test))]
use session::normalize_optional_text;

#[cfg(any(feature = "browser", test))]
use std::time::Duration;

#[cfg(feature = "browser")]
use chromiumoxide::cdp::browser_protocol::page::{CaptureScreenshotFormat, Viewport};
#[cfg(feature = "browser")]
use chromiumoxide::page::ScreenshotParams;
#[cfg(feature = "browser")]
use chromiumoxide::types::ClickOptions;
use crate::page_model::{InteractiveElement, Rect};


pub struct BrowserController {
    #[cfg(feature = "browser")]
    config: BrowserSessionConfig,
    #[cfg(feature = "browser")]
    session: Option<LiveBrowserSession>,
}

impl BrowserController {
    pub fn new(config: BrowserSessionConfig) -> Self {
        #[cfg(not(feature = "browser"))]
        let _ = config;

        Self {
            #[cfg(feature = "browser")]
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
            let selector_literal = serde_json::to_string(selector)
                .map_err(|error| BrowserError::Resolve(error.to_string()))?;

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
            let selector_literal = serde_json::to_string(selector)
                .map_err(|error| BrowserError::Resolve(error.to_string()))?;
            let text_literal = serde_json::to_string(text)
                .map_err(|error| BrowserError::Type(error.to_string()))?;

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
                let mut builder = ScreenshotParams::builder().format(CaptureScreenshotFormat::Png);
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

    pub fn get_html(&mut self, timeout_ms: Option<u64>) -> Result<BrowserHtmlState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            wait_for_page_settle(timeout_ms);
            tauri::async_runtime::block_on(async {
                let html = page
                    .evaluate(
                        "document.documentElement ? document.documentElement.outerHTML : null",
                    )
                    .await
                    .map_err(|error| BrowserError::Inspect(error.to_string()))?
                    .into_value::<Option<String>>()
                    .map_err(|error| BrowserError::Inspect(error.to_string()))?
                    .ok_or_else(|| {
                        BrowserError::Inspect(String::from(
                            "document has no root html element to serialize",
                        ))
                    })?;
                let page_state = snapshot_page_state(&page).await?;

                Ok(BrowserHtmlState {
                    url: page_state.url,
                    title: page_state.title,
                    history: page_state.history,
                    html,
                })
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn eval_js(
        &mut self,
        expression: &str,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserEvalState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            let result = tauri::async_runtime::block_on(async {
                page.evaluate_expression(expression)
                    .await
                    .map_err(|error| BrowserError::Eval(error.to_string()))?
                    .into_value::<serde_json::Value>()
                    .map_err(|error| BrowserError::Eval(error.to_string()))
            })?;
            wait_for_page_settle(timeout_ms);
            let page_state = tauri::async_runtime::block_on(snapshot_page_state(&page))?;

            Ok(BrowserEvalState {
                url: page_state.url,
                title: page_state.title,
                history: page_state.history,
                result,
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = expression;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }
    pub fn switch_visibility(
        &mut self,
        mode: BrowserVisibilityMode,
    ) -> Result<Option<String>, BrowserError> {
        #[cfg(feature = "browser")]
        {
            self.config.visibility = mode;

            let prior_url = self
                .session
                .as_ref()
                .and_then(|session| session.page.clone())
                .and_then(|page| {
                    tauri::async_runtime::block_on(async {
                        page.url()
                            .await
                            .ok()
                            .flatten()
                            .filter(|url| url != "about:blank" && !url.is_empty())
                    })
                });

            self.session = None;

            if let Some(ref url) = prior_url {
                let user_agent = self.config.user_agent.clone();
                let session = self.ensure_session()?;
                let page = session.ensure_page(user_agent.as_deref())?;
                tauri::async_runtime::block_on(async {
                    page.goto(url)
                        .await
                        .map_err(|error| BrowserError::Navigate(error.to_string()))
                })?;
            }

            Ok(prior_url)
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = mode;
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
}
#[cfg(any(feature = "browser", test))]
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
pub(super) fn wait_for_page_settle(timeout_ms: Option<u64>) {
    let wait_ms = timeout_ms.unwrap_or(400).clamp(150, 2_000);
    std::thread::sleep(Duration::from_millis(wait_ms));
}

