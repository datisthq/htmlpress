use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

/// Frame every `<pre data-title="…">` in a figure whose caption is the block's title bar,
/// highlighted or not.
pub(crate) fn add_titles(html: &str) -> Result<String, String> {
    if !html.contains("data-title") {
        return Ok(html.to_owned());
    }
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "pre[data-title]",
        |el| {
            let title = el.get_attribute("data-title").unwrap_or_default();
            // Attribute values come still entity-encoded; only a bare < could open a tag
            let title = title.replace('<', "&lt;");
            el.before(
                &format!("<figure class=\"htmlpress-code-figure\"><figcaption>{title}</figcaption>"),
                ContentType::Html,
            );
            el.after("</figure>", ContentType::Html);
            Ok(())
        }
    ));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "title_unit.rs"]
mod unit;
