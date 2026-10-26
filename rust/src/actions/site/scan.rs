use std::collections::BTreeMap;
use std::path::Path;

use crate::actions::config::load::load_config;
use crate::actions::page::read::{is_layout_name, read_page};
use crate::actions::path::select::{read_bases, select_paths};
use crate::actions::skill::load::load_skills;
use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::{ASSETS, CONFIG_FILE, LAYOUT_ELEMENTS, PROJECT_DIR};

/// Read a project: its config, its pages and the other files under its include bases.
/// Dot-folders and dot-files (`.htmlpress/`, `.git/`) and `node_modules/` are skipped.
pub fn scan_site(root: &Path) -> crate::Result<Site> {
    let config = load_config(root)?;
    let mut paths = vec![];
    // A symlinked base would read, and publish, files from outside the project
    let canonical = root.canonicalize().map_err(Error::io(root))?;
    for base in read_bases(&config) {
        let inside = (root.join(&base).canonicalize())
            .is_ok_and(|folder| folder.is_dir() && folder.starts_with(&canonical));
        if inside {
            list_files(root, &base, &mut paths)?;
        }
    }
    paths.sort();

    paths.retain(|path| path != CONFIG_FILE);
    let selection = select_paths(&config, paths).map_err(|message| Error::Config {
        path: root.join(CONFIG_FILE),
        message,
    })?;
    // stderr, not stdout: under --json stdout carries only the result
    for pattern in &selection.unused {
        eprintln!("warning: {CONFIG_FILE}: \"{pattern}\" matches no file");
    }
    let mut pages = vec![];
    for (path, source) in &selection.pages {
        let file = root.join(source);
        let html = std::fs::read_to_string(&file).map_err(Error::io(&file))?;
        let page = read_page(path, &html).map_err(|error| match error {
            Error::Page { message, .. } => Error::Page {
                path: source.clone(),
                message,
            },
            error => error,
        })?;
        pages.push(Page {
            source: source.clone(),
            ..page
        });
    }
    let files = selection.files;
    pages.sort_by(|a, b| {
        let rank =
            |page: &crate::Page| (page.url != "/", page.order.is_none(), page.order);
        // By URL, not path, so a folder's index (`/api/`) leads its pages (`/api/auth.html`)
        rank(a).cmp(&rank(b)).then_with(|| a.url.cmp(&b.url))
    });

    let assets = read_assets(root)?;
    for page in &pages {
        if let Some(layout) = &page.layout
            && !assets.contains_key(&format!("layouts/{layout}.html"))
        {
            return Err(Error::page(&page.source)(format!(
                "no layout \"{layout}\": add {PROJECT_DIR}/layouts/{layout}.html"
            )));
        }
    }
    let title = config
        .title
        .clone()
        .unwrap_or_else(|| read_folder_name(root));
    let skills = load_skills(root, &config.skills)?;
    Ok(Site {
        root: root.to_owned(),
        config,
        title,
        pages,
        files,
        assets,
        skills,
    })
}

// Only `layout.html` and the files directly in `styles/`, `scripts/`, `components/` and
// `layouts/` are read, so the build output and anything else kept in `.htmlpress/` stay out.
fn read_assets(root: &Path) -> crate::Result<BTreeMap<String, String>> {
    let mut assets: BTreeMap<String, String> = ASSETS
        .iter()
        .map(|asset| (asset.path.to_owned(), asset.content.to_owned()))
        .collect();
    let project = root.join(PROJECT_DIR);
    let mut paths = vec![];
    if project.join("layout.html").is_file() {
        paths.push("layout.html".to_owned());
    }
    for folder in ["styles", "scripts", "components", "layouts"] {
        let path = project.join(folder);
        if !path.is_dir() {
            continue;
        }
        let entries = std::fs::read_dir(&path).map_err(Error::io(&path))?;
        for entry in entries {
            let entry = entry.map_err(Error::io(&path))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') && entry.path().is_file() {
                paths.push(format!("{folder}/{name}"));
            }
        }
    }
    for path in paths {
        let file = project.join(&path);
        check_asset(&path).map_err(|message| Error::Config {
            path: file.clone(),
            message,
        })?;
        let text = std::fs::read_to_string(&file).map_err(Error::io(&file))?;
        assets.insert(path, text);
    }
    Ok(assets)
}

fn check_asset(path: &str) -> Result<(), String> {
    let known = ASSETS.iter().any(|asset| asset.path == path);
    if let Some(name) = path.strip_prefix("layouts/") {
        return match name.strip_suffix(".html") {
            Some(name) if is_layout_name(name) => Ok(()),
            _ => Err(
                "a layout is a .html file named in lowercase letters, digits and hyphens"
                    .to_owned(),
            ),
        };
    }
    let Some(name) = path.strip_prefix("components/") else {
        return match known || path == "layout.html" {
            true => Ok(()),
            false => Err(format!(
                "no built-in {path} to replace; `htmlpress escape` lists them"
            )),
        };
    };
    let (tag, extension) = name.rsplit_once('.').unwrap_or((name, ""));
    if !["html", "css", "js"].contains(&extension) {
        return Err("a component is a .html, .css or .js file".to_owned());
    }
    let valid = tag.starts_with(|char: char| char.is_ascii_lowercase())
        && tag.contains('-')
        && tag.chars().all(|char| {
            char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-'
        });
    if !valid {
        return Err(format!(
            "\"{tag}\" is not a custom element name: lowercase, with a hyphen"
        ));
    }
    let built_in = LAYOUT_ELEMENTS.contains(&tag)
        || ASSETS
            .iter()
            .any(|asset| asset.path.starts_with(&format!("components/{tag}.")));
    if tag.starts_with("htmlpress-") && !built_in {
        return Err(format!(
            "no built-in {tag}; the htmlpress- prefix is reserved, \
             and `htmlpress escape` lists the built-in components"
        ));
    }
    Ok(())
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
