use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::actions::asset::bundle::join_bundle;
use crate::actions::date::format::format_feed_date;
use crate::actions::logo::color::color_logo;
use crate::actions::page::decorate::{Decoration, list_fragments};
use crate::actions::page::read::read_decorated_page;
use crate::actions::page::render::render_page;
use crate::actions::section::generate::list_posts;
use crate::actions::site::describe::{describe_language, describe_site};
use crate::actions::site::index::{create_search_path, index_site};
use crate::actions::site::localize::localize_site;
use crate::actions::site::scan::{Built, Rendered, read_site, reread_page};
use crate::models::asset::Bundle;
use crate::models::build::{BuildOptions, BuildReport};
use crate::models::config::SectionKind;
use crate::models::error::Error;
use crate::models::language::SiteLanguage;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::{
    ASSETS_DIR, NOT_FOUND, NOT_FOUND_PAGE, OUTPUT_DIR, RELOAD, THEME_STYLES,
};

/// Build the project at `options.root` into `options.output`, replacing what is there.
pub fn build_site(options: &BuildOptions) -> crate::Result<BuildReport> {
    build_cached(options).map(|(report, _)| report)
}

/// A finished build, kept by the dev server to rebuild a page alone when only its
/// content changed.
pub(crate) struct Cache {
    site: Site,
    built: Built,
}

/// [`build_site`], keeping what [`rebuild_pages`] needs.
pub(crate) fn build_cached(
    options: &BuildOptions,
) -> crate::Result<(BuildReport, Option<Cache>)> {
    check_output(&options.root, &options.output)?;
    let (site, rendered, built) = read_site(&options.root, Some(options.reload))?;
    // Written beside the output and swapped in whole: the dev server answers requests
    // while it rebuilds, and a page loaded from a half-written output shows unstyled
    let (staging, previous) = create_build_paths(&options.output);
    if staging.exists() {
        std::fs::remove_dir_all(&staging).map_err(Error::io(&staging))?;
    }
    let staged = BuildOptions {
        output: staging.clone(),
        ..options.clone()
    };
    write_site(&staged, &site, &rendered)?;
    swap_output(&staging, &previous, &options.output)?;
    let report = BuildReport {
        output: options.output.clone(),
        pages: site.pages.len(),
        files: site.files.len(),
        inputs: site.inputs.iter().cloned().collect(),
    };
    Ok((report, built.map(|built| Cache { site, built })))
}

/// Rebuild the pages at `changed`, paths relative to the root, in place in the output,
/// with the indexes every page feeds; None when a change reaches beyond a page's own
/// content, which only a full build covers. A path gone and never read is skipped.
pub(crate) fn rebuild_pages(
    options: &BuildOptions,
    cache: &mut Cache,
    changed: &[String],
) -> crate::Result<Option<BuildReport>> {
    let site = &cache.site;
    let known = |path: &String| {
        site.inputs.contains(path)
            || site.files.values().any(|source| source == path)
            || site.pages.iter().any(|page| is_inside(&page.source, path))
    };
    let mut done = vec![];
    for path in changed {
        if !options.root.join(path).exists() && !known(path) {
            continue;
        }
        match reread_page(&options.root, &mut cache.built, path)? {
            Some(page) => done.push(page),
            None => return Ok(None),
        }
    }
    for (page, (path, html, bundle)) in &done {
        write_file(&options.output.join(path), html.as_bytes())?;
        write_bundles(options, &cache.site, bundle.iter())?;
        let own = (cache.site.pages.iter_mut())
            .find(|own| own.source == page.source && own.language == page.language);
        if let Some(own) = own {
            *own = page.clone();
        }
    }
    write_indexes(options, &cache.site)?;
    Ok(Some(BuildReport {
        output: options.output.clone(),
        pages: done.len(),
        files: 0,
        inputs: cache.site.inputs.iter().cloned().collect(),
    }))
}

// The path itself, or a folder holding it: a folder moved or removed takes its pages
fn is_inside(source: &str, path: &str) -> bool {
    source == path
        || source
            .strip_prefix(path)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// The folders a build of `output` writes beside it: the next output, swapped in once
/// complete, and the previous one, removed after the swap.
pub(crate) fn create_build_paths(output: &Path) -> (PathBuf, PathBuf) {
    let name = (output.file_name())
        .map_or("build".into(), |name| name.to_string_lossy().into_owned());
    (
        output.with_file_name(format!("{name}.next")),
        output.with_file_name(format!("{name}.previous")),
    )
}

// Windows refuses to rename or remove a folder holding a file the server has open, so
// the previous output is removed best-effort, and the output in place when it cannot move
fn swap_output(staging: &Path, previous: &Path, output: &Path) -> crate::Result<()> {
    let _ = std::fs::remove_dir_all(previous);
    if output.exists() && std::fs::rename(output, previous).is_err() {
        std::fs::remove_dir_all(output).map_err(Error::io(output))?;
    }
    std::fs::rename(staging, output).map_err(Error::io(output))?;
    let _ = std::fs::remove_dir_all(previous);
    Ok(())
}

fn write_site(
    options: &BuildOptions,
    site: &Site,
    rendered: &[Rendered],
) -> crate::Result<()> {
    // First, so a build that stops halfway leaves a folder `check_output` still replaces
    let assets = options.output.join(ASSETS_DIR);
    std::fs::create_dir_all(&assets).map_err(Error::io(&assets))?;
    let mut bundles = BTreeSet::new();
    for (path, html, used) in rendered {
        write_file(&options.output.join(path), html.as_bytes())?;
        bundles.extend(used.iter().cloned());
    }
    if !site.pages.iter().any(|page| page.path == NOT_FOUND_PAGE) {
        let language = site.read_language("");
        let base = site.config.read_base();
        let fragments = list_fragments(&site.assets);
        let decoration = Decoration {
            language,
            base: &base,
            fragments: &fragments,
        };
        let html = create_not_found(language);
        let page = read_decorated_page(NOT_FOUND_PAGE, &html, Some(&decoration))?;
        let mut built_in = site.clone();
        built_in.config.edit = None;
        let (html, used) = render_page(&built_in, &page, options.reload)?;
        write_file(&options.output.join(NOT_FOUND_PAGE), html.as_bytes())?;
        bundles.extend(used);
    }
    write_bundles(options, site, bundles.iter())?;
    for (file, source) in &site.files {
        let source = options.root.join(source);
        let target = options.output.join(file);
        create_parent(&target)?;
        std::fs::copy(&source, &target).map_err(Error::io(&source))?;
    }
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
    if let Some(logo) = site.assets.get("logo.svg") {
        let tokens = site
            .assets
            .get("styles/tokens.css")
            .map_or("", String::as_str);
        write_file(
            &assets.join("logo.svg"),
            color_logo(logo, tokens).as_bytes(),
        )?;
    }
    if options.reload {
        write_file(&assets.join("reload.js"), RELOAD.as_bytes())?;
    }
    write_indexes(options, site)
}

fn write_bundles<'a>(
    options: &BuildOptions,
    site: &Site,
    bundles: impl Iterator<Item = &'a Bundle>,
) -> crate::Result<()> {
    let assets = options.output.join(ASSETS_DIR);
    for bundle in bundles {
        let path = assets.join(&bundle.path);
        if !path.exists() {
            write_file(&path, join_bundle(&site.assets, bundle).as_bytes())?;
        }
    }
    Ok(())
}

// What every page feeds: search, the agent files, feeds, the home's redirect, robots
// and the sitemap
fn write_indexes(options: &BuildOptions, site: &Site) -> crate::Result<()> {
    let assets = options.output.join(ASSETS_DIR);
    // Each language describes itself through its own view: its config, its pages, and
    // the default pages it lacks listed but not built
    let views: Vec<Site> = (site.languages.iter())
        .map(|language| localize_site(site, &language.code))
        .collect();
    // A project's own file of the same name was copied above and wins; _headers and
    // _redirects merge it
    let shipped = |name: &str| site.files.contains_key(name);
    let mut redirects = Vec::new();
    for (view, language) in views.iter().zip(&site.languages) {
        let index = index_site(view)?;
        write_file(&assets.join(create_search_path(language)), index.as_bytes())?;
        let files = match language.default {
            true => describe_site(view)?,
            false => describe_language(view, &language.prefix)?,
        };
        for (path, bytes) in files {
            if path == "_headers" || !shipped(&path) {
                write_file(&options.output.join(&path), &bytes)?;
            }
        }
        for (path, feed) in create_feeds(view) {
            if !shipped(&path) {
                write_file(&options.output.join(&path), feed.as_bytes())?;
            }
        }
        if let Some(url) = find_home(view, language) {
            let path = format!("{}index.html", language.prefix.trim_start_matches('/'));
            let stub = create_redirect(&url, language);
            write_file(&options.output.join(path), stub.as_bytes())?;
            let from = format!("{}{}", view.config.read_base(), language.prefix);
            redirects.push(format!("{from}  {url}  302\n"));
        }
    }
    if !redirects.is_empty() {
        let rules = create_redirects(site, &redirects.concat())?;
        write_file(&options.output.join("_redirects"), rules.as_bytes())?;
    }
    if !shipped("robots.txt") {
        write_file(
            &options.output.join("robots.txt"),
            create_robots(site).as_bytes(),
        )?;
    }
    if let Some(sitemap) = create_sitemap(site)
        && !shipped("sitemap.xml")
    {
        write_file(&options.output.join("sitemap.xml"), sitemap.as_bytes())?;
    }

    Ok(())
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

// RSS items link absolutely, so a blog gets a feed only with `site`
fn create_feeds(site: &Site) -> Vec<(String, String)> {
    let Some(origin) = site.config.read_origin() else {
        return vec![];
    };
    let escape = |text: &str| {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };
    let pages: Vec<&Page> = site.pages.iter().collect();
    let blogs =
        (site.config.sections.iter()).filter(|section| section.kind == SectionKind::Blog);
    let mut feeds = vec![];
    for section in blogs {
        let prefix = section.prefix.as_deref().unwrap_or_default();
        let link = format!("{origin}{prefix}");
        let mut feed = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n\
             <title>{}</title>\n<link>{}</link>\n<description>{}</description>\n\
             <atom:link href=\"{}rss.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n",
            escape(&section.title),
            escape(&link),
            escape(
                section
                    .site_description
                    .as_deref()
                    .or(site.config.description.as_deref())
                    .unwrap_or_default()
            ),
            escape(&link),
        );
        for post in list_posts(prefix, &pages) {
            let link = if post.untranslated {
                &post.key
            } else {
                &post.url
            };
            let url = escape(&format!("{origin}{link}"));
            feed.push_str(&format!(
                "<item>\n<title>{}</title>\n<link>{url}</link>\n<guid>{url}</guid>\n",
                escape(&post.title)
            ));
            if let Some(date) = post.date.as_deref().and_then(format_feed_date) {
                feed.push_str(&format!("<pubDate>{date}</pubDate>\n"));
            }
            if let Some(description) = &post.description {
                feed.push_str(&format!(
                    "<description>{}</description>\n",
                    escape(description)
                ));
            }
            feed.push_str("</item>\n");
        }
        feed.push_str("</channel>\n</rss>\n");
        feeds.push((format!("{}rss.xml", prefix.trim_start_matches('/')), feed));
    }
    feeds
}

// A sitemap lists absolute URLs only, so a project without `site` gets none.
fn create_sitemap(site: &Site) -> Option<String> {
    let origin = site.config.read_origin()?;
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    let indexed = |page: &&Page| {
        page.path != NOT_FOUND_PAGE && site.read_canonical_url(page) == page.url
    };
    for page in site.pages.iter().filter(indexed) {
        let url = format!("{origin}{}", page.url)
            .replace('&', "&amp;")
            .replace('<', "&lt;");
        let lastmod = page.updated.as_ref().or(page.date.as_ref());
        let lastmod =
            lastmod.map_or(String::new(), |date| format!("<lastmod>{date}</lastmod>"));
        sitemap.push_str(&format!("  <url><loc>{url}</loc>{lastmod}</url>\n"));
    }
    sitemap.push_str("</urlset>\n");
    Some(sitemap)
}

// The output is wiped before each build, so one that holds the project would delete it.
pub(crate) fn check_output(root: &Path, output: &Path) -> crate::Result<()> {
    let root = std::path::absolute(root).map_err(Error::io(root))?;
    let output = std::path::absolute(output).map_err(Error::io(output))?;
    // The build removes the folders it writes beside the output too, so a project at
    // `site.next` built into `site` would be deleted
    let (staging, previous) = create_build_paths(&output);
    if let Some(folder) = [&output, &staging, &previous]
        .into_iter()
        .find(|folder| root.starts_with(folder))
    {
        return Err(Error::Usage(format!(
            "output {} would hold the project {}; pick a folder inside or beside it",
            folder.display(),
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
    // Every build writes `_htmlpress/`, so a folder holding files without it, such as
    // `.git` or the project's escapes, is someone else's and must not be wiped; the
    // default output is htmlpress' own, even when a stopped build left it half-written
    let default = output == root.join(OUTPUT_DIR) && stays_inside(&root, &output);
    if let Some(folder) = [&output, &staging, &previous]
        .into_iter()
        .find(|folder| !default && !is_build_output(folder))
    {
        return Err(Error::Usage(format!(
            "{} is not an htmlpress build output, so it is not replaced; \
             pick another output or remove it yourself",
            folder.display(),
        )));
    }
    Ok(())
}

// A `.htmlpress` symlinked out of the project would make the default output a folder
// elsewhere, such as `~/build`
pub(crate) fn stays_inside(root: &Path, output: &Path) -> bool {
    let (Ok(root), Some(Ok(parent))) =
        (root.canonicalize(), output.parent().map(Path::canonicalize))
    else {
        return true;
    };
    parent.starts_with(root)
}

fn is_build_output(folder: &Path) -> bool {
    let Ok(mut entries) = std::fs::read_dir(folder) else {
        return !folder.exists();
    };
    entries.next().is_none() || folder.join(ASSETS_DIR).is_dir()
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

// A language without a home page of its own sends `/<code>/` to its first page
fn find_home(view: &Site, language: &SiteLanguage) -> Option<String> {
    let home = |page: &Page| page.url == language.prefix && !page.untranslated;
    if view.pages.iter().any(home) {
        return None;
    }
    // A page the project wrote before one a section generated, such as a blog's index or
    // a changelog's version, which shares the changelog's source
    let generated = |page: &Page| {
        page.source.is_empty()
            || (view.config.sections.iter())
                .any(|section| section.source.as_deref() == Some(page.source.as_str()))
    };
    let own = (view.pages.iter())
        .filter(|page| !page.untranslated && page.path != NOT_FOUND_PAGE);
    let first = (own.clone().find(|page| !generated(page))).or(own.clone().next())?;
    Some(format!("{}{}", view.config.read_base(), first.url))
}

// Only for hosts that ignore _redirects: the page paints before the refresh fires
fn create_redirect(url: &str, language: &SiteLanguage) -> String {
    let url = url.replace('&', "&amp;").replace('"', "&quot;");
    format!(
        "<!doctype html>\n<html lang=\"{}\">\n<meta charset=\"utf-8\">\n\
         <meta http-equiv=\"refresh\" content=\"0; url={url}\">\n\
         <link rel=\"canonical\" href=\"{url}\">\n<a href=\"{url}\">{url}</a>\n</html>\n",
        language.locale
    )
}

// The first matching rule wins, so the project's own rules go before ours
fn create_redirects(site: &Site, rules: &str) -> crate::Result<String> {
    let Some(source) = site.files.get("_redirects") else {
        return Ok(rules.to_owned());
    };
    let path = site.root.join(source);
    let existing = std::fs::read_to_string(&path).map_err(Error::io(&path))?;
    Ok(format!("{}\n\n{rules}", existing.trim_end()))
}

// The built-in page, in the default language: a host serves one 404 for every URL
fn create_not_found(language: &SiteLanguage) -> String {
    ["error.title", "error.notFound", "error.home"].iter().fold(
        NOT_FOUND.to_owned(),
        |html, key| {
            let text = language.read_message(key);
            let text = text
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;");
            html.replace(&format!("{{{key}}}"), &text)
        },
    )
}

#[cfg(test)]
#[path = "build_unit.rs"]
mod unit;
