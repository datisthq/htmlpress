use std::collections::BTreeMap;

use textlane::FrozenProcessor;

use crate::models::config::PluginEntry;

/// What a site's page pipeline does beside the prose, math and heading ids every page gets.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PipelineOptions {
    /// Rewrite relative URLs root-relative, from this include base; `None` leaves them.
    pub rebase: Option<String>,
    /// Each page's file to its URL, for links that name a page's file.
    pub links: BTreeMap<String, String>,
    /// The light and dark code themes; `None` leaves code blocks plain.
    pub themes: Option<(String, String)>,
    /// The project's textlane plugins at the `html` hook.
    pub plugins: Vec<PluginEntry>,
}

/// What compiling a Markdown page or partial takes from the project.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct MarkdownOptions {
    /// The project's own components, each a `:::name` directive.
    pub components: Vec<String>,
    /// The project's textlane plugins at the `md` hook.
    pub plugins: Vec<PluginEntry>,
}

/// The textlane processors a page goes through once: one for a full document, one for a
/// fragment, and one that turns the same tree into the page's Markdown copy.
pub(crate) struct Pipeline {
    pub document: FrozenProcessor,
    pub fragment: FrozenProcessor,
    pub markdown: FrozenProcessor,
}
