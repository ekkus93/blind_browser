use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::page_model::{InteractiveElement, PageModel, PageRegion, RegionSource};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExtractionPolicy {
    pub prefer_dom: bool,
    pub enable_sparse_text_checks: bool,
}

impl Default for ExtractionPolicy {
    fn default() -> Self {
        Self {
            prefer_dom: true,
            enable_sparse_text_checks: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum ExtractorError {
    #[error("dom_smoothie extraction support is not enabled in this build")]
    FeatureDisabled,
    #[error("dom_smoothie rejected the document URL because it was not absolute")]
    BadDocumentUrl,
    #[error("dom_smoothie could not parse the document: {0}")]
    Readability(String),
    #[error("dom_smoothie did not return any readable content")]
    NoReadableContent,
}

pub fn extract_page_model_from_html(
    html: &str,
    document_url: Option<&str>,
    interactive_elements: Vec<InteractiveElement>,
) -> Result<PageModel, ExtractorError> {
    #[cfg(feature = "browser")]
    {
        use dom_smoothie::{CandidateSelectMode, Config, Readability};

        let config = Config {
            candidate_select_mode: CandidateSelectMode::DomSmoothie,
            ..Config::default()
        };
        let mut readability =
            Readability::new(html, document_url, Some(config)).map_err(|error| match error {
                dom_smoothie::ReadabilityError::BadDocumentURL => ExtractorError::BadDocumentUrl,
                other => ExtractorError::Readability(other.to_string()),
            })?;
        let article = readability
            .parse()
            .map_err(|error| ExtractorError::Readability(error.to_string()))?;
        let title = normalize_optional_text(Some(article.title.as_str()));
        let url = normalize_optional_text(article.url.as_deref())
            .or_else(|| normalize_optional_text(document_url));
        let mut regions =
            build_regions_from_article_text(article.text_content.as_ref(), title.as_deref());

        if regions.is_empty() {
            return Err(ExtractorError::NoReadableContent);
        }

        if let Some(title_text) = title.as_deref() {
            let duplicates_title = regions
                .first()
                .is_some_and(|region| region.text.trim().eq_ignore_ascii_case(title_text.trim()));
            if !duplicates_title {
                regions.insert(
                    0,
                    PageRegion {
                        region_id: String::from("dom-region-title"),
                        label: Some(String::from("Title")),
                        text: title_text.to_string(),
                        bbox: None,
                        source: RegionSource::Dom,
                    },
                );
            }
        }

        Ok(PageModel {
            title,
            url,
            regions,
            interactive_elements,
        })
    }

    #[cfg(not(feature = "browser"))]
    {
        let _ = html;
        let _ = document_url;
        let _ = interactive_elements;
        Err(ExtractorError::FeatureDisabled)
    }
}

fn build_regions_from_article_text(text: &str, title: Option<&str>) -> Vec<PageRegion> {
    let mut regions = Vec::new();
    let mut current = Vec::<String>::new();

    let flush = |regions: &mut Vec<PageRegion>, current: &mut Vec<String>| {
        let paragraph = current
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        current.clear();
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            return;
        }
        if title.is_some_and(|title| trimmed.eq_ignore_ascii_case(title.trim())) {
            return;
        }

        let next_index = regions.len() + 1;
        regions.push(PageRegion {
            region_id: format!("dom-region-{next_index}"),
            label: None,
            text: trimmed.to_string(),
            bbox: None,
            source: RegionSource::Dom,
        });
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(&mut regions, &mut current);
            continue;
        }

        current.push(trimmed.to_string());
    }
    flush(&mut regions, &mut current);

    regions
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_regions_from_article_text_splits_paragraphs_and_skips_title() {
        let regions = build_regions_from_article_text(
            "Example article\n\nFirst paragraph.\nStill first.\n\nSecond paragraph.",
            Some("Example article"),
        );

        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].region_id, "dom-region-1");
        assert_eq!(regions[0].text, "First paragraph. Still first.");
        assert_eq!(regions[1].text, "Second paragraph.");
    }

    #[test]
    fn build_regions_from_article_text_preserves_paragraph_order() {
        let regions = build_regions_from_article_text(
            "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.",
            None,
        );

        let ordered_text = regions
            .iter()
            .map(|region| region.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            ordered_text,
            vec!["First paragraph.", "Second paragraph.", "Third paragraph."]
        );
        assert!(regions
            .iter()
            .all(|region| region.source == RegionSource::Dom));
    }

    #[cfg(feature = "browser")]
    #[test]
    fn extract_page_model_from_html_uses_dom_smoothie_article_content() {
        let html = r#"
            <html>
                <head><title>Example article</title></head>
                <body>
                    <main>
                        <article>
                            <h1>Example article</h1>
                            <p>First paragraph.</p>
                            <p>Second paragraph.</p>
                        </article>
                    </main>
                </body>
            </html>
        "#;

        let page_model =
            extract_page_model_from_html(html, Some("https://example.com/article"), Vec::new())
                .expect("dom_smoothie extraction should succeed");

        assert_eq!(page_model.title.as_deref(), Some("Example article"));
        assert_eq!(
            page_model.url.as_deref(),
            Some("https://example.com/article")
        );
        assert!(!page_model.regions.is_empty());
        assert!(page_model
            .regions
            .iter()
            .any(|region| region.text.contains("First paragraph")));
    }

    #[cfg(feature = "browser")]
    #[test]
    fn extract_page_model_from_html_builds_target_page_model_shape() {
        let html = r#"
            <html>
                <head><title>Example article</title></head>
                <body>
                    <main>
                        <article>
                            <h1>Example article</h1>
                            <p>First paragraph.</p>
                            <p>Second paragraph.</p>
                        </article>
                    </main>
                </body>
            </html>
        "#;
        let interactive_elements = vec![InteractiveElement {
            element_id: String::from("link-1"),
            dom_locator: Some(String::from("#link-1")),
            role: crate::page_model::ElementRole::Link,
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
        }];

        let page_model = extract_page_model_from_html(
            html,
            Some("https://example.com/article"),
            interactive_elements.clone(),
        )
        .expect("dom_smoothie extraction should succeed");

        assert_eq!(page_model.title.as_deref(), Some("Example article"));
        assert_eq!(
            page_model.url.as_deref(),
            Some("https://example.com/article")
        );
        assert_eq!(page_model.interactive_elements, interactive_elements);

        assert!(!page_model.regions.is_empty());
        let title_regions = page_model
            .regions
            .iter()
            .filter(|region| region.text == "Example article")
            .collect::<Vec<_>>();
        assert!(title_regions.len() <= 1);
        if let Some(title_region) = title_regions.first() {
            assert_eq!(title_region.label.as_deref(), Some("Title"));
            assert_eq!(title_region.source, RegionSource::Dom);
        }

        let body_region_text = page_model
            .regions
            .iter()
            .filter(|region| region.text != "Example article")
            .map(|region| region.text.as_str())
            .collect::<Vec<_>>();
        assert!(!body_region_text.is_empty());
        let combined_body_text = body_region_text.join("\n");
        let first_index = combined_body_text
            .find("First paragraph.")
            .expect("first paragraph should be present");
        let second_index = combined_body_text
            .find("Second paragraph.")
            .expect("second paragraph should be present");
        assert!(first_index < second_index);
        assert!(page_model
            .regions
            .iter()
            .all(|region| region.source == RegionSource::Dom));
    }
}
