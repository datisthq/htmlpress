use textlane::{FrozenProcessor, Processor, html, md};

thread_local! {
    static TRUSTED: Option<FrozenProcessor> = create_processor(true).ok();
    static UNTRUSTED: Option<FrozenProcessor> = create_processor(false).ok();
}

/// Markdown as HTML, with GitHub's tables, task lists, strikethrough and footnotes; a
/// leading YAML frontmatter is dropped.
pub(crate) fn render_markdown(markdown: &str) -> String {
    TRUSTED.with(|processor| process(processor.as_ref(), markdown))
}

/// A release's notes fetched from GitHub, rendered as GitHub renders them: raw HTML kept,
/// then sanitized, so nothing in it can run in the site; headings raised one level and a
/// leading title dropped, as the release's own page already shows one.
pub(crate) fn render_untrusted_markdown(markdown: &str) -> String {
    UNTRUSTED.with(|processor| process(processor.as_ref(), markdown))
}

// A processor that failed to build, or a document it fails on, shows as escaped text
// rather than vanishing
fn process(processor: Option<&FrozenProcessor>, markdown: &str) -> String {
    processor
        .and_then(|processor| processor.process(markdown).ok())
        .map(|output| output.value)
        .unwrap_or_else(|| format!("<pre>{}</pre>", escape_text(markdown)))
}

fn create_processor(trusted: bool) -> Result<FrozenProcessor, textlane::Error> {
    let processor = Processor::new()
        .with(md::Parse::default())
        .with(md::Frontmatter {
            strip: true,
            ..md::Frontmatter::default()
        })
        .with(md::Gfm::default());
    // On the tree, before sanitizing: sanitized HTML edited as a string could be split
    // inside an attribute and let markup through
    let processor = match trusted {
        true => processor,
        // A title `##` becomes an `h1` only once raised, so it is dropped after
        false => processor
            .with(md::ShiftHeading {
                shift: -1,
                drop_title: false,
            })
            .with(md::ShiftHeading {
                shift: 0,
                drop_title: true,
            }),
    };
    let processor = processor.with(md::ToHtml {
        allow_dangerous_html: true,
        ..md::ToHtml::default()
    });
    let processor = match trusted {
        true => processor,
        // Reparsed first: sanitizing raw nodes as strings would let them through whole
        false => processor
            .with(html::Raw::default())
            .with(html::Sanitize::default()),
    };
    processor
        .with(html::Stringify {
            allow_dangerous_html: trusted,
            ..html::Stringify::default()
        })
        .freeze()
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
#[path = "render_unit.rs"]
mod unit;
