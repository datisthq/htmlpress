use textlane::hast::{
    Comment, Doctype, Element, Node, Property, PropertyItem, PropertyValue, Raw, Root,
    Text, Unknown,
};
use textlane::{Allocator, ArenaVec};

/// A deep copy of a node in the same arena, so one tree can be changed and the other
/// kept: a partial included twice, or a page before and after its Markdown copy.
pub(crate) fn copy_node<'a>(node: &Node<'a>, alloc: &'a Allocator) -> Node<'a> {
    match node {
        Node::Root(root) => Node::Root(Root {
            children: copy_nodes(&root.children, alloc),
            position: root.position,
            data: root.data,
            quirks_mode: root.quirks_mode,
        }),
        Node::Element(element) => Node::Element(Element {
            tag_name: element.tag_name,
            properties: copy_properties(&element.properties, alloc),
            children: copy_nodes(&element.children, alloc),
            position: element.position,
            data: element.data,
            content: element.content,
        }),
        Node::Text(text) => Node::Text(Text { ..*text }),
        Node::Comment(comment) => Node::Comment(Comment { ..*comment }),
        Node::Doctype(doctype) => Node::Doctype(Doctype { ..*doctype }),
        Node::Raw(raw) => Node::Raw(Raw { ..*raw }),
        Node::Unknown(unknown) => Node::Unknown(Unknown {
            kind: unknown.kind,
            fields: unknown.fields,
            children: (unknown.children.as_ref()).map(|nodes| copy_nodes(nodes, alloc)),
            position: unknown.position,
            data: unknown.data,
        }),
        // MDX nodes never reach a page's hast, which `md.toHtml` and `html.raw` made
        _ => Node::Text(Text {
            value: "",
            position: None,
            data: None,
        }),
    }
}

/// [`copy_node`] for each node of a list.
pub(crate) fn copy_nodes<'a>(
    nodes: &[Node<'a>],
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let mut copies = ArenaVec::with_capacity_in(nodes.len(), alloc);
    copies.extend(nodes.iter().map(|node| copy_node(node, alloc)));
    copies
}

fn copy_properties<'a>(
    properties: &[Property<'a>],
    alloc: &'a Allocator,
) -> ArenaVec<'a, Property<'a>> {
    let mut copies = ArenaVec::with_capacity_in(properties.len(), alloc);
    copies.extend(properties.iter().map(|property| Property {
        name: property.name,
        value: match &property.value {
            PropertyValue::Null => PropertyValue::Null,
            PropertyValue::Boolean(value) => PropertyValue::Boolean(*value),
            PropertyValue::Number(value) => PropertyValue::Number(*value),
            PropertyValue::String(value) => PropertyValue::String(value),
            PropertyValue::List(items) => {
                let mut list = ArenaVec::with_capacity_in(items.len(), alloc);
                list.extend(items.iter().map(|item| match item {
                    PropertyItem::Number(value) => PropertyItem::Number(*value),
                    PropertyItem::String(value) => PropertyItem::String(value),
                }));
                PropertyValue::List(list)
            }
        },
    }));
    copies
}
