use textlane::hast::Node;

/// Every text under a node, in document order, as written: no whitespace collapsed and
/// nothing between elements, unlike `read_text`'s rendered text.
pub(crate) fn collect_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    let mut stack: Vec<&Node> = nodes.iter().rev().collect();
    while let Some(node) = stack.pop() {
        match node {
            Node::Text(node) => text.push_str(node.value),
            Node::Element(element) => stack.extend(element.children.iter().rev()),
            Node::Root(root) => stack.extend(root.children.iter().rev()),
            _ => {}
        }
    }
    text
}
