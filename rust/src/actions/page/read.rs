use std::cell::RefCell;

use textlane::hast::{Element, Node, Root};
use textlane::html::{read_attribute, write_attribute};
use textlane::{Allocator, ArenaVec, File, FrozenProcessor, Tree};

use crate::actions::icon::render::render_icon;
use crate::actions::node::collect::collect_text;
use crate::actions::node::copy::copy_node;
use crate::actions::node::walk::{Step, walk_nodes};
use crate::actions::node::write::name_references;
use crate::actions::page::decorate::{Decoration, decorate_page, list_tags};
use crate::actions::page::parse::parse_html_source;
use crate::actions::page::prefix::prefix_urls;
use crate::actions::page::prepare::prepare_page;
use crate::actions::path::slugify::slugify_path;
use crate::actions::pipeline::create::create_pipeline;
use crate::actions::site::index::read_chunks;
use crate::models::error::Error;
use crate::models::page::{Heading, Page, SourceAction};
use crate::models::pipeline::{Pipeline, PipelineOptions};
use crate::settings::NOT_FOUND_PAGE;

thread_local! {
    static DEFAULT: RefCell<Result<Pipeline, String>> =
        RefCell::new(create_pipeline(&PipelineOptions::default()));
}

/// Split one HTML source into a [`Page`]. `path` is relative to the project root. A full
/// document and a bare fragment both work: the wrappers are dropped, the content kept.
pub fn read_page(path: &str, html: &str) -> crate::Result<Page> {
    read_decorated_page(path, html, None)
}

/// [`read_page`], its content decorated as a layout shows it when `decoration` is given.
pub(crate) fn read_decorated_page(
    path: &str,
    html: &str,
    decoration: Option<&Decoration>,
) -> crate::Result<Page> {
    DEFAULT.with_borrow_mut(|pipeline| match pipeline {
        Ok(pipeline) => read_page_with(path, html, pipeline, decoration),
        Err(message) => Err(Error::page(path)(message.clone())),
    })
}

/// [`read_decorated_page`] through `pipeline`, parsed into an arena of its own.
pub(crate) fn read_page_with(
    path: &str,
    html: &str,
    pipeline: &mut Pipeline,
    decoration: Option<&Decoration>,
) -> crate::Result<Page> {
    let alloc = Allocator::new();
    let text = alloc.alloc_str(html);
    let tree =
        parse_html_source(pipeline, path, text, &alloc).map_err(Error::page(path))?;
    let mut page = read_source(path, &tree)?;
    finish_page(&mut page, tree, pipeline, decoration, &alloc)?;
    Ok(page)
}

/// A page's facts, read from its parsed source: title, metas, URL. [`finish_page`] then
/// makes the same tree its content.
pub(crate) fn read_source(path: &str, tree: &Node<'_>) -> crate::Result<Page> {
    let facts = read_facts(tree).map_err(Error::page(path))?;
    let output = match &facts.route {
        Some(route) => create_route_path(route).map_err(Error::page(path))?,
        None => slugify_path(path),
    };
    let title = [facts.title, facts.heading]
        .into_iter()
        .map(|text| text.trim().to_owned())
        .find(|text| !text.is_empty())
        .unwrap_or_else(|| read_stem(path));
    Ok(Page {
        url: create_url(&output),
        path: output,
        source: path.to_owned(),
        key: String::new(),
        language: facts.language,
        untranslated: false,
        title,
        label: facts.label,
        icon: facts.icon,
        sidebar: facts.sidebar,
        breadcrumbs: facts.breadcrumbs,
        source_url: facts.source_url,
        source_action: facts.source_action,
        description: facts.description,
        date: facts.date,
        authors: facts.authors,
        tags: facts.tags,
        updated: None,
        image: facts.image,
        order: facts.order,
        layout: facts.layout,
        head: String::new(),
        content: String::new(),
        heading: String::new(),
        toc: vec![],
        markdown: String::new(),
        chunks: vec![],
        components: Default::default(),
    })
}

/// Make a page read by [`read_source`] whole from its tree, never parsing it again: the
/// pipeline's prose, math, heading ids, links and code colors, then its search text and
/// Markdown copy, its content decorated as the layout shows it, and its extra head tags.
pub(crate) fn finish_page<'a>(
    page: &mut Page,
    tree: Node<'a>,
    pipeline: &mut Pipeline,
    decoration: Option<&Decoration>,
    alloc: &'a Allocator,
) -> crate::Result<()> {
    let path = page.source.clone();
    let fail = |message: String| Error::page(&path)(message);
    let failed = |error: textlane::Error| error.message;
    let mut tree = tree;
    prepare_page(&mut tree, alloc).map_err(fail)?;
    let document = is_document_tree(&tree);
    let processor = match document {
        true => pipeline.document.freeze(),
        false => pipeline.fragment.freeze(),
    };
    let processor = processor.map_err(failed).map_err(fail)?;
    let mut file = File::new();
    file.path = Some(page.source.clone());
    let tree = (processor.run_sync(Tree::Hast(tree), &mut file, alloc))
        .map_err(failed)
        .map_err(fail)?;
    // KaTeX reports bad LaTeX as a message and draws it red; a page must not ship that
    let math = file.messages.iter().find(|message| {
        matches!(message.rule.as_deref(), Some("parseerror" | "rangeerror"))
    });
    if let Some(message) = math {
        return Err(fail(format!("invalid math: {}", message.reason)));
    }
    let Tree::Hast(mut tree) = tree else {
        return Err(fail("the page pipeline did not end in HTML".to_owned()));
    };
    let facts = read_facts(&tree).map_err(fail)?;
    let headings = create_ids(facts.headings);
    write_ids(&mut tree, &headings, alloc);
    page.toc = match facts.toc {
        true => (headings.into_iter())
            .filter(|(_, listed)| *listed)
            .map(|(heading, _)| heading)
            .collect(),
        false => vec![],
    };
    let copy = Tree::Hast(copy_node(&tree, alloc));
    let (mut head, mut content) = split_page(tree, document, alloc);
    page.chunks = read_chunks(&content);
    let markdown = pipeline.markdown.freeze().map_err(failed).map_err(fail)?;
    let mut file = File::new();
    let copy = (markdown.run_sync(copy, &mut file, alloc))
        .map_err(failed)
        .map_err(fail)?;
    let body = (markdown.stringify(&copy, &mut file))
        .map_err(failed)
        .map_err(fail)?;
    let mut parts = vec![format!("# {}", page.title)];
    if let Some(description) = &page.description {
        parts.push(format!("> {description}"));
    }
    if !body.trim().is_empty() {
        parts.push(body.trim().to_owned());
    }
    page.markdown = format!("{}\n", parts.join("\n\n"));
    let mut heading = ArenaVec::new_in(alloc);
    let components = match decoration {
        Some(decoration) => {
            let tags = decorate_page(&mut content, &mut heading, page, decoration, alloc)
                .map_err(fail)?;
            if !decoration.base.is_empty() {
                prefix_urls(&mut head, decoration.base, alloc);
            }
            tags
        }
        None => list_tags(&content, &[]),
    };
    page.components = components;
    page.content = write_root(processor, content, &mut file)
        .map_err(fail)?
        .trim()
        .to_owned();
    if !heading.is_empty() {
        page.heading = (write_root(processor, heading, &mut file).map_err(fail)?)
            .trim()
            .to_owned();
    }
    let mut lines = vec![];
    for node in head {
        if matches!(&node, Node::Text(text) if text.value.trim().is_empty()) {
            continue;
        }
        let mut single = ArenaVec::new_in(alloc);
        single.push(node);
        lines.push(
            write_root(processor, single, &mut file)
                .map_err(fail)?
                .trim()
                .to_owned(),
        );
    }
    page.head = lines.join("\n");
    Ok(())
}

#[derive(Default)]
struct Facts {
    title: String,
    heading: String,
    description: Option<String>,
    label: Option<String>,
    icon: Option<String>,
    route: Option<String>,
    sidebar: bool,
    breadcrumbs: Option<bool>,
    toc: bool,
    source_url: Option<String>,
    source_action: SourceAction,
    date: Option<String>,
    authors: Vec<String>,
    tags: Vec<String>,
    image: Option<String>,
    order: Option<i64>,
    layout: Option<String>,
    language: String,
    /// Each heading, and whether it belongs in the table of contents.
    headings: Vec<(Heading, bool)>,
}

// Where a heading is a block's title, not a section of the page: a card's, a callout's
// or a tab's
const HEADING_BOXES: &[&str] = &["htmlpress-callout", "htmlpress-card", "htmlpress-tabs"];

fn read_facts(tree: &Node<'_>) -> Result<Facts, String> {
    let mut facts = Facts {
        sidebar: true,
        toc: true,
        ..Facts::default()
    };
    let mut first_heading = true;
    // Each node with whether a box above it holds its headings
    let mut stack: Vec<(&Node, bool)> = vec![(tree, false)];
    while let Some((node, boxed)) = stack.pop() {
        let element = match node {
            Node::Root(root) => {
                stack.extend(root.children.iter().rev().map(|child| (child, boxed)));
                continue;
            }
            Node::Element(element) => element,
            _ => continue,
        };
        match element.tag_name {
            "title" => facts.title.push_str(&collect_text(&element.children)),
            "h1" if std::mem::take(&mut first_heading) => {
                facts.heading = collect_text(&element.children);
            }
            "h2" | "h3" | "h4" | "h5" | "h6" => {
                facts.headings.push(read_heading(element, boxed)?)
            }
            "meta" => read_meta(element, &mut facts)?,
            _ => {}
        }
        let boxed = boxed || HEADING_BOXES.contains(&element.tag_name);
        stack.extend(element.children.iter().rev().map(|child| (child, boxed)));
    }
    Ok(facts)
}

fn read_heading(element: &Element<'_>, boxed: bool) -> Result<(Heading, bool), String> {
    let heading = Heading {
        id: read_attribute(element, "id")
            .unwrap_or_default()
            .into_owned(),
        title: collect_text(&element.children),
        level: (element.tag_name.as_bytes().get(1)).map_or(2, |digit| digit - b'0'),
    };
    // `data-toc="false"` keeps a heading off the table of contents; "only" keeps it
    // there but off the page, which the theme collapses to nothing in place
    let listed = match read_attribute(element, "data-toc")
        .as_deref()
        .map(str::trim)
    {
        None | Some("only" | "true") => true,
        Some("false") => false,
        Some(other) => {
            return Err(format!(
                "data-toc on a heading must be false or only, got \"{other}\""
            ));
        }
    };
    // An <h5> or <h6> gets an id and an anchor, never a table of contents entry; a step's
    // title is part of its step
    let step = read_attribute(element, "data-step").is_some();
    let listed = listed && heading.level <= 4 && !boxed && !step;
    Ok((heading, listed))
}

fn read_meta(element: &Element<'_>, facts: &mut Facts) -> Result<(), String> {
    let Some(name) = read_attribute(element, "name") else {
        return Ok(());
    };
    let raw = read_attribute(element, "content").map(|content| content.into_owned());
    let content = raw.clone().unwrap_or_default();
    let trimmed = (!content.trim().is_empty()).then(|| content.trim().to_owned());
    let flag = |name: &str| match content.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!(
            "<meta name=\"{name}\"> must be true or false, got \"{other}\""
        )),
    };
    match name.as_ref() {
        "description" => facts.description = raw,
        "date" => {
            facts.date = Some(read_date(&content).ok_or_else(|| {
                format!(
                    "<meta name=\"date\"> must be a YYYY-MM-DD date, got \"{content}\""
                )
            })?);
        }
        "author" => facts.authors.extend(trimmed),
        "keywords" => {
            facts.tags = (content.split(','))
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_owned)
                .collect();
        }
        "image" => facts.image = raw,
        "order" => {
            facts.order = Some(content.trim().parse().map_err(|_| {
                format!("<meta name=\"order\"> must be an integer, got \"{content}\"")
            })?);
        }
        "label" => facts.label = trimmed,
        "icon" => {
            if render_icon(content.trim()).is_none() {
                return Err(format!(
                    "<meta name=\"icon\">: no built-in icon \"{content}\""
                ));
            }
            facts.icon = Some(content.trim().to_owned());
        }
        "path" => facts.route = trimmed,
        "language" => facts.language = trimmed.unwrap_or_default(),
        "sidebar" => facts.sidebar = flag("sidebar")?,
        "breadcrumbs" => facts.breadcrumbs = Some(flag("breadcrumbs")?),
        "toc" => facts.toc = flag("toc")?,
        "source-url" => facts.source_url = trimmed,
        "source-action" => {
            facts.source_action = match content.trim() {
                "edit" => SourceAction::Edit,
                "view" => SourceAction::View,
                _ => {
                    return Err(format!(
                        "<meta name=\"source-action\"> must be edit or view, got \"{content}\""
                    ));
                }
            };
        }
        "layout" => {
            let layout = content.trim();
            if !is_layout_name(layout) {
                return Err(format!(
                    "<meta name=\"layout\"> must be lowercase letters, digits and \
                     hyphens, got \"{layout}\""
                ));
            }
            facts.layout = Some(layout.to_owned());
        }
        _ => {}
    }
    Ok(())
}

// Ids already in the page are reserved first, so a generated slug never steals one that
// a link elsewhere points at.
fn create_ids(headings: Vec<(Heading, bool)>) -> Vec<(Heading, bool)> {
    let mut taken: Vec<String> = headings
        .iter()
        .filter(|(heading, _)| !heading.id.is_empty())
        .map(|(heading, _)| heading.id.clone())
        .collect();
    headings
        .into_iter()
        .map(|(heading, listed)| {
            let title = heading
                .title
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let id = match heading.id.is_empty() {
                true => create_slug(&title, &mut taken),
                false => heading.id,
            };
            (
                Heading {
                    id,
                    title,
                    ..heading
                },
                listed,
            )
        })
        .collect()
}

fn create_slug(title: &str, taken: &mut Vec<String>) -> String {
    let words: Vec<String> = title
        .to_lowercase()
        .split(|char: char| !char.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect();
    let base = match words.is_empty() {
        true => "section".to_owned(),
        false => words.join("-"),
    };
    let mut slug = base.clone();
    let mut count = 1;
    while taken.contains(&slug) {
        count += 1;
        slug = format!("{base}-{count}");
    }
    taken.push(slug.clone());
    slug
}

// In the order `read_facts` met them, every heading without an id gets its own
fn write_ids<'a>(
    tree: &mut Node<'a>,
    headings: &[(Heading, bool)],
    alloc: &'a Allocator,
) {
    let Node::Root(root) = tree else {
        return;
    };
    let mut index = 0;
    let _ = walk_nodes(&mut root.children, &mut |element: &mut Element<'a>| {
        if matches!(element.tag_name, "h2" | "h3" | "h4" | "h5" | "h6") {
            if let Some((heading, _)) = headings.get(index)
                && read_attribute(element, "id").is_none_or(|id| id.is_empty())
            {
                write_attribute(element, "id", &heading.id, alloc);
            }
            index += 1;
        }
        Ok::<_, ()>(Step::Continue)
    });
}

fn is_document_tree(tree: &Node<'_>) -> bool {
    match tree {
        Node::Root(root) => (root.children.iter())
            .any(|child| matches!(child, Node::Element(html) if html.tag_name == "html")),
        _ => false,
    }
}

// A document's head and body; a fragment's leading head tags, which a browser would
// parse into its head, and the rest. Neither keeps what the layout writes itself.
fn split_page<'a>(
    tree: Node<'a>,
    document: bool,
    alloc: &'a Allocator,
) -> (ArenaVec<'a, Node<'a>>, ArenaVec<'a, Node<'a>>) {
    let mut head = ArenaVec::new_in(alloc);
    let mut content = ArenaVec::new_in(alloc);
    let Node::Root(root) = tree else {
        return (head, content);
    };
    match document {
        true => {
            for node in root.children {
                match node {
                    Node::Doctype(_) => {}
                    Node::Element(html) if html.tag_name == "html" => {
                        for child in html.children {
                            match child {
                                Node::Element(part) if part.tag_name == "head" => {
                                    head.extend(part.children)
                                }
                                Node::Element(part) if part.tag_name == "body" => {
                                    content.extend(part.children)
                                }
                                child => content.push(child),
                            }
                        }
                    }
                    node => content.push(node),
                }
            }
        }
        false => {
            let mut started = false;
            for node in root.children {
                started = started
                    || match &node {
                        Node::Element(element) => !HEAD_TAGS.contains(&element.tag_name),
                        Node::Text(text) => !text.value.trim().is_empty(),
                        _ => true,
                    };
                match started {
                    true => content.push(node),
                    false => head.push(node),
                }
            }
        }
    }
    for nodes in [&mut head, &mut content] {
        let _ = walk_nodes(nodes, &mut |element| {
            Ok::<_, ()>(match is_owned_by_layout(element) {
                true => Step::Remove,
                false => Step::Continue,
            })
        });
    }
    (head, content)
}

const HEAD_TAGS: &[&str] = &["title", "meta", "link", "style", "script", "base"];

// The layout writes the charset, viewport, title and description itself; a second copy
// from the page would win or clash depending on the browser.
const OWNED_BY_LAYOUT: &[&str] = &[
    "viewport",
    "description",
    "image",
    "order",
    "layout",
    "label",
    "icon",
    "path",
    "language",
    "sidebar",
    "breadcrumbs",
    "toc",
    "source-url",
    "source-action",
];

fn is_owned_by_layout(element: &Element<'_>) -> bool {
    match element.tag_name {
        "title" => true,
        "meta" => {
            read_attribute(element, "charset").is_some()
                || read_attribute(element, "name")
                    .is_some_and(|name| OWNED_BY_LAYOUT.contains(&name.as_ref()))
        }
        _ => false,
    }
}

fn write_root<'a>(
    processor: &FrozenProcessor,
    nodes: ArenaVec<'a, Node<'a>>,
    file: &mut File,
) -> Result<String, String> {
    let root = Tree::Hast(Node::Root(Root {
        children: nodes,
        position: None,
        data: None,
        quirks_mode: None,
    }));
    let html = processor
        .stringify(&root, file)
        .map_err(|error| error.message)?;
    Ok(name_references(&html))
}

fn read_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".html").unwrap_or(name).to_owned()
}

/// A layout's name, as `<meta name="layout">` gives it and `.htmlpress/layouts/` files it.
pub(crate) fn is_layout_name(name: &str) -> bool {
    name.starts_with(|char: char| char.is_ascii_lowercase())
        && name
            .chars()
            .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-')
}

/// The `YYYY-MM-DD` a date starts with, so a full timestamp is accepted too; None when
/// it does not start with one.
pub(crate) fn read_date(text: &str) -> Option<String> {
    let date = text.trim().get(..10)?;
    let parts: Vec<&str> = date.split('-').collect();
    let valid = matches!(parts.as_slice(), [year, month, day]
        if year.len() == 4 && month.len() == 2 && day.len() == 2
            && parts.iter().all(|part| part.chars().all(|char| char.is_ascii_digit())));
    valid.then(|| date.to_owned())
}

/// The page path a `path` override stands for: `/intro/` → `intro/index.html`,
/// `/intro.html` → `intro.html`, both served at `/intro/`. Anything else is an error,
/// since a URL that is neither a folder nor an `.html` file could not name the page.
pub(crate) fn create_route_path(route: &str) -> Result<String, String> {
    let Some(relative) = route.strip_prefix('/') else {
        return Err(format!("path must start with \"/\", got \"{route}\""));
    };
    if relative.split('/').any(|part| part == "..") {
        return Err(format!("path must stay inside the site, got \"{route}\""));
    }
    match (
        relative.is_empty() || relative.ends_with('/'),
        relative.ends_with(".html"),
    ) {
        (true, _) => Ok(format!("{relative}index.html")),
        (false, true) => Ok(relative.to_owned()),
        (false, false) => Err(format!(
            "path must end with \"/\" or \".html\", got \"{route}\""
        )),
    }
}

/// The folder URL a page is served at: `guides/build.html` → `/guides/build/`, and
/// `guides/index.html` → `/guides/`. The 404 page keeps its file, where hosts look for it.
pub(crate) fn create_url(path: &str) -> String {
    let output = create_output_path(path);
    match output.strip_suffix("index.html") {
        Some(folder) if folder.is_empty() || folder.ends_with('/') => {
            format!("/{folder}")
        }
        _ => format!("/{output}"),
    }
}

/// The file a page is written to: `guides/build.html` → `guides/build/index.html`.
pub(crate) fn create_output_path(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    match path.strip_suffix(".html") {
        Some(stem) if name != "index.html" && path != NOT_FOUND_PAGE => {
            format!("{stem}/index.html")
        }
        _ => path.to_owned(),
    }
}

#[cfg(test)]
#[path = "read_unit.rs"]
mod unit;
