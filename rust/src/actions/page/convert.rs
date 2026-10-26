use std::cell::Cell;

use htmd::element_handler::Handlers;
use htmd::options::{BulletListMarker, Options};
use htmd::{Element, HtmlToMarkdown};
use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

use crate::models::error::Error;
use crate::models::page::Page;

/// Render a page as the Markdown agents read: its title and description, then its content
/// with each `htmlpress-*` component reduced to the plain Markdown it stands for.
pub(crate) fn convert_page(page: &Page) -> crate::Result<String> {
    let html = simplify_components(&page.content).map_err(Error::page(&page.source))?;
    let converter = HtmlToMarkdown::builder()
        .options(Options {
            bullet_list_marker: BulletListMarker::Dash,
            ul_bullet_spacing: 1,
            ..Options::default()
        })
        .skip_tags(vec!["script", "style", "template", "svg"])
        // The LaTeX, not the MathML: `$…$` is what Markdown readers and agents understand,
        // and htmd would escape the source's `_` and `\\` if it went in as text
        .add_handler(
            vec!["htmlpress-math"],
            |_: &dyn Handlers, element: Element| {
                let latex = (element.attrs.iter())
                    .find(|attribute| &*attribute.name.local == "data-latex")
                    .map(|attribute| attribute.value.to_string())?;
                let display = element
                    .attrs
                    .iter()
                    .any(|attribute| &*attribute.name.local == "display");
                Some(match display {
                    true => format!("\n\n$$\n{latex}\n$$\n\n").into(),
                    false => format!("${latex}$").into(),
                })
            },
        )
        .build();
    let body = converter
        .convert(&html)
        .map_err(|error| Error::page(&page.source)(error.to_string()))?;
    let mut parts = vec![format!("# {}", page.title)];
    if let Some(description) = &page.description {
        parts.push(format!("> {description}"));
    }
    if !body.trim().is_empty() {
        parts.push(body.trim().to_owned());
    }
    Ok(format!("{}\n", parts.join("\n\n")))
}

/// Where a page's Markdown sits beside it: `guides/build.html` → `guides/build.md`.
pub(crate) fn create_markdown_path(path: &str) -> String {
    let stem = path.strip_suffix(".html").unwrap_or(path);
    format!("{stem}.md")
}

// The first <h1> repeats the title the Markdown already opens with
fn simplify_components(html: &str) -> Result<String, String> {
    let blocks = list_bare_blocks(html)?;
    let headings = Cell::new(0);
    let index = Cell::new(0);
    let settings = RewriteStrSettings::new()
        // htmd fences only a <pre> holding a <code>, which agents often leave out
        .append_element_content_handler(element!("pre", |el| {
            if blocks.get(index.get()).copied().unwrap_or(false) {
                el.prepend("<code>", ContentType::Html);
                el.append("</code>", ContentType::Html);
            }
            index.set(index.get() + 1);
            Ok(())
        }))
        .append_element_content_handler(element!("h1", |el| {
            headings.set(headings.get() + 1);
            if headings.get() == 1 {
                el.remove();
            }
            Ok(())
        }))
        .append_element_content_handler(element!("htmlpress-callout", |el| {
            let label = el.get_attribute("label").unwrap_or_else(|| {
                let tone = el
                    .get_attribute("type")
                    .unwrap_or_else(|| "info".to_owned());
                let mut chars = tone.chars();
                chars.next().map_or(String::new(), |first| {
                    first.to_uppercase().chain(chars).collect()
                })
            });
            el.set_tag_name("blockquote")?;
            el.prepend(
                &format!("<p><strong>{}</strong></p>", escape_text(&label)),
                ContentType::Html,
            );
            Ok(())
        }))
        .append_element_content_handler(element!(
            "htmlpress-tabs > section[title]",
            |el| {
                let title = el.get_attribute("title").unwrap_or_default();
                el.prepend(
                    &format!("<p><strong>{}</strong></p>", escape_text(&title)),
                    ContentType::Html,
                );
                Ok(())
            }
        ))
        // One image per spot: a dark-only twin would show twice in plain Markdown
        .append_element_content_handler(element!("img[data-theme='dark']", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("pre[data-title]", |el| {
            let title = el.get_attribute("data-title").unwrap_or_default();
            el.before(
                &format!("<p><strong>{}</strong></p>", title.replace('<', "&lt;")),
                ContentType::Html,
            );
            Ok(())
        }))
        .append_element_content_handler(element!("htmlpress-mermaid > pre", |el| {
            el.set_attribute("class", "language-mermaid")?;
            Ok(())
        }))
        .append_element_content_handler(element!("htmlpress-terminal kbd", |el| {
            el.prepend("$ ", ContentType::Text);
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

// Whether each <pre>, in document order, lacks a <code> child
fn list_bare_blocks(html: &str) -> Result<Vec<bool>, String> {
    let blocks = std::cell::RefCell::new(vec![]);
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("pre", |_| {
            blocks.borrow_mut().push(true);
            Ok(())
        }))
        .append_element_content_handler(element!("pre > code", |_| {
            if let Some(bare) = blocks.borrow_mut().last_mut() {
                *bare = false;
            }
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(blocks.into_inner())
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;")
}

#[cfg(test)]
#[path = "convert_unit.rs"]
mod unit;
