use std::cell::{Cell, RefCell};

use lol_html::{RewriteStrSettings, doctype, element, rewrite_str, text};

use crate::models::error::Error;
use crate::models::page::{Heading, Page};

/// Split one HTML source into a [`Page`]. `path` is relative to the project root. A full
/// document and a bare fragment both work: the wrappers are dropped, the content kept.
pub fn read_page(path: &str, html: &str) -> crate::Result<Page> {
    let facts = read_facts(html).map_err(Error::page(path))?;
    let headings = create_ids(facts.headings);
    let content = read_content(html, &headings).map_err(Error::page(path))?;
    let toc = headings
        .into_iter()
        .filter(|(_, listed)| *listed)
        .map(|(heading, _)| heading)
        .collect();
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
        image: facts.image,
        order: facts.order,
        head,
        content,
        toc,
    })
}

#[derive(Default)]
struct Facts {
    title: String,
    heading: String,
    description: Option<String>,
    image: Option<String>,
    order: Option<i64>,
    has_head: bool,
    /// Each heading, and whether it belongs in the table of contents.
    headings: Vec<(Heading, bool)>,
}

fn read_facts(html: &str) -> Result<Facts, String> {
    let facts = RefCell::new(Facts::default());
    let headings = Cell::new(0);
    let mut settings = RewriteStrSettings::new()
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
        .append_element_content_handler(element!("h2, h3", |el| {
            let heading = Heading {
                id: el.get_attribute("id").unwrap_or_default(),
                title: String::new(),
                level: if el.tag_name() == "h2" { 2 } else { 3 },
            };
            facts.borrow_mut().headings.push((heading, true));
            Ok(())
        }))
        .append_element_content_handler(text!("h2, h3", |chunk| {
            if let Some((heading, _)) = facts.borrow_mut().headings.last_mut() {
                heading.title.push_str(chunk.as_str());
            }
            Ok(())
        }));
    // Runs after the handler above on the same heading: a card's or a step's title is
    // part of that block, not a section of the page
    for selector in UNLISTED_HEADINGS {
        settings = settings.append_element_content_handler(element!(selector, |_| {
            if let Some((_, listed)) = facts.borrow_mut().headings.last_mut() {
                *listed = false;
            }
            Ok(())
        }));
    }
    settings = settings
        .append_element_content_handler(element!("meta[name='description']", |el| {
            facts.borrow_mut().description = el.get_attribute("content");
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='image']", |el| {
            facts.borrow_mut().image = el.get_attribute("content");
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

// Ids already in the page are reserved first, so a generated slug never steals one that
// a link elsewhere points at.
fn create_ids(headings: Vec<(Heading, bool)>) -> Vec<(Heading, bool)> {
    let mut taken: Vec<String> = headings
        .iter()
        .filter(|(heading, _)| !heading.id.is_empty())
        .map(|(heading, _)| heading.id.clone())
        .collect();
    headings
        .into_iter()
        .map(|(heading, listed)| {
            let title = decode_entities(
                &heading
                    .title
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            let id = match heading.id.is_empty() {
                true => create_slug(&title, &mut taken),
                false => heading.id,
            };
            (
                Heading {
                    id,
                    title,
                    ..heading
                },
                listed,
            )
        })
        .collect()
}

fn create_slug(title: &str, taken: &mut Vec<String>) -> String {
    let words: Vec<String> = title
        .to_lowercase()
        .split(|char: char| !char.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect();
    let base = match words.is_empty() {
        true => "section".to_owned(),
        false => words.join("-"),
    };
    let mut slug = base.clone();
    let mut count = 1;
    while taken.contains(&slug) {
        count += 1;
        slug = format!("{base}-{count}");
    }
    taken.push(slug.clone());
    slug
}

const UNLISTED_HEADINGS: &[&str] = &[
    "htmlpress-callout h2",
    "htmlpress-callout h3",
    "htmlpress-card h2",
    "htmlpress-card h3",
    "htmlpress-steps h2",
    "htmlpress-steps h3",
    "htmlpress-tabs h2",
    "htmlpress-tabs h3",
];

fn read_content(html: &str, headings: &[(Heading, bool)]) -> Result<String, String> {
    let index = Cell::new(0);
    let mut settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("h2, h3", |el| {
            if let Some((heading, _)) = headings.get(index.get())
                && el.get_attribute("id").is_none_or(|id| id.is_empty())
            {
                el.set_attribute("id", &heading.id)?;
            }
            index.set(index.get() + 1);
            Ok(())
        }))
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
    "meta[name='image']",
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
