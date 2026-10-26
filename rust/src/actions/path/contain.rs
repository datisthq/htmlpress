use std::path::{Component, Path, PathBuf};

use crate::settings::PROJECT_DIR;

// Larger than any page, changelog or code sample; a file this big is a mistake or media
const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Resolve a path a page or the config names, relative to `root`, to the real file it
/// reaches, refusing one that leaves the project (by `..`, an absolute path or a symlink),
/// lives in `node_modules/` or `.htmlpress/`, or, unless `dot` allows it, a dot-entry such
/// as `.env` or `.git/`. Whatever passes is published, so this is what keeps secrets out.
pub(crate) fn contain_path(
    root: &Path,
    path: &str,
    dot: bool,
) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("cannot read the project folder: {error}"))?;
    let joined = root.join(path.trim_start_matches('/'));
    let real = joined
        .canonicalize()
        .map_err(|_| format!("\"{path}\": no such file"))?;
    let inside = real
        .strip_prefix(&root)
        .map_err(|_| format!("\"{path}\": must stay inside the project"))?;
    for part in inside.components() {
        let Component::Normal(name) = part else {
            continue;
        };
        let name = name.to_string_lossy();
        if name == "node_modules"
            || name == PROJECT_DIR
            || (!dot && name.starts_with('.'))
        {
            return Err(format!(
                "\"{path}\": a dot-file, node_modules or {PROJECT_DIR} is never published"
            ));
        }
    }
    if !real.is_file() {
        return Err(format!("\"{path}\": not a file"));
    }
    Ok(real)
}

/// A file [`contain_path`] allows, read as text: UTF-8 and at most 4 MB.
pub(crate) fn read_contained(
    root: &Path,
    path: &str,
    dot: bool,
) -> Result<String, String> {
    let real = contain_path(root, path, dot)?;
    let size = std::fs::metadata(&real)
        .map_err(|error| format!("\"{path}\": {error}"))?
        .len();
    if size > MAX_BYTES {
        return Err(format!("\"{path}\": larger than 4 MB"));
    }
    let bytes = std::fs::read(&real).map_err(|error| format!("\"{path}\": {error}"))?;
    String::from_utf8(bytes).map_err(|_| format!("\"{path}\": not UTF-8 text"))
}

#[cfg(test)]
#[path = "contain_unit.rs"]
mod unit;
