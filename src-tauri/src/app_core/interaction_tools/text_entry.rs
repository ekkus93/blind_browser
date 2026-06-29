use super::click_focus::resolve_clickable_element;
use crate::commands::{
    SubmitActiveFormData, SubmitActiveFormInput, ToolError, ToolName, ToolResult,
    TypeIntoElementData, TypeIntoElementInput,
};
use crate::page_model::{ElementRole, InteractiveElement, PageModel};

impl super::super::AppCore {
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

pub(crate) fn resolve_typeable_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a InteractiveElement, ToolError> {
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
) -> Result<&'a InteractiveElement, ToolError> {
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
