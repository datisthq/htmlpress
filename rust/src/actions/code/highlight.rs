use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use giallo::{HighlightOptions, HtmlRenderer, Registry, RenderOptions, ThemeVariant};
use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str, text};

// Loaded once per process: the built-in grammars and themes are a 1 MB compressed dump
static REGISTRY: OnceLock<Option<Registry>> = OnceLock::new();

/// Color every `<pre><code class="language-…">` in a page's HTML with the light and dark
/// themes, as Shiki would; `data-lines="2,4-5"` highlights lines and `data-line-numbers`
/// (optionally the first number) numbers them. A block in an unknown language, or
/// holding markup of its own, is left as it is.
pub(crate) fn highlight_code(
    html: &str,
    light: &str,
    dark: &str,
) -> Result<String, String> {
    let Some(registry) = load_registry() else {
        return Ok(html.to_owned());
    };
    let blocks = list_blocks(html)?;
    if blocks.iter().all(Option::is_none) {
        return Ok(html.to_owned());
    }
    let index = RefCell::new(0usize);
    let current: Rc<RefCell<Option<(Block, String)>>> = Rc::new(RefCell::new(None));
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("pre > code", |el| {
            let block = blocks.get(*index.borrow()).cloned().flatten();
            *index.borrow_mut() += 1;
            let Some(block) = block else {
                return Ok(());
            };
            *current.borrow_mut() = Some((block, String::new()));
            let current = current.clone();
            let theme = ThemeVariant::Dual {
                light: light.to_owned(),
                dark: dark.to_owned(),
            };
            let handler: lol_html::EndTagHandler<'static> = Box::new(move |end| {
                if let Some((block, source)) = current.borrow_mut().take() {
                    let colored = color_source(registry, &block, &source, &theme);
                    end.before(&colored.unwrap_or(source), ContentType::Html);
                }
                Ok(())
            });
            el.on_end_tag(handler)
        }))
        .append_element_content_handler(text!("pre > code", |chunk| {
            if let Some((_, source)) = current.borrow_mut().as_mut() {
                source.push_str(chunk.as_str());
                chunk.remove();
            }
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

/// Whether the built-in set has a theme by this name, for checking the config.
pub(crate) fn has_theme(name: &str) -> bool {
    load_registry().is_none_or(|registry| registry.contains_theme(name))
}

fn load_registry() -> Option<&'static Registry> {
    let registry = REGISTRY.get_or_init(|| {
        let mut registry = Registry::builtin().ok()?;
        registry.link_grammars();
        Some(registry)
    });
    registry.as_ref()
}

#[derive(Clone)]
struct Block {
    language: String,
    lines: Vec<std::ops::RangeInclusive<usize>>,
    numbers: Option<isize>,
}

// For each `pre > code` in document order: its language, when it names one and holds only
// text, since a block with its own markup (`<mark>`, links) would lose it to the colors
fn list_blocks(html: &str) -> Result<Vec<Option<Block>>, String> {
    let blocks: RefCell<Vec<Option<Block>>> = RefCell::new(vec![]);
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("pre > code", |el| {
            let class = el.get_attribute("class").unwrap_or_default();
            let language = class
                .split_whitespace()
                .find_map(|name| {
                    name.strip_prefix("language-")
                        .or(name.strip_prefix("lang-"))
                })
                .map(str::to_owned);
            let lines = el
                .get_attribute("data-lines")
                .map_or(Ok(vec![]), |text| read_lines(&text))?;
            let numbers = match el.get_attribute("data-line-numbers") {
                Some(start) if start.trim().is_empty() => Some(1),
                Some(start) => Some(start.trim().parse().map_err(|_| {
                    format!(
                        "data-line-numbers must be a number or empty, got \"{start}\""
                    )
                })?),
                None => None,
            };
            blocks.borrow_mut().push(language.map(|language| Block {
                language,
                lines,
                numbers,
            }));
            Ok(())
        }))
        .append_element_content_handler(element!("pre > code *", |_| {
            if let Some(block) = blocks.borrow_mut().last_mut() {
                *block = None;
            }
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(blocks.into_inner())
}

// The token spans only: the page keeps its own `<pre><code>`, so the theme's frame and
// the copy button stay, and colors come from the spans
fn color_source(
    registry: &Registry,
    block: &Block,
    source: &str,
    theme: &ThemeVariant<String>,
) -> Option<String> {
    let theme = match theme {
        ThemeVariant::Dual { light, dark } => ThemeVariant::Dual {
            light: light.as_str(),
            dark: dark.as_str(),
        },
        ThemeVariant::Single(name) => ThemeVariant::Single(name.as_str()),
    };
    let text = decode_entities(source);
    let options = HighlightOptions::new(block.language.to_lowercase(), theme);
    let highlighted = registry
        .highlight(text.trim_end_matches('\n'), &options)
        .ok()?;
    let renderer = HtmlRenderer {
        add_color_scheme: false,
        ..HtmlRenderer::default()
    };
    let options = RenderOptions {
        show_line_numbers: block.numbers.is_some(),
        line_number_start: block.numbers.unwrap_or(1),
        highlight_lines: block.lines.clone(),
        ..RenderOptions::default()
    };
    let html = renderer.render(&highlighted, &options);
    let start = html.find("<code")?;
    let open = start + html[start..].find('>')? + 1;
    let close = html.rfind("</code>")?;
    Some(html[open..close].to_owned())
}

// `2,4-5`: single lines and inclusive ranges, counted from 1
fn read_lines(text: &str) -> Result<Vec<std::ops::RangeInclusive<usize>>, String> {
    let invalid = || format!("data-lines must be like \"2,4-5\", got \"{text}\"");
    let number = |part: &str| part.trim().parse::<usize>().ok().filter(|line| *line > 0);
    (text.split(','))
        .filter(|part| !part.trim().is_empty())
        .map(|part| match part.split_once('-') {
            Some((start, end)) => match (number(start), number(end)) {
                (Some(start), Some(end)) if start <= end => Ok(start..=end),
                _ => Err(invalid()),
            },
            None => number(part).map(|line| line..=line).ok_or_else(invalid),
        })
        .collect()
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&nbsp;", "\u{a0}")
        .replace("&amp;", "&")
}

#[cfg(test)]
#[path = "highlight_unit.rs"]
mod unit;
