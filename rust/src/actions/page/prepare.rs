use std::collections::VecDeque;

use textlane::hast::{Element, Node};
use textlane::html::{read_attribute, remove_attribute, write_attribute};
use textlane::{Allocator, ArenaVec};

use crate::actions::icon::render::{list_icons, render_icon};
use crate::actions::node::collect::collect_text;
use crate::actions::node::create::{create_element, create_text};
use crate::actions::node::parse::parse_fragment;
use crate::actions::package::convert::{MANAGERS, convert_command};

/// What becomes plain HTML before a page's pipeline: package tabs into tabs of code
/// blocks, so they are colored like any other, icons into their SVG, the attributes
/// Livemark's directives leave into the markup they stand for, and themed images into
/// the attribute the theme reads.
pub(crate) fn prepare_page<'a>(
    node: &mut Node<'a>,
    alloc: &'a Allocator,
) -> Result<(), String> {
    match node {
        Node::Root(root) => prepare_nodes(&mut root.children, alloc),
        Node::Element(element) => prepare_nodes(&mut element.children, alloc),
        _ => Ok(()),
    }
}

fn prepare_nodes<'a>(
    nodes: &mut ArenaVec<'a, Node<'a>>,
    alloc: &'a Allocator,
) -> Result<(), String> {
    for node in nodes.iter_mut() {
        let Node::Element(element) = node else {
            continue;
        };
        if prepare_element(element, alloc)? {
            prepare_nodes(&mut element.children, alloc)?;
        }
    }
    Ok(())
}

// Whether the element's children are still the page's, to prepare in turn
fn prepare_element<'a>(
    element: &mut Element<'a>,
    alloc: &'a Allocator,
) -> Result<bool, String> {
    match element.tag_name {
        "pre" => wrap_mermaid(element, alloc),
        // Expanded here, not in the browser, so the tabs component and the copy buttons
        // treat each manager's command like any other code block
        "htmlpress-package-tabs" => {
            let commands = collect_text(&element.children);
            element.tag_name = "htmlpress-tabs";
            if read_attribute(element, "sync").is_none() {
                write_attribute(element, "sync", "package-manager", alloc);
            }
            element.children = create_package_tabs(commands.trim(), alloc);
            return Ok(false);
        }
        "htmlpress-icon" => {
            let name = read_attribute(element, "name").unwrap_or_default();
            let svg = render_icon(&name).ok_or_else(|| {
                format!(
                    "<htmlpress-icon name=\"{name}\">: no such icon; the icons are: {}",
                    list_icons()
                )
            })?;
            element.children = parse_fragment(&svg, alloc);
            return Ok(false);
        }
        // A Markdown run of `tab="…"` fences arrives as one block per tab and their titles
        "htmlpress-tabs" => {
            if let Some(list) = read_attribute(element, "tabs") {
                let titles: Vec<String> = serde_json::from_str(&list)
                    .map_err(|error| format!("<htmlpress-tabs tabs>: {error}"))?;
                remove_attribute(element, "tabs");
                wrap_tabs(element, titles.into(), alloc);
            }
        }
        "details" => {
            if let Some(summary) = read_attribute(element, "summary") {
                let mut node = create_element("summary", &[], alloc);
                node.children = parse_fragment(&summary, alloc);
                remove_attribute(element, "summary");
                element.children.insert(0, Node::Element(node));
            }
        }
        "htmlpress-card" if read_attribute(element, "title").is_some() => {
            let heading = create_card_heading(element, alloc)?;
            for name in ["title", "href", "icon"] {
                remove_attribute(element, name);
            }
            element.children.insert(0, heading);
        }
        "htmlpress-button" => {
            let (href, label) = (
                read_attribute(element, "href"),
                read_attribute(element, "label"),
            );
            let (Some(href), Some(label)) = (href, label) else {
                return Err("<htmlpress-button> needs an href and a label".to_owned());
            };
            let mut link = create_element("a", &[("href", &href)], alloc);
            link.children = parse_fragment(&label, alloc);
            remove_attribute(element, "href");
            remove_attribute(element, "label");
            element.children.clear();
            element.children.push(Node::Element(link));
            return Ok(false);
        }
        "htmlpress-video" => write_media_link(element, "youtube", alloc)?,
        "htmlpress-audio" => write_media_link(element, "soundcloud", alloc)?,
        // Livemark's `#light` and `#dark` image suffixes, as the attribute the theme reads
        "img" => {
            let src = read_attribute(element, "src")
                .unwrap_or_default()
                .into_owned();
            for theme in ["light", "dark"] {
                if let Some(bare) = src.strip_suffix(&format!("#{theme}")) {
                    write_attribute(element, "src", bare, alloc);
                    write_attribute(element, "data-theme", theme, alloc);
                }
            }
        }
        _ => {}
    }
    Ok(true)
}

// Livemark's diagrams are fences, so a bare `language-mermaid` block becomes the diagram,
// its code the `<pre>`'s own text
fn wrap_mermaid<'a>(pre: &mut Element<'a>, alloc: &'a Allocator) {
    let [Node::Element(code)] = &mut pre.children[..] else {
        return;
    };
    let mermaid = pre.properties.is_empty()
        && code.tag_name == "code"
        && code.properties.len() == 1
        && read_attribute(code, "class").as_deref() == Some("language-mermaid");
    if !mermaid {
        return;
    }
    let mut inner = create_element("pre", &[], alloc);
    inner.children = std::mem::replace(&mut code.children, ArenaVec::new_in(alloc));
    pre.tag_name = "htmlpress-mermaid";
    pre.children.clear();
    pre.children.push(Node::Element(inner));
}

fn wrap_tabs<'a>(
    tabs: &mut Element<'a>,
    mut titles: VecDeque<String>,
    alloc: &'a Allocator,
) {
    for child in tabs.children.iter_mut() {
        let Node::Element(pre) = child else {
            continue;
        };
        if pre.tag_name != "pre" {
            continue;
        }
        let Some(title) = titles.pop_front() else {
            break;
        };
        let mut tab = create_element("htmlpress-tab", &[("title", &title)], alloc);
        let block = std::mem::replace(
            child,
            Node::Element(create_element("template", &[], alloc)),
        );
        tab.children.push(block);
        *child = Node::Element(tab);
    }
}

// Written here, not in the browser, so the heading and its link reach search and the
// Markdown copy
fn create_card_heading<'a>(
    card: &Element<'a>,
    alloc: &'a Allocator,
) -> Result<Node<'a>, String> {
    let title = read_attribute(card, "title").unwrap_or_default();
    let mut nodes = ArenaVec::new_in(alloc);
    if let Some(name) = read_attribute(card, "icon") {
        let svg = render_icon(&name).ok_or_else(|| {
            format!(
                "<htmlpress-card icon=\"{name}\">: no such icon; the icons are: {}",
                list_icons()
            )
        })?;
        let mut icon = create_element("htmlpress-icon", &[("name", &name)], alloc);
        icon.children = parse_fragment(&svg, alloc);
        nodes.push(Node::Element(icon));
    }
    nodes.extend(parse_fragment(&title, alloc));
    let mut heading = create_element("h3", &[], alloc);
    match read_attribute(card, "href") {
        Some(href) => {
            let mut link = create_element("a", &[("href", &href)], alloc);
            link.children = nodes;
            heading.children.push(Node::Element(link));
        }
        None => heading.children = nodes,
    }
    Ok(Node::Element(heading))
}

// The embed is the script's; the link it starts as keeps a page without it pointing there
fn write_media_link<'a>(
    element: &mut Element<'a>,
    kind: &str,
    alloc: &'a Allocator,
) -> Result<(), String> {
    let tag = element.tag_name;
    let found = read_attribute(element, "type").unwrap_or_default();
    if found != kind {
        return Err(format!("<{tag}> needs type=\"{kind}\", got \"{found}\""));
    }
    let url = read_attribute(element, "url")
        .unwrap_or_default()
        .into_owned();
    if url.is_empty() {
        return Err(format!("<{tag} type=\"{kind}\"> needs a url"));
    }
    let mut link = create_element("a", &[("href", &url)], alloc);
    link.children.push(create_text(&url, alloc));
    element.children.clear();
    element.children.push(Node::Element(link));
    Ok(())
}

fn create_package_tabs<'a>(
    commands: &str,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    let mut tabs = ArenaVec::new_in(alloc);
    for manager in MANAGERS {
        let mut code = create_element("code", &[], alloc);
        code.children
            .push(create_text(&convert_command(commands, manager), alloc));
        let mut pre = create_element("pre", &[], alloc);
        pre.children.push(Node::Element(code));
        let mut tab = create_element("htmlpress-tab", &[("title", manager)], alloc);
        tab.children.push(Node::Element(pre));
        tabs.push(Node::Element(tab));
    }
    tabs
}
