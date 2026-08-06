use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::page_model::Rect;
use crate::resource_limits::{
    ocr_requests, png_dimensions, truncate_utf8_bytes, validate_image_dimensions,
    MAX_OCR_INPUT_BYTES, MAX_OCR_OUTPUT_BYTES,
};

#[cfg(feature = "ocr")]
use leptess::LepTess;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrSettings {
    pub trigger_on_no_extractable_text: bool,
    pub sparse_text_char_threshold: u32,
    pub sparse_text_region_threshold: u32,
    pub prefer_region_ocr: bool,
}

impl Default for OcrSettings {
    fn default() -> Self {
        Self {
            trigger_on_no_extractable_text: true,
            sparse_text_char_threshold: 200,
            sparse_text_region_threshold: 2,
            prefer_region_ocr: true,
        }
    }
}

#[cfg(any(feature = "ocr", test))]
const OCR_FALLBACK_SOURCE_DPI: i32 = 300;
#[cfg(any(feature = "ocr", test))]
const TESSERACT_LANGUAGE: &str = "eng";

#[derive(Debug, Clone, PartialEq)]
pub struct OcrExtraction {
    pub extracted_text: String,
    pub confidence: Option<f32>,
    pub original_text_bytes: usize,
    pub truncated: bool,
}

#[derive(Debug, Error)]
pub enum OcrRuntimeError {
    #[error("ocr requires the 'ocr' feature to be enabled")]
    FeatureUnavailable,
    #[error("failed to initialize the OCR engine: {reason}")]
    EngineInitFailed { reason: String },
    #[error("failed to load OCR image from {image_path}: {reason}")]
    ImageLoadFailed { image_path: String, reason: String },
    #[error("OCR image used {actual_bytes} bytes, exceeding the {maximum_bytes}-byte limit")]
    ImageTooLarge {
        actual_bytes: u64,
        maximum_bytes: u64,
    },
    #[error("OCR image dimensions are invalid or exceed the configured image budget")]
    ImageDimensionsExceeded,
    #[error("OCR operation was not started: {reason}")]
    OperationLimited { reason: String },
    #[error("ocr bbox coordinates must be finite and positive")]
    InvalidBbox,
    #[error("failed to extract OCR text: {reason}")]
    TextExtractionFailed { reason: String },
}

pub struct OcrController;

impl Default for OcrController {
    fn default() -> Self {
        Self::new()
    }
}

impl OcrController {
    pub fn new() -> Self {
        Self
    }

    pub fn run_ocr(
        &mut self,
        image_path: &Path,
        bbox: Option<&Rect>,
    ) -> Result<OcrExtraction, OcrRuntimeError> {
        #[cfg(feature = "ocr")]
        {
            let _permit = ocr_requests().try_acquire().map_err(|limit| {
                OcrRuntimeError::OperationLimited {
                    reason: limit.to_string(),
                }
            })?;
            validate_ocr_image(image_path)?;
            let mut engine = LepTess::new(None, TESSERACT_LANGUAGE).map_err(|error| {
                OcrRuntimeError::EngineInitFailed {
                    reason: error.to_string(),
                }
            })?;
            engine
                .set_image(image_path)
                .map_err(|error| OcrRuntimeError::ImageLoadFailed {
                    image_path: image_path.display().to_string(),
                    reason: error.to_string(),
                })?;
            engine.set_fallback_source_resolution(OCR_FALLBACK_SOURCE_DPI);

            if let Some(bbox) = bbox {
                let (left, top, width, height) = normalized_ocr_bbox(bbox)?;
                engine.set_rectangle(left, top, width, height);
            }

            let extracted_text = normalize_ocr_text(engine.get_utf8_text().map_err(|error| {
                OcrRuntimeError::TextExtractionFailed {
                    reason: error.to_string(),
                }
            })?);
            let original_text_bytes = extracted_text.len();
            let (bounded_text, truncated) =
                truncate_utf8_bytes(&extracted_text, MAX_OCR_OUTPUT_BYTES);

            Ok(OcrExtraction {
                extracted_text: bounded_text.to_string(),
                confidence: normalize_ocr_confidence(engine.mean_text_conf()),
                original_text_bytes,
                truncated,
            })
        }

        #[cfg(not(feature = "ocr"))]
        {
            let _ = image_path;
            let _ = bbox;
            Err(OcrRuntimeError::FeatureUnavailable)
        }
    }
}

#[cfg(feature = "ocr")]
fn validate_ocr_image(image_path: &Path) -> Result<(), OcrRuntimeError> {
    use std::io::Read;

    let metadata =
        std::fs::metadata(image_path).map_err(|error| OcrRuntimeError::ImageLoadFailed {
            image_path: image_path.display().to_string(),
            reason: error.to_string(),
        })?;
    if metadata.len() > MAX_OCR_INPUT_BYTES {
        return Err(OcrRuntimeError::ImageTooLarge {
            actual_bytes: metadata.len(),
            maximum_bytes: MAX_OCR_INPUT_BYTES,
        });
    }
    let mut header = [0_u8; 24];
    let mut file =
        std::fs::File::open(image_path).map_err(|error| OcrRuntimeError::ImageLoadFailed {
            image_path: image_path.display().to_string(),
            reason: error.to_string(),
        })?;
    file.read_exact(&mut header)
        .map_err(|error| OcrRuntimeError::ImageLoadFailed {
            image_path: image_path.display().to_string(),
            reason: error.to_string(),
        })?;
    let (width, height) =
        png_dimensions(&header).ok_or(OcrRuntimeError::ImageDimensionsExceeded)?;
    validate_image_dimensions(width, height).map_err(|_| OcrRuntimeError::ImageDimensionsExceeded)
}

#[cfg(any(feature = "ocr", test))]
fn normalize_ocr_text(text: String) -> String {
    text.trim().to_string()
}

#[cfg(any(feature = "ocr", test))]
fn normalize_ocr_confidence(confidence: i32) -> Option<f32> {
    if confidence < 0 {
        None
    } else {
        Some((confidence.clamp(0, 100) as f32) / 100.0)
    }
}

#[cfg(any(feature = "ocr", test))]
fn normalized_ocr_bbox(bbox: &Rect) -> Result<(i32, i32, i32, i32), OcrRuntimeError> {
    if !bbox.x.is_finite()
        || !bbox.y.is_finite()
        || !bbox.width.is_finite()
        || !bbox.height.is_finite()
    {
        return Err(OcrRuntimeError::InvalidBbox);
    }

    let left = bbox.x.max(0.0).floor() as i32;
    let top = bbox.y.max(0.0).floor() as i32;
    let width = bbox.width.ceil() as i32;
    let height = bbox.height.ceil() as i32;

    if width <= 0 || height <= 0 {
        return Err(OcrRuntimeError::InvalidBbox);
    }

    Ok((left, top, width, height))
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_ocr_confidence, normalize_ocr_text, normalized_ocr_bbox, OcrRuntimeError,
        OcrSettings,
    };
    use crate::page_model::Rect;
    use crate::resource_limits::{
        ocr_requests, png_dimensions, truncate_utf8_bytes, validate_image_dimensions,
        MAX_OCR_INPUT_BYTES, MAX_OCR_OUTPUT_BYTES,
    };

    #[test]
    fn default_ocr_settings_use_shipped_sparse_text_thresholds() {
        assert_eq!(
            OcrSettings::default(),
            OcrSettings {
                trigger_on_no_extractable_text: true,
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
                prefer_region_ocr: true,
            }
        );
    }

    #[test]
    fn normalize_ocr_text_trims_outer_whitespace() {
        assert_eq!(
            normalize_ocr_text(String::from("  Hello from OCR\n")),
            String::from("Hello from OCR")
        );
    }

    #[test]
    fn normalize_ocr_confidence_maps_percent_to_unit_interval() {
        assert_eq!(normalize_ocr_confidence(87), Some(0.87));
        assert_eq!(normalize_ocr_confidence(150), Some(1.0));
        assert_eq!(normalize_ocr_confidence(-1), None);
    }

    #[test]
    fn normalized_ocr_bbox_clamps_origin_and_rounds_size() {
        assert_eq!(
            normalized_ocr_bbox(&Rect {
                x: -5.2,
                y: 10.8,
                width: 20.1,
                height: 30.0,
            })
            .unwrap(),
            (0, 10, 21, 30)
        );
    }

    #[test]
    fn normalized_ocr_bbox_rejects_non_positive_or_non_finite_values() {
        let invalid = normalized_ocr_bbox(&Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 10.0,
        })
        .unwrap_err();
        assert!(matches!(invalid, OcrRuntimeError::InvalidBbox));

        let non_finite = normalized_ocr_bbox(&Rect {
            x: f32::NAN,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        })
        .unwrap_err();
        assert!(matches!(non_finite, OcrRuntimeError::InvalidBbox));
    }
}
