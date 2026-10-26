use crate::models::nav::NavNode;
use crate::models::page::Page;

/// Nest pages, already in navigation order, by their folders below `prefix`: a folder's
/// `index.html` is its entry, and a folder without one is an entry named by its path.
/// An entry keeps the position of the first page under it. Flat keeps every page at the top.
pub(crate) fn build_nav<'a>(
    pages: &[&'a Page],
    prefix: &str,
    flat: bool,
) -> Vec<NavNode<'a>> {
    let prefix = prefix.trim_start_matches('/');
    let mut roots = vec![];
    for page in pages {
        let relative = page.path.strip_prefix(prefix).unwrap_or(&page.path);
        let mut folders: Vec<&str> = relative.split('/').collect();
        let file = folders.pop().unwrap_or_default();
        if flat {
            folders.clear();
        }
        // The section's own index, and the root one, lead the list instead of holding it
        let index = file == "index.html" && !folders.is_empty();
        insert_page(&mut roots, prefix, &folders, page, index);
    }
    roots
}

fn insert_page<'a>(
    nodes: &mut Vec<NavNode<'a>>,
    parent: &str,
    folders: &[&str],
    page: &'a Page,
    index: bool,
) {
    let Some((name, rest)) = folders.split_first() else {
        nodes.push(NavNode {
            page: Some(page),
            folder: None,
            children: vec![],
        });
        return;
    };
    let folder = format!("{parent}{name}/");
    let path = format!("{folder}index.html");
    let position = match nodes
        .iter()
        .position(|node| node.folder.as_ref() == Some(&path))
    {
        Some(position) => position,
        None => {
            nodes.push(NavNode {
                page: None,
                folder: Some(path),
                children: vec![],
            });
            nodes.len() - 1
        }
    };
    let Some(node) = nodes.get_mut(position) else {
        return;
    };
    match rest.is_empty() && index {
        true => node.page = Some(page),
        false => insert_page(&mut node.children, &folder, rest, page, index),
    }
}

#[cfg(test)]
#[path = "build_unit.rs"]
mod unit;
