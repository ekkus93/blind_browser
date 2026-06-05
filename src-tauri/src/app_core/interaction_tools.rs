use crate::commands::{
    ClickElementData, ClickElementInput, ElementCandidate, FindElementData, FindElementInput,
    FocusElementData, FocusElementInput, ListInteractiveElementsData, ListInteractiveElementsInput,
    SubmitActiveFormData, SubmitActiveFormInput, ToolError, ToolName, ToolResult,
    TypeIntoElementData, TypeIntoElementInput,
};
use crate::narration::find_region_index;
use crate::page_model::{ElementRole, PageModel, PageRegion, Rect};

impl super::AppCore {
    pub fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "list_interactive_elements requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Interactive elements could not be listed because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "list_interactive_elements requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Interactive elements could not be listed because the runtime page model is missing.",
                )],
            );
        };

        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visibility_filter.visible_only(),
            input.roles.as_deref(),
        );
        let visible_count = elements.iter().filter(|element| element.visible).count();

        let mut observations = vec![String::from(
            "Listed deterministic interactive elements from the current runtime page state.",
        )];
        if input.visibility_filter.visible_only() {
            observations.push(String::from(
                "Results were filtered to elements currently marked visible in runtime state.",
            ));
        }
        if let Some(roles) = input.roles.as_ref() {
            observations.push(format!(
                "Results were filtered to {} requested role(s).",
                roles.len()
            ));
        }

        ToolResult::success(
            ToolName::ListInteractiveElements,
            input.request_id,
            ListInteractiveElementsData {
                page_id,
                elements,
                visible_count,
            },
            observations,
        )
    }

    pub fn execute_find_element(
        &mut self,
        input: FindElementInput,
    ) -> ToolResult<FindElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("find_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Element search could not run because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "find_element requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Element search could not run because the runtime page model is missing.",
                )],
            );
        };

        let search_query = match build_find_element_query(&input) {
            Ok(search_query) => search_query,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::FindElement,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Element search was rejected because the search criteria were empty.",
                    )],
                )
            }
        };

        let candidate_limit = input
            .max_candidates
            .unwrap_or(super::DEFAULT_FIND_ELEMENT_MAX_CANDIDATES)
            .clamp(1, super::MAX_FIND_ELEMENT_CANDIDATES);
        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visibility_filter.visible_only(),
            input.role.as_ref().map(std::slice::from_ref),
        );
        let ranked_candidates =
            rank_find_element_candidates(&elements, &search_query, candidate_limit);
        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(
                &ranked_candidates,
                self.config.safety.confirmation_confidence_threshold,
            );

        let mut observations = vec![format!(
            "Searched {} interactive element(s) from the current runtime page state.",
            elements.len()
        )];
        if input.visibility_filter.visible_only() {
            observations.push(String::from(
                "Search was limited to elements currently marked visible in runtime state.",
            ));
        }
        if input
            .max_candidates
            .is_some_and(|value| value > super::MAX_FIND_ELEMENT_CANDIDATES)
        {
            observations.push(format!(
                "Candidate count was clamped to the supported maximum of {}.",
                super::MAX_FIND_ELEMENT_CANDIDATES
            ));
        }
        if ranked_candidates.is_empty() {
            observations.push(String::from(
                "No interactive elements produced a positive match score for the requested search criteria.",
            ));
        } else if requires_confirmation {
            observations.push(String::from(
                "Top candidates are too close to choose deterministically, so planner clarification is required before any side effect.",
            ));
        } else {
            observations.push(String::from(
                "A single strongest candidate was identified from the filtered interactive elements.",
            ));
        }

        ToolResult::success(
            ToolName::FindElement,
            input.request_id,
            FindElementData {
                query_summary: search_query.summary,
                chosen_element_id,
                chosen_confidence,
                candidates: ranked_candidates,
                requires_confirmation,
            },
            observations,
        )
    }

    pub fn execute_click_element(
        &mut self,
        input: ClickElementInput,
    ) -> ToolResult<ClickElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ClickElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("click_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Click could not run because no page has been opened yet.",
                )],
            );
        };

        let element = {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::ClickElement,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "click_element requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Click could not run because the runtime page model is missing.",
                    )],
                );
            };

            match resolve_clickable_element(current_page, &input.element_id) {
                Ok(element) => element.clone(),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::ClickElement,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Click could not run because the requested deterministic element_id was not currently interactable.",
                        )],
                    )
                }
            }
        };

        let browser_click = match self.browser.click_element(
            &element,
            input.click_mode.is_double_click(),
            input.timeout_ms,
        ) {
            Ok(browser_click) => browser_click,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::ClickElement,
                    input.request_id,
                    String::from("Live browser click did not complete successfully."),
                    error,
                )
            }
        };

        if browser_click.page_changed {
            let next_page_id = self.next_page_id(&input.request_id);
            self.state
                .record_navigation(next_page_id, browser_click.url.clone());
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.title = browser_click.title.clone();
            }
        } else if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_click.url.clone());
            current_page.title = browser_click.title.clone();
        }
        self.state.browser_history = browser_click.history.clone();

        let mut observations = vec![format!(
            "Triggered a live Chromium DOM click for element_id={}",
            element.element_id
        )];
        if input.click_mode.is_double_click() {
            observations.push(String::from(
                "The browser backend executed the click with a double-click count.",
            ));
        }
        if browser_click.page_changed {
            observations.push(String::from(
                "The live browser URL changed after the click, so runtime page state advanced to a new page.",
            ));
        } else {
            observations.push(String::from(
                "The click completed without a live browser navigation, so runtime state stayed on the current page.",
            ));
        }

        ToolResult::success(
            ToolName::ClickElement,
            input.request_id,
            ClickElementData {
                element_id: element.element_id.clone(),
                action_performed: true,
                page_changed: browser_click.page_changed,
                navigation_url: browser_click
                    .page_changed
                    .then_some(browser_click.url.clone()),
                resulting_title: browser_click.title,
            },
            observations,
        )
    }

    pub fn execute_focus_element(
        &mut self,
        input: FocusElementInput,
    ) -> ToolResult<FocusElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::FocusElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("focus_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Field focus could not run because no page has been opened yet.",
                )],
            );
        };

        let element = {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::FocusElement,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "focus_element requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Field focus could not run because the runtime page model is missing.",
                    )],
                );
            };

            match resolve_clickable_element(current_page, &input.element_id) {
                Ok(element) => element.clone(),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::FocusElement,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Field focus could not run because the requested deterministic element_id was not currently interactable.",
                        )],
                    )
                }
            }
        };

        let browser_focus = match self.browser.focus_element(&element, input.timeout_ms) {
            Ok(browser_focus) => browser_focus,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::FocusElement,
                    input.request_id,
                    String::from("Live browser field focus did not complete successfully."),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_focus.url.clone());
            current_page.title = browser_focus.title.clone();
        }
        self.state.browser_history = browser_focus.history.clone();
        self.update_recent_field_target(element.element_id.clone());

        ToolResult::success(
            ToolName::FocusElement,
            input.request_id,
            FocusElementData {
                element_id: element.element_id.clone(),
                focused: browser_focus.focused,
                element_role: Some(element.role),
            },
            vec![
                format!(
                    "Moved live browser focus to element_id={}.",
                    element.element_id
                ),
                String::from(
                    "The runtime page state remained on the current page after the focus change.",
                ),
            ],
        )
    }

    pub fn execute_type_into_element(
        &mut self,
        input: TypeIntoElementInput,
    ) -> ToolResult<TypeIntoElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::TypeIntoElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "type_into_element requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Field entry could not run because no page has been opened yet.",
                )],
            );
        };

        let element = {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::TypeIntoElement,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "type_into_element requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Field entry could not run because the runtime page model is missing.",
                    )],
                );
            };

            match resolve_typeable_element(current_page, &input.element_id) {
                Ok(element) => element.clone(),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::TypeIntoElement,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Field entry could not run because the requested deterministic element_id was not currently editable.",
                        )],
                    )
                }
            }
        };

        let browser_type = match self.browser.type_into_element(
            &element,
            &input.text,
            input.text_entry_mode.clears_existing_value(),
            input.submit_mode.submits_after_entry(),
            input.timeout_ms,
        ) {
            Ok(browser_type) => browser_type,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::TypeIntoElement,
                    input.request_id,
                    String::from("Live browser text entry did not complete successfully."),
                    error,
                )
            }
        };

        if browser_type.page_changed {
            let next_page_id = self.next_page_id(&input.request_id);
            self.state
                .record_navigation(next_page_id, browser_type.url.clone());
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.title = browser_type.title.clone();
            }
        } else if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_type.url.clone());
            current_page.title = browser_type.title.clone();
            let requested_element_id = element.element_id.clone();
            if let Some(live_element) = current_page
                .interactive_elements
                .iter_mut()
                .find(|live_element| live_element.element_id == requested_element_id)
            {
                live_element.value = browser_type.value_after.clone();
            }
        }
        self.state.browser_history = browser_type.history.clone();
        if browser_type.page_changed {
            self.clear_recent_field_context();
        } else {
            self.update_recent_field_target(element.element_id.clone());
        }

        let mut observations = vec![format!(
            "Sent text entry to live element_id={}.",
            element.element_id
        )];
        if input.text_entry_mode.clears_existing_value() {
            observations.push(String::from(
                "Existing field contents were cleared before the new text was applied.",
            ));
        }
        if browser_type.page_changed {
            observations.push(String::from(
                "Submitting the field changed the live browser URL, so runtime page state advanced to a new page.",
            ));
        } else {
            observations.push(String::from(
                "The field entry completed without live browser navigation.",
            ));
        }

        ToolResult::success(
            ToolName::TypeIntoElement,
            input.request_id,
            TypeIntoElementData {
                element_id: element.element_id.clone(),
                text_length: input.text.chars().count(),
                value_after: browser_type.value_after,
                accepted_input: browser_type.accepted_input,
            },
            observations,
        )
    }

    pub fn execute_submit_active_form(
        &mut self,
        input: SubmitActiveFormInput,
    ) -> ToolResult<SubmitActiveFormData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::SubmitActiveForm,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "submit_active_form requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Form submission could not run because no page has been opened yet.",
                )],
            );
        };

        let form = match input.form_element_id.as_deref() {
            Some(form_element_id) => {
                let Some(current_page) = self.state.current_page.as_ref() else {
                    return ToolResult::failure(
                        ToolName::SubmitActiveForm,
                        input.request_id,
                        ToolError {
                            code: String::from("missing_page_model"),
                            message: String::from(
                                "submit_active_form requires runtime page data for the active page",
                            ),
                            retryable: false,
                            details: Some(serde_json::json!({ "page_id": page_id })),
                        },
                        vec![String::from(
                            "Form submission could not run because the runtime page model is missing.",
                        )],
                    );
                };

                match resolve_form_element(current_page, form_element_id) {
                    Ok(form) => Some(form.clone()),
                    Err(error) => {
                        return ToolResult::failure(
                            ToolName::SubmitActiveForm,
                            input.request_id,
                            error,
                            vec![String::from(
                                "Form submission could not run because the requested form target was not currently submittable.",
                            )],
                        )
                    }
                }
            }
            None => None,
        };

        let browser_submit = match self
            .browser
            .submit_active_form(form.as_ref(), input.timeout_ms)
        {
            Ok(browser_submit) => browser_submit,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::SubmitActiveForm,
                    input.request_id,
                    String::from("Live browser form submission did not complete successfully."),
                    error,
                )
            }
        };

        if browser_submit.page_changed {
            let next_page_id = self.next_page_id(&input.request_id);
            self.state
                .record_navigation(next_page_id, browser_submit.url.clone());
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.title = browser_submit.title.clone();
            }
            self.clear_recent_field_context();
        } else if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_submit.url.clone());
            current_page.title = browser_submit.title.clone();
        }
        self.state.browser_history = browser_submit.history.clone();

        let mut observations = vec![String::from(
            "Triggered a live browser form submission request.",
        )];
        if let Some(form) = form.as_ref() {
            observations.push(format!(
                "The submission targeted deterministic form element_id={}.",
                form.element_id
            ));
        } else {
            observations.push(String::from(
                "The browser backend resolved the active or uniquely visible form at submit time.",
            ));
        }
        if browser_submit.page_changed {
            observations.push(String::from(
                "Submitting the form changed the live browser URL, so runtime page state advanced to a new page.",
            ));
        } else {
            observations.push(String::from(
                "The submission request completed without a live browser navigation.",
            ));
        }

        ToolResult::success(
            ToolName::SubmitActiveForm,
            input.request_id,
            SubmitActiveFormData {
                form_element_id: form.as_ref().map(|form| form.element_id.clone()),
                submitted: browser_submit.submitted,
                page_changed: browser_submit.page_changed,
                navigation_url: browser_submit
                    .page_changed
                    .then_some(browser_submit.url.clone()),
            },
            observations,
        )
    }
}

pub(super) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(super) fn region_bbox_by_id(
    regions: &[PageRegion],
    region_id: &str,
) -> Result<Rect, crate::commands::ToolError> {
    let Some(region_index) = find_region_index(regions, region_id) else {
        return Err(crate::commands::ToolError {
            code: String::from("unknown_region_id"),
            message: String::from(
                "capture_screenshot could not find the requested region_id in the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "region_id": region_id })),
        });
    };

    let region = &regions[region_index];
    let Some(bbox) = region.bbox.clone() else {
        return Err(crate::commands::ToolError {
            code: String::from("missing_region_bbox"),
            message: String::from(
                "capture_screenshot requires a bounding box for the requested region_id",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "region_id": region_id })),
        });
    };

    if bbox.width <= 0.0 || bbox.height <= 0.0 {
        return Err(crate::commands::ToolError {
            code: String::from("invalid_region_bbox"),
            message: String::from(
                "capture_screenshot requires a positive bounding box for the requested region_id",
            ),
            retryable: false,
            details: Some(serde_json::json!({
                "region_id": region_id,
                "x": bbox.x,
                "y": bbox.y,
                "width": bbox.width,
                "height": bbox.height,
            })),
        });
    }

    Ok(bbox)
}

pub(super) fn focusable_field_elements(
    page: &PageModel,
) -> Vec<crate::page_model::InteractiveElement> {
    filter_interactive_elements(
        &page.interactive_elements,
        true,
        Some(&[
            ElementRole::Input,
            ElementRole::TextArea,
            ElementRole::Select,
        ]),
    )
    .into_iter()
    .filter(|element| {
        element.enabled
            && element
                .dom_locator
                .as_deref()
                .map(str::trim)
                .is_some_and(|locator| !locator.is_empty())
    })
    .collect()
}

pub(super) fn submittable_form_elements(
    page: &PageModel,
) -> Vec<crate::page_model::InteractiveElement> {
    filter_interactive_elements(&page.interactive_elements, true, Some(&[ElementRole::Form]))
        .into_iter()
        .filter(|element| {
            element
                .dom_locator
                .as_deref()
                .map(str::trim)
                .is_some_and(|locator| !locator.is_empty())
        })
        .collect()
}

pub(super) fn summarize_candidate_names(
    page: &PageModel,
    candidates: &[ElementCandidate],
) -> Vec<String> {
    candidates
        .iter()
        .filter_map(|candidate| {
            page.interactive_elements
                .iter()
                .find(|element| element.element_id == candidate.element_id)
                .map(describe_field_element)
        })
        .take(super::MAX_DIRECT_FIELD_CANDIDATE_NAMES)
        .collect()
}

pub(super) fn summarize_form_candidate_names(
    forms: &[crate::page_model::InteractiveElement],
) -> Vec<String> {
    forms
        .iter()
        .map(describe_form_element)
        .take(super::MAX_DIRECT_FIELD_CANDIDATE_NAMES)
        .collect()
}

fn describe_field_element(element: &crate::page_model::InteractiveElement) -> String {
    element
        .accessible_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            element
                .placeholder
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            element
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(String::from)
        .unwrap_or_else(|| element.element_id.clone())
}

pub(super) fn describe_form_element(
    element: &crate::page_model::InteractiveElement,
) -> String {
    element
        .accessible_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            element
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            element
                .attributes
                .get("id")
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(|description| format!("the {description} form"))
        .unwrap_or_else(|| String::from("the current form"))
}

pub(super) fn filter_interactive_elements(
    interactive_elements: &[crate::page_model::InteractiveElement],
    visible_only: bool,
    roles: Option<&[ElementRole]>,
) -> Vec<crate::page_model::InteractiveElement> {
    interactive_elements
        .iter()
        .filter(|element| !visible_only || element.visible)
        .filter(|element| roles.is_none_or(|roles| roles.contains(&element.role)))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FindElementQuery {
    pub(super) summary: String,
    pub(super) description: Option<String>,
    pub(super) text: Option<String>,
    pub(super) role: Option<ElementRole>,
    pub(super) color_hint: Option<String>,
    pub(super) nearby_text: Option<String>,
    pub(super) selector_hint: Option<String>,
}

pub(super) fn build_find_element_query(
    input: &FindElementInput,
) -> Result<FindElementQuery, crate::commands::ToolError> {
    let description = normalize_optional_text(Some(input.description.clone()));
    let text = normalize_optional_text(input.text.clone());
    let color_hint = normalize_optional_text(input.color_hint.clone());
    let nearby_text = normalize_optional_text(input.nearby_text.clone());
    let selector_hint = normalize_optional_text(input.selector_hint.clone());

    if description.is_none()
        && text.is_none()
        && input.role.is_none()
        && color_hint.is_none()
        && nearby_text.is_none()
        && selector_hint.is_none()
    {
        return Err(crate::commands::ToolError {
            code: String::from("invalid_find_query"),
            message: String::from("find_element requires at least one populated search field"),
            retryable: false,
            details: None,
        });
    }

    let mut summary_parts = Vec::new();
    if let Some(description) = description.as_ref() {
        summary_parts.push(format!("description={description}"));
    }
    if let Some(text) = text.as_ref() {
        summary_parts.push(format!("text={text}"));
    }
    if let Some(role) = input.role.as_ref() {
        summary_parts.push(format!("role={role:?}"));
    }
    if let Some(color_hint) = color_hint.as_ref() {
        summary_parts.push(format!("color_hint={color_hint}"));
    }
    if let Some(nearby_text) = nearby_text.as_ref() {
        summary_parts.push(format!("nearby_text={nearby_text}"));
    }
    if let Some(selector_hint) = selector_hint.as_ref() {
        summary_parts.push(format!("selector_hint={selector_hint}"));
    }

    Ok(FindElementQuery {
        summary: summary_parts.join("; "),
        description,
        text,
        role: input.role.clone(),
        color_hint,
        nearby_text,
        selector_hint,
    })
}

pub(super) fn rank_find_element_candidates(
    elements: &[crate::page_model::InteractiveElement],
    query: &FindElementQuery,
    candidate_limit: usize,
) -> Vec<ElementCandidate> {
    let mut candidates = elements
        .iter()
        .filter_map(|element| score_interactive_element(element, query))
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .confidence_bps
            .cmp(&left.confidence_bps)
            .then_with(|| left.element_id.cmp(&right.element_id))
    });
    candidates.truncate(candidate_limit);
    candidates
}

pub(super) fn determine_find_element_resolution(
    candidates: &[ElementCandidate],
    confirmation_confidence_threshold: f32,
) -> (Option<String>, Option<f32>, bool) {
    let Some(top_candidate) = candidates.first() else {
        return (None, None, false);
    };

    let top_confidence = Some(f32::from(top_candidate.confidence_bps) / 10_000.0);
    let required_confidence_bps =
        (confirmation_confidence_threshold.clamp(0.0, 1.0) * 10_000.0).round() as u16;
    let below_threshold = top_candidate.confidence_bps < required_confidence_bps;
    let ambiguous_with_runner_up = candidates.get(1).is_some_and(|second_candidate| {
        top_candidate
            .confidence_bps
            .saturating_sub(second_candidate.confidence_bps)
            <= super::FIND_ELEMENT_AMBIGUITY_MARGIN_BPS
    });

    if below_threshold || ambiguous_with_runner_up {
        (None, top_confidence, true)
    } else {
        (
            Some(top_candidate.element_id.clone()),
            top_confidence,
            false,
        )
    }
}

pub(super) fn resolve_clickable_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a crate::page_model::InteractiveElement, ToolError> {
    let normalized_element_id = element_id.trim();
    if normalized_element_id.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_element_id"),
            message: String::from("click_element requires a non-empty deterministic element_id"),
            retryable: false,
            details: None,
        });
    }

    let Some(element) = page
        .interactive_elements
        .iter()
        .find(|element| element.element_id == normalized_element_id)
    else {
        return Err(ToolError {
            code: String::from("unknown_element_id"),
            message: String::from(
                "click_element requires an element_id that exists in the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    };

    if !element.visible {
        return Err(ToolError {
            code: String::from("element_not_visible"),
            message: String::from("click_element cannot act on an element marked not visible"),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    if !element.enabled {
        return Err(ToolError {
            code: String::from("element_disabled"),
            message: String::from("click_element cannot act on a disabled element"),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    if element
        .dom_locator
        .as_deref()
        .map(str::trim)
        .is_none_or(str::is_empty)
    {
        return Err(ToolError {
            code: String::from("missing_dom_locator"),
            message: String::from(
                "click_element requires the current page model to carry a stable dom_locator",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    Ok(element)
}

pub(super) fn resolve_typeable_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a crate::page_model::InteractiveElement, ToolError> {
    let element = resolve_clickable_element(page, element_id)?;
    if !matches!(
        element.role,
        ElementRole::Input | ElementRole::TextArea | ElementRole::Select
    ) {
        return Err(ToolError {
            code: String::from("element_not_editable"),
            message: String::from(
                "type_into_element requires an input, textarea, or select element",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": element.element_id })),
        });
    }

    Ok(element)
}

pub(super) fn resolve_form_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a crate::page_model::InteractiveElement, ToolError> {
    let element = resolve_clickable_element(page, element_id)?;
    if element.role != ElementRole::Form {
        return Err(ToolError {
            code: String::from("element_not_form"),
            message: String::from(
                "submit_active_form requires a form element from the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": element.element_id })),
        });
    }

    Ok(element)
}

#[derive(Debug, Default)]
struct FindElementScore {
    score_bps: u16,
    matched_on: Vec<String>,
    rationale_codes: Vec<String>,
}

impl FindElementScore {
    fn push_match(&mut self, match_label: &str, rationale_code: impl Into<String>, score_bps: u16) {
        self.score_bps = self.score_bps.saturating_add(score_bps);
        self.matched_on.push(match_label.to_string());
        self.rationale_codes.push(rationale_code.into());
    }
}

struct AttributeHintSpec<'a> {
    match_label: &'a str,
    exact_score_bps: u16,
    contains_score_bps: u16,
}

fn score_interactive_element(
    element: &crate::page_model::InteractiveElement,
    query: &FindElementQuery,
) -> Option<ElementCandidate> {
    let mut score = FindElementScore::default();

    if let Some(role) = query.role.as_ref() {
        if &element.role == role {
            score.push_match("role", "role_match", 1_800);
        } else {
            return None;
        }
    }

    if let Some(description) = query.description.as_ref() {
        let field_match =
            score_text_query_against_element(description, element, "description", &mut score);
        if !field_match && query.role.is_none() {
            return None;
        }
    }

    if let Some(text) = query.text.as_ref() {
        let field_match = score_text_query_against_element(text, element, "text", &mut score);
        if !field_match && query.description.is_none() && query.role.is_none() {
            return None;
        }
    }

    if let Some(nearby_text) = query.nearby_text.as_ref() {
        score_attribute_hint(
            nearby_text,
            element,
            AttributeHintSpec {
                match_label: "nearby_text",
                exact_score_bps: 1_600,
                contains_score_bps: 900,
            },
            &mut score,
        );
    }

    if let Some(selector_hint) = query.selector_hint.as_ref() {
        score_attribute_hint(
            selector_hint,
            element,
            AttributeHintSpec {
                match_label: "selector_hint",
                exact_score_bps: 1_500,
                contains_score_bps: 800,
            },
            &mut score,
        );
    }

    if let Some(color_hint) = query.color_hint.as_ref() {
        score_attribute_hint(
            color_hint,
            element,
            AttributeHintSpec {
                match_label: "color_hint",
                exact_score_bps: 500,
                contains_score_bps: 250,
            },
            &mut score,
        );
    }

    if score.score_bps == 0 {
        return None;
    }

    if element.enabled {
        score.score_bps = score.score_bps.saturating_add(100);
    } else {
        score.rationale_codes.push(String::from("disabled_penalty"));
        score.score_bps = score.score_bps.saturating_sub(300);
    }

    Some(ElementCandidate {
        element_id: element.element_id.clone(),
        confidence_bps: score.score_bps.min(10_000),
        matched_on: score.matched_on,
        rationale_codes: score.rationale_codes,
    })
}

fn score_text_query_against_element(
    query_text: &str,
    element: &crate::page_model::InteractiveElement,
    match_label: &str,
    score: &mut FindElementScore,
) -> bool {
    let normalized_query = normalize_search_text(query_text);
    let accessible_name = element
        .accessible_name
        .as_deref()
        .map(normalize_search_text);
    let visible_text = element.text.as_deref().map(normalize_search_text);
    let placeholder = element.placeholder.as_deref().map(normalize_search_text);

    if accessible_name.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "accessible_name_exact", 4_200);
        return true;
    }
    if visible_text.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "visible_text_exact", 4_000);
        return true;
    }
    if placeholder.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "placeholder_exact", 3_400);
        return true;
    }

    let overlap_score = text_overlap_score(&normalized_query, element);
    if overlap_score > 0 {
        score.push_match(match_label, "lexical_overlap", overlap_score);
        return true;
    }

    false
}

fn score_attribute_hint(
    hint: &str,
    element: &crate::page_model::InteractiveElement,
    spec: AttributeHintSpec<'_>,
    score: &mut FindElementScore,
) -> bool {
    let normalized_hint = normalize_search_text(hint);
    let attribute_blob = element
        .attributes
        .iter()
        .map(|(key, value)| format!("{key} {value}"))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized_attributes = normalize_search_text(&attribute_blob);

    if normalized_attributes.is_empty() {
        return false;
    }

    if normalized_attributes == normalized_hint {
        score.push_match(
            spec.match_label,
            format!("{}_exact", spec.match_label),
            spec.exact_score_bps,
        );
        true
    } else if normalized_attributes.contains(&normalized_hint) {
        score.push_match(
            spec.match_label,
            format!("{}_contains", spec.match_label),
            spec.contains_score_bps,
        );
        true
    } else {
        false
    }
}

fn text_overlap_score(
    query_text: &str,
    element: &crate::page_model::InteractiveElement,
) -> u16 {
    let query_terms = tokenize_search_text(query_text);
    if query_terms.is_empty() {
        return 0;
    }

    let element_blob = [
        element.accessible_name.as_deref().unwrap_or_default(),
        element.text.as_deref().unwrap_or_default(),
        element.placeholder.as_deref().unwrap_or_default(),
        element.href.as_deref().unwrap_or_default(),
        element.value.as_deref().unwrap_or_default(),
    ]
    .join(" ");
    let element_terms = tokenize_search_text(&element_blob);
    if element_terms.is_empty() {
        return 0;
    }

    let overlap = query_terms
        .iter()
        .filter(|term| element_terms.contains(*term))
        .count();
    if overlap == 0 {
        0
    } else {
        let ratio = overlap as f32 / query_terms.len() as f32;
        (900.0 + (ratio * 2_100.0)).round() as u16
    }
}

fn normalize_search_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize_search_text(value: &str) -> Vec<String> {
    normalize_search_text(value)
        .split(' ')
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
