use serde::{Deserialize, Serialize};

/// The project's `htmlpress.toml`. Every key is optional.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Config {
    /// The site title in the header and after each page title; defaults to the folder name.
    pub title: Option<String>,
    /// The site description, used for pages without one of their own.
    pub description: Option<String>,
    /// The public URL the site is served at, such as `https://htmlpress.dev`.
    pub site: Option<String>,
}
