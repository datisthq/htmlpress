use textlane::{FrozenProcessor, Processor, html};

use crate::models::plugin::WrapMath;

// Where text is code, already a link, or not prose a reader sees as such
const LITERAL: &[&str] = &[
    "a",
    "code",
    "pre",
    "kbd",
    "samp",
    "script",
    "style",
    "textarea",
    "title",
    "math",
    "svg",
    "htmlpress-math",
    "htmlpress-terminal",
    "htmlpress-mermaid",
];

thread_local! {
    static DOCUMENT: Option<FrozenProcessor> = create_processor(false).ok();
    static FRAGMENT: Option<FrozenProcessor> = create_processor(true).ok();
}

/// Run a page's HTML through textlane: bare URLs, emails and `:emoji:` shortcodes in its
/// prose, `$…$` math rendered as MathML in `<htmlpress-math>`, and an id on every
/// heading from `<h2>` down that has none, never one the page already uses. Text inside
/// links, code and any element with `data-literal` stays as written; invalid LaTeX fails.
pub(crate) fn transform_page(html: &str) -> Result<String, String> {
    let document = is_document(html);
    let output = match document {
        true => DOCUMENT.with(|processor| run(processor.as_ref(), html)),
        false => FRAGMENT.with(|processor| run(processor.as_ref(), html)),
    }?;
    Ok(output)
}

fn run(processor: Option<&FrozenProcessor>, html: &str) -> Result<String, String> {
    let processor = processor.ok_or("the page pipeline failed to build")?;
    let output = processor.process(html).map_err(|error| error.message)?;
    // KaTeX reports bad LaTeX as a message and draws it red; a page must not ship that
    let math = output.file.messages.iter().find(|message| {
        matches!(message.rule.as_deref(), Some("parseerror" | "rangeerror"))
    });
    if let Some(message) = math {
        return Err(format!("invalid math: {}", message.reason));
    }
    Ok(name_references(&output.value))
}

// textlane writes `&`, `<`, `"` and `'` as hexadecimal references; htmlpress reads
// attribute values as written, so a `&#x27;` would show in a description, escaped twice
pub(crate) fn name_references(html: &str) -> String {
    html.replace("&#x26;", "&amp;")
        .replace("&#x3C;", "&lt;")
        .replace("&#x22;", "&quot;")
        .replace("&#x27;", "'")
}

fn create_processor(fragment: bool) -> Result<FrozenProcessor, textlane::Error> {
    let ignore = Some(
        LITERAL
            .iter()
            .map(|tag| (*tag).to_owned())
            .collect::<Vec<_>>(),
    );
    let ignore_attributes = vec!["data-literal".to_owned()];
    Processor::new()
        .with(html::Parse {
            fragment,
            ..html::Parse::default()
        })
        .with(html::Gemoji {
            ignore: ignore.clone(),
            ignore_attributes: ignore_attributes.clone(),
            ..html::Gemoji::default()
        })
        .with(html::Linkify {
            ignore: ignore.clone(),
            ignore_attributes: ignore_attributes.clone(),
        })
        .with(html::Math {
            ignore,
            ignore_attributes,
            ..html::Math::default()
        })
        .with(WrapMath)
        .with(html::Katex {
            output: Some(html::KatexOutput::Mathml),
            ..html::Katex::default()
        })
        .with(html::Slug {
            reserve_existing: true,
            ranks: Some(vec![2, 3, 4, 5, 6]),
            ..html::Slug::default()
        })
        .with(html::Stringify {
            close_empty_elements: true,
            ..html::Stringify::default()
        })
        .freeze()
}

// A page with its own `<html>` or `<head>` is a document; anything else a fragment, whose
// leading `<title>` and metas stay where they are
fn is_document(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    ["<html", "<head"].iter().any(|tag| {
        lower.match_indices(tag).any(|(at, _)| {
            (lower.as_bytes().get(at + tag.len())).is_some_and(|next| {
                matches!(next, b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/')
            })
        })
    })
}

#[cfg(test)]
#[path = "transform_unit.rs"]
mod unit;
