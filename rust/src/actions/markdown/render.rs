use pulldown_cmark::{Options, Parser, html};

/// Markdown as HTML, with GitHub's tables, task lists, strikethrough and footnotes.
pub(crate) fn render_markdown(markdown: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES;
    let mut output = String::new();
    html::push_html(&mut output, Parser::new_ext(markdown, options));
    output
}
