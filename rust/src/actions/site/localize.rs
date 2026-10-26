use crate::actions::page::read::create_url;
use crate::actions::site::scan::sort_pages;
use crate::models::page::Page;
use crate::models::site::Site;

/// The site as one language builds it: that language's config and title, its own pages,
/// and each default-language page it has no translation of, listed under its prefix but
/// marked untranslated, so the navigation keeps its shape and links to the original.
pub(crate) fn localize_site(site: &Site, code: &str) -> Site {
    let language = site.read_language(code);
    let own = |page: &Page| {
        page.language == language.code || language.default && page.language.is_empty()
    };
    let mut pages: Vec<Page> = site
        .pages
        .iter()
        .filter(|page| own(page))
        .cloned()
        .collect();
    if !language.default {
        let default = site.read_language("");
        let prefix = language.prefix.trim_start_matches('/');
        let missing: Vec<Page> = (site.pages.iter())
            .filter(|page| page.language == default.code)
            .filter(|page| !pages.iter().any(|other| other.key == page.key))
            .map(|page| {
                let path = format!("{prefix}{}", page.path);
                Page {
                    url: create_url(&path),
                    path,
                    untranslated: true,
                    ..page.clone()
                }
            })
            .collect();
        pages.extend(missing);
        sort_pages(&mut pages);
    }
    Site {
        config: language.config.clone(),
        title: language.title.clone(),
        pages,
        ..site.clone()
    }
}

#[cfg(test)]
#[path = "localize_unit.rs"]
mod unit;
