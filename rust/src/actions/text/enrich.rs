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
    {
        return raw.to_owned();
    }
    let text = decode_entities(raw);
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(false);
    let mut html = String::new();
    let mut changed = false;
    let mut last = 0;
    for link in finder.links(&text) {
        let url = link.as_str();
        let href = match link.kind() {
            LinkKind::Email => format!("mailto:{url}"),
            _ if url.contains("://") => url.to_owned(),
            // A bare domain could be a file name (`index.html`); only `www.` reads as a site
            _ if url.starts_with("www.") => format!("https://{url}"),
            _ => continue,
        };
        html.push_str(&replace_shortcodes(&text[last..link.start()], &mut changed));
        html.push_str(&format!(
            "<a href=\"{}\">{}</a>",
            escape_text(&href),
            escape_text(url)
        ));
        changed = true;
        last = link.end();
    }
    html.push_str(&replace_shortcodes(&text[last..], &mut changed));
    match changed {
        true => html,
        false => raw.to_owned(),
    }
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
