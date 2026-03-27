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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ExtractedArticleBlockKind {
    Title,
    Paragraph,
    Heading,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExtractedArticleBlock {
    pub block_id: String,
    pub kind: ExtractedArticleBlockKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ExtractedArticle {
    pub title: Option<String>,
    pub url: Option<String>,
    pub blocks: Vec<ExtractedArticleBlock>,
    pub interactive_elements: Vec<InteractiveElement>,
}

impl ExtractedArticle {
    pub fn into_page_model(self) -> PageModel {
        let regions = self
            .blocks
            .into_iter()
            .map(|block| PageRegion {
                region_id: block.block_id,
                label: match block.kind {
                    ExtractedArticleBlockKind::Title => Some(String::from("Title")),
                    ExtractedArticleBlockKind::Paragraph => None,
                    ExtractedArticleBlockKind::Heading => Some(String::from("Heading")),
                },
                text: block.text,
                bbox: None,
                source: RegionSource::Dom,
            })
            .collect();

        PageModel {
            title: self.title,
            url: self.url,
            regions,
            interactive_elements: self.interactive_elements,
        }
    }
}

pub fn extract_structured_article_from_html(
    html: &str,
    document_url: Option<&str>,
    interactive_elements: Vec<InteractiveElement>,
) -> Result<ExtractedArticle, ExtractorError> {
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
        let mut blocks =
            build_article_blocks_from_text(article.text_content.as_ref(), title.as_deref());

        if blocks.is_empty() {
            return Err(ExtractorError::NoReadableContent);
        }

        if let Some(title_text) = title.as_deref() {
            let duplicates_title = blocks
                .first()
                .is_some_and(|block| block.text.trim().eq_ignore_ascii_case(title_text.trim()));
            if !duplicates_title {
                blocks.insert(
                    0,
                    ExtractedArticleBlock {
                        block_id: String::from("dom-block-title"),
                        kind: ExtractedArticleBlockKind::Title,
                        text: title_text.to_string(),
                    },
                );
            }
        }

        Ok(ExtractedArticle {
            title,
            url,
            blocks,
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

fn build_article_blocks_from_text(
    text: &str,
    title: Option<&str>,
) -> Vec<ExtractedArticleBlock> {
    let mut blocks = Vec::new();
    let mut current = Vec::<String>::new();

    let flush = |blocks: &mut Vec<ExtractedArticleBlock>, current: &mut Vec<String>| {
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

        let next_index = blocks.len() + 1;
        blocks.push(ExtractedArticleBlock {
            block_id: format!("dom-block-{next_index}"),
            kind: ExtractedArticleBlockKind::Paragraph,
            text: trimmed.to_string(),
        });
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(&mut blocks, &mut current);
            continue;
        }

        current.push(trimmed.to_string());
    }
    flush(&mut blocks, &mut current);

    blocks
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
    fn build_article_blocks_from_text_splits_paragraphs_and_skips_title() {
        let blocks = build_article_blocks_from_text(
            "Example article\n\nFirst paragraph.\nStill first.\n\nSecond paragraph.",
            Some("Example article"),
        );

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_id, "dom-block-1");
        assert_eq!(blocks[0].kind, ExtractedArticleBlockKind::Paragraph);
        assert_eq!(blocks[0].text, "First paragraph. Still first.");
        assert_eq!(blocks[1].text, "Second paragraph.");
    }

    #[test]
    fn build_article_blocks_from_text_preserves_paragraph_order() {
        let blocks = build_article_blocks_from_text(
            "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.",
            None,
        );

        let ordered_text = blocks
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            ordered_text,
            vec!["First paragraph.", "Second paragraph.", "Third paragraph."]
        );
        assert!(blocks
            .iter()
            .all(|block| block.kind == ExtractedArticleBlockKind::Paragraph));
    }

    #[test]
    fn extracted_article_into_page_model_preserves_block_order() {
        let article = ExtractedArticle {
            title: Some(String::from("Example article")),
            url: Some(String::from("https://example.com/article")),
            blocks: vec![
                ExtractedArticleBlock {
                    block_id: String::from("dom-block-title"),
                    kind: ExtractedArticleBlockKind::Title,
                    text: String::from("Example article"),
                },
                ExtractedArticleBlock {
                    block_id: String::from("dom-block-1"),
                    kind: ExtractedArticleBlockKind::Paragraph,
                    text: String::from("First paragraph."),
                },
                ExtractedArticleBlock {
                    block_id: String::from("dom-block-2"),
                    kind: ExtractedArticleBlockKind::Heading,
                    text: String::from("Section heading"),
                },
            ],
            interactive_elements: Vec::new(),
        };

        let page_model = article.into_page_model();

        assert_eq!(page_model.regions.len(), 3);
        assert_eq!(page_model.regions[0].region_id, "dom-block-title");
        assert_eq!(page_model.regions[0].label.as_deref(), Some("Title"));
        assert_eq!(page_model.regions[1].region_id, "dom-block-1");
        assert_eq!(page_model.regions[1].label, None);
        assert_eq!(page_model.regions[2].region_id, "dom-block-2");
        assert_eq!(page_model.regions[2].label.as_deref(), Some("Heading"));
        assert!(page_model
            .regions
            .iter()
            .all(|region| region.source == RegionSource::Dom));
    }

    #[cfg(feature = "browser")]
    #[test]
    fn extract_structured_article_from_html_uses_dom_smoothie_article_content() {
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

        let article = extract_structured_article_from_html(
            html,
            Some("https://example.com/article"),
            Vec::new(),
        )
        .expect("dom_smoothie extraction should succeed");

        assert_eq!(article.title.as_deref(), Some("Example article"));
        assert_eq!(
            article.url.as_deref(),
            Some("https://example.com/article")
        );
        assert!(!article.blocks.is_empty());
        assert!(article
            .blocks
            .iter()
            .any(|block| block.text.contains("First paragraph")));
    }

    #[cfg(feature = "browser")]
    #[test]
    fn extract_structured_article_from_html_builds_target_page_model_shape() {
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

        let article = extract_structured_article_from_html(
            html,
            Some("https://example.com/article"),
            interactive_elements.clone(),
        )
        .expect("dom_smoothie extraction should succeed");

        assert_eq!(article.title.as_deref(), Some("Example article"));
        assert_eq!(
            article.url.as_deref(),
            Some("https://example.com/article")
        );
        assert_eq!(article.interactive_elements, interactive_elements);

        assert!(!article.blocks.is_empty());
        let title_blocks = article
            .blocks
            .iter()
            .filter(|block| block.text == "Example article")
            .collect::<Vec<_>>();
        assert!(title_blocks.len() <= 1);
        if let Some(title_block) = title_blocks.first() {
            assert_eq!(title_block.kind, ExtractedArticleBlockKind::Title);
        }

        let body_block_text = article
            .blocks
            .iter()
            .filter(|block| block.text != "Example article")
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>();
        assert!(!body_block_text.is_empty());
        let combined_body_text = body_block_text.join("\n");
        let first_index = combined_body_text
            .find("First paragraph.")
            .expect("first paragraph should be present");
        let second_index = combined_body_text
            .find("Second paragraph.")
            .expect("second paragraph should be present");
        assert!(first_index < second_index);
        assert!(article
            .blocks
            .iter()
            .all(|block| {
                matches!(
                    block.kind,
                    ExtractedArticleBlockKind::Title | ExtractedArticleBlockKind::Paragraph
                )
            }));
    }
}
