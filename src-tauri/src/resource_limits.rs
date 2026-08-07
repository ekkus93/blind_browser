use std::fmt;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::blocking::Response;

pub(crate) const MAX_REMOTE_REGIONS: usize = 64;
pub(crate) const MAX_REMOTE_ELEMENTS: usize = 128;
pub(crate) const MAX_REMOTE_HISTORY_ENTRIES: usize = 32;
pub(crate) const MAX_REMOTE_HISTORY_BYTES: usize = 64 * 1024;
pub(crate) const MAX_REMOTE_SKILLS: usize = 32;
pub(crate) const MAX_REGION_TEXT_CHARS: usize = 2_000;
pub(crate) const MAX_ELEMENT_TEXT_CHARS: usize = 512;
pub(crate) const MAX_OBSERVATION_CHARS: usize = 512;
pub(crate) const MAX_IDENTIFIER_CHARS: usize = 128;
pub(crate) const MAX_URL_CHARS: usize = 2_048;
pub(crate) const MAX_INTENT_TAGS: usize = 16;
pub(crate) const MAX_REMOTE_PLANNER_PROMPT_BYTES: usize = 256 * 1024;

pub(crate) const MAX_SCREENSHOT_WIDTH: u32 = 8_192;
pub(crate) const MAX_SCREENSHOT_HEIGHT: u32 = 8_192;
pub(crate) const MAX_SCREENSHOT_PIXELS: u64 = 32 * 1024 * 1024;
pub(crate) const MAX_SCREENSHOT_ENCODED_BYTES: u64 = 32 * 1024 * 1024;
pub(crate) const SCREENSHOT_CACHE_MAX_COUNT: usize = 32;
pub(crate) const SCREENSHOT_CACHE_MAX_BYTES: u64 = 128 * 1024 * 1024;

pub(crate) const MAX_OCR_INPUT_BYTES: u64 = MAX_SCREENSHOT_ENCODED_BYTES;
pub(crate) const MAX_OCR_OUTPUT_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_PLANNER_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_TTS_RESPONSE_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_ASR_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_ASR_AUDIO_UPLOAD_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_TTS_INPUT_TEXT_BYTES: usize = 64 * 1024;
pub(crate) const MAX_MODEL_LIST_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_MODEL_DOWNLOAD_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub(crate) const SYNTHESIZED_SPEECH_CACHE_MAX_COUNT: usize = 8;
pub(crate) const SYNTHESIZED_SPEECH_CACHE_MAX_BYTES: usize = 64 * 1024 * 1024;

pub(crate) const MAX_CONCURRENT_PLANNER_REQUESTS: usize = 2;
pub(crate) const MAX_CONCURRENT_MODEL_LIST_REQUESTS: usize = 2;
pub(crate) const MAX_CONCURRENT_SCREENSHOTS: usize = 1;
pub(crate) const MAX_CONCURRENT_OCR_REQUESTS: usize = 1;
pub(crate) const MAX_CONCURRENT_MODEL_DOWNLOADS: usize = 1;
pub(crate) const MAX_CONCURRENT_TTS_REQUESTS: usize = 2;
pub(crate) const MAX_CONCURRENT_ASR_REQUESTS: usize = 1;

const ONE_MINUTE_MS: u64 = 60_000;
const ONE_HOUR_MS: u64 = 60 * ONE_MINUTE_MS;
pub(crate) const MAX_PLANNER_REQUESTS_PER_MINUTE: usize = 30;
pub(crate) const MAX_MODEL_LIST_REQUESTS_PER_MINUTE: usize = 20;
pub(crate) const MAX_SCREENSHOTS_PER_MINUTE: usize = 30;
pub(crate) const MAX_OCR_REQUESTS_PER_MINUTE: usize = 20;
pub(crate) const MAX_MODEL_DOWNLOADS_PER_HOUR: usize = 4;
pub(crate) const MAX_TTS_REQUESTS_PER_MINUTE: usize = 60;
pub(crate) const MAX_ASR_REQUESTS_PER_MINUTE: usize = 30;

static PLANNER_REQUESTS: OperationLimiter = OperationLimiter::rated(
    MAX_CONCURRENT_PLANNER_REQUESTS,
    MAX_PLANNER_REQUESTS_PER_MINUTE,
    ONE_MINUTE_MS,
);
static MODEL_LIST_REQUESTS: OperationLimiter = OperationLimiter::rated(
    MAX_CONCURRENT_MODEL_LIST_REQUESTS,
    MAX_MODEL_LIST_REQUESTS_PER_MINUTE,
    ONE_MINUTE_MS,
);
static SCREENSHOTS: OperationLimiter = OperationLimiter::rated(
    MAX_CONCURRENT_SCREENSHOTS,
    MAX_SCREENSHOTS_PER_MINUTE,
    ONE_MINUTE_MS,
);
static OCR_REQUESTS: OperationLimiter = OperationLimiter::rated(
    MAX_CONCURRENT_OCR_REQUESTS,
    MAX_OCR_REQUESTS_PER_MINUTE,
    ONE_MINUTE_MS,
);
static MODEL_DOWNLOADS: OperationLimiter = OperationLimiter::rated(
    MAX_CONCURRENT_MODEL_DOWNLOADS,
    MAX_MODEL_DOWNLOADS_PER_HOUR,
    ONE_HOUR_MS,
);
static TTS_REQUESTS: OperationLimiter = OperationLimiter::rated(
    MAX_CONCURRENT_TTS_REQUESTS,
    MAX_TTS_REQUESTS_PER_MINUTE,
    ONE_MINUTE_MS,
);
static ASR_REQUESTS: OperationLimiter = OperationLimiter::rated(
    MAX_CONCURRENT_ASR_REQUESTS,
    MAX_ASR_REQUESTS_PER_MINUTE,
    ONE_MINUTE_MS,
);

pub(crate) fn planner_requests() -> &'static OperationLimiter {
    &PLANNER_REQUESTS
}

pub(crate) fn model_list_requests() -> &'static OperationLimiter {
    &MODEL_LIST_REQUESTS
}

pub(crate) fn screenshots() -> &'static OperationLimiter {
    &SCREENSHOTS
}

pub(crate) fn ocr_requests() -> &'static OperationLimiter {
    &OCR_REQUESTS
}

pub(crate) fn model_downloads() -> &'static OperationLimiter {
    &MODEL_DOWNLOADS
}

pub(crate) fn tts_requests() -> &'static OperationLimiter {
    &TTS_REQUESTS
}

pub(crate) fn asr_requests() -> &'static OperationLimiter {
    &ASR_REQUESTS
}

#[derive(Debug, Clone, Copy)]
struct RateLimit {
    maximum: usize,
    window_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct RateWindow {
    started_at_ms: u64,
    used: usize,
}

#[derive(Debug)]
pub(crate) struct OperationLimiter {
    active: AtomicUsize,
    maximum_concurrent: usize,
    rate_limit: Option<RateLimit>,
    rate_window: Mutex<RateWindow>,
}

impl OperationLimiter {
    #[cfg(test)]
    pub(crate) const fn new(maximum_concurrent: usize) -> Self {
        Self {
            active: AtomicUsize::new(0),
            maximum_concurrent,
            rate_limit: None,
            rate_window: Mutex::new(RateWindow {
                started_at_ms: 0,
                used: 0,
            }),
        }
    }

    pub(crate) const fn rated(
        maximum_concurrent: usize,
        maximum_requests: usize,
        window_ms: u64,
    ) -> Self {
        Self {
            active: AtomicUsize::new(0),
            maximum_concurrent,
            rate_limit: Some(RateLimit {
                maximum: maximum_requests,
                window_ms,
            }),
            rate_window: Mutex::new(RateWindow {
                started_at_ms: 0,
                used: 0,
            }),
        }
    }

    pub(crate) fn try_acquire(&self) -> Result<OperationPermit<'_>, OperationLimitExceeded> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OperationLimitExceeded::ClockUnavailable)?
            .as_millis();
        let now_ms = u64::try_from(now_ms).map_err(|_| OperationLimitExceeded::ClockUnavailable)?;
        self.try_acquire_at(now_ms)
    }

    fn try_acquire_at(&self, now_ms: u64) -> Result<OperationPermit<'_>, OperationLimitExceeded> {
        if self.maximum_concurrent == 0 {
            return Err(OperationLimitExceeded::Concurrency {
                maximum: self.maximum_concurrent,
            });
        }
        let acquired = self
            .active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < self.maximum_concurrent).then_some(active + 1)
            })
            .is_ok();
        if !acquired {
            return Err(OperationLimitExceeded::Concurrency {
                maximum: self.maximum_concurrent,
            });
        }

        if let Err(error) = self.consume_rate_budget(now_ms) {
            let previous = self.active.fetch_sub(1, Ordering::AcqRel);
            debug_assert!(previous > 0, "operation permit counter underflow");
            return Err(error);
        }

        Ok(OperationPermit { limiter: self })
    }

    fn consume_rate_budget(&self, now_ms: u64) -> Result<(), OperationLimitExceeded> {
        let Some(rate_limit) = self.rate_limit else {
            return Ok(());
        };
        if rate_limit.maximum == 0 || rate_limit.window_ms == 0 {
            return Err(OperationLimitExceeded::Rate {
                maximum: rate_limit.maximum,
                window_ms: rate_limit.window_ms,
                retry_after_ms: rate_limit.window_ms,
            });
        }

        let mut window = self
            .rate_window
            .lock()
            .map_err(|_| OperationLimitExceeded::StateUnavailable)?;
        let elapsed = now_ms.saturating_sub(window.started_at_ms);
        if window.started_at_ms == 0 || elapsed >= rate_limit.window_ms {
            window.started_at_ms = now_ms;
            window.used = 0;
        }
        if window.used >= rate_limit.maximum {
            return Err(OperationLimitExceeded::Rate {
                maximum: rate_limit.maximum,
                window_ms: rate_limit.window_ms,
                retry_after_ms: rate_limit.window_ms.saturating_sub(elapsed),
            });
        }
        window.used = window.used.saturating_add(1);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn active(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }
}

#[derive(Debug)]
pub(crate) struct OperationPermit<'a> {
    limiter: &'a OperationLimiter,
}

impl Drop for OperationPermit<'_> {
    fn drop(&mut self) {
        let previous = self.limiter.active.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0, "operation permit counter underflow");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperationLimitExceeded {
    Concurrency {
        maximum: usize,
    },
    Rate {
        maximum: usize,
        window_ms: u64,
        retry_after_ms: u64,
    },
    ClockUnavailable,
    StateUnavailable,
}

impl OperationLimitExceeded {
    pub(crate) fn code(self, operation: &str) -> String {
        match self {
            Self::Concurrency { .. } => format!("{operation}_concurrency_limited"),
            Self::Rate { .. } => format!("{operation}_rate_limited"),
            Self::ClockUnavailable | Self::StateUnavailable => {
                format!("{operation}_budget_unavailable")
            }
        }
    }

    pub(crate) fn details(self) -> serde_json::Value {
        match self {
            Self::Concurrency { maximum } => {
                serde_json::json!({ "maximum_concurrent_requests": maximum })
            }
            Self::Rate {
                maximum,
                window_ms,
                retry_after_ms,
            } => serde_json::json!({
                "maximum_requests": maximum,
                "window_ms": window_ms,
                "retry_after_ms": retry_after_ms,
            }),
            Self::ClockUnavailable | Self::StateUnavailable => serde_json::json!({}),
        }
    }
}

impl fmt::Display for OperationLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Concurrency { maximum } => write!(
                formatter,
                "operation concurrency limit of {maximum} is already in use"
            ),
            Self::Rate {
                maximum,
                window_ms,
                retry_after_ms,
            } => write!(
                formatter,
                "operation rate limit of {maximum} per {window_ms} ms is exhausted; retry after {retry_after_ms} ms"
            ),
            Self::ClockUnavailable => {
                write!(formatter, "operation rate-limit clock is unavailable")
            }
            Self::StateUnavailable => {
                write!(formatter, "operation rate-limit state is unavailable")
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum BoundedResponseError {
    DeclaredTooLarge { declared: u64, maximum: usize },
    BodyTooLarge { maximum: usize },
    ReadFailed(std::io::Error),
}

impl fmt::Display for BoundedResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeclaredTooLarge { declared, maximum } => write!(
                formatter,
                "response declared {declared} bytes, exceeding the {maximum}-byte limit"
            ),
            Self::BodyTooLarge { maximum } => {
                write!(formatter, "response exceeded the {maximum}-byte limit")
            }
            Self::ReadFailed(error) => write!(formatter, "failed to read response body: {error}"),
        }
    }
}

pub(crate) fn read_bounded_response(
    response: Response,
    maximum: usize,
) -> Result<Vec<u8>, BoundedResponseError> {
    let declared = response.content_length();
    read_bounded_reader(response, declared, maximum)
}

pub(crate) fn read_bounded_reader<R: Read>(
    reader: R,
    declared: Option<u64>,
    maximum: usize,
) -> Result<Vec<u8>, BoundedResponseError> {
    if let Some(declared) = declared {
        if declared > maximum as u64 {
            return Err(BoundedResponseError::DeclaredTooLarge { declared, maximum });
        }
    }

    let capacity = match declared {
        Some(length) => match usize::try_from(length) {
            Ok(length) => length.min(maximum),
            Err(_) => maximum,
        },
        None => 0,
    };
    let mut body = Vec::with_capacity(capacity);
    reader
        .take((maximum as u64).saturating_add(1))
        .read_to_end(&mut body)
        .map_err(BoundedResponseError::ReadFailed)?;
    if body.len() > maximum {
        return Err(BoundedResponseError::BodyTooLarge { maximum });
    }
    Ok(body)
}

pub(crate) fn png_dimensions(image_bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if image_bytes.len() < 24 || &image_bytes[..8] != PNG_SIGNATURE {
        return None;
    }
    let mut width_bytes = [0_u8; 4];
    width_bytes.copy_from_slice(&image_bytes[16..20]);
    let mut height_bytes = [0_u8; 4];
    height_bytes.copy_from_slice(&image_bytes[20..24]);
    Some((
        u32::from_be_bytes(width_bytes),
        u32::from_be_bytes(height_bytes),
    ))
}

pub(crate) fn validate_png_resource_limits(
    image_bytes: &[u8],
) -> Result<(u32, u32), ImageLimitExceeded> {
    validate_encoded_image_bytes(image_bytes.len())?;
    let (width, height) =
        png_dimensions(image_bytes).ok_or(ImageLimitExceeded::InvalidDimensions {
            width: 0,
            height: 0,
        })?;
    validate_image_dimensions(width, height)?;
    Ok((width, height))
}

pub(crate) fn validate_image_dimensions(width: u32, height: u32) -> Result<(), ImageLimitExceeded> {
    if width == 0 || height == 0 {
        return Err(ImageLimitExceeded::InvalidDimensions { width, height });
    }
    if width > MAX_SCREENSHOT_WIDTH || height > MAX_SCREENSHOT_HEIGHT {
        return Err(ImageLimitExceeded::Dimensions {
            width,
            height,
            maximum_width: MAX_SCREENSHOT_WIDTH,
            maximum_height: MAX_SCREENSHOT_HEIGHT,
        });
    }
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    if pixels > MAX_SCREENSHOT_PIXELS {
        return Err(ImageLimitExceeded::Pixels {
            pixels,
            maximum: MAX_SCREENSHOT_PIXELS,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageLimitExceeded {
    InvalidDimensions {
        width: u32,
        height: u32,
    },
    Dimensions {
        width: u32,
        height: u32,
        maximum_width: u32,
        maximum_height: u32,
    },
    Pixels {
        pixels: u64,
        maximum: u64,
    },
    EncodedBytes {
        bytes: u64,
        maximum: u64,
    },
}

pub(crate) fn validate_encoded_image_bytes(bytes: usize) -> Result<(), ImageLimitExceeded> {
    let bytes = match u64::try_from(bytes) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err(ImageLimitExceeded::EncodedBytes {
                bytes: u64::MAX,
                maximum: MAX_SCREENSHOT_ENCODED_BYTES,
            });
        }
    };
    if bytes > MAX_SCREENSHOT_ENCODED_BYTES {
        return Err(ImageLimitExceeded::EncodedBytes {
            bytes,
            maximum: MAX_SCREENSHOT_ENCODED_BYTES,
        });
    }
    Ok(())
}

pub(crate) fn truncate_utf8_bytes(value: &str, maximum: usize) -> (&str, bool) {
    if value.len() <= maximum {
        return (value, false);
    }
    let mut end = maximum;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    (&value[..end], true)
}

pub(crate) fn record_resource_size(resource: &'static str, bytes: usize) {
    tracing::debug!(resource, bytes, "bounded resource size");
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;

    use super::*;

    #[test]
    fn operation_limiter_bounds_concurrency_and_releases_on_drop() {
        let limiter = Arc::new(OperationLimiter::new(2));
        let first = limiter.try_acquire().expect("first permit should fit");
        let second = limiter.try_acquire().expect("second permit should fit");
        assert_eq!(limiter.active(), 2);
        assert_eq!(
            limiter.try_acquire().unwrap_err(),
            OperationLimitExceeded::Concurrency { maximum: 2 }
        );
        drop(first);
        assert_eq!(limiter.active(), 1);
        drop(second);
        assert_eq!(limiter.active(), 0);
    }

    #[test]
    fn operation_limiter_is_atomic_across_threads() {
        let limiter = Arc::new(OperationLimiter::new(1));
        let permit = limiter.try_acquire().expect("permit should fit");
        let worker_limiter = Arc::clone(&limiter);
        let worker = thread::spawn(move || worker_limiter.try_acquire().is_err());
        assert!(worker.join().expect("worker should join"));
        drop(permit);
        assert!(limiter.try_acquire().is_ok());
    }

    #[test]
    fn rated_limiter_fails_closed_until_the_window_resets() {
        let limiter = OperationLimiter::rated(2, 2, 1_000);
        let first = limiter
            .try_acquire_at(10)
            .expect("first request should fit");
        drop(first);
        let second = limiter
            .try_acquire_at(20)
            .expect("second request should fit");
        drop(second);
        assert_eq!(
            limiter.try_acquire_at(30).unwrap_err(),
            OperationLimitExceeded::Rate {
                maximum: 2,
                window_ms: 1_000,
                retry_after_ms: 980,
            }
        );
        assert!(limiter.try_acquire_at(1_010).is_ok());
    }

    #[test]
    fn rate_rejection_releases_the_concurrency_slot() {
        let limiter = OperationLimiter::rated(1, 1, 1_000);
        let permit = limiter
            .try_acquire_at(10)
            .expect("first request should fit");
        drop(permit);
        assert!(matches!(
            limiter.try_acquire_at(20),
            Err(OperationLimitExceeded::Rate { .. })
        ));
        assert_eq!(limiter.active(), 0);
    }

    #[test]
    fn bounded_reader_rejects_declared_and_streamed_overflow() {
        use std::io::Cursor;

        assert!(matches!(
            read_bounded_reader(Cursor::new(vec![0_u8; 4]), Some(5), 4),
            Err(BoundedResponseError::DeclaredTooLarge { .. })
        ));
        assert!(matches!(
            read_bounded_reader(Cursor::new(vec![0_u8; 5]), None, 4),
            Err(BoundedResponseError::BodyTooLarge { .. })
        ));
        assert_eq!(
            read_bounded_reader(Cursor::new(vec![1_u8; 4]), Some(4), 4)
                .expect("bounded body should be accepted"),
            vec![1_u8; 4]
        );
    }

    #[test]
    fn utf8_truncation_does_not_split_a_scalar() {
        let (value, truncated) = truncate_utf8_bytes("abéz", 3);
        assert_eq!(value, "ab");
        assert!(truncated);
    }

    #[test]
    fn representative_image_fits_but_pathological_image_does_not() {
        assert!(validate_image_dimensions(1_920, 1_080).is_ok());
        assert!(matches!(
            validate_image_dimensions(100_000, 100_000),
            Err(ImageLimitExceeded::Dimensions { .. })
        ));
    }
}
