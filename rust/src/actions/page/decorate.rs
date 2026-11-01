use std::collections::{BTreeMap, BTreeSet};

use textlane::hast::Node;
use textlane::html::{read_attribute, remove_attribute, write_attribute};
use textlane::{Allocator, ArenaVec};

use crate::actions::code::title::add_titles;
use crate::actions::node::parse::parse_fragment;
use crate::actions::node::walk::{Step, walk_nodes};
use crate::actions::page::prefix::prefix_urls;
use crate::actions::page::render::create_toc_up_to;
use crate::models::asset::read_component_path;
use crate::models::language::SiteLanguage;
use crate::models::page::Page;

/// What turns a page's content into what its layout shows: the language its UI strings
/// are in, the site's base, and each project component's tag with its `.html`.
pub(crate) struct Decoration<'d> {
    pub(crate) language: &'d SiteLanguage,
    pub(crate) base: &'d str,
    pub(crate) fragments: &'d [(String, &'d str)],
}

/// The content as its layout shows it, after its search text and Markdown copy were
/// read: its own `<htmlpress-toc>`s filled, each callout's title in the page's language,
/// code blocks framed, project components filled, with a base every root-relative URL
/// under it, and its leading `<h1>` moved out into `heading`. Gives back the tags it
/// holds (see [`list_tags`]), read before the components are filled.
pub(crate) fn decorate_page<'a>(
    nodes: &mut ArenaVec<'a, Node<'a>>,
    heading: &mut ArenaVec<'a, Node<'a>>,
    page: &Page,
    decoration: &Decoration,
    alloc: &'a Allocator,
) -> Result<BTreeSet<String>, String> {
    let language = decoration.language;
    walk_nodes(nodes, &mut |element| match element.tag_name {
        "htmlpress-toc" => {
            let level = match read_attribute(element, "max-level")
                .as_deref()
                .map(str::trim)
            {
                None => 4,
                Some("2") => 2,
                Some("3") => 3,
                Some("4") => 4,
                Some(other) => {
                    return Err(format!(
                        "<htmlpress-toc max-level> must be 2, 3 or 4, got \"{other}\""
                    ));
                }
            };
            Ok(match create_toc_up_to(page, level, language) {
                Some(html) => {
                    element.children = parse_fragment(&html, alloc);
                    Step::Skip
                }
                None => Step::Remove,
            })
        }
        // A callout's default title is CSS, which cannot know the page's language, so
        // each gets its own or its type's; as `data-title`, since `title` is a tooltip
        "htmlpress-callout" => {
            let title = match read_attribute(element, "title") {
                Some(title) => title.into_owned(),
                None => {
                    let key =
                        match read_attribute(element, "type").as_deref().map(str::trim) {
                            Some("note") => "callout.note",
                            Some("tip") => "callout.tip",
                            Some("warning") => "callout.warning",
                            Some("danger") => "callout.danger",
                            _ => "callout.info",
                        };
                    language.read_message(key).to_owned()
                }
            };
            remove_attribute(element, "title");
            write_attribute(element, "data-title", &title, alloc);
            Ok(Step::Continue)
        }
        _ => Ok(Step::Continue),
    })?;
    add_titles(nodes, alloc)?;
    let tags = list_tags(nodes, decoration.fragments);
    fill_components(nodes, decoration.fragments, alloc);
    if !decoration.base.is_empty() {
        prefix_urls(nodes, decoration.base, alloc);
    }
    take_heading(nodes, heading);
    Ok(tags)
}

/// Each project component's tag with its `.html`, from the site's assets.
pub(crate) fn list_fragments(assets: &BTreeMap<String, String>) -> Vec<(String, &str)> {
    (assets.iter())
        .filter_map(|(path, html)| {
            let (tag, extension) = read_component_path(path)?;
            (extension == "html").then_some((tag, html.as_str()))
        })
        .collect()
}

/// Every tag in these nodes, and in the `.html` of each project component among them,
/// one level deep: the components whose CSS and JS the page links.
pub(crate) fn list_tags(
    nodes: &[Node<'_>],
    fragments: &[(String, &str)],
) -> BTreeSet<String> {
    let mut tags = BTreeSet::new();
    read_tags(nodes, &mut tags);
    let alloc = Allocator::new();
    for (tag, html) in fragments {
        if tags.contains(tag) {
            read_tags(&parse_fragment(html, &alloc), &mut tags);
        }
    }
    tags
}

fn read_tags(nodes: &[Node<'_>], tags: &mut BTreeSet<String>) {
    let mut stack: Vec<&Node> = nodes.iter().collect();
    while let Some(node) = stack.pop() {
        if let Node::Element(element) = node {
            tags.insert(element.tag_name.to_owned());
            stack.extend(element.children.iter());
        }
    }
}

/// Fill each project component's tag with its `.html`; markup filled in is left as it
/// is, so a component never fills itself.
pub(crate) fn fill_components<'a>(
    nodes: &mut ArenaVec<'a, Node<'a>>,
    fragments: &[(String, &str)],
    alloc: &'a Allocator,
) {
    if fragments.is_empty() {
        return;
    }
    let _ = walk_nodes(nodes, &mut |element| match fragments
        .iter()
        .find(|(tag, _)| tag == element.tag_name)
    {
        Some((_, html)) => {
            element.children = parse_fragment(html, alloc);
            Ok::<_, ()>(Step::Skip)
        }
        None => Ok(Step::Continue),
    });
}

// The first element, when it is an `<h1>`, as Livemark drops a body's leading one
fn take_heading<'a>(
    nodes: &mut ArenaVec<'a, Node<'a>>,
    heading: &mut ArenaVec<'a, Node<'a>>,
) {
    let first = nodes.iter().position(|node| match node {
        Node::Text(text) => !text.value.trim().is_empty(),
        Node::Comment(_) => false,
        _ => true,
    });
    if let Some(at) = first
        && matches!(nodes.get(at), Some(Node::Element(element)) if element.tag_name == "h1")
    {
        heading.push(nodes.remove(at));
    }
}
