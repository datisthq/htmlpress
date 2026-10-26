use std::path::Path;

use crate::actions::page::read::read_page;
use crate::actions::page::render::render_page;
use crate::actions::site::describe::describe_site;
use crate::actions::site::scan::scan_site;
use crate::models::build::{BuildOptions, BuildReport};
use crate::models::error::Error;
use crate::models::site::Site;
use crate::settings::{ASSETS_DIR, NOT_FOUND, NOT_FOUND_PAGE, RELOAD, THEME_STYLES};

/// Build the project at `options.root` into `options.output`, replacing what is there.
pub fn build_site(options: &BuildOptions) -> crate::Result<BuildReport> {
    check_output(&options.root, &options.output)?;
    let site = scan_site(&options.root)?;

    if options.output.exists() {
        std::fs::remove_dir_all(&options.output).map_err(Error::io(&options.output))?;
    }
    for page in &site.pages {
        let html = render_page(&site, page, options.reload)?;
        write_file(&options.output.join(&page.path), html.as_bytes())?;
    }
    if !site.pages.iter().any(|page| page.path == NOT_FOUND_PAGE) {
        let page = read_page(NOT_FOUND_PAGE, NOT_FOUND)?;
        let mut built_in = site.clone();
        built_in.config.edit = None;
        let html = render_page(&built_in, &page, options.reload)?;
        write_file(&options.output.join(NOT_FOUND_PAGE), html.as_bytes())?;
    }
    for (file, source) in &site.files {
        let source = options.root.join(source);
        let target = options.output.join(file);
        create_parent(&target)?;
        std::fs::copy(&source, &target).map_err(Error::io(&source))?;
    }
    let assets = options.output.join(ASSETS_DIR);
    let theme: Vec<&str> = THEME_STYLES
        .iter()
        .filter_map(|path| site.assets.get(*path).map(String::as_str))
        .collect();
    write_file(&assets.join("theme.css"), theme.join("\n").as_bytes())?;
    for (path, text) in &site.assets {
        let served = path.starts_with("scripts/")
            || path.starts_with("components/") && !path.ends_with(".html");
        if served {
            write_file(&assets.join(path), text.as_bytes())?;
        }
    }
    if options.reload {
        write_file(&assets.join("reload.js"), RELOAD.as_bytes())?;
    }
    // A project's own file of the same name was copied above and wins; _headers merges it
    let shipped = |name: &str| site.files.contains_key(name);
    for (path, bytes) in describe_site(&site)? {
        if path == "_headers" || !shipped(&path) {
            write_file(&options.output.join(&path), &bytes)?;
        }
    }
    if !shipped("robots.txt") {
        write_file(
            &options.output.join("robots.txt"),
            create_robots(&site).as_bytes(),
        )?;
    }
    if let Some(sitemap) = create_sitemap(&site)
        && !shipped("sitemap.xml")
    {
        write_file(&options.output.join("sitemap.xml"), sitemap.as_bytes())?;
    }

    Ok(BuildReport {
        output: options.output.clone(),
        pages: site.pages.len(),
        files: site.files.len(),
    })
}

// Agents read Content-Signal only inside a user-agent group, so it must follow one.
fn create_robots(site: &Site) -> String {
    let signals = &site.config.content_signals;
    let flag = |allowed: bool| if allowed { "yes" } else { "no" };
    let mut robots = format!(
        "User-agent: *\nContent-Signal: search={}, ai-input={}, ai-train={}\nAllow: /\n",
        flag(signals.search),
        flag(signals.ai_input),
        flag(signals.ai_train),
    );
    if let Some(origin) = site.config.read_origin() {
        robots.push_str(&format!("\nSitemap: {origin}/sitemap.xml\n"));
    }
    robots
}

// A sitemap lists absolute URLs only, so a project without `site` gets none.
fn create_sitemap(site: &Site) -> Option<String> {
    let origin = site.config.read_origin()?;
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in site.pages.iter().filter(|page| page.path != NOT_FOUND_PAGE) {
        let url = format!("{origin}{}", page.url)
            .replace('&', "&amp;")
            .replace('<', "&lt;");
        sitemap.push_str(&format!("  <url><loc>{url}</loc></url>\n"));
    }
    sitemap.push_str("</urlset>\n");
    Some(sitemap)
}

// The output is wiped before each build, so one that holds the project would delete it.
pub(crate) fn check_output(root: &Path, output: &Path) -> crate::Result<()> {
    let root = std::path::absolute(root).map_err(Error::io(root))?;
    let output = std::path::absolute(output).map_err(Error::io(output))?;
    if root.starts_with(&output) {
        return Err(Error::Usage(format!(
            "output {} would hold the project {}; pick a folder inside or beside it",
            output.display(),
            root.display(),
        )));
    }
    let inside = output.strip_prefix(&root).ok();
    let hidden = inside
        .and_then(|path| path.components().next())
        .is_some_and(|part| {
            part.as_os_str().to_string_lossy().starts_with('.')
                || part.as_os_str() == "node_modules"
        });
    if inside.is_some() && !hidden {
        return Err(Error::Usage(format!(
            "output {} is inside the project, so its pages would be read back as sources; \
             use a dot-folder such as .htmlpress/build",
            output.display(),
        )));
    }
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> crate::Result<()> {
    create_parent(path)?;
    std::fs::write(path, bytes).map_err(Error::io(path))
}

fn create_parent(path: &Path) -> crate::Result<()> {
    match path.parent() {
        Some(parent) => std::fs::create_dir_all(parent).map_err(Error::io(parent)),
        None => Ok(()),
    }
}

#[cfg(test)]
#[path = "build_unit.rs"]
mod unit;
