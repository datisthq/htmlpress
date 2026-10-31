use textlane::hast::{Element, Node, Property, PropertyItem, PropertyValue, Text};
use textlane::html::{read_attribute, read_text, write_attribute};
use textlane::{Allocator, ArenaVec};

/// Make every formula an `<htmlpress-math>` holding the `code.language-math` that
/// `html.katex` renders: one written in a page keeps its tag, and the `$…$` of the prose
/// and Markdown math fences get one. The LaTeX stays in `data-latex` for the Markdown copy.
pub(crate) fn wrap_math<'a>(node: &mut Node<'a>, alloc: &'a Allocator) {
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => &mut element.children,
            _ => continue,
        };
        for child in children.iter_mut() {
            let Node::Element(element) = child else {
                continue;
            };
            let math = match element.tag_name {
                "htmlpress-math" => Some((
                    read_text(element).trim().to_owned(),
                    read_attribute(element, "display").is_some(),
                )),
                "code" | "pre" => read_marked(element),
                _ => None,
            };
            match math {
                Some((latex, display)) => *child = create_math(&latex, display, alloc),
                None => stack.push(child),
            }
        }
    }
}

// The LaTeX and display mode of a `code.math-inline`/`math-display`, or of a `<pre>`
// around one, as `html.math` and `md.math` write them
fn read_marked(element: &Element<'_>) -> Option<(String, bool)> {
    let code = match element.tag_name {
        "pre" => element.children.iter().find_map(|child| match child {
            Node::Element(code) if code.tag_name == "code" => Some(code),
            _ => None,
        })?,
        _ => element,
    };
    let classes = read_attribute(code, "class")?;
    let classes: Vec<&str> = classes.split_ascii_whitespace().collect();
    let display = classes.contains(&"math-display") || element.tag_name == "pre";
    let marked = classes.contains(&"math-inline") || classes.contains(&"math-display");
    marked.then(|| (read_text(code).trim().to_owned(), display))
}

fn create_math<'a>(latex: &str, display: bool, alloc: &'a Allocator) -> Node<'a> {
    let mut classes = ArenaVec::with_capacity_in(2, alloc);
    classes.push(PropertyItem::String("language-math"));
    classes.push(PropertyItem::String(match display {
        true => "math-display",
        false => "math-inline",
    }));
    let mut code_properties = ArenaVec::with_capacity_in(1, alloc);
    code_properties.push(Property {
        name: "className",
        value: PropertyValue::List(classes),
    });
    let mut text = ArenaVec::with_capacity_in(1, alloc);
    text.push(Node::Text(Text {
        value: alloc.alloc_str(latex),
        position: None,
        data: None,
    }));
    let mut code = ArenaVec::with_capacity_in(1, alloc);
    code.push(Node::Element(Element {
        tag_name: "code",
        properties: code_properties,
        children: text,
        position: None,
        data: None,
        content: None,
    }));
    let mut math = Element {
        tag_name: "htmlpress-math",
        properties: ArenaVec::new_in(alloc),
        children: code,
        position: None,
        data: None,
        content: None,
    };
    if display {
        write_attribute(&mut math, "display", "", alloc);
    }
    write_attribute(&mut math, "data-latex", latex, alloc);
    Node::Element(math)
}
