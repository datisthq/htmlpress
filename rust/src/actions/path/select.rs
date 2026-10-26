use crate::models::config::Config;

/// The project's files sorted by the config's `include` and `exclude`; excluded ones are
/// in neither list.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Selection {
    pub pages: Vec<String>,
    pub files: Vec<String>,
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
/// `**/*.html` when empty) and files copied as-is; `exclude` drops a path from both.
pub(crate) fn select_paths(
    config: &Config,
    paths: Vec<String>,
) -> Result<Selection, String> {
    let include = match config.include.is_empty() {
        true => vec![DEFAULT_INCLUDE.to_owned()],
        false => config.include.clone(),
    };
    let include = compile_patterns("include", &include)?;
    let exclude = compile_patterns("exclude", &config.exclude)?;
    let matches = |patterns: &[(String, glob::Pattern)], path: &str| {
        (patterns.iter()).any(|(_, pattern)| pattern.matches_with(path, OPTIONS))
    };
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
        if matches(&exclude, &path) {
            continue;
        }
        match path.ends_with(".html") && matches(&include, &path) {
            true => selection.pages.push(path),
            false => selection.files.push(path),
        }
    }
    Ok(selection)
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
