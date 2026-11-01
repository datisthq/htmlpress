use textlane::hast::{Element, Node};
use textlane::html::{read_attribute, write_attribute};
use textlane::{Allocator, ArenaVec};

use crate::actions::node::create::{create_element, create_text};
use crate::actions::node::parse::parse_fragment;
use crate::actions::node::walk::{Step, walk_nodes};
use crate::language_icons::{LANGUAGE_ALIASES, LANGUAGE_ICONS};

/// Frame every `<pre data-title="…">` in a figure whose caption is the block's title bar,
/// with its language's logo, highlighted or not; give every `<pre data-collapse="N">` the
/// height of N lines, which the script then offers to expand.
pub(crate) fn add_titles<'a>(
    nodes: &mut ArenaVec<'a, Node<'a>>,
    alloc: &'a Allocator,
) -> Result<(), String> {
    walk_nodes(nodes, &mut |element| {
        if element.tag_name != "pre" {
            return Ok(Step::Continue);
        }
        if let Some(lines) = read_attribute(element, "data-collapse") {
            let count: u32 = (lines.trim().parse().ok())
                .filter(|lines| *lines > 0)
                .ok_or_else(|| {
                    format!("data-collapse must be a number of lines, got \"{lines}\"")
                })?;
            let style = format!("--htmlpress-code-lines: {count}");
            write_attribute(element, "style", &style, alloc);
        }
        let Some(title) = read_attribute(element, "data-title") else {
            return Ok(Step::Skip);
        };
        let icon = create_icon(&read_language(element).unwrap_or_default());
        let mut caption = create_element("figcaption", &[], alloc);
        caption.children = parse_fragment(&icon, alloc);
        caption.children.push(create_text(&title, alloc));
        // The element itself becomes the figure, the block moving inside it
        let block = Element {
            tag_name: "pre",
            properties: std::mem::replace(
                &mut element.properties,
                ArenaVec::new_in(alloc),
            ),
            children: std::mem::replace(&mut element.children, ArenaVec::new_in(alloc)),
            position: element.position,
            data: element.data,
            content: None,
        };
        element.tag_name = "figure";
        write_attribute(element, "class", "htmlpress-code-figure", alloc);
        element.children.push(Node::Element(caption));
        element.children.push(Node::Element(block));
        Ok(Step::Skip)
    })
}

// The language a block's `<code>` names
fn read_language(pre: &Element<'_>) -> Option<String> {
    pre.children.iter().find_map(|child| match child {
        Node::Element(code) if code.tag_name == "code" => {
            let class = read_attribute(code, "class")?;
            (class.split_whitespace())
                .find_map(|name| {
                    name.strip_prefix("language-")
                        .or(name.strip_prefix("lang-"))
                })
                .map(str::to_lowercase)
        }
        _ => None,
    })
}

// A language without a logo of its own gets the generic file
fn create_icon(language: &str) -> String {
    let name = (LANGUAGE_ALIASES.iter())
        .find(|(alias, _)| *alias == language)
        .map_or(language, |(_, name)| name);
    let found = LANGUAGE_ICONS.binary_search_by(|(icon, _, _)| (*icon).cmp(name));
    let index = found.or_else(|_| {
        LANGUAGE_ICONS.binary_search_by(|(icon, _, _)| (*icon).cmp("default"))
    });
    let Some((_, view_box, path)) =
        index.ok().and_then(|index| LANGUAGE_ICONS.get(index))
    else {
        return String::new();
    };
    format!(
        "<svg viewBox=\"{view_box}\" fill=\"currentColor\" aria-hidden=\"true\"><path d=\"{path}\"/></svg>"
    )
}

#[cfg(test)]
#[path = "title_unit.rs"]
mod unit;
