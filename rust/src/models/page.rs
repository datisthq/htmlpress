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
    /// `<title>`, else the first `<h1>`, else the file name.
    pub title: String,
    /// `<meta name="description">`.
    pub description: Option<String>,
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
    /// The `<h2>` and `<h3>` headings, in order, for the page's table of contents.
    pub toc: Vec<Heading>,
}

/// One heading of a page, linked from its table of contents.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Heading {
    /// The heading's `id`: its own, else a slug of its text made unique on the page.
    pub id: String,
    pub title: String,
    /// 2 for `<h2>`, 3 for `<h3>`.
    pub level: u8,
}
