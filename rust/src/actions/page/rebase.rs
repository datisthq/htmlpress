use std::collections::BTreeMap;

use lol_html::{RewriteStrSettings, element, rewrite_str};

use crate::models::page::Page;

/// Make a page's relative URLs (`href`, `src`, `srcset`, `poster`), in its content and
/// head, and its `image` root-relative, resolved from the folder its source file sits in:
/// served at its own folder URL, a page would otherwise point one level too deep. A link
/// to a page's file (`/guide.html`, `guide.es.html`) then takes that page's URL from `links`.
pub(crate) fn rebase_page(
    page: &mut Page,
    bases: &[String],
    links: &BTreeMap<String, String>,
) -> Result<(), String> {
    if page.source.is_empty() {
        return Ok(());
    }
    let folder = read_source_folder(&page.source, bases);
    if let Some(image) = &page.image {
        page.image = Some(resolve_url(image, &folder).unwrap_or_else(|| image.clone()));
    }
    page.content = rebase_urls(&page.content, &folder, links)?;
    page.head = rebase_urls(&page.head, &folder, links)?;
    Ok(())
}

/// Each page's file, as a root-relative URL, mapped to the URL it is served at: by its
/// path and by its source without the include base, since a link may name either.
pub(crate) fn list_links(pages: &[Page], bases: &[String]) -> BTreeMap<String, String> {
    let mut links = BTreeMap::new();
    for page in pages {
        links.insert(format!("/{}", page.path), page.url.clone());
        if let Some(name) = page
            .source
            .rsplit('/')
            .next()
            .filter(|_| !page.source.is_empty())
        {
            let folder = read_source_folder(&page.source, bases);
            links
                .entry(format!("{folder}{name}"))
                .or_insert_with(|| page.url.clone());
        }
    }
    links
}

fn rebase_urls(
    html: &str,
    folder: &str,
    links: &BTreeMap<String, String>,
) -> Result<String, String> {
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "[href], [src], [srcset], [poster]",
        |el| {
            for name in ["href", "src", "poster"] {
                if let Some(value) = el.get_attribute(name) {
                    let url =
                        resolve_url(&value, folder).unwrap_or_else(|| value.clone());
                    let url = match name {
                        "href" => link_page(&url, links).unwrap_or(url),
                        _ => url,
                    };
                    if url != value {
                        el.set_attribute(name, &url)?;
                    }
                }
            }
            if let Some(srcset) = el.get_attribute("srcset") {
                let candidates: Vec<String> = (srcset.split(','))
                    .map(|candidate| {
                        let trimmed = candidate.trim_start();
                        let lead = &candidate[..candidate.len() - trimmed.len()];
                        let (url, rest) = trimmed.split_at(
                            trimmed.find(char::is_whitespace).unwrap_or(trimmed.len()),
                        );
                        let url =
                            resolve_url(url, folder).unwrap_or_else(|| url.to_owned());
                        format!("{lead}{url}{rest}")
                    })
                    .collect();
                el.set_attribute("srcset", &candidates.join(","))?;
            }
            Ok(())
        }
    ));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

// `/guide.html#flags` → `/guide/#flags`; None for a URL that names no page's file
fn link_page(url: &str, links: &BTreeMap<String, String>) -> Option<String> {
    let end = url.find(['?', '#']).unwrap_or(url.len());
    let page = links.get(&url[..end])?;
    Some(format!("{page}{}", &url[end..]))
}

/// `/guides/` for `docs/guides/setup.html` when `docs/` is an include base.
fn read_source_folder(source: &str, bases: &[String]) -> String {
    let folder = source.rfind('/').map_or("", |end| &source[..=end]);
    let folder = (bases.iter())
        .filter(|base| !base.is_empty())
        .filter_map(|base| folder.strip_prefix(base.as_str()))
        .min_by_key(|rest| rest.len())
        .unwrap_or(folder);
    format!("/{folder}")
}

// None for a URL that is not relative: root-relative, absolute, `data:`, a fragment or a query
fn resolve_url(url: &str, folder: &str) -> Option<String> {
    let scheme = url.find(':').is_some_and(|colon| {
        let scheme = &url[..colon];
        !scheme.is_empty()
            && !scheme.contains(['/', '?', '#'])
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
    });
    if url.is_empty() || scheme || url.starts_with(['/', '#', '?']) {
        return None;
    }
    let end = url.find(['?', '#']).unwrap_or(url.len());
    let mut segments: Vec<&str> = folder.split('/').filter(|s| !s.is_empty()).collect();
    for segment in url[..end].split('/') {
        match segment {
            "." => {}
            ".." => {
                segments.pop();
            }
            segment => segments.push(segment),
        }
    }
    Some(format!("/{}{}", segments.join("/"), &url[end..]))
}

#[cfg(test)]
#[path = "rebase_unit.rs"]
mod unit;
