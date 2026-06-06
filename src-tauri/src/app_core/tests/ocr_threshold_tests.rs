use super::*;

#[test]
fn should_trigger_no_extractable_text_ocr_fallback_when_dom_regions_are_empty() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("   "),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn extracted_text_metrics_counts_trimmed_text_and_regions() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("  Visible DOM text  "),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from(" "),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(extracted_text_metrics(&page), (16, 1));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_when_text_is_below_char_threshold() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("Short text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        sparse_text_char_threshold: 20,
        sparse_text_region_threshold: 1,
        ..OcrSettings::default()
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true, &page, &settings
    ));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_when_region_count_is_below_threshold() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("This region has enough text to pass the char threshold alone."),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        sparse_text_char_threshold: 10,
        sparse_text_region_threshold: 2,
        ..OcrSettings::default()
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true, &page, &settings
    ));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_at_default_char_boundary() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: "a".repeat(100),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: "b".repeat(100),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_at_default_region_boundary() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: "a".repeat(201),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn should_not_trigger_extract_page_model_ocr_fallback_above_default_boundaries() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: "a".repeat(101),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: "b".repeat(100),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert!(!should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn should_not_trigger_extract_page_model_ocr_fallback_when_thresholds_are_satisfied() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from(
                    "This first region contains comfortably more than twenty characters.",
                ),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from("This second region also contains enough text."),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        sparse_text_char_threshold: 20,
        sparse_text_region_threshold: 2,
        ..OcrSettings::default()
    };

    assert!(!should_trigger_extract_page_model_ocr_fallback(
        true, &page, &settings
    ));
}

#[test]
fn should_not_trigger_extract_page_model_ocr_fallback_when_disabled_or_non_dom() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::new(),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let disabled_settings = OcrSettings {
        trigger_on_no_extractable_text: false,
        ..OcrSettings::default()
    };

    assert!(!should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &disabled_settings
    ));
    assert!(!should_trigger_extract_page_model_ocr_fallback(
        false,
        &page,
        &OcrSettings::default()
    ));
}
