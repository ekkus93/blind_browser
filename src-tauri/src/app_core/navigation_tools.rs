use super::RecentFieldContext;
use crate::browser::{BrowserError, LoadState};
use crate::commands::{
    GoBackData, GoBackInput, GoForwardData, GoForwardInput, OpenUrlData, OpenUrlInput,
    ReloadPageData, ReloadPageInput, ToolError, ToolName, ToolResult,
};
use crate::page_model::PageModel;
use crate::state::AppState;

/// Normalize a planner/user-supplied browser navigation URL, failing closed to
/// `http`/`https` via the shared [`crate::url_policy`] so the validator and runtime
/// paths cannot drift. The name is kept for existing call sites; the behavior is now
/// "normalize an allowed web navigation URL," not "any absolute URL."
pub(crate) fn normalize_absolute_url(url: &str) -> Result<String, ToolError> {
    crate::url_policy::normalize_browser_navigation_url(url).map_err(|error| ToolError {
        code: String::from("invalid_url"),
        message: String::from(error.user_message()),
        retryable: false,
        details: Some(error.details()),
    })
}

pub(crate) fn refresh_current_page_after_navigation(
    current_page: &mut Option<PageModel>,
    url: Option<String>,
    title: Option<String>,
) {
    if let Some(current_page) = current_page.as_mut() {
        current_page.url = url;
        current_page.title = title;
        current_page.regions.clear();
        current_page.interactive_elements.clear();
    }
}

pub(crate) fn clear_navigation_follow_up_state(
    state: &mut AppState,
    recent_field_context: &mut Option<RecentFieldContext>,
) {
    state.narration_cursor = Default::default();
    *recent_field_context = None;
}

pub(crate) fn browser_error_to_tool_error(message: String, error: BrowserError) -> ToolError {
    let code = match &error {
        BrowserError::FeatureDisabled => "browser_feature_disabled",
        BrowserError::Launch(_) => "browser_launch_failed",
        BrowserError::CreatePage(_) => "browser_page_creation_failed",
        BrowserError::Navigate(_) => "browser_navigation_failed",
        BrowserError::Inspect(_) => "browser_state_read_failed",
        BrowserError::NoActivePage => "browser_no_active_page",
        BrowserError::MissingDomLocator { .. } => "missing_dom_locator",
        BrowserError::Resolve(_) => "browser_element_resolution_failed",
        BrowserError::ElementNotFound { .. } => "browser_element_not_found",
        BrowserError::Click(_) => "browser_click_failed",
        BrowserError::Focus(_) => "browser_focus_failed",
        BrowserError::Type(_) => "browser_type_failed",
        BrowserError::Submit(_) => "browser_submit_failed",
        BrowserError::History(_) => "browser_history_failed",
        BrowserError::Reload(_) => "browser_reload_failed",
        BrowserError::Eval(_) => "browser_eval_failed",
        BrowserError::Scroll(_) => "browser_scroll_failed",
        BrowserError::Screenshot(_) => "browser_screenshot_failed",
    };

    ToolError {
        code: String::from(code),
        message,
        retryable: matches!(
            error,
            BrowserError::Launch(_)
                | BrowserError::CreatePage(_)
                | BrowserError::Navigate(_)
                | BrowserError::Inspect(_)
                | BrowserError::History(_)
                | BrowserError::Reload(_)
                | BrowserError::Eval(_)
                | BrowserError::Scroll(_)
                | BrowserError::Screenshot(_)
        ),
        details: Some(serde_json::json!({ "reason": error.to_string() })),
    }
}

impl super::AppCore {
    pub fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        let final_url = match normalize_absolute_url(&input.url) {
            Ok(url) => url,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::OpenUrl,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Navigation request was rejected because the URL was not an absolute URL.",
                    )],
                )
            }
        };

        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_page = match self
            .browser
            .open_url(&final_url, load_state, input.timeout_ms)
        {
            Ok(browser_page) => browser_page,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::OpenUrl,
                    input.request_id,
                    String::from("Browser navigation did not complete successfully."),
                    error,
                )
            }
        };

        let page_id = self.next_page_id(&input.request_id);
        self.stop_narration_playback();
        self.state
            .record_navigation(page_id.clone(), browser_page.url.clone());
        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.title = browser_page.title.clone();
        }
        self.state.browser_history = browser_page.history.clone();
        self.clear_recent_field_context();

        ToolResult::success(
            ToolName::OpenUrl,
            input.request_id,
            OpenUrlData {
                final_url: browser_page.url,
                title: browser_page.title,
                page_id,
                load_state,
                http_status: None,
                history: browser_page.history,
            },
            vec![
                String::from(
                    "Validated the requested absolute URL and navigated the live Chromium page.",
                ),
                String::from(
                    "Runtime navigation state now reflects the live browser URL and document title.",
                ),
            ],
        )
    }

    pub fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::GoBack, input.request_id);
        }

        let requested_steps = input.steps.unwrap_or(1).clamp(1, super::MAX_HISTORY_STEPS);
        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_navigation =
            match self
                .browser
                .go_back(requested_steps, load_state, input.timeout_ms)
            {
                Ok(browser_navigation) => browser_navigation,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::GoBack,
                        input.request_id,
                        String::from(
                            "Live browser history navigation did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

        self.state.browser_history = browser_navigation.history.clone();
        if browser_navigation.navigated {
            self.stop_narration_playback();
            refresh_current_page_after_navigation(
                &mut self.state.current_page,
                browser_navigation.url.clone(),
                browser_navigation.title.clone(),
            );
            clear_navigation_follow_up_state(&mut self.state, &mut self.recent_field_context);
        }

        let mut observations = vec![format!(
            "Requested backward history navigation for up to {} step(s).",
            requested_steps
        )];
        if input
            .steps
            .is_some_and(|steps| steps > super::MAX_HISTORY_STEPS)
        {
            observations.push(format!(
                "Requested steps were clamped to the supported maximum of {}.",
                super::MAX_HISTORY_STEPS
            ));
        }
        observations.push(if browser_navigation.navigated {
            String::from("The live browser moved backward in history and runtime page metadata was refreshed.")
        } else {
            String::from("The live browser was already at the earliest reachable history entry.")
        });

        ToolResult::success(
            ToolName::GoBack,
            input.request_id,
            GoBackData {
                navigated: browser_navigation.navigated,
                actual_steps: if browser_navigation.navigated {
                    requested_steps
                } else {
                    0
                },
                final_url: browser_navigation.url,
                title: browser_navigation.title,
                load_state: browser_navigation.navigated.then_some(load_state),
                history: browser_navigation.history,
            },
            observations,
        )
    }

    pub fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::GoForward, input.request_id);
        }

        let requested_steps = input.steps.unwrap_or(1).clamp(1, super::MAX_HISTORY_STEPS);
        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_navigation =
            match self
                .browser
                .go_forward(requested_steps, load_state, input.timeout_ms)
            {
                Ok(browser_navigation) => browser_navigation,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::GoForward,
                        input.request_id,
                        String::from(
                            "Live browser forward navigation did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

        self.state.browser_history = browser_navigation.history.clone();
        if browser_navigation.navigated {
            self.stop_narration_playback();
            refresh_current_page_after_navigation(
                &mut self.state.current_page,
                browser_navigation.url.clone(),
                browser_navigation.title.clone(),
            );
            clear_navigation_follow_up_state(&mut self.state, &mut self.recent_field_context);
        }

        let mut observations = vec![format!(
            "Requested forward history navigation for up to {} step(s).",
            requested_steps
        )];
        if input
            .steps
            .is_some_and(|steps| steps > super::MAX_HISTORY_STEPS)
        {
            observations.push(format!(
                "Requested steps were clamped to the supported maximum of {}.",
                super::MAX_HISTORY_STEPS
            ));
        }
        observations.push(if browser_navigation.navigated {
            String::from("The live browser moved forward in history and runtime page metadata was refreshed.")
        } else {
            String::from("The live browser was already at the latest reachable history entry.")
        });

        ToolResult::success(
            ToolName::GoForward,
            input.request_id,
            GoForwardData {
                navigated: browser_navigation.navigated,
                actual_steps: if browser_navigation.navigated {
                    requested_steps
                } else {
                    0
                },
                final_url: browser_navigation.url,
                title: browser_navigation.title,
                load_state: browser_navigation.navigated.then_some(load_state),
                history: browser_navigation.history,
            },
            observations,
        )
    }

    pub fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::ReloadPage, input.request_id);
        }

        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_page = match self.browser.reload_page(
            input.mode.uses_cache_bypass(),
            load_state,
            input.timeout_ms,
        ) {
            Ok(browser_page) => browser_page,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::ReloadPage,
                    input.request_id,
                    String::from("Live browser reload did not complete successfully."),
                    error,
                )
            }
        };

        self.state.browser_history = browser_page.history.clone();
        self.stop_narration_playback();
        refresh_current_page_after_navigation(
            &mut self.state.current_page,
            Some(browser_page.url.clone()),
            browser_page.title.clone(),
        );
        clear_navigation_follow_up_state(&mut self.state, &mut self.recent_field_context);

        let mut observations = vec![String::from(
            "Reloaded the live browser page and refreshed runtime page metadata.",
        )];
        if input.mode.uses_cache_bypass() {
            observations.push(String::from(
                "The reload ignored browser cache as requested.",
            ));
        }

        ToolResult::success(
            ToolName::ReloadPage,
            input.request_id,
            ReloadPageData {
                reloaded: true,
                final_url: browser_page.url,
                title: browser_page.title,
                load_state,
                http_status: None,
                history: browser_page.history,
            },
            observations,
        )
    }
}
