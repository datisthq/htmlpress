use std::collections::BTreeMap;
use std::path::Path;

use crate::actions::config::load::load_config;
use crate::actions::git::read::read_dates;
use crate::actions::language::detect::{create_key, detect_language};
use crate::actions::language::resolve::resolve_languages;
use crate::actions::markdown::render::render_markdown;
use crate::actions::page::include::resolve_includes;
use crate::actions::page::patch::patch_page;
use crate::actions::page::read::{is_layout_name, read_page};
use crate::actions::page::rebase::{list_links, rebase_page};
use crate::actions::path::contain::read_contained;
use crate::actions::path::select::{read_bases, select_paths};
use crate::actions::section::generate::generate_pages;
use crate::actions::skill::load::load_skills;
use crate::models::asset::{COMPONENT_PREFIX, read_component_path};
use crate::models::config::Config;
use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::{ASSETS, CONFIG_FILE, NOT_FOUND_PAGE, PROJECT_DIR};

/// Read a project: its config, its pages and the other files under its include bases.
/// Dot-folders and dot-files (`.htmlpress/`, `.git/`) and `node_modules/` are skipped.
pub fn scan_site(root: &Path) -> crate::Result<Site> {
    let languages = resolve_languages(&load_config(root)?, &read_folder_name(root));
    let Some(language) = languages.first() else {
        return Err(Error::Usage("no language to build".to_owned()));
    };
    // The default language's copy, so every text the build reads is already localized
    let config = language.config.clone();
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
        let html = resolve_includes(root, source, &html).map_err(Error::page(source))?;
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
    apply_patches(root, &config, &mut pages)?;
    let suffixes = !config.languages.is_empty();
    for page in &mut pages {
        detect_language(page, &languages, suffixes).map_err(Error::page(&page.source))?;
    }
    check_urls(&pages)?;
    // Before the generated pages join: a changelog's versions share its source file, whose
    // last change says nothing about when one version was written
    let dates = read_dates(root);
    for page in &mut pages {
        page.updated = dates.get(&page.source).cloned();
    }
    let mut generated = vec![];
    for language in &languages {
        for mut page in generate_pages(root, language, &pages)? {
            page.key = create_key(&page.url, language);
            page.language = language.code.clone();
            generated.push(page);
        }
    }
    pages.extend(generated);
    let bases = read_bases(&config);
    let links = list_links(&pages, &bases);
    for page in &mut pages {
        rebase_page(page, &bases, &links).map_err(Error::page(&page.source))?;
    }
    sort_pages(&mut pages);

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
    let title = language.title.clone();
    let translations = list_translations(&pages);
    let skills = load_skills(root, &config.skills)?;
    Ok(Site {
        root: root.to_owned(),
        config,
        title,
        pages,
        files,
        assets,
        skills,
        translations,
        languages,
    })
}

// A patch names a page by its source; a patched `.md` file, not a page until now, is
// rendered from Markdown and built beside where it sits, as an `.html` page
fn apply_patches(
    root: &Path,
    config: &Config,
    pages: &mut Vec<Page>,
) -> crate::Result<()> {
    let bases = read_bases(config);
    for patch in &config.patches {
        let fail = |message: String| Error::Config {
            path: root.join(CONFIG_FILE),
            message: format!("patch \"{}\": {message}", patch.file),
        };
        let page = match pages.iter().position(|page| page.source == patch.file) {
            Some(index) => pages.remove(index),
            None => read_patched_file(root, &bases, &patch.file).map_err(fail)?,
        };
        pages.push(patch_page(page, &patch.article).map_err(fail)?);
    }
    Ok(())
}

fn read_patched_file(root: &Path, bases: &[String], file: &str) -> Result<Page, String> {
    let stem = match (file.strip_suffix(".md"), file.strip_suffix(".html")) {
        (Some(stem), _) | (_, Some(stem)) => stem,
        _ => return Err("a patched file is an .html page or a .md file".to_owned()),
    };
    let text = read_contained(root, file, false)?;
    let html = match file.ends_with(".md") {
        true => render_markdown(&text),
        false => text,
    };
    let html = resolve_includes(root, file, &html)?;
    let output = (bases.iter())
        .filter(|base| !base.is_empty())
        .find_map(|base| stem.strip_prefix(&format!("{base}/")))
        .unwrap_or(stem);
    let page =
        read_page(&format!("{output}.html"), &html).map_err(|error| error.to_string())?;
    Ok(Page {
        source: file.to_owned(),
        ..page
    })
}

fn list_translations(pages: &[Page]) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut translations: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for page in pages.iter().filter(|page| page.path != NOT_FOUND_PAGE) {
        (translations.entry(page.key.clone()).or_default())
            .insert(page.language.clone(), page.url.clone());
    }
    translations
}

/// Sort pages into navigation order: each language's home first, then, as in Livemark,
/// ordered pages ascending, the rest, then negative orders with -1 last.
pub(crate) fn sort_pages(pages: &mut [Page]) {
    pages.sort_by(|a, b| {
        let rank = |page: &Page| {
            let group = match page.order {
                Some(order) if order < 0 => 2,
                Some(_) => 0,
                None => 1,
            };
            (page.key != "/" && page.url != "/", group, page.order)
        };
        // By URL, not path, so a folder's index (`/api/`) leads its pages (`/api/auth.html`)
        rank(a).cmp(&rank(b)).then_with(|| a.url.cmp(&b.url))
    });
}

// Two pages on one URL would overwrite each other's output without a word
fn check_urls(pages: &[Page]) -> crate::Result<()> {
    for (index, page) in pages.iter().enumerate() {
        if let Some(other) = pages[..index].iter().find(|other| other.url == page.url) {
            return Err(Error::page(&page.source)(format!(
                "builds to {} like {}; give one a different path",
                page.url, other.source
            )));
        }
    }
    Ok(())
}

// Only `layout.html`, the files directly in `styles/`, `scripts/`, `components/` and
// `layouts/`, and those one folder into `components/` are read, so the build output and
// anything else kept in `.htmlpress/` stay out.
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
        list_assets(&project, folder, &mut paths)?;
    }
    let folders: Vec<String> = paths
        .iter()
        .filter(|path| path.starts_with("components/") && project.join(path).is_dir())
        .cloned()
        .collect();
    paths.retain(|path| !folders.contains(path));
    for folder in &folders {
        list_assets(&project, folder, &mut paths)?;
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

fn list_assets(
    project: &Path,
    folder: &str,
    paths: &mut Vec<String>,
) -> crate::Result<()> {
    let path = project.join(folder);
    if !path.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(&path).map_err(Error::io(&path))?;
    for entry in entries {
        let entry = entry.map_err(Error::io(&path))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let kept =
            entry.path().is_file() || folder == "components" && entry.path().is_dir();
        if !name.starts_with('.') && kept {
            paths.push(format!("{folder}/{name}"));
        }
    }
    Ok(())
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
    let Some(file) = path.strip_prefix("components/") else {
        return match known || path == "layout.html" {
            true => Ok(()),
            false => Err(format!(
                "no built-in {path} to replace; `htmlpress escape` lists them"
            )),
        };
    };
    let (folder, _) = file.split_once('/').unwrap_or(("", file));
    let stem = file.rsplit('/').next().unwrap_or(file);
    let stem = stem.rsplit_once('.').map_or(stem, |(stem, _)| stem);
    let name = stem.strip_prefix(COMPONENT_PREFIX).unwrap_or(stem);
    let moved =
        || format!("a component's files are components/{name}/{name}.html, .css or .js");
    // A leftover `components/htmlpress-tabs/` would add `htmlpress-htmlpress-tabs` and
    // silently stop replacing the built-in
    if folder.starts_with(COMPONENT_PREFIX) {
        return Err(moved());
    }
    let Some((_, extension)) = read_component_path(path) else {
        return Err(moved());
    };
    if !["html", "css", "js"].contains(&extension) {
        return Err("a component is a .html, .css or .js file".to_owned());
    }
    let valid = folder.starts_with(|char: char| char.is_ascii_lowercase())
        && folder.chars().all(|char| {
            char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-'
        });
    match valid {
        true => Ok(()),
        false => Err(format!(
            "\"{folder}\" is not a component name: lowercase letters, digits and hyphens"
        )),
    }
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
