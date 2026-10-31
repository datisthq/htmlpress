use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};

use textlane::{File, FrozenProcessor, Processor, html, resolve_url};

use crate::actions::page::transform::name_references;

use crate::models::page::Page;

/// Make a page's relative URLs (`href`, `src`, `srcset`, `poster`), in its content and
/// head, and its `image` root-relative, resolved from the folder its source file sits in:
/// served at its own folder URL, a page would otherwise point one level too deep. A link
/// to a page's file (`/guide.html`, `guide.es.html`) then takes that page's URL from `links`.
pub(crate) fn rebase_page(
    page: &mut Page,
    bases: &[String],
    links: &BTreeMap<String, String>,
    processors: &mut HashMap<Option<String>, FrozenProcessor>,
) -> Result<(), String> {
    if page.source.is_empty() {
        return Ok(());
    }
    let root = read_root(&page.source, bases);
    // One per include base: the links are the same for every page under it
    let processor = match processors.entry(root.clone()) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => entry.insert(create_processor(root.clone(), links)?),
    };
    let rebase = |html: &str| -> Result<String, String> {
        let mut file = File::new();
        file.path = Some(page.source.clone());
        let output =
            (processor.process_file(html, file)).map_err(|error| error.message)?;
        Ok(name_references(&output.value))
    };
    page.content = rebase(&page.content)?;
    page.head = rebase(&page.head)?;
    if let Some(image) = &page.image
        && let Some(resolved) = resolve_url(image, Some(&page.source), root.as_deref())
    {
        let slash = if resolved.directory && !resolved.file.is_empty() {
            "/"
        } else {
            ""
        };
        page.image = Some(format!("/{}{slash}{}", resolved.file, resolved.suffix));
    }
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

fn create_processor(
    root: Option<String>,
    links: &BTreeMap<String, String>,
) -> Result<FrozenProcessor, String> {
    let options = html::PathRewrite {
        root,
        pages: (links.iter())
            .map(|(file, url)| (file.trim_start_matches('/').to_owned(), url.clone()))
            .collect(),
        ..html::PathRewrite::default()
    };
    Processor::new()
        .with(html::Parse {
            fragment: true,
            ..html::Parse::default()
        })
        .with(options)
        .with(html::Stringify {
            close_empty_elements: true,
            ..html::Stringify::default()
        })
        .freeze()
        .map_err(|error| error.message)
}

// The include base a source sits under, which its relative URLs count from
fn read_root(source: &str, bases: &[String]) -> Option<String> {
    (bases.iter())
        .filter(|base| !base.is_empty() && source.starts_with(base.as_str()))
        .max_by_key(|base| base.len())
        .map(|base| base.trim_end_matches('/').to_owned())
}

#[cfg(test)]
#[path = "rebase_unit.rs"]
mod unit;
