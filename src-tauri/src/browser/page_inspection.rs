#[cfg(feature = "browser")]
use super::session::snapshot_page_state;
#[cfg(feature = "browser")]
use chromiumoxide::cdp::browser_protocol::page::{CaptureScreenshotFormat, Viewport};
#[cfg(feature = "browser")]
use chromiumoxide::page::ScreenshotParams;

use super::{BrowserError, BrowserEvalState, BrowserHtmlState, BrowserScreenshotState};
use crate::page_model::Rect;
use crate::resource_limits::{
    screenshots, validate_image_dimensions, validate_png_resource_limits, ImageLimitExceeded,
};

#[cfg(feature = "browser")]
#[derive(serde::Deserialize)]
struct DocumentScreenshotDimensions {
    width: u32,
    height: u32,
}

impl super::BrowserController {
    pub fn capture_screenshot(
        &mut self,
        full_page: bool,
        bbox: Option<Rect>,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserScreenshotState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let _permit = screenshots().try_acquire().map_err(|limit| {
                BrowserError::Screenshot(format!("screenshot capture was not started: {limit}"))
            })?;
            if let Some(bbox) = bbox.as_ref() {
                let width = bbox.width.ceil().max(0.0) as u32;
                let height = bbox.height.ceil().max(0.0) as u32;
                validate_image_dimensions(width, height).map_err(image_limit_to_browser_error)?;
            }
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            let screenshot_bytes = tauri::async_runtime::block_on(async {
                let mut builder = ScreenshotParams::builder().format(CaptureScreenshotFormat::Png);
                if full_page {
                    let dimensions = page
                        .evaluate(
                            "({ width: Math.ceil(Math.max(document.documentElement?.scrollWidth || 0, document.body?.scrollWidth || 0, window.innerWidth || 0)), height: Math.ceil(Math.max(document.documentElement?.scrollHeight || 0, document.body?.scrollHeight || 0, window.innerHeight || 0)) })",
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
                            "bbox screenshots require positive width and height",
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

            super::wait_for_page_settle(timeout_ms);
            let after = tauri::async_runtime::block_on(snapshot_page_state(&page))?;
            let (width, height) = validate_png_resource_limits(&screenshot_bytes)
                .map_err(image_limit_to_browser_error)?;

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

fn image_limit_to_browser_error(error: ImageLimitExceeded) -> BrowserError {
    let message = match error {
        ImageLimitExceeded::InvalidDimensions { .. } => {
            String::from("captured screenshot was not a valid positive-size PNG image")
        }
        ImageLimitExceeded::Dimensions {
            width,
            height,
            maximum_width,
            maximum_height,
        } => format!(
            "screenshot dimensions {width}x{height} exceed the {maximum_width}x{maximum_height} limit"
        ),
        ImageLimitExceeded::Pixels { pixels, maximum } => format!(
            "screenshot contains {pixels} pixels, exceeding the {maximum}-pixel limit"
        ),
        ImageLimitExceeded::EncodedBytes { bytes, maximum } => format!(
            "screenshot contains {bytes} encoded bytes, exceeding the {maximum}-byte limit"
        ),
    };
    BrowserError::Screenshot(message)
}
