use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{Map, Value};
use textlane::hast::Node;
use textlane::{Allocator, ArenaVec};

use crate::models::page::Page;
use crate::models::pipeline::{MarkdownOptions, Pipeline};

/// A Markdown file as hast: its body's nodes and the frontmatter it carried.
pub(crate) struct Compiled<'a> {
    pub(crate) body: ArenaVec<'a, Node<'a>>,
    pub(crate) matter: Map<String, Value>,
}

/// A project file as hast, as its parse left it: never changed, only copied.
pub(crate) enum Parsed<'a> {
    /// An HTML file, parsed as a document when it has its own `<html>` or `<head>`.
    Html(Node<'a>),
    Markdown(Compiled<'a>),
}

/// Every source a build reads, each parsed once into the build's arena: pages and the
/// partials they include, by their canonical path.
pub(crate) struct Sources<'a> {
    pub(crate) alloc: &'a Allocator,
    pub(crate) markdown: MarkdownOptions,
    /// Only parses: the project's `html` plugins may reconfigure `html.parse`.
    pub(crate) parser: Pipeline,
    pub(crate) files: HashMap<PathBuf, Parsed<'a>>,
    /// Each file's length in bytes, which an include spends of a page's budget.
    pub(crate) sizes: HashMap<PathBuf, usize>,
}

/// A page read from its source, with the tree it was read from, until its pipeline
/// makes that tree its content.
pub(crate) struct Draft<'a> {
    pub(crate) page: Page,
    pub(crate) tree: Node<'a>,
}
