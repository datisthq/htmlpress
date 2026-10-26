use crate::models::config::{Section, SectionKind};

/// The index of the `article` section owning a page's URL: the one whose prefix is the
/// longest that URL starts with. None when no section owns it.
pub(crate) fn find_section(sections: &[Section], url: &str) -> Option<usize> {
    (sections.iter().enumerate())
        .filter(|(_, section)| section.kind == SectionKind::Article)
        .filter_map(|(index, section)| Some((index, section.prefix.as_deref()?)))
        .filter(|(_, prefix)| url.starts_with(prefix))
        .max_by_key(|(_, prefix)| prefix.len())
        .map(|(index, _)| index)
}

/// Whether a `custom` section's internal URL is the open page or a folder above it. An
/// external URL never is, and `/` only on the home page, or it would match every page.
pub(crate) fn is_custom_current(url: &str, page_url: &str) -> bool {
    if !url.starts_with('/') || url.starts_with("//") {
        return false;
    }
    let target = url.split(['?', '#']).next().unwrap_or_default();
    match target {
        "/" => page_url == "/",
        _ if target.ends_with('/') => page_url.starts_with(target),
        _ => page_url == target,
    }
}

#[cfg(test)]
#[path = "find_unit.rs"]
mod unit;
