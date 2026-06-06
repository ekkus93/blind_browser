use super::*;

pub(super) fn format_browser_visibility_mode(mode: BrowserVisibilityMode) -> String {
    match mode {
        BrowserVisibilityMode::Visible => String::from("visible"),
        BrowserVisibilityMode::Headless => String::from("headless"),
    }
}
pub(super) fn current_page_label(agent_state: &AgentStateData) -> String {
    normalized_optional_text(agent_state.title.as_deref())
        .or_else(|| normalized_optional_text(agent_state.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

pub(super) fn format_runtime_status_summary(runtime_status: &GetRuntimeStatusData) -> String {
    let page_summary = current_page_label_from_runtime_status(runtime_status);
    let browser_mode = format_browser_visibility_mode(runtime_status.browser_visibility);
    let listening = if runtime_status.listening_state.is_listening {
        "on"
    } else {
        "off"
    };
    let speaking = if runtime_status.speaking {
        "active"
    } else {
        "idle"
    };
    let back = if runtime_status.browser_history.can_go_back {
        "available"
    } else {
        "unavailable"
    };
    let forward = if runtime_status.browser_history.can_go_forward {
        "available"
    } else {
        "unavailable"
    };

    format!(
        "Current page is {page_summary}. Browser mode is {browser_mode}. Listening is {listening}. Speech output is {speaking}. Back is {back}. Forward is {forward}."
    )
}

pub(super) fn current_page_label_from_runtime_status(runtime_status: &GetRuntimeStatusData) -> String {
    normalized_optional_text(runtime_status.title.as_deref())
        .or_else(|| normalized_optional_text(runtime_status.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

pub(super) fn format_back_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_back {
        String::from("Back navigation is available.")
    } else {
        String::from("Back navigation is not available.")
    }
}

pub(super) fn format_forward_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_forward {
        String::from("Forward navigation is available.")
    } else {
        String::from("Forward navigation is not available.")
    }
}

pub(super) fn format_listening_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.listening_state.is_listening {
        String::from("Listening is on.")
    } else {
        String::from("Listening is off.")
    }
}

pub(super) fn format_speaking_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.speaking {
        String::from("Speech output is active.")
    } else {
        String::from("Speech output is idle.")
    }
}

pub(super) fn format_browser_mode_summary(runtime_status: &GetRuntimeStatusData) -> String {
    format!(
        "Browser mode is {}.",
        format_browser_visibility_mode(runtime_status.browser_visibility)
    )
}
