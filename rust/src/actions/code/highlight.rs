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

type Lines = Vec<std::ops::RangeInclusive<usize>>;

#[derive(Clone)]
struct Block {
    language: String,
    lines: Lines,
    numbers: Option<isize>,
    /// Each line mark: the attribute it sets on the line, its value and the lines.
    marks: Vec<(&'static str, &'static str, Lines)>,
    /// Lines left at full strength while the rest are dimmed; empty for no focus.
    focus: Lines,
    words: Vec<String>,
}

// The `data-*` on a block's <code> that mark lines, and what each sets on them
const LINE_MARKS: &[(&str, &str, &str)] = &[
    ("data-add", "data-diff", "add"),
    ("data-remove", "data-diff", "remove"),
    ("data-error", "data-severity", "error"),
    ("data-warning", "data-severity", "warning"),
];

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
            let mut marks = vec![];
            for (attribute, name, value) in LINE_MARKS {
                if let Some(text) = el.get_attribute(attribute) {
                    let lines = read_lines(&text)
                        .map_err(|error| error.replace("data-lines", attribute))?;
                    marks.push((*name, *value, lines));
                }
            }
            let focus = el
                .get_attribute("data-focus")
                .map_or(Ok(vec![]), |text| read_lines(&text))
                .map_err(|error| error.replace("data-lines", "data-focus"))?;
            let words = (el
                .get_attribute("data-words")
                .unwrap_or_default()
                .split(','))
            .map(str::trim)
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect();
            blocks.borrow_mut().push(language.map(|language| Block {
                language,
                lines,
                numbers,
                marks,
                focus,
                words,
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
    let html = mark_lines(&html[open..close], block);
    Some(mark_words(&html, &block.words))
}

const LINE_START: &str = "<span class=\"giallo-l\"";

// giallo writes each line as a sibling `<span class="giallo-l">`, in order, so the nth
// one is line n; a mark is an attribute on it that the theme styles
fn mark_lines(html: &str, block: &Block) -> String {
    if block.marks.is_empty() && block.focus.is_empty() {
        return html.to_owned();
    }
    let mut output = String::with_capacity(html.len());
    for (line, part) in html.split(LINE_START).enumerate() {
        if line == 0 {
            output.push_str(part);
            continue;
        }
        output.push_str(LINE_START);
        for (name, value, lines) in &block.marks {
            if lines.iter().any(|range| range.contains(&line)) {
                output.push_str(&format!(" {name}=\"{value}\""));
            }
        }
        if !block.focus.is_empty()
            && !block.focus.iter().any(|range| range.contains(&line))
        {
            output.push_str(" data-dim");
        }
        output.push_str(part);
    }
    output
}

// Only in the text between tags, so a word never matches an attribute or a tag name; a
// word split across two tokens by the grammar is not marked
fn mark_words(html: &str, words: &[String]) -> String {
    if words.is_empty() {
        return html.to_owned();
    }
    // Attribute values come entity-encoded or not, so each is decoded, then escaped as the
    // rendered tokens are
    let escaped: Vec<String> = (words.iter())
        .map(|word| escape_text(&decode_entities(word)))
        .collect();
    let mut output = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        output.push_str(&mark_text(&rest[..open], &escaped));
        let close = rest[open..]
            .find('>')
            .map_or(rest.len(), |end| open + end + 1);
        output.push_str(&rest[open..close]);
        rest = &rest[close..];
    }
    output.push_str(&mark_text(rest, &escaped));
    output
}

fn mark_text(text: &str, words: &[String]) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        let next = (words.iter())
            .filter_map(|word| rest.find(word.as_str()).map(|at| (at, word)))
            .min_by_key(|(at, word)| (*at, std::cmp::Reverse(word.len())));
        let Some((at, word)) = next else { break };
        output.push_str(&rest[..at]);
        output.push_str(&format!("<mark>{word}</mark>"));
        rest = &rest[at + word.len()..];
    }
    output.push_str(rest);
    output
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
