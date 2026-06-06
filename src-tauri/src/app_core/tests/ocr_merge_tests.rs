use super::*;

#[test]
fn region_first_ocr_target_ids_prefers_bbox_backed_readable_regions() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable text"),
                bbox: Some(Rect {
                    x: 1.0,
                    y: 2.0,
                    width: 30.0,
                    height: 40.0,
                }),
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable but no bbox"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-3"),
                role: RegionRole::Other,
                label: None,
                text: String::from(""),
                bbox: Some(Rect {
                    x: 5.0,
                    y: 6.0,
                    width: 50.0,
                    height: 60.0,
                }),
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-4"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable but invalid bbox"),
                bbox: Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 10.0,
                }),
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        region_first_ocr_target_ids(&page, &OcrSettings::default()),
        vec![String::from("region-1")]
    );
}

#[test]
fn region_first_ocr_target_ids_respects_preference_toggle() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("Readable text"),
            bbox: Some(Rect {
                x: 1.0,
                y: 2.0,
                width: 30.0,
                height: 40.0,
            }),
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        prefer_region_ocr: false,
        ..OcrSettings::default()
    };

    assert!(region_first_ocr_target_ids(&page, &settings).is_empty());
}

#[test]
fn merged_region_text_prefers_more_complete_or_combined_text() {
    assert_eq!(
        merged_region_text("Short label", "Short label with extra detail"),
        String::from("Short label with extra detail")
    );
    assert_eq!(
        merged_region_text("DOM text", "OCR text"),
        String::from("DOM text\n\nOCR text")
    );
}

#[test]
fn merge_ocr_text_into_page_model_updates_existing_region_as_mixed_and_adopts_bbox() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Section,
            label: Some(String::from("Main")),
            text: String::from("DOM summary"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let updated_region_ids = merge_ocr_text_into_page_model(
        &mut page,
        Some("region-1"),
        "OCR detail",
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }),
        String::from("unused"),
    )
    .expect("merge should update the requested region");

    assert_eq!(updated_region_ids, vec![String::from("region-1")]);
    assert_eq!(page.regions[0].source, RegionSource::Mixed);
    assert_eq!(
        page.regions[0].text,
        String::from("DOM summary\n\nOCR detail")
    );
    assert_eq!(
        page.regions[0].bbox,
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        })
    );
}

#[test]
fn merge_ocr_text_into_page_model_preserves_existing_region_bbox() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Section,
            label: Some(String::from("Main")),
            text: String::from("DOM summary"),
            bbox: Some(Rect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }),
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let updated_region_ids = merge_ocr_text_into_page_model(
        &mut page,
        Some("region-1"),
        "OCR detail",
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }),
        String::from("unused"),
    )
    .expect("merge should update the requested region");

    assert_eq!(updated_region_ids, vec![String::from("region-1")]);
    assert_eq!(
        page.regions[0].bbox,
        Some(Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        })
    );
}

#[test]
fn merge_ocr_text_into_page_model_appends_new_ocr_region_when_target_missing() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let updated_region_ids = merge_ocr_text_into_page_model(
        &mut page,
        None,
        "Recovered OCR text",
        Some(Rect {
            x: 5.0,
            y: 6.0,
            width: 70.0,
            height: 80.0,
        }),
        String::from("ocr-region-generated"),
    )
    .expect("merge should create a new OCR region when no target region_id is supplied");

    assert_eq!(
        updated_region_ids,
        vec![String::from("ocr-region-generated")]
    );
    assert_eq!(page.regions.len(), 1);
    assert_eq!(page.regions[0].region_id, "ocr-region-generated");
    assert_eq!(page.regions[0].source, RegionSource::Ocr);
    assert_eq!(page.regions[0].text, "Recovered OCR text");
    assert_eq!(
        page.regions[0].bbox,
        Some(Rect {
            x: 5.0,
            y: 6.0,
            width: 70.0,
            height: 80.0,
        })
    );
}

#[test]
fn merge_ocr_text_into_page_model_rejects_blank_ocr_text() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Existing text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let error = merge_ocr_text_into_page_model(
        &mut page,
        Some("region-1"),
        "   ",
        None,
        String::from("ocr-region-1"),
    )
    .unwrap_err();

    assert_eq!(error.code, "invalid_ocr_text");
    assert_eq!(page.regions[0].text, "Existing text");
    assert_eq!(page.regions[0].source, RegionSource::Dom);
}

#[test]
fn merge_ocr_text_into_page_model_rejects_unknown_target_region() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Existing text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let error = merge_ocr_text_into_page_model(
        &mut page,
        Some("missing-region"),
        "Scanned text",
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        }),
        String::from("ocr-region-1"),
    )
    .unwrap_err();

    assert_eq!(error.code, "unknown_region_id");
    assert_eq!(
        error.details,
        Some(serde_json::json!({ "region_id": "missing-region" }))
    );
    assert_eq!(page.regions.len(), 1);
    assert_eq!(page.regions[0].text, "Existing text");
}
