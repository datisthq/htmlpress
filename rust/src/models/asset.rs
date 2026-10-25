/// A built-in file a project can escape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Asset {
    /// Its path under `.htmlpress/`, such as `components/htmlpress-tabs.js`.
    pub path: &'static str,
    pub content: &'static str,
}

/// A built-in asset and whether the project replaces it.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetStatus {
    pub path: &'static str,
    pub escaped: bool,
}
