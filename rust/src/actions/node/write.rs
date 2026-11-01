use textlane::ArenaVec;
use textlane::hast::{Node, Root};
use textlane::html::{Stringify, stringify_hast};

/// Nodes as HTML, raw nodes written as they are: what the layout writes, the page's
/// content already among them as raw markup.
pub(crate) fn write_nodes<'a>(nodes: ArenaVec<'a, Node<'a>>) -> Result<String, String> {
    let root = Node::Root(Root {
        children: nodes,
        position: None,
        data: None,
        quirks_mode: None,
    });
    let options = Stringify {
        allow_dangerous_html: true,
        close_empty_elements: true,
        ..Stringify::default()
    };
    let html = stringify_hast(&root, &options).map_err(|error| error.message)?;
    Ok(name_references(&html))
}

/// textlane writes `&`, `<`, `"` and `'` as hexadecimal references; the named ones (and a
/// bare `'`) read as an author writes them. Raw text, where neither appears, is untouched.
pub(crate) fn name_references(html: &str) -> String {
    html.replace("&#x26;", "&amp;")
        .replace("&#x3C;", "&lt;")
        .replace("&#x22;", "&quot;")
        .replace("&#x27;", "'")
}
