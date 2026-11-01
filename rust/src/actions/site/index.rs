use textlane::hast::Node;
use textlane::html::read_attribute;

use crate::actions::icon::render::render_icon;
use crate::actions::node::collect::collect_text;
use crate::actions::section::find::find_section;
use crate::models::error::Error;
use crate::models::language::SiteLanguage;
use crate::models::page::{Page, SearchChunk};
use crate::models::site::Site;
use crate::settings::NOT_FOUND_PAGE;

/// Where the build writes the search index, under the assets folder.
pub(crate) const SEARCH_INDEX: &str = "search.json";

/// Where a language's search index is written under `_htmlpress/`: `search.json` for the
/// default language, `search.<code>.json` for the others.
pub(crate) fn create_search_path(language: &SiteLanguage) -> String {
    match language.default {
        true => SEARCH_INDEX.to_owned(),
        false => format!("search.{}.json", language.code),
    }
}

// A chunk this long already shows what it is about; the rest only grows the download
const MAX_TEXT: usize = 1500;

/// The search index: each page's title, URL (with the base), section and text, split at
/// its `<h2>` and `<h3>` headings so a hit can land on the heading it is under.
pub(crate) fn index_site(site: &Site) -> crate::Result<String> {
    let base = site.config.read_base();
    let mut pages = vec![];
    let pages_in = |page: &&Page| page.path != NOT_FOUND_PAGE && !page.untranslated;
    for page in site.pages.iter().filter(pages_in) {
        let section = find_section(&site.config.sections, &page.url)
            .and_then(|index| site.config.sections.get(index))
            .map(|section| section.title.clone());
        pages.push(serde_json::json!({
            "title": page.title,
            "url": format!("{base}{}", page.url),
            "section": section,
            "group": create_group(&page.key),
            "description": page.description,
            "icon": page.icon.as_deref().and_then(render_icon),
            "chunks": page.chunks
                .iter()
                .map(|chunk| serde_json::json!({
                    "id": chunk.id,
                    "heading": chunk.heading,
                    "text": chunk.text,
                }))
                .collect::<Vec<_>>(),
        }));
    }
    serde_json::to_string(&serde_json::json!({ "pages": pages }))
        .map_err(|error| Error::Usage(format!("cannot write {SEARCH_INDEX}: {error}")))
}

// Livemark's grouping: a page two folders deep or more is grouped under the folders
// between its first and its own, title-cased; the rest fall under the "Pages" group
fn create_group(key: &str) -> Option<String> {
    let path = key.trim_matches('/').trim_end_matches(".html");
    let path = path.strip_suffix("/index").unwrap_or(path);
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() <= 2 {
        return None;
    }
    let folders = segments[1..segments.len() - 1].iter().map(|folder| {
        let words = folder.split('-').map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect::<String>()
            })
        });
        words.collect::<Vec<_>>().join(" ")
    });
    Some(folders.collect::<Vec<_>>().join(" › "))
}

/// A page's content as search reads it, in document order: a heading opens a new chunk,
/// and `<h1>`s, code, scripts and styles are left out, since none is prose a reader
/// searches.
pub(crate) fn read_chunks(nodes: &[Node<'_>]) -> Vec<SearchChunk> {
    let mut chunks = vec![SearchChunk::default()];
    let mut stack: Vec<&Node> = nodes.iter().rev().collect();
    while let Some(node) = stack.pop() {
        match node {
            Node::Text(text) => {
                if let Some(last) = chunks.last_mut() {
                    last.text.push_str(text.value);
                }
            }
            Node::Element(element) => {
                // Words are separated at element starts, as a browser would show them
                if let Some(last) = chunks.last_mut() {
                    last.text.push(' ');
                }
                let id = read_attribute(element, "id");
                match element.tag_name {
                    "h2" | "h3" if id.is_some() => chunks.push(SearchChunk {
                        id: id.map(|id| id.into_owned()),
                        heading: Some(collect_text(&element.children)),
                        text: String::new(),
                    }),
                    "h1" | "pre" | "script" | "style" | "svg" | "template" => {}
                    _ => stack.extend(element.children.iter().rev()),
                }
            }
            _ => {}
        }
    }
    chunks
        .into_iter()
        .map(|chunk| SearchChunk {
            heading: chunk.heading.map(|heading| collapse_text(&heading)),
            text: shorten_text(&collapse_text(&chunk.text)),
            ..chunk
        })
        .filter(|chunk| chunk.heading.is_some() || !chunk.text.is_empty())
        .collect()
}

fn collapse_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn shorten_text(text: &str) -> String {
    match text.char_indices().nth(MAX_TEXT) {
        Some((end, _)) => text[..end].to_owned(),
        None => text.to_owned(),
    }
}

#[cfg(test)]
#[path = "index_unit.rs"]
mod unit;
