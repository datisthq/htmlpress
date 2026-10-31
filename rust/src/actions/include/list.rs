use std::cell::RefCell;

use lol_html::{RewriteStrSettings, element, rewrite_str};

/// The project-relative paths of the files `<htmlpress-include src>` names in `html`, a
/// page or partial at `source`, resolved as `resolve_includes` resolves them.
pub(crate) fn list_includes(source: &str, html: &str) -> Result<Vec<String>, String> {
    if !html.contains("<htmlpress-include") {
        return Ok(vec![]);
    }
    let folder = source.rsplit_once('/').map_or("", |(folder, _)| folder);
    let paths = RefCell::new(vec![]);
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "htmlpress-include[src]",
        |el| {
            let src = el.get_attribute("src").unwrap_or_default();
            let path = match (src.strip_prefix('/'), folder.is_empty()) {
                (Some(rooted), _) => rooted.to_owned(),
                (None, true) => src,
                (None, false) => format!("{folder}/{src}"),
            };
            paths.borrow_mut().push(normalize_path(&path));
            Ok(())
        }
    ));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(paths.into_inner())
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
