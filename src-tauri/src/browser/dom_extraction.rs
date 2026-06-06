#[cfg(feature = "browser")]
use std::collections::BTreeMap;
#[cfg(feature = "browser")]
use chromiumoxide::Page;
use serde::Deserialize;

use super::{BrowserController, BrowserError};
use crate::page_model::{ElementRole, InteractiveElement, PageModel, PageRegion, Rect, RegionRole, RegionSource};

impl BrowserController {
    pub fn extract_page_model(&mut self) -> Result<PageModel, BrowserError> {
        #[cfg(feature = "browser")]
        {
            let session = self.ensure_session()?;
            let page = session.page.clone().ok_or(BrowserError::NoActivePage)?;
            tauri::async_runtime::block_on(extract_live_page_model(&page))
        }

        #[cfg(not(feature = "browser"))]
        {
            Err(BrowserError::FeatureDisabled)
        }
    }

}

#[derive(Debug, Deserialize)]
struct LiveExtractedPage {
    title: Option<String>,
    url: Option<String>,
    regions: Vec<LiveExtractedRegion>,
    interactive_elements: Vec<LiveExtractedInteractiveElement>,
}
#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveExtractedRegion {
    region_id: String,
    role: String,
    label: Option<String>,
    text: String,
    bbox: Option<Rect>,
    source: String,
}

#[cfg(feature = "browser")]
#[derive(Debug, Deserialize)]
struct LiveExtractedInteractiveElement {
    element_id: String,
    dom_locator: Option<String>,
    role: String,
    tag_name: String,
    text: Option<String>,
    accessible_name: Option<String>,
    placeholder: Option<String>,
    href: Option<String>,
    value: Option<String>,
    bbox: Option<Rect>,
    visible: bool,
    enabled: bool,
    attributes: BTreeMap<String, String>,
}

#[cfg(feature = "browser")]
async fn extract_live_page_model(page: &Page) -> Result<PageModel, BrowserError> {
    let evaluation = r#"(() => {
        const normalizeText = (value) => {
            const text = String(value ?? '').replace(/\s+/g, ' ').trim();
            return text.length > 0 ? text : null;
        };

        const isVisible = (node) => {
            if (!(node instanceof Element)) {
                return false;
            }
            const style = window.getComputedStyle(node);
            if (style.display === 'none' || style.visibility === 'hidden' || style.visibility === 'collapse') {
                return false;
            }
            const rect = node.getBoundingClientRect();
            return rect.width > 0 && rect.height > 0;
        };

        const isEnabled = (node) => {
            if (!(node instanceof Element)) {
                return false;
            }
            return !node.hasAttribute('disabled') && node.getAttribute('aria-disabled') !== 'true';
        };

        const collectLabelText = (node) => {
            if (!(node instanceof Element) || !('labels' in node) || !node.labels) {
                return null;
            }
            return normalizeText(Array.from(node.labels).map((label) => label.textContent ?? '').join(' '));
        };

        const accessibleNameFor = (node) => {
            if (!(node instanceof Element)) {
                return null;
            }
            const ariaLabel = normalizeText(node.getAttribute('aria-label'));
            if (ariaLabel) {
                return ariaLabel;
            }
            const labelledBy = normalizeText(node.getAttribute('aria-labelledby'));
            if (labelledBy) {
                const referenced = labelledBy
                    .split(' ')
                    .map((id) => document.getElementById(id))
                    .filter(Boolean)
                    .map((element) => element.textContent ?? '')
                    .join(' ');
                const normalized = normalizeText(referenced);
                if (normalized) {
                    return normalized;
                }
            }
            const labelText = collectLabelText(node);
            if (labelText) {
                return labelText;
            }
            return (
                normalizeText(node.getAttribute('title')) ||
                normalizeText(node.getAttribute('alt')) ||
                normalizeText(node.innerText) ||
                normalizeText(node.textContent)
            );
        };

        const uniqueSelector = (node) => {
            if (!(node instanceof Element)) {
                return null;
            }
            if (node.id) {
                const idSelector = `#${CSS.escape(node.id)}`;
                if (document.querySelectorAll(idSelector).length === 1) {
                    return idSelector;
                }
            }

            const parts = [];
            let current = node;
            while (current && current.nodeType === Node.ELEMENT_NODE) {
                const tagName = current.tagName.toLowerCase();
                let part = tagName;
                if (current.parentElement) {
                    const sameTagSiblings = Array.from(current.parentElement.children)
                        .filter((sibling) => sibling.tagName === current.tagName);
                    if (sameTagSiblings.length > 1) {
                        part += `:nth-of-type(${sameTagSiblings.indexOf(current) + 1})`;
                    }
                }
                parts.unshift(part);
                const selector = parts.join(' > ');
                if (document.querySelectorAll(selector).length === 1) {
                    return selector;
                }
                current = current.parentElement;
            }

            return parts.length > 0 ? parts.join(' > ') : null;
        };

        const roleFor = (node) => {
            if (!(node instanceof Element)) {
                return 'Other';
            }
            const explicitRole = normalizeText(node.getAttribute('role'));
            switch (explicitRole) {
                case 'link': return "Link";
                case 'button': return "Button";
                case 'textbox': return "Input";
                case 'combobox': return "Select";
                case 'checkbox': return "Checkbox";
                case 'radio': return "Radio";
                case 'form': return "Form";
                case 'navigation':
                case 'main':
                case 'banner':
                case 'contentinfo': return "Landmark";
                default: break;
            }

            const tagName = node.tagName.toLowerCase();
            if (tagName === 'a' && node.hasAttribute('href')) return "Link";
            if (tagName === 'button') return "Button";
            if (tagName === 'textarea') return "TextArea";
            if (tagName === 'select') return "Select";
            if (tagName === 'form') return "Form";
            if (tagName === 'input') {
                const type = normalizeText(node.getAttribute('type'));
                if (type === 'checkbox') return "Checkbox";
                if (type === 'radio') return "Radio";
                return "Input";
            }
            return 'Other';
        };

        const interactiveSelector = [
            'a[href]',
            'button',
            'input',
            'textarea',
            'select',
            'form',
            '[role="button"]',
            '[role="link"]',
            '[role="textbox"]',
            '[role="combobox"]',
            '[role="checkbox"]',
            '[role="radio"]',
            '[role="form"]',
            '[role="navigation"]',
            '[role="main"]',
            '[role="banner"]',
            '[role="contentinfo"]'
        ].join(',');

        const interactive_elements = Array.from(document.querySelectorAll(interactiveSelector)).map((node, index) => {
            const rect = node.getBoundingClientRect();
            const text = normalizeText(node.innerText || node.textContent);
            const placeholder = normalizeText(node.getAttribute('placeholder'));
            const href = 'href' in node ? normalizeText(node.href) : normalizeText(node.getAttribute('href'));
            const value = 'value' in node ? normalizeText(node.value) : null;
            return {
                element_id: `element-${index + 1}`,
                dom_locator: uniqueSelector(node),
                role: roleFor(node),
                tag_name: node.tagName.toLowerCase(),
                text,
                accessible_name: accessibleNameFor(node),
                placeholder,
                href,
                value,
                bbox: isVisible(node) ? {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                } : null,
                visible: isVisible(node),
                enabled: isEnabled(node),
                attributes: Object.fromEntries(Array.from(node.attributes).map((attribute) => [attribute.name, attribute.value]))
            };
        });

        const regionCandidates = Array.from(document.querySelectorAll('main, article, section, nav, aside, p, li, blockquote, pre, h1, h2, h3, h4, h5, h6'));
        const seenTexts = new Set();
        const regions = [];
        const regionRoleFor = (node) => {
            if (!(node instanceof Element)) {
                return 'Other';
            }
            const tagName = node.tagName.toLowerCase();
            if (tagName === 'h1' || tagName === 'h2' || tagName === 'h3' || tagName === 'h4' || tagName === 'h5' || tagName === 'h6') {
                return 'Heading';
            }
            if (tagName === 'p' || tagName === 'li' || tagName === 'blockquote' || tagName === 'pre') {
                return 'Paragraph';
            }
            if (tagName === 'main' || tagName === 'article' || tagName === 'section' || tagName === 'nav' || tagName === 'aside') {
                return 'Section';
            }
            return 'Other';
        };
        for (const node of regionCandidates) {
            if (!isVisible(node)) {
                continue;
            }
            const text = normalizeText(node.innerText || node.textContent);
            if (!text || seenTexts.has(text)) {
                continue;
            }
            seenTexts.add(text);
            regions.push({
                region_id: `dom-region-${regions.length + 1}`,
                role: regionRoleFor(node),
                label: normalizeText(node.getAttribute('aria-label')),
                text,
                bbox: (() => {
                    const rect = node.getBoundingClientRect();
                    return {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    };
                })(),
                source: 'Dom'
            });
        }

        return {
            title: normalizeText(document.title),
            url: normalizeText(window.location.href),
            regions,
            interactive_elements,
        };
    })()"#;

    let extracted = page
        .evaluate(evaluation)
        .await
        .map_err(|error| BrowserError::Inspect(error.to_string()))?
        .into_value::<LiveExtractedPage>()
        .map_err(|error| BrowserError::Inspect(error.to_string()))?;

    Ok(PageModel {
        title: extracted.title,
        url: extracted.url,
        regions: extracted
            .regions
            .into_iter()
            .map(|region| PageRegion {
                region_id: region.region_id,
                role: match region.role.as_str() {
                    "Heading" => RegionRole::Heading,
                    "Paragraph" => RegionRole::Paragraph,
                    "Section" => RegionRole::Section,
                    _ => RegionRole::Other,
                },
                label: region.label,
                text: region.text,
                bbox: region.bbox,
                source: match region.source.as_str() {
                    "Mixed" => RegionSource::Mixed,
                    "Ocr" => RegionSource::Ocr,
                    _ => RegionSource::Dom,
                },
            })
            .collect(),
        interactive_elements: extracted
            .interactive_elements
            .into_iter()
            .map(|element| InteractiveElement {
                element_id: element.element_id,
                dom_locator: element.dom_locator.and_then(|locator| {
                    let trimmed = locator.trim();
                    (!trimmed.is_empty()).then(|| trimmed.to_string())
                }),
                role: match element.role.as_str() {
                    "Link" => ElementRole::Link,
                    "Button" => ElementRole::Button,
                    "Input" => ElementRole::Input,
                    "TextArea" => ElementRole::TextArea,
                    "Select" => ElementRole::Select,
                    "Checkbox" => ElementRole::Checkbox,
                    "Radio" => ElementRole::Radio,
                    "Form" => ElementRole::Form,
                    "Landmark" => ElementRole::Landmark,
                    _ => ElementRole::Other,
                },
                tag_name: element.tag_name,
                text: element.text,
                accessible_name: element.accessible_name,
                placeholder: element.placeholder,
                href: element.href,
                value: element.value,
                bbox: element.bbox,
                visible: element.visible,
                enabled: element.enabled,
                attributes: element.attributes,
            })
            .collect(),
    })
}
