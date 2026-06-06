#[cfg(feature = "browser")]
use chromiumoxide::Page;
use serde::Deserialize;

use super::{
    BrowserController, BrowserError, BrowserPageMetrics, BrowserScrollState, ScrollDirection,
    ScrollTarget,
};

impl BrowserController {
    pub fn get_page_metrics(&mut self) -> Result<BrowserPageMetrics, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            tauri::async_runtime::block_on(read_page_metrics(&page))
        }

        #[cfg(not(feature = "browser"))]
        {
            Err(BrowserError::FeatureDisabled)
        }
    }

    pub fn scroll_page(
        &mut self,
        direction: ScrollDirection,
        amount_px: Option<f32>,
        target: Option<ScrollTarget>,
        timeout_ms: Option<u64>,
    ) -> Result<BrowserScrollState, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let page = self
                .ensure_session()?
                .page
                .clone()
                .ok_or(BrowserError::NoActivePage)?;

            tauri::async_runtime::block_on(async {
                let previous_scroll_y = current_scroll_y(&page).await?;
                let scroll_instruction =
                    build_scroll_instruction(direction, amount_px, target, &page).await?;
                let result = page
                    .evaluate_expression(scroll_instruction)
                    .await
                    .map_err(|error| BrowserError::Scroll(error.to_string()))?
                    .into_value::<LiveScrollResult>()
                    .map_err(|error| BrowserError::Scroll(error.to_string()))?;
                super::wait_for_page_settle(timeout_ms);

                Ok(BrowserScrollState {
                    previous_scroll_y,
                    current_scroll_y: result.current_scroll_y,
                    reached_boundary: result.reached_boundary,
                })
            })
        }

        #[cfg(not(feature = "browser"))]
        {
            let _ = direction;
            let _ = amount_px;
            let _ = target;
            let _ = timeout_ms;
            Err(BrowserError::FeatureDisabled)
        }
    }
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LivePageMetrics {
    scroll_y: f64,
    viewport_width: f64,
    viewport_height: f64,
    document_height: f64,
}

#[cfg(feature = "browser")]
async fn current_scroll_y(page: &Page) -> Result<f32, BrowserError> {
    let scroll_y = page
        .evaluate_expression("window.scrollY")
        .await
        .map_err(|error| BrowserError::Scroll(error.to_string()))?
        .into_value::<f64>()
        .map_err(|error| BrowserError::Scroll(error.to_string()))?;
    Ok(scroll_y as f32)
}

#[cfg(feature = "browser")]
async fn read_page_metrics(page: &Page) -> Result<BrowserPageMetrics, BrowserError> {
    let live_metrics = page
        .evaluate(
            r#"(() => {
                const doc = document.documentElement;
                const body = document.body;
                const viewportWidth = window.innerWidth || doc?.clientWidth || 0;
                const viewportHeight = window.innerHeight || doc?.clientHeight || 0;
                const documentHeight = Math.max(
                    doc?.scrollHeight || 0,
                    body?.scrollHeight || 0,
                    doc?.clientHeight || 0,
                    body?.clientHeight || 0
                );
                return {
                    scroll_y: window.scrollY || 0,
                    viewport_width: viewportWidth,
                    viewport_height: viewportHeight,
                    document_height: documentHeight
                };
            })()"#,
        )
        .await
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .into_value::<LivePageMetrics>()
        .map_err(|error| BrowserError::Inspect(error.to_string()))?;
    Ok(normalize_page_metrics(live_metrics))
}

#[cfg(feature = "browser")]
fn normalize_page_metrics(metrics: LivePageMetrics) -> BrowserPageMetrics {
    fn clean(value: f64) -> f32 {
        if value.is_finite() && value >= 0.0 {
            value as f32
        } else {
            0.0
        }
    }

    BrowserPageMetrics {
        scroll_y: clean(metrics.scroll_y),
        viewport_width: clean(metrics.viewport_width),
        viewport_height: clean(metrics.viewport_height),
        document_height: clean(metrics.document_height),
    }
}

#[cfg(feature = "browser")]
async fn build_scroll_instruction(
    direction: ScrollDirection,
    amount_px: Option<f32>,
    target: Option<ScrollTarget>,
    page: &Page,
) -> Result<String, BrowserError> {
    let metrics = page
        .layout_metrics()
        .await
        .map_err(|error| BrowserError::Scroll(error.to_string()))?;
    let viewport_height = metrics.css_layout_viewport.client_height as f32;
    let default_amount = (viewport_height * 0.85).max(200.0);
    let requested_amount = amount_px.unwrap_or(default_amount).max(0.0);
    let axis = match direction {
        ScrollDirection::Up | ScrollDirection::Down => "y",
        ScrollDirection::Left | ScrollDirection::Right => "x",
    };
    let signed_amount = match direction {
        ScrollDirection::Up | ScrollDirection::Left => -requested_amount,
        ScrollDirection::Down | ScrollDirection::Right => requested_amount,
    };
    let target_expression = match target {
        Some(ScrollTarget::Top) => "window.scrollTo({ top: 0, behavior: 'auto' });".to_string(),
        Some(ScrollTarget::Bottom) => {
            "window.scrollTo({ top: document.documentElement.scrollHeight, behavior: 'auto' });"
                .to_string()
        }
        Some(ScrollTarget::NextSection) => {
            "window.scrollBy({ top: window.innerHeight * 0.9, behavior: 'auto' });".to_string()
        }
        Some(ScrollTarget::PreviousSection) => {
            "window.scrollBy({ top: -window.innerHeight * 0.9, behavior: 'auto' });".to_string()
        }
        None => format!(
            "window.scrollBy({{ {}: {}, behavior: 'auto' }});",
            if axis == "y" { "top" } else { "left" },
            signed_amount
        ),
    };

    Ok(format!(
        "(() => {{\
            const previousY = window.scrollY;\
            {target_expression}\
            const maxY = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);\
            return {{\
                current_scroll_y: window.scrollY,\
                reached_boundary: window.scrollY <= 0 || window.scrollY >= maxY || window.scrollY === previousY\
            }};\
        }})()"
    ))
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveScrollResult {
    current_scroll_y: f32,
    reached_boundary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "browser")]
    #[test]
    fn normalize_page_metrics_keeps_finite_non_negative_values() {
        let metrics = normalize_page_metrics(LivePageMetrics {
            scroll_y: 42.5,
            viewport_width: 1280.0,
            viewport_height: 720.0,
            document_height: 2400.25,
        });

        assert_eq!(
            metrics,
            BrowserPageMetrics {
                scroll_y: 42.5,
                viewport_width: 1280.0,
                viewport_height: 720.0,
                document_height: 2400.25,
            }
        );
    }

    #[cfg(feature = "browser")]
    #[test]
    fn normalize_page_metrics_clamps_invalid_values_to_zero() {
        let metrics = normalize_page_metrics(LivePageMetrics {
            scroll_y: f64::NAN,
            viewport_width: -10.0,
            viewport_height: f64::INFINITY,
            document_height: -0.5,
        });

        assert_eq!(
            metrics,
            BrowserPageMetrics {
                scroll_y: 0.0,
                viewport_width: 0.0,
                viewport_height: 0.0,
                document_height: 0.0,
            }
        );
    }
}
