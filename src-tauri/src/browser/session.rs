#[cfg(feature = "browser")]
use chromiumoxide::cdp::browser_protocol::page::GetNavigationHistoryParams;
#[cfg(feature = "browser")]
use chromiumoxide::cdp::browser_protocol::page::NavigationEntry;
#[cfg(feature = "browser")]
use chromiumoxide::{Browser, BrowserConfig, Page};
#[cfg(feature = "browser")]
use futures::StreamExt;

use super::{BrowserError, BrowserPageState, BrowserSessionConfig, BrowserVisibilityMode};
use crate::page_model::InteractiveElement;

#[cfg(feature = "browser")]
pub(super) struct LiveBrowserSession {
    pub(super) browser: Browser,
    pub(super) page: Option<Page>,
}

#[cfg(feature = "browser")]
impl LiveBrowserSession {
    pub(super) fn launch(config: &BrowserSessionConfig) -> Result<Self, BrowserError> {
        let browser_config = build_browser_config(config)?;
        let (browser, mut handler) = tauri::async_runtime::block_on(async {
            Browser::launch(browser_config)
                .await
                .map_err(|error| BrowserError::Launch(error.to_string()))
        })?;

        tauri::async_runtime::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(error) = event {
                    tracing::error!(error = %error, "chromium handler stopped");
                    break;
                }
            }
        });

        Ok(Self {
            browser,
            page: None,
        })
    }

    pub(super) fn ensure_page(&mut self, user_agent: Option<&str>) -> Result<Page, BrowserError> {
        if let Some(page) = self.page.clone() {
            return Ok(page);
        }

        let page = tauri::async_runtime::block_on(async {
            let page = self
                .browser
                .new_page("about:blank")
                .await
                .map_err(|error| BrowserError::CreatePage(error.to_string()))?;

            if let Some(user_agent) = user_agent.filter(|value| !value.trim().is_empty()) {
                page.set_user_agent(user_agent)
                    .await
                    .map_err(|error| BrowserError::CreatePage(error.to_string()))?;
            }

            Ok::<Page, BrowserError>(page)
        })?;

        self.page = Some(page.clone());
        Ok(page)
    }
}

#[cfg(feature = "browser")]
pub(super) fn build_browser_config(
    config: &BrowserSessionConfig,
) -> Result<BrowserConfig, BrowserError> {
    let mut builder = BrowserConfig::builder();
    if matches!(config.visibility, BrowserVisibilityMode::Visible) {
        builder = builder.with_head();
    }

    builder.build().map_err(BrowserError::Launch)
}

#[cfg(feature = "browser")]
pub(super) async fn snapshot_page_state(page: &Page) -> Result<BrowserPageState, BrowserError> {
    let url = page
        .url()
        .await
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .unwrap_or_else(|| String::from("about:blank"));
    let title = page
        .evaluate("document.title || null")
        .await
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .into_value::<Option<String>>()
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .and_then(|value| normalize_optional_text(&value));

    Ok(BrowserPageState {
        url,
        title,
        history: read_browser_history(page).await?,
    })
}

#[cfg(feature = "browser")]
pub(super) fn stable_dom_selector(element: &InteractiveElement) -> Result<&str, BrowserError> {
    element
        .dom_locator
        .as_deref()
        .map(str::trim)
        .filter(|locator| !locator.is_empty())
        .ok_or_else(|| BrowserError::MissingDomLocator {
            element_id: element.element_id.clone(),
        })
}

#[cfg(feature = "browser")]
pub(super) fn ensure_live_element(
    page: &Page,
    element: &InteractiveElement,
    selector: &str,
) -> Result<(), BrowserError> {
    tauri::async_runtime::block_on(async {
        page.find_element(selector)
            .await
            .map(|_| ())
            .map_err(|_| BrowserError::ElementNotFound {
                element_id: element.element_id.clone(),
                locator: selector.to_string(),
            })
    })
}

#[cfg(feature = "browser")]
pub(super) async fn read_browser_history(
    page: &Page,
) -> Result<crate::state::BrowserHistoryState, BrowserError> {
    Ok(read_navigation_history_snapshot(page).await?.history)
}

#[cfg(feature = "browser")]
pub(super) async fn read_navigation_history_snapshot(
    page: &Page,
) -> Result<LiveNavigationHistorySnapshot, BrowserError> {
    let history = page
        .execute(GetNavigationHistoryParams::default())
        .await
        .map_err(|error| BrowserError::History(error.to_string()))?
        .result;
    let current_index = isize::try_from(history.current_index).ok();
    let entry_count = history.entries.len();

    Ok(LiveNavigationHistorySnapshot {
        current_index,
        history: crate::state::BrowserHistoryState {
            can_go_back: current_index.is_some_and(|index| index > 0),
            can_go_forward: current_index.is_some_and(|index| (index as usize) + 1 < entry_count),
            current_entry_index: current_index.and_then(|index| usize::try_from(index).ok()),
            entry_count,
        },
        entries: history.entries,
    })
}

#[cfg(feature = "browser")]
pub(super) struct LiveNavigationHistorySnapshot {
    pub(super) current_index: Option<isize>,
    pub(super) history: crate::state::BrowserHistoryState,
    pub(super) entries: Vec<NavigationEntry>,
}

#[cfg(any(feature = "browser", test))]
pub(super) fn normalize_optional_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(feature = "browser")]
#[derive(Debug, serde::Deserialize)]
pub(super) struct LiveFocusResult {
    pub(super) found: bool,
    pub(super) focused: bool,
}

#[cfg(feature = "browser")]
#[derive(Debug, serde::Deserialize)]
pub(super) struct LiveTypeResult {
    pub(super) found: bool,
    pub(super) accepted_input: bool,
    pub(super) value_after: Option<String>,
}

#[cfg(feature = "browser")]
#[derive(Debug, serde::Deserialize)]
pub(super) struct LiveSubmitResult {
    pub(super) found: bool,
    pub(super) ambiguous: bool,
    pub(super) submitted: bool,
}
