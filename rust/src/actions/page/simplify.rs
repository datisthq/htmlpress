use textlane::hast::{Element, Node, Text};
use textlane::html::{read_attribute, remove_attribute, write_attribute};
use textlane::{Allocator, ArenaVec};

/// Reduce a page's tree to what plain Markdown can say, before `html.toMarkdown`: each
/// `htmlpress-*` component to the Markdown it stands for, a titled block's title before
/// it, and what only the page shows (its first `<h1>`, a dark image's twin, line numbers)
/// gone.
pub(crate) fn simplify_page<'a>(node: &mut Node<'a>, alloc: &'a Allocator) {
    let mut first_heading = true;
    // Each node with whether it sits in a terminal, whose commands take a prompt
    let mut stack: Vec<(&mut Node<'a>, Option<&'a str>, bool)> =
        vec![(node, None, false)];
    while let Some((node, parent, terminal)) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => {
                let tag = element.tag_name;
                if (tag == "h1" && std::mem::take(&mut first_heading))
                    || (tag == "img"
                        && read_attribute(element, "data-theme").as_deref()
                            == Some("dark"))
                    || (tag == "span" && has_class(element, "line-number"))
                {
                    drop_element(element);
                    continue;
                }
                match tag {
                    "pre" => {
                        simplify_block(element, parent, alloc);
                        // The element itself becomes the wrapper, the block moving inside it
                        if let Some(title) = read_attribute(element, "data-title") {
                            let mut block = Element {
                                tag_name: "pre",
                                properties: std::mem::replace(
                                    &mut element.properties,
                                    ArenaVec::new_in(alloc),
                                ),
                                children: std::mem::replace(
                                    &mut element.children,
                                    ArenaVec::new_in(alloc),
                                ),
                                position: None,
                                data: None,
                                content: None,
                            };
                            remove_attribute(&mut block, "data-title");
                            element.tag_name = "div";
                            element.children.push(create_label(&title, alloc));
                            element.children.push(Node::Element(block));
                        }
                    }
                    "htmlpress-callout" => {
                        let label = read_attribute(element, "label")
                            .map(|label| label.into_owned())
                            .unwrap_or_else(|| {
                                let tone = read_attribute(element, "type")
                                    .map_or("info".to_owned(), |tone| tone.into_owned());
                                capitalize(&tone)
                            });
                        element.tag_name = "blockquote";
                        element.children.insert(0, create_label(&label, alloc));
                    }
                    "section" if parent == Some("htmlpress-tabs") => {
                        if let Some(title) = read_attribute(element, "title") {
                            element.children.insert(0, create_label(&title, alloc));
                        }
                    }
                    "kbd" if terminal => {
                        element.children.insert(0, create_text("$ ", alloc))
                    }
                    _ => {}
                }
                let terminal = terminal || element.tag_name == "htmlpress-terminal";
                let tag = element.tag_name;
                stack.extend(
                    element
                        .children
                        .iter_mut()
                        .map(|child| (child, Some(tag), terminal)),
                );
                continue;
            }
            _ => continue,
        };
        stack.extend(children.iter_mut().map(|child| (child, None, terminal)));
    }
}

// A fence comes only from a <pre> holding a <code>, which agents often leave out, and the
// language is read from that <code>
fn simplify_block<'a>(pre: &mut Element<'a>, parent: Option<&str>, alloc: &'a Allocator) {
    if parent == Some("htmlpress-mermaid") {
        write_attribute(pre, "class", "language-mermaid", alloc);
    }
    let bare = !pre
        .children
        .iter()
        .any(|child| matches!(child, Node::Element(code) if code.tag_name == "code"));
    if !bare {
        return;
    }
    let mut code = Element {
        tag_name: "code",
        properties: ArenaVec::new_in(alloc),
        children: std::mem::replace(&mut pre.children, ArenaVec::new_in(alloc)),
        position: None,
        data: None,
        content: None,
    };
    if let Some(class) = read_attribute(pre, "class") {
        write_attribute(&mut code, "class", &class, alloc);
    }
    pre.children.push(Node::Element(code));
}

// `html.toMarkdown` writes nothing for a `template`, so a node becomes one to be dropped
// without rebuilding its parent's children
fn drop_element(element: &mut Element<'_>) {
    element.tag_name = "template";
    element.children.clear();
}

fn has_class(element: &Element<'_>, name: &str) -> bool {
    read_attribute(element, "class").is_some_and(|classes| {
        classes.split_ascii_whitespace().any(|class| class == name)
    })
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or(String::new(), |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

// A bold paragraph: a callout's or a tab's label, a block's title
fn create_label<'a>(text: &str, alloc: &'a Allocator) -> Node<'a> {
    let mut strong = create_element("strong", alloc);
    if let Node::Element(element) = &mut strong {
        element.children.push(create_text(text, alloc));
    }
    let mut paragraph = create_element("p", alloc);
    if let Node::Element(element) = &mut paragraph {
        element.children.push(strong);
    }
    paragraph
}

fn create_element<'a>(tag_name: &'a str, alloc: &'a Allocator) -> Node<'a> {
    Node::Element(Element {
        tag_name,
        properties: ArenaVec::new_in(alloc),
        children: ArenaVec::new_in(alloc),
        position: None,
        data: None,
        content: None,
    })
}

fn create_text<'a>(value: &str, alloc: &'a Allocator) -> Node<'a> {
    Node::Text(Text {
        value: alloc.alloc_str(value),
        position: None,
        data: None,
    })
}
