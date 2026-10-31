use std::collections::BTreeMap;

use textlane::resolve_url;

use crate::models::page::Page;

/// Make a page's `image` root-relative, resolved from the folder its source file sits in,
/// as its links are by the page pipeline: served at its own folder URL, a page would
/// otherwise point one level too deep.
pub(crate) fn rebase_image(page: &mut Page, bases: &[String]) {
    if page.source.is_empty() {
        return;
    }
    let root = read_root(&page.source, bases);
    if let Some(image) = &page.image
        && let Some(resolved) = resolve_url(image, Some(&page.source), root.as_deref())
    {
        let slash = match resolved.directory && !resolved.file.is_empty() {
            true => "/",
            false => "",
        };
        page.image = Some(format!("/{}{slash}{}", resolved.file, resolved.suffix));
    }
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

/// The include base a source sits under, which its relative URLs count from.
pub(crate) fn read_root(source: &str, bases: &[String]) -> Option<String> {
    (bases.iter())
        .filter(|base| !base.is_empty() && source.starts_with(base.as_str()))
        .max_by_key(|base| base.len())
        .map(|base| base.trim_end_matches('/').to_owned())
}

#[cfg(test)]
#[path = "rebase_unit.rs"]
mod unit;
