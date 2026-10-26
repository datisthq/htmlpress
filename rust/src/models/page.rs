use serde::Serialize;

/// One HTML source page, split into what the layout needs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Page {
    /// The path it is built to, relative to the output, with `/` separators.
    pub path: String,
    /// The source file's path relative to the project root, which `include`'s base
    /// prefixes: `docs/guide.html` for the page built to `guide.html`.
    pub source: String,
    /// The root-relative URL it is served at: `/`, `/guides/`, `/guides/build.html`.
    pub url: String,
    /// The URL it shares with its translations: its own without the language's prefix.
    pub key: String,
    /// `<meta name="language">`, else the `.<code>.html` suffix: the code of the language it
    /// is written in; the default language's when empty.
    pub language: String,
    /// A default-language page listed in another language's navigation for want of a
    /// translation; it links to `key`, and is built only in its own language.
    #[serde(skip)]
    pub untranslated: bool,
    /// `<title>`, else the first `<h1>`, else the file name.
    pub title: String,
    /// `<meta name="label">`: a shorter title for the navigation.
    pub label: Option<String>,
    /// `<meta name="icon">`: a built-in icon before the page's top-level navigation entry,
    /// or its folder's group for an index page.
    pub icon: Option<String>,
    /// `<meta name="sidebar" content="false">` leaves the page out of the navigation and
    /// the previous and next links; it is still built.
    pub sidebar: bool,
    /// `<meta name="source-url">`: where "Edit this page" leads instead of `edit`'s URL.
    pub source_url: Option<String>,
    /// `<meta name="source-action" content="view">` labels that link "View source".
    pub source_action: SourceAction,
    /// `<meta name="description">`.
    pub description: Option<String>,
    /// `<meta name="date">`: when the page was published, `YYYY-MM-DD`.
    pub date: Option<String>,
    /// Each `<meta name="author">`.
    pub authors: Vec<String>,
    /// `<meta name="keywords">`, split at commas: a blog post's tags.
    pub tags: Vec<String>,
    /// When the source last changed, from git, `YYYY-MM-DD`.
    pub updated: Option<String>,
    /// `<meta name="image">`: the social preview image's URL.
    pub image: Option<String>,
    /// `<meta name="order">`: lower comes first in the navigation.
    pub order: Option<i64>,
    /// `<meta name="layout">`: the page is wrapped in `.htmlpress/layouts/<name>.html`
    /// instead of `layout.html`.
    pub layout: Option<String>,
    /// What the page's `<head>` holds besides its title and the metas htmlpress reads.
    pub head: String,
    /// The page's body, without the `<html>`, `<head>` and `<body>` wrappers.
    pub content: String,
    /// The `<h2>` to `<h4>` headings, in order, for the page's table of contents.
    pub toc: Vec<Heading>,
}

/// What the page's source link offers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceAction {
    #[default]
    Edit,
    View,
}

/// One heading of a page, linked from its table of contents.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Heading {
    /// The heading's `id`: its own, else a slug of its text made unique on the page.
    pub id: String,
    pub title: String,
    /// 2 for `<h2>` to 4 for `<h4>`.
    pub level: u8,
}
