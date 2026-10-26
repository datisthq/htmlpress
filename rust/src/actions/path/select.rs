use std::collections::BTreeMap;

use crate::models::config::Config;

/// The project's files sorted by the config's `include` and `exclude`, each keyed by the
/// path it is served at; excluded ones and those outside every include base are in neither.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Selection {
    /// Served path to source path, both project-relative with `/` separators.
    pub pages: BTreeMap<String, String>,
    pub files: BTreeMap<String, String>,
    /// The patterns, as written, that match no file: most likely typos.
    pub unused: Vec<String>,
}

const DEFAULT_INCLUDE: &str = "**/*.html";

// `*` must stop at `/`, or `*.html` would take every page in every folder
const OPTIONS: glob::MatchOptions = glob::MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

/// Sort project-relative, `/`-separated paths into pages (`.html` matched by `include`,
/// `**/*.html` when empty) and files copied as-is (any other path under an include's
/// base); `exclude` drops a path from both. A path is served without its base.
pub(crate) fn select_paths(
    config: &Config,
    paths: Vec<String>,
) -> Result<Selection, String> {
    let include = compile_patterns("include", &read_include(config))?;
    let exclude = compile_patterns("exclude", &config.exclude)?;
    let unused = (include.iter().chain(&exclude))
        .filter(|(_, pattern)| {
            !paths.iter().any(|path| pattern.matches_with(path, OPTIONS))
        })
        .map(|(text, _)| text.clone())
        .collect();
    let mut selection = Selection {
        unused,
        ..Selection::default()
    };
    for path in paths {
        if (exclude.iter()).any(|(_, pattern)| pattern.matches_with(&path, OPTIONS)) {
            continue;
        }
        let page = (include.iter()).find(|(_, pattern)| {
            path.ends_with(".html") && pattern.matches_with(&path, OPTIONS)
        });
        let (base, is_page) = match page {
            Some((text, _)) => (read_base(text), true),
            None => match (include.iter())
                .map(|(text, _)| read_base(text))
                .filter(|base| path.starts_with(base.as_str()))
                .max_by_key(String::len)
            {
                Some(base) => (base, false),
                None => continue,
            },
        };
        let served = path[base.len()..].to_owned();
        let taken = (selection.pages.get(&served))
            .or_else(|| selection.files.get(&served))
            .cloned();
        if let Some(other) = taken {
            return Err(format!("{other} and {path} are both served at /{served}"));
        }
        match is_page {
            true => selection.pages.insert(served, path),
            false => selection.files.insert(served, path),
        };
    }
    Ok(selection)
}

/// The folders, each empty or ending in `/`, that `include` reaches: only these are read.
pub(crate) fn read_bases(config: &Config) -> Vec<String> {
    let mut bases: Vec<String> = read_include(config)
        .iter()
        .map(|text| read_base(text))
        .collect();
    bases.sort();
    bases.dedup();
    bases
}

fn read_include(config: &Config) -> Vec<String> {
    match config.include.is_empty() {
        true => vec![DEFAULT_INCLUDE.to_owned()],
        false => config.include.clone(),
    }
}

// The folders before the first one holding a wildcard: `docs/` for `docs/**/*.html`
fn read_base(pattern: &str) -> String {
    let folders: Vec<&str> = pattern.split('/').collect();
    let literal = folders[..folders.len() - 1]
        .iter()
        .take_while(|folder| !folder.contains(['*', '?', '[']));
    literal.map(|folder| format!("{folder}/")).collect()
}

fn compile_patterns(
    key: &str,
    patterns: &[String],
) -> Result<Vec<(String, glob::Pattern)>, String> {
    (patterns.iter())
        .map(|text| {
            glob::Pattern::new(text)
                .map(|pattern| (text.clone(), pattern))
                .map_err(|error| format!("{key}: \"{text}\": {error}"))
        })
        .collect()
}

#[cfg(test)]
#[path = "select_unit.rs"]
mod unit;
