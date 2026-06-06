pub mod config;
pub use config::*;

mod session;
mod navigation;
mod page_metrics;
mod dom_extraction;
mod element_interaction;
mod page_inspection;

#[cfg(feature = "browser")]
use session::{LiveBrowserSession, snapshot_page_state};

#[cfg(any(feature = "browser", test))]
use std::time::Duration;


pub struct BrowserController {
    #[cfg(feature = "browser")]
    config: BrowserSessionConfig,
    #[cfg(feature = "browser")]
    session: Option<LiveBrowserSession>,
}

impl BrowserController {
    pub fn new(config: BrowserSessionConfig) -> Self {
        #[cfg(not(feature = "browser"))]
        let _ = config;

        Self {
            #[cfg(feature = "browser")]
            config,
            #[cfg(feature = "browser")]
            session: None,
        }
    }

    pub fn open_url(
        &mut self,
        url: &str,
        load_state: LoadState,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserPageState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let _ = load_state;
            let _ = timeout_ms;
            let user_agent = self.config.user_agent.clone();
            let session = self.ensure_session()?;
            let page = session.ensure_page(user_agent.as_deref())?;

            tauri::async_runtime::block_on(async {
                page.goto(url)
                    .await
                    .map_err(|error| BrowserError::Navigate(error.to_string()))?;

                snapshot_page_state(&page).await
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = url;
            let _ = load_state;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn switch_visibility(
        &mut self,
        mode: BrowserVisibilityMode,
    ) -> Result<Option<String>, BrowserError> {
        #[cfg(feature = "browser")]
        {
            self.config.visibility = mode;

            let prior_url = self
                .session
                .as_ref()
                .and_then(|session| session.page.clone())
                .and_then(|page| {
                    tauri::async_runtime::block_on(async {
                        page.url()
                            .await
                            .ok()
                            .flatten()
                            .filter(|url| url != "about:blank" && !url.is_empty())
                    })
                });

            self.session = None;

            if let Some(ref url) = prior_url {
                let user_agent = self.config.user_agent.clone();
                let session = self.ensure_session()?;
                let page = session.ensure_page(user_agent.as_deref())?;
                tauri::async_runtime::block_on(async {
                    page.goto(url)
                        .await
                        .map_err(|error| BrowserError::Navigate(error.to_string()))
                })?;
            }

            Ok(prior_url)
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = mode;
            Err(BrowserError::FeatureDisabled)
        }
    }

    #[cfg(feature = "browser")]
    fn ensure_session(&mut self) -> Result<&mut LiveBrowserSession, BrowserError> {
        if self.session.is_none() {
            self.session = Some(LiveBrowserSession::launch(&self.config)?);
        }

        self.session.as_mut().ok_or_else(|| {
            BrowserError::Launch(String::from("chromium session was not initialized"))
        })
    }
}

#[cfg(feature = "browser")]
pub(super) fn wait_for_page_settle(timeout_ms: Option<u64>) {
    let wait_ms = timeout_ms.unwrap_or(400).clamp(150, 2_000);
    std::thread::sleep(Duration::from_millis(wait_ms));
}

