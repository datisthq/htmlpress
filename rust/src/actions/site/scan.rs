use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use textlane::Allocator;
use textlane::hast::Node;

use crate::actions::changelog::fetch::is_github_source;
use crate::actions::config::load::load_config;
use crate::actions::git::read::read_dates;
use crate::actions::include::list::list_includes;
use crate::actions::language::detect::{create_key, detect_language};
use crate::actions::language::resolve::resolve_languages;
use crate::actions::page::compile::create_processor;
use crate::actions::page::decorate::{Decoration, list_fragments};
use crate::actions::page::include::resolve_includes;
use crate::actions::page::parse::{create_page_tree, load_source};
use crate::actions::page::patch::patch_page;
use crate::actions::page::read::{finish_page, is_layout_name, read_source};
use crate::actions::page::rebase::{list_links, read_root, rebase_image};
use crate::actions::path::contain::{contain_path, read_contained};
use crate::actions::path::select::{Selection, read_bases, select_paths};
use crate::actions::pipeline::create::create_pipeline;
use crate::actions::section::generate::generate_pages;
use crate::actions::skill::load::load_skills;
use crate::models::asset::{COMPONENT_PREFIX, read_component_path};
use crate::models::config::Config;
use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::pipeline::{MarkdownOptions, Pipeline, PipelineOptions};
use crate::models::site::Site;
use crate::models::source::{Draft, Parsed, Sources};
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
    // A project of HTML pages only is the common case, not a typo
    let quiet = |pattern: &String| config.include.is_empty() && pattern == "**/*.md";
    for pattern in selection.unused.iter().filter(|pattern| !quiet(pattern)) {
        match config.include.is_empty() && !config.exclude.contains(pattern) {
            true => {
                eprintln!("warning: no file matches the default include \"{pattern}\"")
            }
            false => eprintln!("warning: {CONFIG_FILE}: \"{pattern}\" matches no file"),
        }
    }
    let mut selection = selection;
    let assets = read_assets(root)?;
    let markdown = MarkdownOptions {
        components: list_components(&assets),
        plugins: config.plugins.md.clone(),
    };
    let config_error = |message: String| Error::Config {
        path: root.join(CONFIG_FILE),
        message,
    };
    // Up front, so a bad plugin is the config's error, not the first Markdown page's
    create_processor(&markdown).map_err(config_error)?;
    let parser = PipelineOptions {
        plugins: config.plugins.html.clone(),
        ..PipelineOptions::default()
    };
    // One arena for the build: every source is parsed into it once, and each page's tree
    // lives there from its parse to its content
    let alloc = Allocator::new();
    let mut sources = Sources {
        alloc: &alloc,
        markdown,
        parser: create_pipeline(&parser).map_err(config_error)?,
        files: HashMap::new(),
        sizes: HashMap::new(),
    };
    let mut inputs = keep_partials(root, &mut selection, &mut sources)?;
    let mut drafts = vec![];
    for (path, source) in &selection.pages {
        let tree =
            read_page_tree(root, source, &mut sources).map_err(Error::page(source))?;
        let tree = match tree {
            Some(tree) => tree,
            None => {
                let file = root.join(source);
                let error = (std::fs::read_to_string(&file).err())
                    .unwrap_or_else(|| std::io::Error::other("cannot be read"));
                return Err(Error::io(&file)(error));
            }
        };
        let page = read_source(path, &tree).map_err(|error| match error {
            Error::Page { message, .. } => Error::Page {
                path: source.clone(),
                message,
            },
            error => error,
        })?;
        let page = Page {
            source: source.clone(),
            ..page
        };
        drafts.push(Draft { page, tree });
    }
    let files = selection.files;
    apply_patches(root, &config, &mut drafts, &mut sources)?;
    let suffixes = !config.languages.is_empty();
    for draft in &mut drafts {
        let page = &mut draft.page;
        detect_language(page, &languages, suffixes).map_err(Error::page(&page.source))?;
    }
    check_urls(&drafts)?;
    // Before the generated pages join: a changelog's versions share its source file, whose
    // last change says nothing about when one version was written
    let dates = read_dates(root);
    for draft in &mut drafts {
        draft.page.updated = dates.get(&draft.page.source).cloned();
    }
    let mut generated = vec![];
    for language in &languages {
        let read: Vec<Page> = drafts.iter().map(|draft| draft.page.clone()).collect();
        for mut draft in generate_pages(root, language, &read, &mut sources)? {
            draft.page.key = create_key(&draft.page.url, language);
            draft.page.language = language.code.clone();
            generated.push(draft);
        }
    }
    drafts.extend(generated);
    let bases = read_bases(&config);
    let read: Vec<Page> = drafts.iter().map(|draft| draft.page.clone()).collect();
    let links = list_links(&read, &bases);
    // One pipeline a page, now that every page's URL is known for the links between them
    let (light, dark) = config.read_code_themes();
    let base = config.read_base();
    let fragments = list_fragments(&assets);
    let mut pipelines: HashMap<Option<String>, Pipeline> = HashMap::new();
    let mut pages = vec![];
    for Draft { mut page, tree } in drafts {
        rebase_image(&mut page, &bases);
        let root_base = match page.source.is_empty() {
            true => None,
            false => Some(read_root(&page.source, &bases).unwrap_or_default()),
        };
        let pipeline = match pipelines.entry(root_base.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let options = PipelineOptions {
                    rebase: root_base,
                    links: links.clone(),
                    themes: Some((light.to_owned(), dark.to_owned())),
                    plugins: config.plugins.html.clone(),
                };
                entry.insert(create_pipeline(&options).map_err(config_error)?)
            }
        };
        let language = (languages.iter())
            .find(|language| language.code == page.language)
            .unwrap_or(language);
        let decoration = Decoration {
            language,
            base: &base,
            fragments: &fragments,
        };
        finish_page(&mut page, tree, pipeline, Some(&decoration), &alloc)?;
        pages.push(page);
    }
    sort_pages(&mut pages);

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
    inputs.extend(config.patches.iter().map(|patch| patch.file.clone()));
    for language in &languages {
        let sources = (language.config.sections.iter())
            .filter_map(|section| section.source.as_deref())
            .filter(|source| !is_github_source(source));
        inputs.extend(sources.map(|source| source.trim_start_matches('/').to_owned()));
    }
    let real = root.canonicalize().unwrap_or_else(|_| root.to_owned());
    for skill in &skills {
        if let Ok(path) = skill.path.strip_prefix(&real) {
            inputs.insert(path.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(Site {
        root: root.to_owned(),
        config,
        title,
        pages,
        files,
        assets,
        skills,
        inputs,
        translations,
        languages,
    })
}

// A patch names a page by its source; a patched `.md` file, not a page until now, is
// rendered from Markdown and built beside where it sits, as an `.html` page
fn apply_patches<'a>(
    root: &Path,
    config: &Config,
    drafts: &mut Vec<Draft<'a>>,
    sources: &mut Sources<'a>,
) -> crate::Result<()> {
    let bases = read_bases(config);
    for patch in &config.patches {
        let fail = |message: String| Error::Config {
            path: root.join(CONFIG_FILE),
            message: format!("patch \"{}\": {message}", patch.file),
        };
        let draft = match drafts
            .iter()
            .position(|draft| draft.page.source == patch.file)
        {
            Some(index) => drafts.remove(index),
            None => {
                read_patched_file(root, &bases, &patch.file, sources).map_err(fail)?
            }
        };
        let page = patch_page(draft.page, &patch.article).map_err(fail)?;
        drafts.push(Draft {
            page,
            tree: draft.tree,
        });
    }
    Ok(())
}

fn read_patched_file<'a>(
    root: &Path,
    bases: &[String],
    file: &str,
    sources: &mut Sources<'a>,
) -> Result<Draft<'a>, String> {
    let stem = match (file.strip_suffix(".md"), file.strip_suffix(".html")) {
        (Some(stem), _) | (_, Some(stem)) => stem,
        _ => return Err("a patched file is an .html page or a .md file".to_owned()),
    };
    let key = contain_path(root, file, false)?;
    let read = || read_contained(root, file, false);
    let alloc = sources.alloc;
    let parsed = load_source(sources, key, file, file.ends_with(".md"), read)?;
    let mut tree = create_page_tree(parsed, alloc)?;
    resolve_includes(root, file, &mut tree, sources)?;
    let output = (bases.iter())
        .filter(|base| !base.is_empty())
        .find_map(|base| stem.strip_prefix(&format!("{base}/")))
        .unwrap_or(stem);
    let page = read_source(&format!("{output}.html"), &tree)
        .map_err(|error| error.to_string())?;
    Ok(Draft {
        page: Page {
            source: file.to_owned(),
            ..page
        },
        tree,
    })
}

// A page's source as its own tree, its includes resolved; None when it cannot be read
fn read_page_tree<'a>(
    root: &Path,
    source: &str,
    sources: &mut Sources<'a>,
) -> Result<Option<Node<'a>>, String> {
    let alloc = sources.alloc;
    let Some(parsed) = read_page_source(root, source, sources)? else {
        return Ok(None);
    };
    let mut tree = create_page_tree(parsed, alloc)?;
    resolve_includes(root, source, &mut tree, sources)?;
    Ok(Some(tree))
}

// A page's source parsed, by its canonical path, so an include of it shares the parse
fn read_page_source<'s, 'a>(
    root: &Path,
    source: &str,
    sources: &'s mut Sources<'a>,
) -> Result<Option<&'s Parsed<'a>>, String> {
    let file = root.join(source);
    let (Ok(key), Ok(text)) = (file.canonicalize(), std::fs::read_to_string(&file))
    else {
        return Ok(None);
    };
    load_source(sources, key, source, source.ends_with(".md"), || Ok(text)).map(Some)
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
fn check_urls(drafts: &[Draft]) -> crate::Result<()> {
    for (index, page) in drafts.iter().map(|draft| &draft.page).enumerate() {
        let earlier = drafts[..index].iter().map(|draft| &draft.page);
        if let Some(other) = earlier.clone().find(|other| other.url == page.url) {
            return Err(Error::page(&page.source)(format!(
                "builds to {} like {}; give one a different path",
                page.url, other.source
            )));
        }
    }
    Ok(())
}

// A Markdown file another page includes is a partial, as before Markdown pages: copied
// as a file, never built into a page of its own, nor listed. Each file read here is
// parsed into `sources`, where the pages and includes read it from. Returns every
// included path
fn keep_partials(
    root: &Path,
    selection: &mut Selection,
    sources: &mut Sources<'_>,
) -> crate::Result<BTreeSet<String>> {
    let mut pending: Vec<String> = selection.pages.values().cloned().collect();
    let mut seen: Vec<String> = vec![];
    let mut partials: Vec<String> = vec![];
    while let Some(source) = pending.pop() {
        if seen.contains(&source) {
            continue;
        }
        seen.push(source.clone());
        // A file that cannot be read or parsed fails where a page or include reads it
        let Ok(Some(parsed)) = read_page_source(root, &source, sources) else {
            continue;
        };
        let nodes = match parsed {
            Parsed::Html(node) => std::slice::from_ref(node),
            Parsed::Markdown(compiled) => &compiled.body[..],
        };
        for path in list_includes(&source, nodes) {
            if path.ends_with(".md") && !partials.contains(&path) {
                partials.push(path.clone());
            }
            pending.push(path);
        }
    }
    let served: Vec<String> = (selection.pages.iter())
        .filter(|(_, source)| partials.contains(source))
        .map(|(served, _)| served.clone())
        .collect();
    for served in served {
        if let Some(source) = selection.pages.remove(&served) {
            let file = served
                .strip_suffix(".html")
                .map_or(served.clone(), |stem| format!("{stem}.md"));
            selection.files.insert(file, source);
        }
    }
    let pages: Vec<&String> = selection.pages.values().collect();
    Ok(seen
        .into_iter()
        .filter(|path| !pages.contains(&path))
        .collect())
}

// The names, without the prefix, of every component built in or the project's
fn list_components(assets: &BTreeMap<String, String>) -> Vec<String> {
    let mut names: Vec<String> = (assets.keys())
        .filter_map(|path| read_component_path(path))
        .filter_map(|(tag, _)| tag.strip_prefix(COMPONENT_PREFIX).map(str::to_owned))
        .collect();
    names.sort();
    names.dedup();
    names
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
