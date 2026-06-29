use crate::commands::{
    ClickElementData, ClickElementInput, FocusElementData, FocusElementInput, ToolError, ToolName,
    ToolResult,
};
use crate::page_model::{InteractiveElement, PageModel};

impl super::super::AppCore {
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
}

pub(crate) fn resolve_clickable_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a InteractiveElement, ToolError> {
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
