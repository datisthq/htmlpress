use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

/// Markdown as HTML, with GitHub's tables, task lists, strikethrough and footnotes.
pub(crate) fn render_markdown(markdown: &str) -> String {
    let mut output = String::new();
    html::push_html(&mut output, Parser::new_ext(markdown, OPTIONS));
    output
}

/// Markdown written by someone else, such as a fetched release's notes: its raw HTML shows
/// as text and a link or image to a script URL loses it, so it cannot run in the site.
pub(crate) fn render_untrusted_markdown(markdown: &str) -> String {
    let events = Parser::new_ext(markdown, OPTIONS).map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: drop_script(dest_url),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: drop_script(dest_url),
            title,
            id,
        }),
        event => event,
    });
    let mut output = String::new();
    html::push_html(&mut output, events);
    output
}

const OPTIONS: Options = Options::ENABLE_TABLES
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_FOOTNOTES);

// Browsers ignore whitespace and control characters inside a scheme, `java\tscript:`
fn drop_script(url: CowStr) -> CowStr {
    let scheme: String = (url.chars())
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .take_while(|c| *c != ':')
        .collect::<String>()
        .to_ascii_lowercase();
    let plain = url.contains(':')
        && ["javascript", "vbscript", "data"].contains(&scheme.as_str());
    match plain {
        true => CowStr::Borrowed("#"),
        false => url,
    }
}
