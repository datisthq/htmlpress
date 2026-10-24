use std::cell::{Cell, RefCell};

use lol_html::{RewriteStrSettings, doctype, element, rewrite_str, text};

use crate::models::error::Error;
use crate::models::page::Page;

/// Split one HTML source into a [`Page`]. `path` is relative to the project root. A full
/// document and a bare fragment both work: the wrappers are dropped, the content kept.
pub fn read_page(path: &str, html: &str) -> crate::Result<Page> {
    let facts = read_facts(html).map_err(Error::page(path))?;
    let content = read_content(html).map_err(Error::page(path))?;
    let head = match facts.has_head {
        true => read_head(html).map_err(Error::page(path))?,
        false => String::new(),
    };
    let title = [facts.title, facts.heading]
        .into_iter()
        .map(|text| decode_entities(text.trim()))
        .find(|text| !text.is_empty())
        .unwrap_or_else(|| read_stem(path));
    Ok(Page {
        path: path.to_owned(),
        url: create_url(path),
        title,
        description: facts.description,
        order: facts.order,
        head,
        content,
    })
}

#[derive(Default)]
struct Facts {
    title: String,
    heading: String,
    description: Option<String>,
    order: Option<i64>,
    has_head: bool,
}

fn read_facts(html: &str) -> Result<Facts, String> {
    let facts = RefCell::new(Facts::default());
    let headings = Cell::new(0);
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("head", |_| {
            facts.borrow_mut().has_head = true;
            Ok(())
        }))
        .append_element_content_handler(text!("title", |chunk| {
            facts.borrow_mut().title.push_str(chunk.as_str());
            Ok(())
        }))
        .append_element_content_handler(element!("h1", |_| {
            headings.set(headings.get() + 1);
            Ok(())
        }))
        .append_element_content_handler(text!("h1", |chunk| {
            if headings.get() == 1 {
                facts.borrow_mut().heading.push_str(chunk.as_str());
            }
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='description']", |el| {
            facts.borrow_mut().description = el.get_attribute("content");
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='order']", |el| {
            let order = el.get_attribute("content").unwrap_or_default();
            let order = order.trim().parse().map_err(|_| {
                format!("<meta name=\"order\"> must be an integer, got \"{order}\"")
            })?;
            facts.borrow_mut().order = Some(order);
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(facts.into_inner())
}

fn read_content(html: &str) -> Result<String, String> {
    let mut settings = RewriteStrSettings::new()
        .append_document_content_handler(doctype!(|doctype| {
            doctype.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("head", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("html", |el| {
            el.remove_and_keep_content();
            Ok(())
        }))
        .append_element_content_handler(element!("body", |el| {
            el.remove_and_keep_content();
            Ok(())
        }));
    // A fragment carries its title and metas without a <head>; they are read, not shown
    for selector in OWNED_BY_LAYOUT {
        settings = settings.append_element_content_handler(element!(selector, |el| {
            el.remove();
            Ok(())
        }));
    }
    let content = rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(content.trim().to_owned())
}

// The layout writes the charset, viewport, title and description itself; a second copy
// from the page would win or clash depending on the browser.
const OWNED_BY_LAYOUT: &[&str] = &[
    "title",
    "meta[charset]",
    "meta[name='viewport']",
    "meta[name='description']",
    "meta[name='order']",
];

fn read_head(html: &str) -> Result<String, String> {
    let mut settings = RewriteStrSettings::new()
        .append_document_content_handler(doctype!(|doctype| {
            doctype.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("body", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("html > :not(head)", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("html", |el| {
            el.remove_and_keep_content();
            Ok(())
        }))
        .append_element_content_handler(element!("head", |el| {
            el.remove_and_keep_content();
            Ok(())
        }));
    for selector in OWNED_BY_LAYOUT {
        settings = settings.append_element_content_handler(element!(selector, |el| {
            el.remove();
            Ok(())
        }));
    }
    let head = rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(head
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

pub(crate) fn create_url(path: &str) -> String {
    match path.strip_suffix("index.html") {
        Some(folder) if folder.is_empty() || folder.ends_with('/') => {
            format!("/{folder}")
        }
        _ => format!("/{path}"),
    }
}

fn read_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".html").unwrap_or(name).to_owned()
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", "\u{a0}")
        .replace("&amp;", "&")
}

#[cfg(test)]
#[path = "read_unit.rs"]
mod unit;
