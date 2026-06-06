#[cfg(feature = "browser")]
use chromiumoxide::cdp::browser_protocol::page::{NavigateToHistoryEntryParams, ReloadParams};

use super::session::{normalize_optional_text, read_navigation_history_snapshot, snapshot_page_state};
use super::{BrowserController, BrowserError, BrowserNavigationState, BrowserPageState, LoadState};

impl BrowserController {
    pub fn go_back(
        &mut self,
        steps: u8,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserNavigationState, BrowserError> {
        self.navigate_history(-(steps as isize), load_state, timeout_ms)
    }

    pub fn go_forward(
        &mut self,
        steps: u8,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserNavigationState, BrowserError> {
        self.navigate_history(steps as isize, load_state, timeout_ms)
    }

    pub fn reload_page(
        &mut self,
        hard_reload: bool,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserPageState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            tauri::async_runtime::block_on(async {
                if hard_reload {
                    page.execute(ReloadParams::builder().ignore_cache(true).build())
                        .await
                        .map_err(|error| BrowserError::Reload(error.to_string()))?;
                    page.wait_for_navigation()
                        .await
                        .map_err(|error| BrowserError::Reload(error.to_string()))?;
                } else {
                    page.reload()
                        .await
                        .map_err(|error| BrowserError::Reload(error.to_string()))?;
                }

                let _ = load_state;
                let _ = timeout_ms;
                snapshot_page_state(&page).await
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = hard_reload;
            let _ = load_state;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    fn navigate_history(
        &mut self,
        delta: isize,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserNavigationState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            tauri::async_runtime::block_on(async {
                let history_snapshot = read_navigation_history_snapshot(&page).await?;
                let Some(current_index) = history_snapshot.current_index else {
                    return Ok(BrowserNavigationState {
                        navigated: false,
                        url: None,
                        title: None,
                        history: history_snapshot.history,
                    });
                };

                let target_index = current_index + delta;
                if target_index < 0 || target_index >= history_snapshot.entries.len() as isize {
                    let current_entry = history_snapshot.entries.get(current_index as usize);
                    return Ok(BrowserNavigationState {
                        navigated: false,
                        url: current_entry.map(|entry| entry.url.clone()),
                        title: current_entry
                            .and_then(|entry| normalize_optional_text(&entry.title)),
                        history: history_snapshot.history,
                    });
                }

                let target_entry = &history_snapshot.entries[target_index as usize];
                page.execute(NavigateToHistoryEntryParams::new(target_entry.id))
                    .await
                    .map_err(|error| BrowserError::Navigate(error.to_string()))?;
                page.wait_for_navigation()
                    .await
                    .map_err(|error| BrowserError::Navigate(error.to_string()))?;
                let _ = load_state;
                let _ = timeout_ms;
                let current_page = snapshot_page_state(&page).await?;

                Ok(BrowserNavigationState {
                    navigated: true,
                    url: Some(current_page.url),
                    title: current_page.title,
                    history: current_page.history,
                })
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = delta;
            let _ = load_state;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }
}
