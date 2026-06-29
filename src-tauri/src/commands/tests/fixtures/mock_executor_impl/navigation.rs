use super::*;

pub(super) fn execute_open_url(
    ex: &mut MockExecutor,
    input: OpenUrlInput,
) -> ToolResult<OpenUrlData> {
    ex.last_open_url = Some(input.url.clone());
    ToolResult::success(
        ToolName::OpenUrl,
        input.request_id,
        OpenUrlData {
            final_url: input.url,
            title: None,
            page_id: String::from("page-1"),
            load_state: input.wait_for_load_state.unwrap_or(LoadState::Load),
            http_status: None,
            history: BrowserHistoryState {
                can_go_back: false,
                can_go_forward: false,
                current_entry_index: Some(0),
                entry_count: 1,
            },
        },
        vec![String::from("opened url")],
    )
}

pub(super) fn execute_go_back(ex: &mut MockExecutor, input: GoBackInput) -> ToolResult<GoBackData> {
    ex.last_go_back_request = Some(input.clone());
    ToolResult::success(
        ToolName::GoBack,
        input.request_id,
        GoBackData {
            navigated: true,
            actual_steps: input.steps.unwrap_or(1),
            final_url: Some(String::from("https://example.com/previous")),
            title: Some(String::from("Previous page")),
            load_state: Some(input.wait_for_load_state.unwrap_or(LoadState::Load)),
            history: BrowserHistoryState {
                can_go_back: false,
                can_go_forward: true,
                current_entry_index: Some(0),
                entry_count: 2,
            },
        },
        vec![String::from("went back in history")],
    )
}

pub(super) fn execute_go_forward(
    ex: &mut MockExecutor,
    input: GoForwardInput,
) -> ToolResult<GoForwardData> {
    ex.last_go_forward_request = Some(input.clone());
    ToolResult::success(
        ToolName::GoForward,
        input.request_id,
        GoForwardData {
            navigated: true,
            actual_steps: input.steps.unwrap_or(1),
            final_url: Some(String::from("https://example.com/next")),
            title: Some(String::from("Next page")),
            load_state: Some(input.wait_for_load_state.unwrap_or(LoadState::Load)),
            history: BrowserHistoryState {
                can_go_back: true,
                can_go_forward: false,
                current_entry_index: Some(1),
                entry_count: 2,
            },
        },
        vec![String::from("went forward in history")],
    )
}

pub(super) fn execute_reload_page(
    ex: &mut MockExecutor,
    input: ReloadPageInput,
) -> ToolResult<ReloadPageData> {
    ex.last_reload_request = Some(input.clone());
    ToolResult::success(
        ToolName::ReloadPage,
        input.request_id,
        ReloadPageData {
            reloaded: true,
            final_url: String::from("https://example.com/current"),
            title: Some(String::from("Current page")),
            load_state: input.wait_for_load_state.unwrap_or(LoadState::Load),
            http_status: None,
            history: BrowserHistoryState {
                can_go_back: true,
                can_go_forward: false,
                current_entry_index: Some(1),
                entry_count: 2,
            },
        },
        vec![String::from("reloaded the page")],
    )
}

pub(super) fn execute_get_html(
    ex: &mut MockExecutor,
    input: GetHtmlInput,
) -> ToolResult<GetHtmlData> {
    ex.last_get_html_request = Some(input.clone());
    let html = String::from("<html><body><main>Example article</main></body></html>");
    let html_length = html.len();
    ToolResult::success(
        ToolName::GetHtml,
        input.request_id,
        GetHtmlData {
            page_id: String::from("page-1"),
            url: String::from("https://example.com/article"),
            title: Some(String::from("Example article")),
            html,
            html_length,
        },
        vec![String::from("read the current page HTML")],
    )
}

pub(super) fn execute_eval_js(ex: &mut MockExecutor, input: EvalJsInput) -> ToolResult<EvalJsData> {
    ex.last_eval_js_request = Some(input.clone());
    ToolResult::success(
        ToolName::EvalJs,
        input.request_id,
        EvalJsData {
            page_id: String::from("page-1"),
            url: String::from("https://example.com/article"),
            title: Some(String::from("Example article")),
            result: serde_json::json!({
                "headline": "Example article",
                "regionCount": 3
            }),
        },
        vec![String::from(
            "evaluated the requested JavaScript expression",
        )],
    )
}

pub(super) fn execute_scroll_page(
    ex: &mut MockExecutor,
    input: ScrollPageInput,
) -> ToolResult<ScrollPageData> {
    ex.last_scroll_request = Some(input.clone());
    ToolResult::success(
        ToolName::ScrollPage,
        input.request_id,
        ScrollPageData {
            previous_scroll_y: 120.0,
            current_scroll_y: 640.0,
            reached_boundary: false,
        },
        vec![String::from("scrolled the page")],
    )
}
