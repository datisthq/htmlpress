use std::path::Path;

use crate::actions::config::load::load_config;
use crate::actions::page::read::read_page;
use crate::models::error::Error;
use crate::models::site::Site;
use crate::settings::{CONFIG_FILE, LAYOUT_ESCAPE};

/// Read a project: its config, every `.html` page and every other file under the root.
/// Dot-folders and dot-files (`.htmlpress/`, `.git/`) and `node_modules/` are skipped.
pub fn scan_site(root: &Path) -> crate::Result<Site> {
    let config = load_config(root)?;
    let mut paths = vec![];
    list_files(root, "", &mut paths)?;
    paths.sort();

    let mut pages = vec![];
    let mut files = vec![];
    for path in paths {
        if path == CONFIG_FILE {
            continue;
        }
        if path.ends_with(".html") {
            let file = root.join(&path);
            let html = std::fs::read_to_string(&file).map_err(Error::io(&file))?;
            pages.push(read_page(&path, &html)?);
        } else {
            files.push(path);
        }
    }
    pages.sort_by(|a, b| {
        let rank =
            |page: &crate::Page| (page.url != "/", page.order.is_none(), page.order);
        rank(a).cmp(&rank(b)).then_with(|| a.path.cmp(&b.path))
    });

    let escape = root.join(LAYOUT_ESCAPE);
    let layout = match escape.is_file() {
        true => Some(std::fs::read_to_string(&escape).map_err(Error::io(&escape))?),
        false => None,
    };
    let title = config
        .title
        .clone()
        .unwrap_or_else(|| read_folder_name(root));
    Ok(Site {
        root: root.to_owned(),
        config,
        title,
        pages,
        files,
        layout,
    })
}

fn list_files(root: &Path, prefix: &str, paths: &mut Vec<String>) -> crate::Result<()> {
    let folder = root.join(prefix);
    let entries = std::fs::read_dir(&folder).map_err(Error::io(&folder))?;
    for entry in entries {
        let entry = entry.map_err(Error::io(&folder))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == "node_modules" {
            continue;
        }
        let path = format!("{prefix}{name}");
        let kind = entry.file_type().map_err(Error::io(entry.path()))?;
        if kind.is_dir() {
            list_files(root, &format!("{path}/"), paths)?;
        } else if kind.is_file() {
            paths.push(path);
        }
    }
    Ok(())
}

fn read_folder_name(root: &Path) -> String {
    std::path::absolute(root)
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "Htmlpress".to_owned())
}

#[cfg(test)]
#[path = "scan_unit.rs"]
mod unit;
