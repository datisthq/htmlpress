use std::cell::RefCell;

use lol_html::{RewriteStrSettings, element, rewrite_str, text};

use crate::actions::section::find::find_section;
use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::NOT_FOUND_PAGE;

/// Where the build writes the search index, under the assets folder.
pub(crate) const SEARCH_INDEX: &str = "search.json";

// A chunk this long already shows what it is about; the rest only grows the download
const MAX_TEXT: usize = 1500;

/// The search index: each page's title, URL (with the base), section and text, split at
/// its `<h2>` and `<h3>` headings so a hit can land on the heading it is under.
pub(crate) fn index_site(site: &Site) -> crate::Result<String> {
    let base = site.config.read_base();
    let mut pages = vec![];
    for page in site.pages.iter().filter(|page| page.path != NOT_FOUND_PAGE) {
        let section = find_section(&site.config.sections, &page.url)
            .and_then(|index| site.config.sections.get(index))
            .map(|section| section.title.clone());
        let chunks = read_chunks(page).map_err(Error::page(&page.source))?;
        pages.push(serde_json::json!({
            "title": page.title,
            "url": format!("{base}{}", page.url),
            "section": section,
            "chunks": chunks
                .into_iter()
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

#[derive(Default)]
struct Chunk {
    id: Option<String>,
    heading: Option<String>,
    text: String,
}

// The content's text in document order; a heading opens a new chunk, and the first
// <h1> and code, scripts and styles are left out, since none is prose a reader searches
fn read_chunks(page: &Page) -> Result<Vec<Chunk>, String> {
    let chunks = RefCell::new(vec![Chunk::default()]);
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("h2[id], h3[id]", |el| {
            chunks.borrow_mut().push(Chunk {
                id: el.get_attribute("id"),
                heading: Some(String::new()),
                text: String::new(),
            });
            Ok(())
        }))
        .append_element_content_handler(text!("h2[id], h3[id]", |chunk| {
            if let Some(last) = chunks.borrow_mut().last_mut()
                && let Some(heading) = &mut last.heading
            {
                heading.push_str(chunk.as_str());
            }
            Ok(())
        }))
        // lol_html may cut one text node into several chunks, so words are separated at
        // element starts, not between chunks
        .append_element_content_handler(element!("*", |_| {
            if let Some(last) = chunks.borrow_mut().last_mut() {
                last.text.push(' ');
            }
            Ok(())
        }))
        .append_element_content_handler(text!(
            ":not(h1, h2[id], h3[id], script, style, svg, template)",
            |chunk| {
                if let Some(last) = chunks.borrow_mut().last_mut() {
                    last.text.push_str(chunk.as_str());
                }
                Ok(())
            }
        ));
    rewrite_str(&page.content, settings).map_err(|error| error.to_string())?;
    Ok(chunks
        .into_inner()
        .into_iter()
        .map(|chunk| Chunk {
            heading: chunk.heading.map(|heading| collapse_text(&heading)),
            text: shorten_text(&collapse_text(&chunk.text)),
            ..chunk
        })
        .filter(|chunk| chunk.heading.is_some() || !chunk.text.is_empty())
        .collect())
}

fn collapse_text(text: &str) -> String {
    let text = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&");
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
