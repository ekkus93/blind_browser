use crate::commands::{
    ClickElementData, ClickElementInput, FindElementData, FindElementInput, FocusElementData,
    FocusElementInput, ListInteractiveElementsData, ListInteractiveElementsInput,
    SubmitActiveFormData, SubmitActiveFormInput, ToolError, ToolName, ToolResult,
    TypeIntoElementData, TypeIntoElementInput,
};
use crate::page_model::{ElementRole, PageModel};

use super::element_scoring::{
    build_find_element_query, determine_find_element_resolution, filter_interactive_elements,
    rank_find_element_candidates,
};

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

pub(crate) fn resolve_clickable_element<'a>(
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

pub(crate) fn resolve_typeable_element<'a>(
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

pub(crate) fn resolve_form_element<'a>(
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
