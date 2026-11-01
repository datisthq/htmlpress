use std::path::PathBuf;

use textlane::hast::Node;
use textlane::{Allocator, ArenaVec, File, Tree};

use crate::actions::node::copy::{copy_node, copy_nodes};
use crate::actions::page::compile::{compile_markdown, create_document};
use crate::models::pipeline::Pipeline;
use crate::models::source::{Compiled, Parsed, Sources};

/// The source at `key`, its canonical path, parsed on its first read, when `read` gives
/// its text: compiled when `markdown`, else parsed as HTML.
pub(crate) fn load_source<'s, 'a>(
    sources: &'s mut Sources<'a>,
    key: PathBuf,
    path: &str,
    markdown: bool,
    read: impl FnOnce() -> Result<String, String>,
) -> Result<&'s Parsed<'a>, String> {
    if !sources.files.contains_key(&key) {
        let alloc = sources.alloc;
        let text = alloc.alloc_str(&read()?);
        sources.sizes.insert(key.clone(), text.len());
        let parsed = match markdown {
            true => Parsed::Markdown(compile_markdown(text, &sources.markdown, alloc)?),
            false => {
                Parsed::Html(parse_html_source(&mut sources.parser, path, text, alloc)?)
            }
        };
        sources.files.insert(key.clone(), parsed);
    }
    (sources.files.get(&key)).ok_or_else(|| format!("{path} was not read"))
}

/// An HTML page's text as hast, through the parser of the project's pipeline: a document
/// when it has its own `<html>` or `<head>`, else a fragment.
pub(crate) fn parse_html_source<'a>(
    parser: &mut Pipeline,
    path: &str,
    text: &'a str,
    alloc: &'a Allocator,
) -> Result<Node<'a>, String> {
    let parser = match is_document(text) {
        true => &mut parser.document,
        false => &mut parser.fragment,
    };
    let processor = parser.freeze().map_err(|error| error.message)?;
    let mut file = File::new();
    file.path = Some(path.to_owned());
    match processor.parse(text, &mut file, alloc) {
        Ok(Tree::Hast(node)) => Ok(node),
        Ok(_) => Err("the HTML parser did not return hast".to_owned()),
        Err(error) => Err(error.message),
    }
}

/// A parsed source as the page it builds, a Markdown one as a document with its
/// frontmatter's head; a copy, so the source stays as parsed for an include of it.
pub(crate) fn create_page_tree<'a>(
    parsed: &Parsed<'a>,
    alloc: &'a Allocator,
) -> Result<Node<'a>, String> {
    match parsed {
        Parsed::Html(node) => Ok(copy_node(node, alloc)),
        Parsed::Markdown(compiled) => create_document(
            Compiled {
                body: copy_nodes(&compiled.body, alloc),
                matter: compiled.matter.clone(),
            },
            alloc,
        ),
    }
}

/// A parsed source as the nodes an include puts in a page: an HTML document's body, an
/// HTML fragment or a Markdown body whole, its frontmatter dropped.
pub(crate) fn create_partial_nodes<'a>(
    parsed: &Parsed<'a>,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    match parsed {
        Parsed::Markdown(compiled) => copy_nodes(&compiled.body, alloc),
        Parsed::Html(Node::Root(root)) => match find_body(&root.children) {
            Some(body) => copy_nodes(body, alloc),
            None => copy_nodes(&root.children, alloc),
        },
        Parsed::Html(node) => copy_nodes(std::slice::from_ref(node), alloc),
    }
}

/// Whether a page's text has its own `<html>` or `<head>`, which makes it a document;
/// anything else is a fragment, whose leading `<title>` and metas stay where they are.
pub(crate) fn is_document(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    ["<html", "<head"].iter().any(|tag| {
        lower.match_indices(tag).any(|(at, _)| {
            (lower.as_bytes().get(at + tag.len())).is_some_and(|next| {
                matches!(next, b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/')
            })
        })
    })
}

fn find_body<'n, 'a>(nodes: &'n [Node<'a>]) -> Option<&'n [Node<'a>]> {
    nodes.iter().find_map(|node| match node {
        Node::Element(html) if html.tag_name == "html" => {
            html.children.iter().find_map(|child| match child {
                Node::Element(body) if body.tag_name == "body" => {
                    Some(&body.children[..])
                }
                _ => None,
            })
        }
        _ => None,
    })
}
