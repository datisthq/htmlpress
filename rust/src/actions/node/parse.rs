use textlane::hast::Node;
use textlane::html::{Parse, parse_html};
use textlane::{Allocator, ArenaVec, File};

/// Markup no tree holds yet, as nodes: what htmlpress writes itself, such as an icon or
/// the navigation, or a frontmatter's raw `head`; never a page's source, parsed once.
pub(crate) fn parse_fragment<'a>(
    html: &str,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let options = Parse {
        fragment: true,
        ..Parse::default()
    };
    match parse_html(html, &options, &mut File::new(), alloc) {
        Node::Root(root) => root.children,
        node => {
            let mut nodes = ArenaVec::new_in(alloc);
            nodes.push(node);
            nodes
        }
    }
}
