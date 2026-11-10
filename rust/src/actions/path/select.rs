use std::collections::BTreeMap;

use crate::models::config::Config;

/// The project's files sorted by the config's `include` and `exclude`, each keyed by the
/// path it is served at; excluded ones and those outside every include base are in neither.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Selection {
    /// Served path to source path, both project-relative with `/` separators.
    pub pages: BTreeMap<String, String>,
    pub files: BTreeMap<String, String>,
    /// A page's source to the `.md` beside it, published as the page's Markdown copy.
    pub twins: BTreeMap<String, String>,
    /// The patterns, as written, that match no file: most likely typos.
    pub unused: Vec<String>,
}

/// The include of a config naming none: every page, HTML or Markdown.
pub(crate) const DEFAULT_INCLUDE: &[&str] = &["**/*.html", "**/*.md"];

// `*` must stop at `/`, or `*.html` would take every page in every folder
const OPTIONS: glob::MatchOptions = glob::MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

/// Sort project-relative, `/`-separated paths into pages (`.html` and `.md` matched by
/// `include`, both when empty; a `.md` page served at `.html`) and files copied as-is
/// (any other path under an include's base, an `.md` beside its `.html` page, a
/// changelog section's `source` and the `skills` files among them); `exclude` drops a path
/// from both. A path is served without its base.
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
    let skills = compile_patterns("skills", &config.skills)?;
    let excluded = |path: &str| {
        (exclude.iter()).any(|(_, pattern)| pattern.matches_with(path, OPTIONS))
    };
    let included = |path: &str| {
        (include.iter()).any(|(_, pattern)| pattern.matches_with(path, OPTIONS))
    };
    // An `.md` beside its `.html` page is the project's own Markdown copy of that page
    let twins: BTreeMap<String, String> = (paths.iter())
        .filter(|path| !excluded(path) && included(path))
        .filter_map(|path| Some((format!("{}.md", path.strip_suffix(".html")?), path)))
        .filter(|(twin, _)| paths.contains(twin))
        .map(|(twin, page)| (twin, page.clone()))
        .collect();
    // A changelog section reads its file into pages of its own
    let sources: Vec<&str> = (config.sections.iter())
        .filter_map(|section| section.source.as_deref())
        .collect();
    let mut selection = Selection {
        unused,
        ..Selection::default()
    };
    for path in paths {
        if excluded(&path) {
            continue;
        }
        if let Some(page) = twins.get(&path) {
            selection.twins.insert(page.clone(), path);
            continue;
        }
        let source = is_page(&path)
            && !twins.contains_key(&path)
            && !sources.contains(&path.as_str())
            && !(skills.iter()).any(|(_, pattern)| pattern.matches_with(&path, OPTIONS));
        let is_page = (include.iter())
            .any(|(_, pattern)| source && pattern.matches_with(&path, OPTIONS));
        // As Livemark's: the longest base of any include, not of the one that matched
        let Some(base) = (include.iter())
            .map(|(text, _)| read_base(text))
            .filter(|base| path.starts_with(base.as_str()))
            .max_by_key(String::len)
        else {
            continue;
        };
        let served = path[base.len()..].to_owned();
        // A Markdown page is served where its `.html` twin would be
        let served = match (is_page, served.strip_suffix(".md")) {
            (true, Some(stem)) => format!("{stem}.html"),
            _ => served,
        };
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

/// Whether a path is a page's source: an `.html` or `.md` file.
pub(crate) fn is_page(path: &str) -> bool {
    path.ends_with(".html") || path.ends_with(".md")
}

/// The folders, each empty or ending in `/`, that `include` reaches: only these are read.
pub(crate) fn read_bases(config: &Config) -> Vec<String> {
    let mut bases: Vec<String> = read_include(config)
        .iter()
        .filter(|text| check_pattern("include", text).is_ok())
        .map(|text| read_base(text))
        .collect();
    bases.sort();
    bases.dedup();
    bases
}

fn read_include(config: &Config) -> Vec<String> {
    match config.include.is_empty() {
        true => DEFAULT_INCLUDE
            .iter()
            .map(|pattern| (*pattern).to_owned())
            .collect(),
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
            check_pattern(key, text)?;
            glob::Pattern::new(text)
                .map(|pattern| (text.clone(), pattern))
                .map_err(|error| format!("{key}: \"{text}\": {error}"))
        })
        .collect()
}

// A base outside the project would read, and publish, files the project never held
fn check_pattern(key: &str, text: &str) -> Result<(), String> {
    match text.starts_with('/')
        || text.contains(['\\', ':'])
        || text.split('/').any(|part| part == "..")
    {
        true => Err(format!("{key}: \"{text}\": must stay inside the project")),
        false => Ok(()),
    }
}

#[cfg(test)]
#[path = "select_unit.rs"]
mod unit;
