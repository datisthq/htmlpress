use std::cell::RefCell;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

use crate::language_icons::{LANGUAGE_ALIASES, LANGUAGE_ICONS};

/// Frame every `<pre data-title="…">` in a figure whose caption is the block's title bar,
/// with its language's logo, highlighted or not; give every `<pre data-collapse="N">` the
/// height of N lines, which the script then offers to expand.
pub(crate) fn add_titles(html: &str) -> Result<String, String> {
    if !html.contains("data-title") && !html.contains("data-collapse") {
        return Ok(html.to_owned());
    }
    let languages = list_languages(html)?;
    let index = RefCell::new(0usize);
    let settings = RewriteStrSettings::new().append_element_content_handler(element!("pre", |el| {
        let language = languages.get(*index.borrow()).cloned().flatten();
        *index.borrow_mut() += 1;
        if let Some(lines) = el.get_attribute("data-collapse") {
            let lines: u32 = lines.trim().parse().ok().filter(|lines| *lines > 0).ok_or_else(|| {
                format!("data-collapse must be a number of lines, got \"{lines}\"")
            })?;
            el.set_attribute("style", &format!("--htmlpress-code-lines: {lines}"))?;
        }
        let Some(title) = el.get_attribute("data-title") else {
            return Ok(());
        };
        // Attribute values come still entity-encoded; only a bare < could open a tag
        let title = title.replace('<', "&lt;");
        let icon = create_icon(language.as_deref().unwrap_or_default());
        el.before(
            &format!("<figure class=\"htmlpress-code-figure\"><figcaption>{icon}{title}</figcaption>"),
            ContentType::Html,
        );
        el.after("</figure>", ContentType::Html);
        Ok(())
    }));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

// For each <pre> in document order, the language its <code> names
fn list_languages(html: &str) -> Result<Vec<Option<String>>, String> {
    let languages: RefCell<Vec<Option<String>>> = RefCell::new(vec![]);
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("pre", |_| {
            languages.borrow_mut().push(None);
            Ok(())
        }))
        .append_element_content_handler(element!("pre > code[class]", |el| {
            let class = el.get_attribute("class").unwrap_or_default();
            let language = (class.split_whitespace()).find_map(|name| {
                name.strip_prefix("language-")
                    .or(name.strip_prefix("lang-"))
            });
            if let Some(last) = languages.borrow_mut().last_mut() {
                *last = language.map(str::to_lowercase);
            }
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(languages.into_inner())
}

// A language without a logo of its own gets the generic file
fn create_icon(language: &str) -> String {
    let name = (LANGUAGE_ALIASES.iter())
        .find(|(alias, _)| *alias == language)
        .map_or(language, |(_, name)| name);
    let found = LANGUAGE_ICONS.binary_search_by(|(icon, _, _)| (*icon).cmp(name));
    let index = found.or_else(|_| {
        LANGUAGE_ICONS.binary_search_by(|(icon, _, _)| (*icon).cmp("default"))
    });
    let Some((_, view_box, path)) =
        index.ok().and_then(|index| LANGUAGE_ICONS.get(index))
    else {
        return String::new();
    };
    format!(
        "<svg viewBox=\"{view_box}\" fill=\"currentColor\" aria-hidden=\"true\"><path d=\"{path}\"/></svg>"
    )
}

#[cfg(test)]
#[path = "title_unit.rs"]
mod unit;
