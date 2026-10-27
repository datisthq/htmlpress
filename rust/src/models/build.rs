use std::path::PathBuf;

/// What a build reads and where it writes.
#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub root: PathBuf,
    /// The output folder; [`BuildOptions::new`] puts it at `.htmlpress/build` in the root.
    pub output: PathBuf,
    /// Add the dev server's reload script to every page.
    pub reload: bool,
}

impl BuildOptions {
    /// Options for a production build of `root` into its default output folder.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let output = root.join(crate::settings::OUTPUT_DIR);
        Self {
            root,
            output,
            reload: false,
        }
    }
}

/// A count and its noun, singular for one: `1 page`, `2 pages`.
pub fn format_count(count: usize, noun: &str) -> String {
    match count {
        1 => format!("1 {noun}"),
        _ => format!("{count} {noun}s"),
    }
}

/// What a build wrote.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildReport {
    pub output: PathBuf,
    pub pages: usize,
    pub files: usize,
}
