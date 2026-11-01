use textlane::hast::Node;
use textlane::html::read_attribute;

/// The project-relative paths of the files `<htmlpress-include src>`, or in Markdown
/// `::include{file}`, names in a page or partial at `source`, read from its parsed nodes
/// and resolved as `resolve_includes` resolves them.
pub(crate) fn list_includes(source: &str, nodes: &[Node<'_>]) -> Vec<String> {
    let folder = source.rsplit_once('/').map_or("", |(folder, _)| folder);
    let mut srcs = vec![];
    let mut stack: Vec<&Node> = nodes.iter().rev().collect();
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &root.children,
            Node::Element(element) => {
                if element.tag_name == "htmlpress-include"
                    && let Some(src) = read_attribute(element, "src")
                {
                    srcs.push(src.into_owned());
                }
                &element.children
            }
            _ => continue,
        };
        stack.extend(children.iter().rev());
    }
    srcs.into_iter()
        .map(|src| {
            let path = match (src.strip_prefix('/'), folder.is_empty()) {
                (Some(rooted), _) => rooted.to_owned(),
                (None, true) => src,
                (None, false) => format!("{folder}/{src}"),
            };
            normalize_path(&path)
        })
        .collect()
}

// `a/./b/../c.md` → `a/c.md`, so a path compares with the scanned ones
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = vec![];
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}

#[cfg(test)]
#[path = "list_unit.rs"]
mod unit;
