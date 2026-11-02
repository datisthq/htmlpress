use std::collections::{BTreeMap, BTreeSet, HashMap};

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

/// Each language's links: a page's file, as a root-relative URL, to the URL it is served
/// at, its translation's in that language when it has one, as Livemark links. A link may
/// name the page's path or its source without the include base, percent-encoded or not.
pub(crate) fn list_links(
    pages: &[Page],
    bases: &[String],
) -> BTreeMap<String, BTreeMap<String, String>> {
    let files = list_files(pages, bases);
    let translations: HashMap<(&str, &str), &Page> = (pages.iter())
        .filter(|page| !page.untranslated && !page.key.is_empty())
        .map(|page| ((page.key.as_str(), page.language.as_str()), page))
        .collect();
    let languages: BTreeSet<&str> =
        pages.iter().map(|page| page.language.as_str()).collect();
    (languages.into_iter())
        .map(|language| {
            let links = (files.iter())
                .map(|(file, page)| {
                    let page = (translations.get(&(page.key.as_str(), language)))
                        .unwrap_or(page);
                    (file.clone(), page.url.clone())
                })
                .collect();
            (language.to_owned(), links)
        })
        .collect()
}

// Every spelling of each page's file: its path, then its source, then the source's
// `.html` for a `.md` page and every one percent-encoded, an exact one never displaced
fn list_files<'p>(pages: &'p [Page], bases: &[String]) -> BTreeMap<String, &'p Page> {
    let mut files = BTreeMap::new();
    for page in pages {
        files.insert(format!("/{}", page.path), page);
    }
    let sources: Vec<(Option<String>, &Page)> = (pages.iter())
        .map(|page| (read_source_file(&page.source, bases), page))
        .collect();
    for (source, page) in &sources {
        if let Some(source) = source {
            files.entry(source.clone()).or_insert(page);
        }
    }
    for (source, page) in &sources {
        let mut spellings = vec![format!("/{}", page.path)];
        if let Some(source) = source {
            if let Some(stem) = source.strip_suffix(".md") {
                spellings.push(format!("{stem}.html"));
            }
            spellings.push(source.clone());
        }
        let encoded: Vec<String> = spellings.iter().map(|it| encode_path(it)).collect();
        for spelling in spellings.into_iter().chain(encoded) {
            files.entry(spelling).or_insert(page);
        }
    }
    files
}

// `/guides/setup.html` for `docs/guides/setup.html` when `docs/` is an include base
fn read_source_file(source: &str, bases: &[String]) -> Option<String> {
    let name = source.rsplit('/').next().filter(|_| !source.is_empty())?;
    Some(format!("{}{name}", read_source_folder(source, bases)))
}

// A link spells a space or a non-ASCII letter as `%20`, `%C3%A9`, which the lookup
// compares as written
fn encode_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte.is_ascii_alphanumeric() || b"-._~/!$&'()*+,;=:@".contains(&byte) {
            true => encoded.push(char::from(byte)),
            false => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
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
