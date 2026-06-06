#[cfg(feature = "browser")]
use super::session::snapshot_page_state;
#[cfg(feature = "browser")]
use chromiumoxide::cdp::browser_protocol::page::{CaptureScreenshotFormat, Viewport};
#[cfg(feature = "browser")]
use chromiumoxide::page::ScreenshotParams;

use crate::page_model::Rect;
use super::{BrowserEvalState, BrowserError, BrowserHtmlState, BrowserScreenshotState};

impl super::BrowserController {
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

            super::wait_for_page_settle(timeout_ms);
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

            super::wait_for_page_settle(timeout_ms);
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
            super::wait_for_page_settle(timeout_ms);
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
}

#[cfg(any(feature = "browser", test))]
pub(super) fn png_dimensions(image_bytes: &[u8]) -> Result<(u32, u32), BrowserError> {
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
