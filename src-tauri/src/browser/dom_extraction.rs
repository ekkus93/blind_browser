#[cfg(feature = "browser")]
use chromiumoxide::Page;
use serde::Deserialize;
#[cfg(feature = "browser")]
use std::collections::BTreeMap;

use super::{BrowserController, BrowserError};
use crate::page_model::{
    ElementRole, InteractiveElement, PageModel, PageRegion, Rect, RegionRole, RegionSource,
};

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

// Module-level (rather than a local inside extract_live_page_model) so a
// test can assert on its content directly -- there is no live-browser test
// harness in this codebase to exercise the script end-to-end (see the
// coordinate-space regression test below for why that gap is accepted for
// now), so a string-content check on the script itself is the narrowest
// thing that still catches someone reintroducing the viewport-relative bug
// this script was fixed for.
#[cfg(feature = "browser")]
const EXTRACTION_SCRIPT: &str = r#"(() => {
        const normalizeText = (value) => {
            const text = String(value ?? '').replace(/\s+/g, ' ').trim();
            return text.length > 0 ? text : null;
        };

        // getBoundingClientRect() is viewport-relative (it moves as the page
        // scrolls); every bbox consumer (CDP Page.captureScreenshot's clip,
        // and Tesseract set_rectangle cropping a full-page raster) expects
        // document/page-absolute coordinates instead. Adding the current
        // scroll offset here, once, at the source, means every consumer
        // inherits the fix rather than each having to know to correct for
        // scroll itself.
        const documentAbsoluteRect = (rect) => ({
            x: rect.x + window.scrollX,
            y: rect.y + window.scrollY,
            width: rect.width,
            height: rect.height,
        });

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
            // Form-control values remain local private state and are never extracted.
            const value = null;
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
                bbox: isVisible(node) ? documentAbsoluteRect(rect) : null,
                visible: isVisible(node),
                enabled: isEnabled(node),
                attributes: Object.fromEntries(
                    Array.from(node.attributes)
                        .filter((attribute) => [
                            'type', 'role', 'aria-label', 'aria-labelledby',
                            'placeholder', 'checked', 'selected', 'disabled',
                            'name', 'autocomplete'
                        ].includes(attribute.name.toLowerCase()))
                        .map((attribute) => [attribute.name.toLowerCase(), attribute.value])
                )
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
                bbox: documentAbsoluteRect(node.getBoundingClientRect()),
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

#[cfg(feature = "browser")]
async fn extract_live_page_model(page: &Page) -> Result<PageModel, BrowserError> {
    let extracted = page
        .evaluate(EXTRACTION_SCRIPT)
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

// CR3 P1.3: region/element bboxes were being extracted viewport-relative
// (getBoundingClientRect(), which moves with scroll) but consumed as
// document-absolute (CDP Page.captureScreenshot's clip.x/y, confirmed
// empirically to always be document-absolute regardless of
// captureBeyondViewport; and Tesseract set_rectangle cropping a full-page
// raster whose pixel origin is the document origin). On a scrolled page,
// "read that section" silently captured/OCR'd the wrong part of the page.
//
// There is no live-browser (real CDP/chromiumoxide session) test harness
// anywhere in this codebase to exercise the fix end-to-end -- every
// browser-dependent test here uses a mock executor instead (see
// commands/tests/tool_dispatch/*.rs) -- so this is a narrower,
// string-content regression test on the actual extraction script: it
// catches someone reintroducing a raw getBoundingClientRect() bbox (the
// exact shape of this bug) without asserting anything about a live page.
// The empirical CDP verification that document-absolute clip coordinates
// are in fact what's needed (raw viewport-relative clip on a page scrolled
// 1500px landed on blank content; document-absolute clip landed on the
// intended element) was done once, ad hoc, during the review that found
// this bug, and is recorded in docs/BB_CODE_REVIEW3_TODO.md's P1.3 note
// rather than re-run on every test invocation.
#[cfg(all(test, feature = "browser"))]
mod tests {
    use super::EXTRACTION_SCRIPT;

    #[test]
    fn extraction_script_corrects_both_bbox_sources_for_scroll() {
        assert!(
            EXTRACTION_SCRIPT.contains("x: rect.x + window.scrollX"),
            "the extraction script must add window.scrollX to every bbox.x, \
             not just some -- see the documentAbsoluteRect helper"
        );
        assert!(
            EXTRACTION_SCRIPT.contains("y: rect.y + window.scrollY"),
            "the extraction script must add window.scrollY to every bbox.y, \
             not just some -- see the documentAbsoluteRect helper"
        );
        // Both the interactive-element bbox and the region bbox must route
        // through the shared correction helper -- not just one of the two
        // sources getBoundingClientRect() is called from in this script.
        // (The helper's own definition, `documentAbsoluteRect = (rect) =>`,
        // doesn't match this substring itself -- only call sites do.)
        assert_eq!(
            EXTRACTION_SCRIPT.matches("documentAbsoluteRect(").count(),
            2,
            "expected exactly two call sites of the helper (interactive-\
             element bbox, region bbox); a bbox source that stopped \
             calling it would silently reintroduce the viewport-relative \
             bug"
        );
        // No remaining raw `rect.x`/`rect.y` bbox construction outside the
        // helper itself: every occurrence of `rect.x`/`rect.y` in the
        // script must be inside documentAbsoluteRect's own body.
        let helper_start = EXTRACTION_SCRIPT
            .find("const documentAbsoluteRect")
            .expect("helper definition should exist");
        let helper_end = helper_start
            + EXTRACTION_SCRIPT[helper_start..]
                .find("});")
                .expect("helper definition should close")
            + "});".len();
        for needle in ["rect.x", "rect.y"] {
            let mut search_from = 0;
            while let Some(found) = EXTRACTION_SCRIPT[search_from..].find(needle) {
                let absolute = search_from + found;
                assert!(
                    absolute >= helper_start && absolute < helper_end,
                    "found a raw `{needle}` bbox reference at byte {absolute}, \
                     outside documentAbsoluteRect's body ({helper_start}..{helper_end}) -- \
                     every bbox construction must go through the scroll-correcting helper"
                );
                search_from = absolute + needle.len();
            }
        }
    }
}
