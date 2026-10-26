use std::cell::{Cell, RefCell};
use std::rc::Rc;

use linkify::{LinkFinder, LinkKind};
use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, doc_text, element, rewrite_str};

// Where text is code, already a link, or not prose a reader sees as such
const LITERAL: &str = "a, code, pre, kbd, samp, script, style, textarea, title, \
                       htmlpress-terminal, htmlpress-mermaid, [data-literal]";

/// Turn the bare URLs and emails in a page's prose into links, and GitHub's emoji
/// shortcodes (`:rocket:`) into emoji, as GitHub-flavored Markdown does. Text inside
/// links, code, and any element with `data-literal` stays as written.
pub(crate) fn enrich_text(html: &str) -> Result<String, String> {
    let literal = Rc::new(Cell::new(0usize));
    let buffer = RefCell::new(String::new());
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!(LITERAL, |el| {
            // A void element (`<img data-literal>`) holds no text and never ends, so it
            // must not open a literal stretch
            let Some(handlers) = el.end_tag_handlers() else {
                return Ok(());
            };
            literal.set(literal.get() + 1);
            let literal = literal.clone();
            let handler: lol_html::EndTagHandler<'static> = Box::new(move |_| {
                literal.set(literal.get().saturating_sub(1));
                Ok(())
            });
            handlers.push(handler);
            Ok(())
        }))
        // lol_html may cut one text node into several chunks, so a node is gathered whole
        // before a URL or a shortcode in it is looked for
        .append_document_content_handler(doc_text!(|chunk| {
            if literal.get() > 0 {
                return Ok(());
            }
            buffer.borrow_mut().push_str(chunk.as_str());
            let last = chunk.last_in_text_node();
            chunk.remove();
            if last {
                let text = std::mem::take(&mut *buffer.borrow_mut());
                chunk.after(&enrich_node(&text), ContentType::Html);
            }
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

// The node as written when it holds nothing to change, so its entities stay exactly
fn enrich_node(raw: &str) -> String {
    if !raw.contains("://")
        && !raw.contains("www.")
        && !raw.contains('@')
        && !raw.contains(':')
        && !raw.contains('$')
    {
        return raw.to_owned();
    }
    let text = decode_entities(raw);
    let mut html = String::new();
    let mut changed = false;
    for segment in split_math(&text) {
        match segment {
            Segment::Text(text) => html.push_str(&link_text(&text, &mut changed)),
            Segment::Math(latex, display) => {
                let display = if display { " display" } else { "" };
                html.push_str(&format!(
                    "<htmlpress-math{display}>{}</htmlpress-math>",
                    escape_text(&latex)
                ));
                changed = true;
            }
            Segment::Dollar => {
                html.push('$');
                changed = true;
            }
        }
    }
    match changed {
        true => html,
        false => raw.to_owned(),
    }
}

fn link_text(text: &str, changed: &mut bool) -> String {
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(false);
    let mut html = String::new();
    let mut last = 0;
    for link in finder.links(text) {
        let url = link.as_str();
        let href = match link.kind() {
            LinkKind::Email => format!("mailto:{url}"),
            _ if url.contains("://") => url.to_owned(),
            // A bare domain could be a file name (`index.html`); only `www.` reads as a site
            _ if url.starts_with("www.") => format!("https://{url}"),
            _ => continue,
        };
        html.push_str(&replace_shortcodes(&text[last..link.start()], changed));
        html.push_str(&format!(
            "<a href=\"{}\">{}</a>",
            escape_text(&href),
            escape_text(url)
        ));
        *changed = true;
        last = link.end();
    }
    html.push_str(&replace_shortcodes(&text[last..], changed));
    html
}

enum Segment {
    Text(String),
    /// LaTeX source, and whether it is display math (`$$`).
    Math(String, bool),
    /// An escaped `\$`, a literal dollar sign.
    Dollar,
}

// Livemark's `$…$` and `$$…$$`, read by pandoc's rule so prices stay text: an opening `$`
// is followed by a non-space, a closing one follows a non-space and is not followed by a
// digit, so neither `$5 and $10` nor `costs $ 5` is math
fn split_math(text: &str) -> Vec<Segment> {
    let mut segments = vec![];
    let mut plain = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('$') {
        plain.push_str(&rest[..at]);
        let after = &rest[at..];
        if plain.ends_with('\\') {
            plain.pop();
            segments.push(Segment::Text(std::mem::take(&mut plain)));
            segments.push(Segment::Dollar);
            rest = &after[1..];
            continue;
        }
        let found = match after.strip_prefix("$$") {
            Some(inner) => inner.find("$$").and_then(|end| {
                let latex = inner[..end].trim();
                (!latex.is_empty()).then(|| (latex.to_owned(), true, end + 4))
            }),
            None => find_inline(&after[1..])
                .map(|(latex, end)| (latex.to_owned(), false, end + 1)),
        };
        match found {
            Some((latex, display, length)) => {
                segments.push(Segment::Text(std::mem::take(&mut plain)));
                segments.push(Segment::Math(latex, display));
                rest = &after[length..];
            }
            None => {
                plain.push('$');
                rest = &after[1..];
            }
        }
    }
    plain.push_str(rest);
    segments.push(Segment::Text(plain));
    segments
}

// The LaTeX of an inline `$…$` starting right after its opening `$`, and how far its
// closing `$` reaches
fn find_inline(text: &str) -> Option<(&str, usize)> {
    if text.starts_with(char::is_whitespace) || text.is_empty() {
        return None;
    }
    let mut search = 0;
    while let Some(offset) = text[search..].find('$') {
        let end = search + offset;
        let before = text[..end].chars().next_back();
        let after = text[end + 1..].chars().next();
        let closes = before.is_some_and(|char| !char.is_whitespace() && char != '\\')
            && !after.is_some_and(|char| char.is_ascii_digit());
        if closes && end > 0 {
            return Some((&text[..end], end + 1));
        }
        search = end + 1;
    }
    None
}

// `:name:` with a GitHub shortcode becomes its emoji; anything else, `10:30:00` included,
// stays as text
fn replace_shortcodes(text: &str, changed: &mut bool) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find(':') {
        let after = &rest[open + 1..];
        let name_end = after
            .find(|char: char| !(char.is_ascii_alphanumeric() || "_+-".contains(char)));
        let emoji = match name_end {
            Some(end) if end > 0 && after[end..].starts_with(':') => {
                emojis::get_by_shortcode(&after[..end]).map(|emoji| (emoji, end))
            }
            _ => None,
        };
        match emoji {
            Some((emoji, end)) => {
                output.push_str(&escape_text(&rest[..open]));
                output.push_str(emoji.as_str());
                *changed = true;
                rest = &after[end + 1..];
            }
            None => {
                output.push_str(&escape_text(&rest[..=open]));
                rest = after;
            }
        }
    }
    output.push_str(&escape_text(rest));
    output
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", "\u{a0}")
        .replace("&amp;", "&")
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "enrich_unit.rs"]
mod unit;
