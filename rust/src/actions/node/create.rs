use textlane::hast::{Element, Node, Text};
use textlane::html::write_attribute;
use textlane::{Allocator, ArenaVec};

/// An element with these attributes, by their HTML names, and no children.
pub(crate) fn create_element<'a>(
    tag: &str,
    attributes: &[(&str, &str)],
    alloc: &'a Allocator,
) -> Element<'a> {
    let mut element = Element {
        tag_name: alloc.alloc_str(tag),
        properties: ArenaVec::new_in(alloc),
        children: ArenaVec::new_in(alloc),
        position: None,
        data: None,
        content: None,
    };
    for (name, value) in attributes {
        write_attribute(&mut element, name, value, alloc);
    }
    element
}

/// A text node.
pub(crate) fn create_text<'a>(value: &str, alloc: &'a Allocator) -> Node<'a> {
    Node::Text(Text {
        value: alloc.alloc_str(value),
        position: None,
        data: None,
    })
}
