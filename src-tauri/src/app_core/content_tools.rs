use std::fs;

use crate::commands::{
    CaptureScreenshotData, CaptureScreenshotInput, EvalJsData, EvalJsInput, GetHtmlData,
    GetHtmlInput, ScrollPageData, ScrollPageInput, ToolError, ToolName, ToolResult,
};

impl super::AppCore {
    pub fn execute_get_html(&mut self, input: GetHtmlInput) -> ToolResult<GetHtmlData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return Self::browser_runtime_missing_page(ToolName::GetHtml, input.request_id);
        };

        let browser_html = match self.browser.get_html(input.timeout_ms) {
            Ok(browser_html) => browser_html,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::GetHtml,
                    input.request_id,
                    String::from("Live browser HTML retrieval did not complete successfully."),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_html.url.clone());
            current_page.title = browser_html.title.clone();
        }
        self.state.browser_history = browser_html.history.clone();

        let html_length = browser_html.html.len();
        ToolResult::success(
            ToolName::GetHtml,
            input.request_id,
            GetHtmlData {
                page_id,
                url: browser_html.url,
                title: browser_html.title,
                html: browser_html.html,
                html_length,
            },
            vec![
                String::from("Read the live browser document HTML from the current active page."),
                String::from(
                    "Runtime page metadata was refreshed from the live browser without altering the current page model.",
                ),
            ],
        )
    }

    pub fn execute_eval_js(&mut self, input: EvalJsInput) -> ToolResult<EvalJsData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return Self::browser_runtime_missing_page(ToolName::EvalJs, input.request_id);
        };

        let browser_eval = match self
            .browser
            .eval_js(input.expression.trim(), input.timeout_ms)
        {
            Ok(browser_eval) => browser_eval,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::EvalJs,
                    input.request_id,
                    String::from(
                        "Live browser JavaScript evaluation did not complete successfully.",
                    ),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_eval.url.clone());
            current_page.title = browser_eval.title.clone();
        }
        self.state.browser_history = browser_eval.history.clone();

        ToolResult::success(
            ToolName::EvalJs,
            input.request_id,
            EvalJsData {
                page_id,
                url: browser_eval.url,
                title: browser_eval.title,
                result: browser_eval.result,
            },
            vec![
                String::from(
                    "Evaluated the requested JavaScript expression against the live browser page.",
                ),
                String::from(
                    "Runtime page metadata was refreshed from the live browser, but page-model regions were not automatically re-extracted.",
                ),
            ],
        )
    }

    pub fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::ScrollPage, input.request_id);
        }

        if input.amount_px.is_none() && input.target.is_none() {
            return ToolResult::failure(
                ToolName::ScrollPage,
                input.request_id,
                ToolError {
                    code: String::from("invalid_scroll_request"),
                    message: String::from(
                        "scroll_page requires amount_px or target to be provided",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Scroll request was rejected because it did not specify an amount or a target.",
                )],
            );
        }

        let clamped_amount = input
            .amount_px
            .map(|amount| amount.clamp(0.0, super::MAX_SCROLL_AMOUNT_PX));
        let scroll_state = match self.browser.scroll_page(
            input.direction,
            clamped_amount,
            input.target,
            input.timeout_ms,
        ) {
            Ok(scroll_state) => scroll_state,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::ScrollPage,
                    input.request_id,
                    String::from("Live browser scrolling did not complete successfully."),
                    error,
                )
            }
        };

        let mut observations = vec![String::from("Executed a live browser scroll request.")];
        if input
            .amount_px
            .zip(clamped_amount)
            .is_some_and(|(requested, clamped)| (requested - clamped).abs() > f32::EPSILON)
        {
            observations.push(format!(
                "Requested scroll amount was clamped to the supported maximum of {} px.",
                super::MAX_SCROLL_AMOUNT_PX
            ));
        }
        if scroll_state.reached_boundary {
            observations.push(String::from(
                "The scroll request reached a document boundary.",
            ));
        }

        ToolResult::success(
            ToolName::ScrollPage,
            input.request_id,
            ScrollPageData {
                previous_scroll_y: scroll_state.previous_scroll_y,
                current_scroll_y: scroll_state.current_scroll_y,
                reached_boundary: scroll_state.reached_boundary,
            },
            observations,
        )
    }

    pub fn execute_capture_screenshot(
        &mut self,
        input: CaptureScreenshotInput,
    ) -> ToolResult<CaptureScreenshotData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(
                ToolName::CaptureScreenshot,
                input.request_id,
            );
        }

        let region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);
        let region_id_active = input.region_id.as_deref().is_some();
        let targeting_modes = usize::from(input.scope.captures_full_page())
            + usize::from(region_id_active)
            + usize::from(input.bbox.is_some());
        if targeting_modes > 1 {
            return ToolResult::failure(
                ToolName::CaptureScreenshot,
                input.request_id,
                ToolError {
                    code: String::from("invalid_screenshot_target"),
                    message: String::from(
                        "capture_screenshot accepts at most one targeting mode from scope = FullPage, region_id, or bbox",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Screenshot request was rejected because it combined multiple targeting modes.",
                )],
            );
        }

        if let Some(bbox) = input.bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::CaptureScreenshot,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_screenshot_bbox"),
                        message: String::from(
                            "capture_screenshot bbox requires positive width and height",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({
                            "x": bbox.x,
                            "y": bbox.y,
                            "width": bbox.width,
                            "height": bbox.height,
                        })),
                    },
                    vec![String::from(
                        "Screenshot request was rejected because the requested bbox was not positive-sized.",
                    )],
                );
            }
        }

        let screenshot_bbox = if let Some(region_id) = region_id.as_deref() {
            let regions = match self.readable_regions() {
                Ok(regions) => regions,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::CaptureScreenshot,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted screenshot capture requires readable regions in the current page model.",
                        )],
                    )
                }
            };

            let bbox = match super::region_bbox_by_id(regions, region_id) {
                Ok(bbox) => bbox,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::CaptureScreenshot,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted screenshot capture could not resolve a usable bounding box for the requested region.",
                        )],
                    )
                }
            };

            Some(bbox)
        } else {
            input.bbox.clone()
        };

        let browser_screenshot = match self.browser.capture_screenshot(
            input.scope.captures_full_page(),
            screenshot_bbox.clone(),
            input.timeout_ms,
        ) {
            Ok(browser_screenshot) => browser_screenshot,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::CaptureScreenshot,
                    input.request_id,
                    String::from("Live browser screenshot capture did not complete successfully."),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_screenshot.url.clone());
            current_page.title = browser_screenshot.title.clone();
        }
        self.state.browser_history = browser_screenshot.history.clone();

        let image_id = self.next_image_id(&input.request_id);
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
        if input.scope.captures_full_page() {
            observations.push(String::from(
                "The screenshot targeted the full page rather than only the current viewport.",
            ));
        } else if region_id.is_some() {
            observations.push(String::from(
                "The screenshot was clipped to the requested page region using its stored bounding box.",
            ));
        } else if browser_screenshot.bbox.is_some() {
            observations.push(String::from(
                "The screenshot was clipped to the explicitly requested bounding box.",
            ));
        } else {
            observations.push(String::from(
                "The screenshot used the current viewport because no full-page or bbox target was requested.",
            ));
        }

        ToolResult::success(
            ToolName::CaptureScreenshot,
            input.request_id,
            CaptureScreenshotData {
                image_id,
                path: screenshot_path.display().to_string(),
                bbox: browser_screenshot.bbox,
                width: browser_screenshot.width,
                height: browser_screenshot.height,
            },
            observations,
        )
    }
}
