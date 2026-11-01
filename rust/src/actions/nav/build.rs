use crate::models::nav::NavNode;
use crate::models::page::Page;

/// Nest pages, already in navigation order, as Livemark does: a page goes under its
/// folder's `index.html` below `prefix` and a folder's index under its parent folder's;
/// with no such page it stays at the top, so a folder without an index adds no entry.
/// Flat keeps every page at the top.
pub(crate) fn build_nav<'a>(
    pages: &[&'a Page],
    prefix: &str,
    flat: bool,
) -> Vec<NavNode<'a>> {
    let prefix = prefix.trim_start_matches('/');
    let relatives: Vec<&str> = (pages.iter())
        .map(|page| page.path.strip_prefix(prefix).unwrap_or(&page.path))
        .collect();
    let parents: Vec<Option<usize>> = (relatives.iter())
        .map(|relative| {
            let parent = read_parent(relative).filter(|_| !flat)?;
            relatives.iter().position(|other| *other == parent)
        })
        .collect();
    create_nodes(pages, &parents, None)
}

fn create_nodes<'a>(
    pages: &[&'a Page],
    parents: &[Option<usize>],
    parent: Option<usize>,
) -> Vec<NavNode<'a>> {
    (pages.iter().enumerate())
        .filter(|(index, _)| parents.get(*index).copied().flatten() == parent)
        .map(|(index, page)| NavNode {
            page,
            children: create_nodes(pages, parents, Some(index)),
        })
        .collect()
}

// `a/b/c.html` and `a/b/index.html` both answer to `a/index.html` or `a/b/index.html`:
// a folder's index belongs to the folder above it, a top page to none
fn read_parent(relative: &str) -> Option<String> {
    let folder = relative.rsplit_once('/')?.0;
    let folder = match relative.ends_with("/index.html") {
        true => folder.rsplit_once('/')?.0,
        false => folder,
    };
    Some(format!("{folder}/index.html"))
}

#[cfg(test)]
#[path = "build_unit.rs"]
mod unit;
