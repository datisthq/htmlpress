/// A built-in file a project can escape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Asset {
    /// Its path under `.htmlpress/`, such as `components/tabs/tabs.js`.
    pub path: &'static str,
    pub content: &'static str,
}

/// A built-in asset and whether the project replaces it.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetStatus {
    pub path: &'static str,
    pub escaped: bool,
}

/// The prefix of every component's tag, left out of its path.
pub const COMPONENT_PREFIX: &str = "htmlpress-";

/// The path of a component's file, `htmlpress-tabs` at `components/tabs/tabs.<extension>`,
/// or `None` for a tag that is not a component's.
pub fn create_component_path(tag: &str, extension: &str) -> Option<String> {
    let name = tag.strip_prefix(COMPONENT_PREFIX)?;
    Some(format!("components/{name}/{name}.{extension}"))
}

/// A component file's tag and extension, or `None` for any other path.
pub fn read_component_path(path: &str) -> Option<(String, &str)> {
    let (name, file) = path.strip_prefix("components/")?.split_once('/')?;
    let (stem, extension) = file.rsplit_once('.')?;
    (stem == name).then(|| (format!("{COMPONENT_PREFIX}{name}"), extension))
}
