use crate::commands::{
    CaptureScreenshotInput, ExtractPageModelData, ExtractPageModelInput, GetPageSnapshotInput,
    MergeOcrIntoPageModelData, MergeOcrIntoPageModelInput, PageSnapshotData, RunOcrData,
    RunOcrInput, ToolError, ToolName, ToolResult,
};
use crate::extractor::extract_structured_article_from_html;
use crate::ocr::OcrSettings;
use crate::page_model::PageModel;
use super::ocr_merge::{extracted_text_metrics, merge_ocr_text_into_page_model, region_first_ocr_target_ids};
use super::page_model_builder::{
    build_extracted_page_model, build_visible_text_excerpt, extract_page_model_internal_failure,
    infer_extraction_source, nested_tool_failure_as_extract_page_model, ocr_runtime_error_to_tool_error,
};

impl super::AppCore {
    pub fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
        let image_id = input
            .image_id
            .as_deref()
            .map(str::trim)
            .filter(|image_id| !image_id.is_empty())
            .map(ToOwned::to_owned);
        let region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);

        if image_id.is_none() && region_id.is_none() && input.bbox.is_none() {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("invalid_ocr_request"),
                    message: String::from(
                        "run_ocr requires at least one source from image_id, region_id, or bbox",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR request was rejected because it did not identify any image or target area.",
                )],
            );
        }

        if let Some(bbox) = input.bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_ocr_bbox"),
                        message: String::from("run_ocr bbox requires positive width and height"),
                        retryable: false,
                        details: Some(serde_json::json!({
                            "x": bbox.x,
                            "y": bbox.y,
                            "width": bbox.width,
                            "height": bbox.height,
                        })),
                    },
                    vec![String::from(
                        "OCR request was rejected because the requested bbox was not positive-sized.",
                    )],
                );
            }
        }

        let ocr_bbox = if let Some(region_id) = region_id.as_deref() {
            let regions = match self.readable_regions() {
                Ok(regions) => regions,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::RunOcr,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted OCR requires readable regions in the current page model.",
                        )],
                    )
                }
            };

            match super::region_bbox_by_id(regions, region_id) {
                Ok(bbox) => Some(bbox),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::RunOcr,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted OCR could not resolve a usable bounding box for the requested region.",
                        )],
                    )
                }
            }
        } else {
            input.bbox.clone()
        };

        let Some(image_id) = image_id else {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("missing_ocr_image_id"),
                    message: String::from(
                        "run_ocr currently requires image_id so it can resolve a persisted screenshot",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR needs a persisted screenshot image_id because implicit image selection is not supported.",
                )],
            );
        };

        let image_path = match self.cached_image_path(&image_id) {
            Ok(path) => path,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR could not resolve the cached screenshot path.",
                    )],
                )
            }
        };

        if !image_path.is_file() {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("ocr_image_not_found"),
                    message: String::from(
                        "run_ocr could not find the cached screenshot for the requested image_id",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "image_id": image_id,
                        "path": image_path.display().to_string(),
                    })),
                },
                vec![String::from(
                    "OCR could not start because the requested cached screenshot does not exist.",
                )],
            );
        }

        let ocr_result = match self.ocr.run_ocr(&image_path, ocr_bbox.as_ref()) {
            Ok(result) => result,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    ocr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "OCR could not extract text from the requested screenshot.",
                    )],
                )
            }
        };

        let mut observations = vec![String::from(
            "Ran deterministic OCR on the requested cached screenshot.",
        )];
        if region_id.is_some() {
            observations.push(String::from(
                "OCR was limited to the requested page region using its stored bounding box.",
            ));
        } else if ocr_bbox.is_some() {
            observations.push(String::from(
                "OCR was limited to the explicitly requested bounding box within the cached image.",
            ));
        } else {
            observations.push(String::from(
                "OCR used the full cached screenshot because no bbox override was provided.",
            ));
        }
        if ocr_result.extracted_text.is_empty() {
            observations.push(String::from(
                "OCR completed successfully but did not extract any readable text.",
            ));
        }

        let extracted_text = ocr_result.extracted_text;
        let text_length = extracted_text.len();

        ToolResult::success(
            ToolName::RunOcr,
            input.request_id,
            RunOcrData {
                image_id: Some(image_id),
                extracted_text,
                text_length,
                confidence: ocr_result.confidence,
                source_bbox: ocr_bbox,
            },
            observations,
        )
    }

    pub fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData> {
        let requested_page_id = input.page_id.trim().to_string();
        if requested_page_id.is_empty() {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("invalid_page_id"),
                    message: String::from(
                        "merge_ocr_into_page_model requires a non-empty page_id",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR merge was rejected because it did not identify a target page.",
                )],
            );
        }

        let Some(active_page_id) = self.state.current_page_id.as_deref() else {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "merge_ocr_into_page_model requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": requested_page_id })),
                },
                vec![String::from(
                    "OCR merge could not run because no active page is loaded in runtime state.",
                )],
            );
        };

        if active_page_id != requested_page_id {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("page_id_mismatch"),
                    message: String::from(
                        "merge_ocr_into_page_model page_id must match the active runtime page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "active_page_id": active_page_id,
                        "page_id": requested_page_id,
                    })),
                },
                vec![String::from(
                    "OCR merge was rejected because it targeted a page that is not currently active.",
                )],
            );
        }

        let normalized_ocr_text = input.ocr_text.trim().to_string();
        if normalized_ocr_text.is_empty() {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("invalid_ocr_text"),
                    message: String::from(
                        "merge_ocr_into_page_model requires non-empty ocr_text",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR merge was rejected because there was no recognized text to merge.",
                )],
            );
        }

        if let Some(bbox) = input.source_bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_source_bbox"),
                        message: String::from(
                            "merge_ocr_into_page_model source_bbox requires positive width and height",
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
                        "OCR merge was rejected because the supplied source bounding box was invalid.",
                    )],
                );
            }
        }

        let next_region_id = self.next_ocr_region_id(&input.request_id);
        let requested_region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);

        let merge_outcome = {
            let Some(current_page) = self.state.current_page.as_mut() else {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "merge_ocr_into_page_model requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": requested_page_id })),
                    },
                    vec![String::from(
                        "OCR merge could not update the page because the runtime page model is missing.",
                    )],
                );
            };

            merge_ocr_text_into_page_model(
                current_page,
                requested_region_id.as_deref(),
                &normalized_ocr_text,
                input.source_bbox.clone(),
                next_region_id,
            )
        };

        let updated_region_ids = match merge_outcome {
            Ok(updated_region_ids) => updated_region_ids,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR merge could not apply the recognized text to the runtime page model.",
                    )],
                )
            }
        };

        let mut observations = vec![String::from(
            "Merged OCR text into the active runtime page model.",
        )];
        if requested_region_id.is_some() {
            observations.push(String::from(
                "OCR text updated an existing page region and marked it as mixed DOM/OCR content.",
            ));
        } else {
            observations.push(String::from(
                "OCR text was added as a new OCR region because no existing target region was supplied.",
            ));
        }

        ToolResult::success(
            ToolName::MergeOcrIntoPageModel,
            input.request_id,
            MergeOcrIntoPageModelData {
                page_id: requested_page_id,
                updated_region_ids,
                merged_text_length: normalized_ocr_text.len(),
            },
            observations,
        )
    }

    pub fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "get_page_snapshot requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Page snapshot could not be created because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "get_page_snapshot requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Page snapshot could not be created because the runtime page model is missing.",
                )],
            );
        };

        let Some(url) = current_page.url.clone() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_url"),
                    message: String::from(
                        "get_page_snapshot requires a current page URL in runtime state",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Page snapshot could not be created because the runtime page URL is missing.",
                )],
            );
        };

        let title = current_page.title.clone();
        let visible_text_excerpt =
            build_visible_text_excerpt(current_page, input.text_excerpt_max_chars);
        let interactive_elements = if input.include_interactive_elements {
            current_page.interactive_elements.clone()
        } else {
            Vec::new()
        };
        let page_metrics = match self.browser.get_page_metrics() {
            Ok(page_metrics) => page_metrics,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::GetPageSnapshot,
                    input.request_id,
                    String::from(
                        "Live page snapshot metrics could not be read from the active browser page.",
                    ),
                    error,
                )
            }
        };

        ToolResult::success(
            ToolName::GetPageSnapshot,
            input.request_id,
            PageSnapshotData {
                page_id,
                url,
                title,
                visible_text_excerpt,
                interactive_elements,
                scroll_y: page_metrics.scroll_y,
                viewport_width: page_metrics.viewport_width,
                viewport_height: page_metrics.viewport_height,
                document_height: page_metrics.document_height,
            },
            vec![
                String::from(
                    "Built a deterministic page snapshot from the current runtime page state.",
                ),
                String::from(
                    "Included live scroll and viewport metrics from the active browser page.",
                ),
            ],
        )
    }

    pub fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ExtractPageModel,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "extract_page_model requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Page model extraction could not run because no page has been opened yet.",
                )],
            );
        };

        let mut used_dom_smoothie = false;
        let mut dom_smoothie_fallback_reason: Option<String> = None;
        let base_page_model = if input.use_dom_extraction {
            let browser_page_model = match self.browser.extract_page_model() {
                Ok(extracted_page_model) => extracted_page_model,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::ExtractPageModel,
                        input.request_id,
                        String::from(
                            "Live browser page-model extraction did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

            let extracted_page_model = match self.browser.get_html(input.timeout_ms) {
                Ok(browser_html) => match extract_structured_article_from_html(
                    &browser_html.html,
                    browser_page_model.url.as_deref(),
                    browser_page_model.interactive_elements.clone(),
                ) {
                    Ok(extracted_article) => {
                        used_dom_smoothie = true;
                        extracted_article.into_page_model()
                    }
                    Err(error) => {
                        dom_smoothie_fallback_reason = Some(error.to_string());
                        tracing::warn!(error = %error, "dom_smoothie extraction fell back to browser DOM model");
                        browser_page_model
                    }
                },
                Err(error) => {
                    dom_smoothie_fallback_reason = Some(error.to_string());
                    tracing::warn!(error = %error, "HTML retrieval failed during dom_smoothie extraction; falling back to browser DOM model");
                    browser_page_model
                }
            };

            self.state.current_page = Some(extracted_page_model.clone());
            extracted_page_model
        } else {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::ExtractPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "extract_page_model requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Page model extraction could not run because the runtime page model is missing.",
                    )],
                );
            };

            current_page.clone()
        };

        let mut observations = if input.use_dom_extraction {
            if used_dom_smoothie {
                vec![String::from(
                    "Built a deterministic page model from live HTML using dom_smoothie and persisted it into runtime state.",
                )]
            } else {
                let mut observations = vec![String::from(
                    "dom_smoothie extraction fell back to the live Chromium DOM model.",
                )];
                if let Some(reason) = dom_smoothie_fallback_reason.as_deref() {
                    observations.push(format!("dom_smoothie fallback reason: {reason}"));
                }
                observations.push(String::from(
                    "Built a deterministic page model from the live Chromium DOM and persisted it into runtime state.",
                ));
                observations
            }
        } else {
            vec![String::from(
                "Built a deterministic page model from the current runtime page state.",
            )]
        };
        if !input.include_links {
            observations.push(String::from(
                "Link elements were omitted from the extracted page model as requested.",
            ));
        }
        if input.include_headings {
            observations.push(String::from(
                "Heading-specific extraction is not yet distinguished in the current page model schema, so regions were returned unchanged.",
            ));
        }
        if !input.use_dom_extraction {
            observations.push(String::from(
                "A non-DOM extraction request currently reuses runtime page state until OCR-specific extraction is wired.",
            ));
        }

        if should_trigger_extract_page_model_ocr_fallback(
            input.use_dom_extraction,
            &base_page_model,
            &self.config.ocr,
        ) {
            observations.push(String::from(
                "Live DOM extraction did not produce enough readable text, so OCR fallback was triggered.",
            ));

            let screenshot_result = self.execute_capture_screenshot(CaptureScreenshotInput {
                request_id: format!("{}-ocr-fallback-screenshot", input.request_id),
                timeout_ms: input.timeout_ms,
                scope: crate::commands::ScreenshotScope::FullPage,
                region_id: None,
                bbox: None,
            });
            if !screenshot_result.ok {
                return nested_tool_failure_as_extract_page_model(
                    input.request_id,
                    observations,
                    screenshot_result,
                    String::from(
                        "OCR fallback could not capture a screenshot for page-model recovery.",
                    ),
                );
            }
            observations.extend(screenshot_result.observations.clone());
            let Some(screenshot_data) = screenshot_result.data else {
                return extract_page_model_internal_failure(
                    input.request_id,
                    String::from("OCR fallback screenshot result did not include screenshot data"),
                    observations,
                );
            };

            let region_first_targets =
                region_first_ocr_target_ids(&base_page_model, &self.config.ocr);
            let had_region_first_targets = !region_first_targets.is_empty();
            let mut recovered_region_text = false;

            if !region_first_targets.is_empty() {
                observations.push(format!(
                    "OCR fallback is trying {} bbox-backed readable regions before broader OCR.",
                    region_first_targets.len()
                ));

                for region_id in region_first_targets {
                    let ocr_result = self.execute_run_ocr(RunOcrInput {
                        request_id: format!("{}-ocr-fallback-run-{region_id}", input.request_id),
                        timeout_ms: input.timeout_ms,
                        image_id: Some(screenshot_data.image_id.clone()),
                        region_id: Some(region_id.clone()),
                        bbox: None,
                    });
                    observations.extend(ocr_result.observations.clone());

                    if !ocr_result.ok {
                        let failure_code = ocr_result
                            .error
                            .as_ref()
                            .map(|error| error.code.as_str())
                            .unwrap_or("unknown_ocr_error");
                        observations.push(format!(
                            "Region-first OCR for region_id={region_id} did not succeed ({failure_code}), so broader OCR may still be needed."
                        ));
                        continue;
                    }

                    let Some(ocr_data) = ocr_result.data else {
                        return extract_page_model_internal_failure(
                            input.request_id,
                            String::from("Region-first OCR result did not include OCR data"),
                            observations,
                        );
                    };

                    if ocr_data.extracted_text.is_empty() {
                        observations.push(format!(
                            "Region-first OCR completed for region_id={region_id}, but it did not recover readable text."
                        ));
                        continue;
                    }

                    let merge_result =
                        self.execute_merge_ocr_into_page_model(MergeOcrIntoPageModelInput {
                            request_id: format!(
                                "{}-ocr-fallback-merge-{region_id}",
                                input.request_id
                            ),
                            timeout_ms: input.timeout_ms,
                            page_id: page_id.clone(),
                            region_id: Some(region_id.clone()),
                            ocr_text: ocr_data.extracted_text,
                            source_bbox: ocr_data.source_bbox,
                        });
                    if !merge_result.ok {
                        return nested_tool_failure_as_extract_page_model(
                            input.request_id,
                            observations,
                            merge_result,
                            String::from(
                                "Region-first OCR recovered text, but merging it into the page model failed.",
                            ),
                        );
                    }
                    observations.extend(merge_result.observations);
                    observations.push(format!(
                        "Region-first OCR recovered readable text for region_id={region_id}."
                    ));
                    recovered_region_text = true;
                }
            }

            if !recovered_region_text {
                if self.config.ocr.prefer_region_ocr {
                    if !had_region_first_targets {
                        observations.push(String::from(
                            "Region-first OCR was unavailable because no bbox-backed readable regions were present, so fallback widened to the full-page screenshot.",
                        ));
                    } else {
                        observations.push(String::from(
                            "Region-first OCR did not recover enough text, so fallback widened to the full-page screenshot.",
                        ));
                    }
                }

                let ocr_result = self.execute_run_ocr(RunOcrInput {
                    request_id: format!("{}-ocr-fallback-run", input.request_id),
                    timeout_ms: input.timeout_ms,
                    image_id: Some(screenshot_data.image_id.clone()),
                    region_id: None,
                    bbox: None,
                });
                if !ocr_result.ok {
                    return nested_tool_failure_as_extract_page_model(
                        input.request_id,
                        observations,
                        ocr_result,
                        String::from(
                            "OCR fallback could not extract readable text from the recovery screenshot.",
                        ),
                    );
                }
                observations.extend(ocr_result.observations.clone());
                let Some(ocr_data) = ocr_result.data else {
                    return extract_page_model_internal_failure(
                        input.request_id,
                        String::from("OCR fallback result did not include OCR data"),
                        observations,
                    );
                };

                if ocr_data.extracted_text.is_empty() {
                    observations.push(String::from(
                        "OCR fallback completed, but it still did not recover readable text.",
                    ));
                } else {
                    let merge_result =
                        self.execute_merge_ocr_into_page_model(MergeOcrIntoPageModelInput {
                            request_id: format!("{}-ocr-fallback-merge", input.request_id),
                            timeout_ms: input.timeout_ms,
                            page_id: page_id.clone(),
                            region_id: None,
                            ocr_text: ocr_data.extracted_text,
                            source_bbox: ocr_data.source_bbox,
                        });
                    if !merge_result.ok {
                        return nested_tool_failure_as_extract_page_model(
                            input.request_id,
                            observations,
                            merge_result,
                            String::from(
                                "OCR fallback recovered text, but merging it into the page model failed.",
                            ),
                        );
                    }
                    observations.extend(merge_result.observations);
                    observations.push(String::from(
                        "OCR fallback recovered readable text and merged it into the runtime page model.",
                    ));
                }
            }
        }

        let runtime_page_model = self
            .state
            .current_page
            .clone()
            .unwrap_or_else(|| base_page_model.clone());
        let extracted_page_model = build_extracted_page_model(&runtime_page_model, &input);
        let region_count = extracted_page_model.regions.len();
        let readable_region_count = extracted_page_model
            .regions
            .iter()
            .filter(|region| !region.text.trim().is_empty())
            .count();
        let extraction_source = infer_extraction_source(
            &runtime_page_model,
            input.use_dom_extraction,
            used_dom_smoothie,
        );

        ToolResult::success(
            ToolName::ExtractPageModel,
            input.request_id,
            ExtractPageModelData {
                page_model: extracted_page_model,
                region_count,
                readable_region_count,
                extraction_source,
            },
            observations,
        )
    }
}

pub(crate) fn should_trigger_extract_page_model_ocr_fallback(
    use_dom_extraction: bool,
    page: &PageModel,
    ocr_settings: &OcrSettings,
) -> bool {
    if !use_dom_extraction || !ocr_settings.trigger_on_no_extractable_text {
        return false;
    }

    let (readable_char_count, readable_region_count) = extracted_text_metrics(page);

    readable_region_count == 0
        || readable_char_count <= ocr_settings.sparse_text_char_threshold as usize
        || readable_region_count < ocr_settings.sparse_text_region_threshold as usize
}
