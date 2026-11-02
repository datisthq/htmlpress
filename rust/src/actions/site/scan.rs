use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::{Arc, mpsc};

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
use crate::actions::page::read::{
    create_output_path, finish_page, is_layout_name, read_source,
};
use crate::actions::page::rebase::{list_links, read_root, rebase_image};
use crate::actions::page::render::render_page;
use crate::actions::path::contain::{contain_path, read_contained};
use crate::actions::path::select::{Selection, is_page, read_bases, select_paths};
use crate::actions::pipeline::create::create_pipeline;
use crate::actions::section::generate::generate_pages;
use crate::actions::site::localize::localize_site;
use crate::actions::skill::load::load_skills;
use crate::models::asset::{COMPONENT_PREFIX, read_component_path};
use crate::models::config::{Config, PluginEntry};
use crate::models::error::Error;
use crate::models::language::SiteLanguage;
use crate::models::page::Page;
use crate::models::pipeline::{MarkdownOptions, Pipeline, PipelineOptions};
use crate::models::site::Site;
use crate::models::source::{Draft, Parsed, Sources};
use crate::settings::{ASSETS, CONFIG_FILE, NOT_FOUND_PAGE, PROJECT_DIR, ROUTES_DIR};

/// Read a project: its config, its pages and the other files under its include bases.
/// Dot-folders and dot-files (`.htmlpress/`, `.git/`) and `node_modules/` are skipped.
pub fn scan_site(root: &Path) -> crate::Result<Site> {
    Ok(read_site(root, None)?.0)
}

/// What a rendering build keeps to read one page again on its own: the facts every
/// page needs of the others, as the build settled them.
pub(crate) struct Built {
    finish: Finish,
    markdown: MarkdownOptions,
    parser: PipelineOptions,
    // Each article's and route's source to its index and the path `read_source` takes
    selected: HashMap<String, (usize, String)>,
    patched: BTreeSet<String>,
    inputs: BTreeSet<String>,
    suffixes: bool,
}

/// A page rendered into its layout: the path it is written to and its HTML.
pub(crate) type Rendered = (String, String);

/// [`scan_site`], each page also rendered into its layout when `reload` is given, on
/// every core: a worker parses its share of the pages and keeps their trees until they
/// are rendered, and only the facts every page needs of the others meet in between.
pub(crate) fn read_site(
    root: &Path,
    reload: Option<bool>,
) -> crate::Result<(Site, Vec<Rendered>, Option<Built>)> {
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
    add_routes(root, &canonical, &mut selection)?;
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
    create_pipeline(&parser).map_err(config_error)?;
    let selected: Vec<(usize, String, String)> = (selection.pages.iter())
        .enumerate()
        .map(|(index, (path, source))| (index, path.clone(), source.clone()))
        .collect();
    let count = (std::thread::available_parallelism().map_or(1, |count| count.get()))
        .min(selected.len())
        .max(1);
    let context = Context {
        root,
        markdown: &markdown,
        parser: &parser,
    };
    // Generated and patched pages are the main thread's own, parsed into this arena
    let alloc = Allocator::new();
    let mut sources = Sources {
        alloc: &alloc,
        markdown: markdown.clone(),
        parser: create_pipeline(&parser).map_err(config_error)?,
        files: HashMap::new(),
        sizes: HashMap::new(),
    };
    std::thread::scope(|scope| {
        let mut workers = vec![];
        for worker in 0..count {
            let share: Vec<(usize, String, String)> = (selected.iter())
                .filter(|(index, _, _)| index % count == worker)
                .cloned()
                .collect();
            let (commands, inbox) = mpsc::channel();
            let (outbox, reports) = mpsc::channel();
            let context = &context;
            scope.spawn(move || run_worker(context, share, inbox, outbox));
            workers.push((commands, reports));
        }
        // Step 1: every page's includes, which make a Markdown file a partial
        let mut seen: BTreeSet<String> = selection.pages.values().cloned().collect();
        let mut partials = BTreeSet::new();
        for (_, reports) in &workers {
            let Ok(Report::Includes(included)) = reports.recv() else {
                return Err(lost_worker());
            };
            for path in included {
                if path.ends_with(".md") {
                    partials.insert(path.clone());
                }
                seen.insert(path);
            }
        }
        let partials = Arc::new(partials);
        for (commands, _) in &workers {
            let _ = commands.send(Command::Keep(partials.clone()));
        }
        let served: Vec<String> = (selection.pages.iter())
            .filter(|(_, source)| partials.contains(*source))
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
        let kept: BTreeSet<&String> = selection.pages.values().collect();
        let selected: HashMap<String, (usize, String)> = (selected.iter())
            .filter(|(_, _, source)| kept.contains(source))
            .map(|(index, path, source)| (source.clone(), (*index, path.clone())))
            .collect();
        let mut inputs: BTreeSet<String> = seen
            .into_iter()
            .filter(|path| !kept.contains(path))
            .collect();
        // Step 1 still: every page's facts, read from the tree its worker keeps
        let mut facts: Vec<(usize, Page)> = vec![];
        let mut failures = vec![];
        for (_, reports) in &workers {
            match reports.recv() {
                Ok(Report::Facts(Ok(read))) => facts.extend(read),
                Ok(Report::Facts(Err(failure))) => failures.push(failure),
                _ => return Err(lost_worker()),
            }
        }
        if let Some(error) = take_first(failures) {
            return Err(error);
        }
        facts.sort_by_key(|(index, _)| *index);
        // Step 2: what every page needs of the others, on this thread alone
        let files = std::mem::take(&mut selection.files);
        let mut drafts: Vec<(usize, Node)> = vec![];
        let mut next = selected.len();
        apply_patches(
            root,
            &config,
            &mut facts,
            &mut drafts,
            &mut next,
            &mut sources,
        )?;
        let suffixes = !config.languages.is_empty();
        for (_, page) in &mut facts {
            detect_language(page, &languages, suffixes)
                .map_err(Error::page(&page.source))?;
        }
        // A route is drawn bare and reached by its URL alone, as in Livemark
        for (_, page) in facts.iter_mut().filter(|(_, page)| page.is_route()) {
            page.sidebar = false;
        }
        check_urls(&facts)?;
        // Before the generated pages join: a changelog's versions share its source file,
        // whose last change says nothing about when one version was written
        let dates = read_dates(root);
        for (_, page) in &mut facts {
            page.updated = dates.get(&page.source).cloned();
        }
        for language in &languages {
            let read: Vec<Page> = facts.iter().map(|(_, page)| page.clone()).collect();
            for mut draft in generate_pages(root, language, &read, &mut sources)? {
                draft.page.key = create_key(&draft.page.url, language);
                draft.page.language = language.code.clone();
                facts.push((next, draft.page));
                drafts.push((next, draft.tree));
                next += 1;
            }
        }
        let mut bases = read_bases(&config);
        bases.push(ROUTES_DIR.to_owned());
        let read: Vec<Page> = facts.iter().map(|(_, page)| page.clone()).collect();
        let links = list_links(&read, &bases);
        for (_, page) in &mut facts {
            rebase_image(page, &bases);
            if let Some(layout) = &page.layout
                && !assets.contains_key(&format!("layouts/{layout}.html"))
            {
                return Err(Error::page(&page.source)(format!(
                    "no layout \"{layout}\": add {PROJECT_DIR}/layouts/{layout}.html"
                )));
            }
        }
        let mut pages: Vec<Page> = facts.iter().map(|(_, page)| page.clone()).collect();
        sort_pages(&mut pages);
        let skills = load_skills(root, &config.skills)?;
        inputs.extend(config.patches.iter().map(|patch| patch.file.clone()));
        for language in &languages {
            let sources = (language.config.sections.iter())
                .filter_map(|section| section.source.as_deref())
                .filter(|source| !is_github_source(source));
            inputs
                .extend(sources.map(|source| source.trim_start_matches('/').to_owned()));
        }
        let real = root.canonicalize().unwrap_or_else(|_| root.to_owned());
        for skill in &skills {
            if let Ok(path) = skill.path.strip_prefix(&real) {
                inputs.insert(path.to_string_lossy().replace('\\', "/"));
            }
        }
        let mut site = Site {
            root: root.to_owned(),
            config: config.clone(),
            title: language.title.clone(),
            translations: list_translations(&pages),
            pages,
            files,
            assets: assets.clone(),
            skills,
            inputs,
            languages: languages.clone(),
        };
        let views = match reload {
            Some(_) => (languages.iter())
                .map(|language| localize_site(&site, &language.code))
                .collect(),
            None => vec![],
        };
        let mut final_facts: Vec<Option<Page>> = vec![None; next];
        for (index, page) in facts {
            if let Some(slot) = final_facts.get_mut(index) {
                *slot = Some(page);
            }
        }
        let (light, dark) = config.read_code_themes();
        let finish = Arc::new(Finish {
            pages: final_facts,
            links,
            bases,
            themes: (light.to_owned(), dark.to_owned()),
            plugins: config.plugins.html.clone(),
            languages: languages.clone(),
            base: config.read_base(),
            fragments: (list_fragments(&assets).into_iter())
                .map(|(tag, html)| (tag, html.to_owned()))
                .collect(),
            views,
            reload,
        });
        for (commands, _) in &workers {
            let _ = commands.send(Command::Finish(finish.clone()));
        }
        // Step 3: every page through its pipeline and into its layout, the main thread
        // doing its own while the workers do theirs
        let mut done = vec![];
        let mut failures = vec![];
        let mut pipelines = HashMap::new();
        for (index, tree) in drafts {
            match complete_page(&finish, index, tree, &mut pipelines, &alloc) {
                Ok(result) => done.push(result),
                Err(error) => {
                    failures.push((index, error));
                    break;
                }
            }
        }
        for (_, reports) in &workers {
            match reports.recv() {
                Ok(Report::Done(Ok(results))) => done.extend(results),
                Ok(Report::Done(Err(failure))) => failures.push(failure),
                _ => return Err(lost_worker()),
            }
        }
        if let Some(error) = take_first(failures) {
            return Err(error);
        }
        done.sort_by_key(|(index, _, _)| *index);
        let mut rendered = vec![];
        let mut pages = vec![];
        for (_, page, html) in done {
            rendered.extend(html);
            pages.push(page);
        }
        sort_pages(&mut pages);
        site.pages = pages;
        let built = Built {
            finish: Finish::clone(&finish),
            markdown: markdown.clone(),
            parser: parser.clone(),
            selected,
            patched: (config.patches.iter())
                .map(|patch| patch.file.clone())
                .collect(),
            inputs: site.inputs.clone(),
            suffixes,
        };
        Ok((site, rendered, reload.map(|_| built)))
    })
}

/// Read and render the page at `source` again, alone; None when it is no page the last
/// build read, or its edit changed a fact other pages read, which only a full build covers.
pub(crate) fn reread_page(
    root: &Path,
    built: &mut Built,
    source: &str,
) -> crate::Result<Option<(Page, Rendered)>> {
    let Some((index, path)) = built.selected.get(source).cloned() else {
        return Ok(None);
    };
    let Some(Some(old)) = built.finish.pages.get(index) else {
        return Ok(None);
    };
    if built.patched.contains(source) || !root.join(source).is_file() {
        return Ok(None);
    }
    let alloc = Allocator::new();
    let mut sources = Sources {
        alloc: &alloc,
        markdown: built.markdown.clone(),
        parser: create_pipeline(&built.parser).map_err(Error::Usage)?,
        files: HashMap::new(),
        sizes: HashMap::new(),
    };
    // A new include is a file the watcher does not know yet, and may make a partial
    let included = list_page_includes(root, source, &mut sources);
    if included.iter().any(|path| !built.inputs.contains(path)) {
        return Ok(None);
    }
    let Draft { mut page, tree } = read_draft(root, &path, source, &mut sources)?;
    detect_language(&mut page, &built.finish.languages, built.suffixes)
        .map_err(Error::page(source))?;
    if page.is_route() {
        page.sidebar = false;
    }
    page.updated = old.updated.clone();
    rebase_image(&mut page, &built.finish.bases);
    if read_facts(&page) != read_facts(old) {
        return Ok(None);
    }
    if let Some(slot) = built.finish.pages.get_mut(index) {
        *slot = Some(page);
    }
    let (_, page, html) =
        complete_page(&built.finish, index, tree, &mut HashMap::new(), &alloc)?;
    Ok(html.map(|html| (page, html)))
}

// What other pages read of a page: everything but what its own render and the indexes
// take from its content
fn read_facts(page: &Page) -> Page {
    Page {
        head: String::new(),
        content: String::new(),
        heading: String::new(),
        toc: vec![],
        markdown: String::new(),
        chunks: vec![],
        components: BTreeSet::new(),
        ..page.clone()
    }
}

// What every worker reads and none changes
struct Context<'c> {
    root: &'c Path,
    markdown: &'c MarkdownOptions,
    parser: &'c PipelineOptions,
}

// What the main thread tells a worker, once each, in this order
enum Command {
    /// The Markdown files some page includes, which are partials and not pages.
    Keep(Arc<BTreeSet<String>>),
    Finish(Arc<Finish>),
}

// What a worker tells the main thread, once each, in this order; a failure carries the
// page's index, so the first page to fail is the one reported, as on one thread
enum Report {
    Includes(Vec<String>),
    Facts(Result<Vec<(usize, Page)>, Failure>),
    Done(Result<Vec<Completed>, Failure>),
}

// A page's index with what stopped it
type Failure = (usize, Error);

// A page's index, the page finished and, when rendering, its output
type Completed = (usize, Page, Option<Rendered>);

// What finishing any page needs: every page's facts by index, and the site they make
#[derive(Clone)]
struct Finish {
    pages: Vec<Option<Page>>,
    links: BTreeMap<String, BTreeMap<String, String>>,
    bases: Vec<String>,
    themes: (String, String),
    plugins: Vec<PluginEntry>,
    languages: Vec<SiteLanguage>,
    base: String,
    fragments: Vec<(String, String)>,
    /// Each language's view of the site, when the pages are rendered.
    views: Vec<Site>,
    reload: Option<bool>,
}

// A worker's pages, from their parse to their rendered HTML, in one arena of its own:
// a tree borrows its arena, which no other thread may touch
fn run_worker(
    context: &Context,
    share: Vec<(usize, String, String)>,
    inbox: mpsc::Receiver<Command>,
    outbox: mpsc::Sender<Report>,
) {
    let alloc = Allocator::new();
    let parser = match create_pipeline(context.parser) {
        Ok(parser) => parser,
        Err(_) => return,
    };
    let mut sources = Sources {
        alloc: &alloc,
        markdown: context.markdown.clone(),
        parser,
        files: HashMap::new(),
        sizes: HashMap::new(),
    };
    let root = context.root;
    let mut included = vec![];
    for (_, _, source) in &share {
        included.extend(list_page_includes(root, source, &mut sources));
    }
    if outbox.send(Report::Includes(included)).is_err() {
        return;
    }
    let Ok(Command::Keep(partials)) = inbox.recv() else {
        return;
    };
    let mut drafts = vec![];
    let mut read = vec![];
    for (index, path, source) in share {
        if partials.contains(&source) {
            continue;
        }
        match read_draft(root, &path, &source, &mut sources) {
            Ok(Draft { page, tree }) => {
                read.push((index, page));
                drafts.push((index, tree));
            }
            Err(error) => {
                let _ = outbox.send(Report::Facts(Err((index, error))));
                return;
            }
        }
    }
    if outbox.send(Report::Facts(Ok(read))).is_err() {
        return;
    }
    let Ok(Command::Finish(finish)) = inbox.recv() else {
        return;
    };
    let mut done = vec![];
    let mut pipelines = HashMap::new();
    for (index, tree) in drafts {
        match complete_page(&finish, index, tree, &mut pipelines, &alloc) {
            Ok(result) => done.push(result),
            Err(error) => {
                let _ = outbox.send(Report::Done(Err((index, error))));
                return;
            }
        }
    }
    let _ = outbox.send(Report::Done(Ok(done)));
}

// Every path a page includes, and those its partials include in turn
fn list_page_includes(
    root: &Path,
    source: &str,
    sources: &mut Sources<'_>,
) -> Vec<String> {
    let mut pending = vec![source.to_owned()];
    let mut seen: Vec<String> = vec![];
    let mut included = vec![];
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
            included.push(path.clone());
            pending.push(path);
        }
    }
    included
}

fn read_draft<'a>(
    root: &Path,
    path: &str,
    source: &str,
    sources: &mut Sources<'a>,
) -> crate::Result<Draft<'a>> {
    let tree = read_page_tree(root, source, sources).map_err(Error::page(source))?;
    let Some(tree) = tree else {
        let file = root.join(source);
        let error = (std::fs::read_to_string(&file).err())
            .unwrap_or_else(|| std::io::Error::other("cannot be read"));
        return Err(Error::io(&file)(error));
    };
    let page = read_source(path, &tree).map_err(|error| match error {
        Error::Page { message, .. } => Error::Page {
            path: source.to_owned(),
            message,
        },
        error => error,
    })?;
    let page = Page {
        source: source.to_owned(),
        ..page
    };
    Ok(Draft { page, tree })
}

// One page through its pipeline, decorated in its language, and with `views` into its
// layout, its content then dropped: the HTML holds it
fn complete_page<'a>(
    finish: &Finish,
    index: usize,
    tree: Node<'a>,
    pipelines: &mut HashMap<(Option<String>, String), Pipeline>,
    alloc: &'a Allocator,
) -> crate::Result<Completed> {
    let Some(Some(page)) = finish.pages.get(index) else {
        return Err(Error::Usage(format!("no page {index} to finish")));
    };
    let mut page = page.clone();
    let root_base = match page.source.is_empty() {
        true => None,
        false => Some(read_root(&page.source, &finish.bases).unwrap_or_default()),
    };
    let pipeline = match pipelines.entry((root_base.clone(), page.language.clone())) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            let links = (finish.links.get(&page.language))
                .or_else(|| finish.links.values().next())
                .cloned()
                .unwrap_or_default();
            let options = PipelineOptions {
                rebase: root_base,
                links,
                themes: Some(finish.themes.clone()),
                plugins: finish.plugins.clone(),
            };
            entry.insert(create_pipeline(&options).map_err(Error::Usage)?)
        }
    };
    let found =
        (finish.languages.iter()).position(|language| language.code == page.language);
    let Some(language) = found
        .and_then(|at| finish.languages.get(at))
        .or(finish.languages.first())
    else {
        return Err(Error::Usage("no language to build".to_owned()));
    };
    let fragments: Vec<(String, &str)> = (finish.fragments.iter())
        .map(|(tag, html)| (tag.clone(), html.as_str()))
        .collect();
    // A bare route's content is its layout, which `render_page` prefixes whole
    let bare = page.is_route() && page.layout.is_none();
    let decoration = Decoration {
        language,
        base: if bare { "" } else { &finish.base },
        fragments: &fragments,
    };
    finish_page(&mut page, tree, pipeline, Some(&decoration), alloc)?;
    let Some(reload) = finish.reload else {
        return Ok((index, page, None));
    };
    // The view `localize_site` lists the page in as its own, as the build renders it
    let view = (finish.languages.iter().zip(&finish.views))
        .find(|(language, _)| {
            page.language == language.code || language.default && page.language.is_empty()
        })
        .map(|(_, view)| view);
    let Some(view) = view else {
        return Err(Error::page(&page.source)(
            "no language renders this page".to_owned(),
        ));
    };
    let html = render_page(view, &page, reload)?;
    let path = create_output_path(&page.path);
    page.content.clear();
    Ok((index, page, Some((path, html))))
}

// The error of the page that comes first, as a build on one thread would report it
fn take_first(failures: Vec<Failure>) -> Option<Error> {
    (failures.into_iter())
        .min_by_key(|(index, _)| *index)
        .map(|(_, error)| error)
}

fn lost_worker() -> Error {
    Error::Usage("a build worker stopped before its pages were read".to_owned())
}

// A patch names a page by its source; a patched `.md` file, not a page until now, is
// rendered from Markdown and built beside where it sits, as an `.html` page the main
// thread keeps, its tree in `drafts`
fn apply_patches<'a>(
    root: &Path,
    config: &Config,
    facts: &mut Vec<(usize, Page)>,
    drafts: &mut Vec<(usize, Node<'a>)>,
    next: &mut usize,
    sources: &mut Sources<'a>,
) -> crate::Result<()> {
    let bases = read_bases(config);
    for patch in &config.patches {
        let fail = |message: String| Error::Config {
            path: root.join(CONFIG_FILE),
            message: format!("patch \"{}\": {message}", patch.file),
        };
        let at = facts.iter().position(|(_, page)| page.source == patch.file);
        let (index, page) = match at {
            Some(at) => facts.remove(at),
            None => {
                let draft = read_patched_file(root, &bases, &patch.file, sources)
                    .map_err(fail)?;
                drafts.push((*next, draft.tree));
                *next += 1;
                (*next - 1, draft.page)
            }
        };
        facts.push((index, patch_page(page, &patch.article).map_err(fail)?));
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
fn check_urls(facts: &[(usize, Page)]) -> crate::Result<()> {
    for (index, page) in facts.iter().map(|(_, page)| page).enumerate() {
        let earlier = facts[..index].iter().map(|(_, page)| page);
        if let Some(other) = earlier.clone().find(|other| other.url == page.url) {
            return Err(Error::page(&page.source)(format!(
                "builds to {} like {}; give one a different path",
                page.url, other.source
            )));
        }
    }
    Ok(())
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

// Each `.html` and `.md` file under the routes folder, none in or named with a leading
// `-`, is a page at its own path, in place of any article served there
fn add_routes(
    root: &Path,
    canonical: &Path,
    selection: &mut Selection,
) -> crate::Result<()> {
    let inside = (root.join(ROUTES_DIR).canonicalize())
        .is_ok_and(|folder| folder.is_dir() && folder.starts_with(canonical));
    if !inside {
        return Ok(());
    }
    let mut sources = vec![];
    list_files(root, ROUTES_DIR, &mut sources)?;
    sources.sort();
    for source in sources {
        let path = &source[ROUTES_DIR.len()..];
        if !is_page(path) || path.split('/').any(|part| part.starts_with('-')) {
            continue;
        }
        let served = match path.strip_suffix(".md") {
            Some(stem) => format!("{stem}.html"),
            None => path.to_owned(),
        };
        selection.files.remove(&served);
        if let Some(article) = selection.pages.insert(served.clone(), source.clone()) {
            eprintln!("warning: {source} takes the place of {article} at /{served}");
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
