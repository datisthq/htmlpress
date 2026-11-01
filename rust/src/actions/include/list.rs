use std::cell::RefCell;

use lol_html::{RewriteStrSettings, element, rewrite_str};

/// The project-relative paths of the files `<htmlpress-include src>`, or in Markdown
/// `::include{file}`, names in `html`, a page or partial at `source`, resolved as
/// `resolve_includes` resolves them.
pub(crate) fn list_includes(source: &str, html: &str) -> Result<Vec<String>, String> {
    let folder = source.rsplit_once('/').map_or("", |(folder, _)| folder);
    let mut srcs = match source.ends_with(".md") {
        true => list_directive_files(html),
        false => vec![],
    };
    if html.contains("<htmlpress-include") {
        let found = RefCell::new(vec![]);
        let settings = RewriteStrSettings::new().append_element_content_handler(
            element!("htmlpress-include[src]", |el| {
                found
                    .borrow_mut()
                    .push(el.get_attribute("src").unwrap_or_default());
                Ok(())
            }),
        );
        rewrite_str(html, settings).map_err(|error| error.to_string())?;
        srcs.extend(found.into_inner());
    }
    Ok(srcs
        .into_iter()
        .map(|src| {
            let path = match (src.strip_prefix('/'), folder.is_empty()) {
                (Some(rooted), _) => rooted.to_owned(),
                (None, true) => src,
                (None, false) => format!("{folder}/{src}"),
            };
            normalize_path(&path)
        })
        .collect())
}

// Livemark's `::include{file="…"}` lines, outside fences, where a page shows the syntax
fn list_directive_files(markdown: &str) -> Vec<String> {
    let mut files = vec![];
    let mut fenced = false;
    for line in markdown.lines().map(str::trim_start) {
        if line.starts_with("```") || line.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        let Some(attributes) = line.strip_prefix("::include{").filter(|_| !fenced) else {
            continue;
        };
        let file = attributes
            .split_once("file=\"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(file, _)| file.to_owned());
        files.extend(file);
    }
    files
}

// `a/./b/../c.md` → `a/c.md`, so a path compares with the scanned ones
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = vec![];
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}

#[cfg(test)]
#[path = "list_unit.rs"]
mod unit;
