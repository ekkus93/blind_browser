use super::*;

#[test]
fn build_visible_text_excerpt_joins_regions_and_applies_limit() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("First paragraph"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Second paragraph"),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        build_visible_text_excerpt(&page, None),
        String::from("First paragraph\n\nSecond paragraph")
    );
    assert_eq!(
        build_visible_text_excerpt(&page, Some(5)),
        String::from("First")
    );
}

#[test]
fn region_bbox_by_id_returns_region_geometry_when_available() {
    let regions = vec![PageRegion {
        region_id: String::from("region-1"),
        role: RegionRole::Section,
        label: Some(String::from("Main")),
        text: String::from("Text"),
        bbox: Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }),
        source: RegionSource::Dom,
    }];

    assert_eq!(
        region_bbox_by_id(&regions, "region-1").expect("region bbox should resolve"),
        Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }
    );
}

#[test]
fn build_extracted_page_model_can_omit_links() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("link-1"),
                dom_locator: Some(String::from("#link-1")),
                role: ElementRole::Link,
                tag_name: String::from("a"),
                text: Some(String::from("Read more")),
                accessible_name: Some(String::from("Read more")),
                placeholder: None,
                href: Some(String::from("https://example.com/more")),
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#button-1")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: true,
        include_headings: true,
        include_links: false,
    };

    let extracted = build_extracted_page_model(&page, &input);

    assert_eq!(extracted.interactive_elements.len(), 1);
    assert_eq!(extracted.interactive_elements[0].role, ElementRole::Button);
}

#[test]
fn build_extracted_page_model_preserves_link_metadata_when_requested() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("link-1"),
            dom_locator: Some(String::from("#link-1")),
            role: ElementRole::Link,
            tag_name: String::from("a"),
            text: Some(String::from("Read more")),
            accessible_name: Some(String::from("Read more about examples")),
            placeholder: None,
            href: Some(String::from("https://example.com/more")),
            value: None,
            bbox: Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 12.0,
            }),
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::from([(
                String::from("rel"),
                String::from("noopener"),
            )]),
        }],
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: true,
        include_headings: true,
        include_links: true,
    };

    let extracted = build_extracted_page_model(&page, &input);

    assert_eq!(extracted.interactive_elements.len(), 1);
    let link = &extracted.interactive_elements[0];
    assert_eq!(link.role, ElementRole::Link);
    assert_eq!(link.href.as_deref(), Some("https://example.com/more"));
    assert_eq!(link.text.as_deref(), Some("Read more"));
    assert_eq!(
        link.accessible_name.as_deref(),
        Some("Read more about examples")
    );
    assert_eq!(
        link.bbox,
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 12.0,
        })
    );
    assert_eq!(
        link.attributes.get("rel").map(String::as_str),
        Some("noopener")
    );
}

#[test]
fn build_extracted_page_model_preserves_region_order_and_sources() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("dom-region-title"),
                role: RegionRole::Title,
                label: Some(String::from("Title")),
                text: String::from("Example"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("dom-region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("First paragraph."),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("ocr-region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Recovered OCR text."),
                bbox: None,
                source: RegionSource::Ocr,
            },
        ],
        interactive_elements: Vec::new(),
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: true,
        include_headings: true,
        include_links: true,
    };

    let extracted = build_extracted_page_model(&page, &input);

    let ordered_region_ids = extracted
        .regions
        .iter()
        .map(|region| region.region_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ordered_region_ids,
        vec!["dom-region-title", "dom-region-1", "ocr-region-1"]
    );
    assert_eq!(
        extracted
            .regions
            .iter()
            .map(|region| region.source.clone())
            .collect::<Vec<_>>(),
        vec![RegionSource::Dom, RegionSource::Dom, RegionSource::Ocr]
    );
}

#[test]
fn build_extracted_page_model_leaves_heading_regions_unchanged_when_disabled() {
    let page = PageModel {
        title: Some(String::from("Example article")),
        url: Some(String::from("https://example.com/article")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-title"),
                role: RegionRole::Title,
                label: Some(String::from("Title")),
                text: String::from("Example article"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-heading"),
                role: RegionRole::Heading,
                label: Some(String::from("Heading")),
                text: String::from("Section one"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-paragraph"),
                role: RegionRole::Paragraph,
                label: None,
                text: String::from("First paragraph."),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: false,
        include_headings: false,
        include_links: true,
    };

    let extracted = build_extracted_page_model(&page, &input);

    assert_eq!(extracted.title, page.title);
    assert_eq!(extracted.url, page.url);
    assert_eq!(extracted.regions, page.regions);
}

#[test]
fn infer_extraction_source_detects_merged_models() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("dom-region"),
                role: RegionRole::Other,
                label: None,
                text: String::from("DOM text"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("ocr-region"),
                role: RegionRole::Other,
                label: None,
                text: String::from("OCR text"),
                bbox: None,
                source: RegionSource::Ocr,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        infer_extraction_source(&page, true, false),
        ExtractionSource::Merged
    );
}

#[test]
fn infer_extraction_source_treats_mixed_regions_as_merged() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("mixed-region"),
            role: RegionRole::Other,
            label: None,
            text: String::from("DOM text\n\nOCR text"),
            bbox: None,
            source: RegionSource::Mixed,
        }],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        infer_extraction_source(&page, true, false),
        ExtractionSource::Merged
    );
}

#[test]
fn infer_extraction_source_reports_dom_smoothie_when_dom_only() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("dom-region"),
            role: RegionRole::Other,
            label: None,
            text: String::from("Readable text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        infer_extraction_source(&page, true, true),
        ExtractionSource::DomSmoothie
    );
    assert_eq!(
        infer_extraction_source(&page, true, false),
        ExtractionSource::DomFallback
    );
}
