use serde::Serialize;

/// One HTML source page, split into what the layout needs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Page {
    /// The source path relative to the project root, with `/` separators.
    pub path: String,
    /// The root-relative URL it is served at: `/`, `/guides/`, `/guides/build.html`.
    pub url: String,
    /// `<title>`, else the first `<h1>`, else the file name.
    pub title: String,
    /// `<meta name="description">`.
    pub description: Option<String>,
    /// `<meta name="order">`: lower comes first in the navigation.
    pub order: Option<i64>,
    /// What the page's `<head>` holds besides its title and the metas htmlpress reads.
    pub head: String,
    /// The page's body, without the `<html>`, `<head>` and `<body>` wrappers.
    pub content: String,
}
