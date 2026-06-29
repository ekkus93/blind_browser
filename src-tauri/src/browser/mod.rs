pub mod config;
pub use config::*;

mod dom_extraction;
mod element_interaction;
mod navigation;
mod page_inspection;
mod page_metrics;
mod session;

#[cfg(feature = "browser")]
use session::{snapshot_page_state, LiveBrowserSession};

#[cfg(any(feature = "browser", test))]
use std::time::Duration;

/// Reject load-wait states that are not yet implemented, rather than silently
/// degrading them. `Load`/`DomContentLoaded` map to the best available navigation
/// wait; `NetworkIdle` is explicitly unsupported until a real network-idle wait is
/// implemented (the spec forbids silently treating it as `Load`).
#[cfg(feature = "browser")]
pub(super) fn ensure_supported_load_state(load_state: LoadState) -> Result<(), BrowserError> {
    match load_state {
        LoadState::DomContentLoaded | LoadState::Load => Ok(()),
        LoadState::NetworkIdle => Err(BrowserError::Navigate(String::from(
            "NetworkIdle load waiting is not implemented yet; use Load or DomContentLoaded",
        ))),
    }
}

/// Bound a browser navigation future by `timeout_ms` (default 30s, clamped to
/// 1ms..=120s). On timeout the operation future is dropped (cancelling the await)
/// and a structured `BrowserError::Navigate` is returned. Runs inside
/// `tauri::async_runtime::block_on`, whose tokio runtime has the time driver
/// enabled, so `tokio::time::timeout` is valid here.
#[cfg(feature = "browser")]
pub(super) async fn with_browser_timeout<T, F>(
    operation: F,
    timeout_ms: Option<u64>,
    operation_name: &'static str,
) -> Result<T, BrowserError>
where
    F: std::future::Future<Output = Result<T, BrowserError>>,
{
    let timeout_ms = timeout_ms.unwrap_or(30_000).clamp(1, 120_000);
    let timeout = Duration::from_millis(timeout_ms);
    match tokio::time::timeout(timeout, operation).await {
        Ok(result) => result,
        Err(_) => Err(BrowserError::Navigate(format!(
            "{operation_name} timed out after {timeout_ms}ms"
        ))),
    }
}

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
            ensure_supported_load_state(load_state)?;
            let user_agent = self.config.user_agent.clone();
            let session = self.ensure_session()?;
            let page = session.ensure_page(user_agent.as_deref())?;

            tauri::async_runtime::block_on(async {
                with_browser_timeout(
                    async {
                        page.goto(url)
                            .await
                            .map_err(|error| BrowserError::Navigate(error.to_string()))?;
                        Ok(())
                    },
                    timeout_ms,
                    "browser navigation",
                )
                .await?;

                snapshot_page_state(&page).await
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = (url, load_state, timeout_ms);
            Err(BrowserError::FeatureDisabled)
        }
    }

    /// Switch browser visibility two-phase, so the working session is never lost on
    /// a failed switch: read the prior URL, launch a *candidate* session with the new
    /// visibility, navigate it to the prior URL, and only then commit the new config
    /// and session. If any step fails the `?` returns early with the old session and
    /// config still in place.
    pub fn switch_visibility(
        &mut self,
        mode: BrowserVisibilityMode,
    ) -> Result<Option<String>, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let prior_url = self.read_current_non_blank_url()?;

            let mut candidate_config = self.config.clone();
            candidate_config.visibility = mode;
            let mut candidate_session = LiveBrowserSession::launch(&candidate_config)?;

            if let Some(ref url) = prior_url {
                let user_agent = candidate_config.user_agent.clone();
                let page = candidate_session.ensure_page(user_agent.as_deref())?;
                tauri::async_runtime::block_on(async {
                    page.goto(url)
                        .await
                        .map_err(|error| BrowserError::Navigate(error.to_string()))
                })?;
            }

            // Candidate is proven; commit it and drop the old session only now.
            self.config = candidate_config;
            self.session = Some(candidate_session);
            Ok(prior_url)
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = mode;
            Err(BrowserError::FeatureDisabled)
        }
    }

    /// Read the current page URL, treating `about:blank`/empty as "no URL to
    /// preserve". Failures to read the URL are surfaced as `BrowserError::Inspect`
    /// rather than swallowed with `.ok().flatten()`, since the read failing changes
    /// whether the prior page is recovered after a visibility switch.
    #[cfg(feature = "browser")]
    fn read_current_non_blank_url(&self) -> Result<Option<String>, BrowserError> {
        let Some(session) = self.session.as_ref() else {
            return Ok(None);
        };
        let Some(page) = session.page.clone() else {
            return Ok(None);
        };

        tauri::async_runtime::block_on(async {
            page.url()
                .await
                .map_err(|error| BrowserError::Inspect(error.to_string()))
                .map(|url| url.filter(|url| url != "about:blank" && !url.is_empty()))
        })
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

#[cfg(all(test, feature = "browser"))]
mod navigation_policy_tests {
    use super::*;

    #[test]
    fn ensure_supported_load_state_accepts_dom_and_load_rejects_network_idle() {
        assert!(ensure_supported_load_state(LoadState::Load).is_ok());
        assert!(ensure_supported_load_state(LoadState::DomContentLoaded).is_ok());

        let error = ensure_supported_load_state(LoadState::NetworkIdle)
            .expect_err("NetworkIdle must be rejected as unsupported, not silently degraded");
        match error {
            BrowserError::Navigate(message) => assert!(message.contains("NetworkIdle")),
            other => panic!("expected an unsupported-state navigate error, got {other:?}"),
        }
    }

    #[test]
    fn with_browser_timeout_returns_structured_error_on_timeout() {
        let result: Result<(), BrowserError> = tauri::async_runtime::block_on(async {
            with_browser_timeout(
                async {
                    tokio::time::sleep(Duration::from_secs(3_600)).await;
                    Ok(())
                },
                Some(1),
                "test navigation",
            )
            .await
        });

        match result.expect_err("a slow operation must time out") {
            BrowserError::Navigate(message) => {
                assert!(
                    message.contains("timed out"),
                    "unexpected message: {message}"
                );
                assert!(message.contains("test navigation"));
            }
            other => panic!("expected a navigate timeout error, got {other:?}"),
        }
    }

    #[test]
    fn with_browser_timeout_passes_through_fast_success() {
        let result: Result<u8, BrowserError> = tauri::async_runtime::block_on(async {
            with_browser_timeout(async { Ok(7u8) }, Some(120_000), "test navigation").await
        });
        assert_eq!(result.expect("fast operations should pass through"), 7);
    }
}
