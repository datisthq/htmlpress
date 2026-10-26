use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt::Write;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

use crate::actions::icon::render::render_icon;
use crate::actions::page::convert::create_markdown_path;
use crate::actions::page::prefix::prefix_urls;
use crate::actions::section::find::{find_section, is_custom_current};
use crate::actions::site::index::SEARCH_INDEX;
use crate::models::config::{Link, Position, Section, SectionKind};
use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::{ASSETS_DIR, NOT_FOUND_PAGE, RELOAD_PATH};

/// Wrap one page of the site in its layout, filling its `<head>`, each project
/// component's tag with its `.html`, and every `htmlpress-*` layout element.
pub(crate) fn render_page(
    site: &Site,
    page: &Page,
    reload: bool,
) -> crate::Result<String> {
    let fail = |error: String| Error::page(&page.source)(error);
    let layout = match &page.layout {
        Some(name) => format!("layouts/{name}.html"),
        None => "layout.html".to_owned(),
    };
    let layout = site
        .assets
        .get(&layout)
        .ok_or_else(|| fail(format!("no layout {layout}")))?
        .as_str();
    let fragments: Vec<(&str, &str)> = site
        .assets
        .iter()
        .filter_map(|(path, html)| {
            let tag = path.strip_prefix("components/")?.strip_suffix(".html")?;
            Some((tag, html.as_str()))
        })
        .collect();
    let content = swap_components(&page.content, &fragments).map_err(fail)?;
    let tags = list_tags(&[layout, &page.content], &fragments).map_err(fail)?;
    let head = create_head(site, page, &tags, reload);
    let nav = create_nav(site, page);
    let replaced = |tag: &str| fragments.iter().any(|(name, _)| *name == tag);

    let mut settings = RewriteStrSettings::new().append_element_content_handler(
        element!("head", |el| {
            el.prepend(&head, ContentType::Html);
            el.append(&page.head, ContentType::Html);
            Ok(())
        }),
    );
    for (tag, html) in &fragments {
        settings = settings.append_element_content_handler(element!(tag, |el| {
            el.set_inner_content(html, ContentType::Html);
            Ok(())
        }));
    }
    if !replaced("htmlpress-content") {
        settings = settings.append_element_content_handler(element!(
            "htmlpress-content",
            |el| {
                el.replace(&content, ContentType::Html);
                Ok(())
            }
        ));
    }
    if !replaced("htmlpress-brand") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-brand", |el| {
                el.set_inner_content(&create_brand(site, page), ContentType::Html);
                Ok(())
            }));
    }
    if !replaced("htmlpress-banner") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-banner", |el| {
                fill_element(el, site.config.banner.clone());
                Ok(())
            }));
    }
    if !replaced("htmlpress-breadcrumbs") {
        settings = settings.append_element_content_handler(element!(
            "htmlpress-breadcrumbs",
            |el| {
                fill_element(el, create_breadcrumbs(site, page));
                Ok(())
            }
        ));
    }
    if !replaced("htmlpress-footer") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-footer", |el| {
                el.set_inner_content(FOOTER, ContentType::Html);
                Ok(())
            }));
    }
    if !replaced("htmlpress-nav") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-nav", |el| {
                el.set_inner_content(&nav, ContentType::Html);
                Ok(())
            }));
    }
    if !replaced("htmlpress-links") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-links", |el| {
                fill_element(el, create_links(site, page));
                Ok(())
            }));
    }
    if !replaced("htmlpress-pager") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-pager", |el| {
                fill_element(el, create_pager(site, page));
                Ok(())
            }));
    }
    if !replaced("htmlpress-toc") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-toc", |el| {
                fill_element(el, create_toc(page));
                Ok(())
            }));
    }
    if !replaced("htmlpress-search") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-search", |el| {
                el.set_attribute("data-index", &format!("/{ASSETS_DIR}/{SEARCH_INDEX}"))?;
                Ok(())
            }));
    }
    if !replaced("htmlpress-toolbar") {
        settings = settings.append_element_content_handler(element!(
            "htmlpress-toolbar",
            |el| {
                fill_element(el, create_toolbar(page));
                Ok(())
            }
        ));
    }
    if !replaced("htmlpress-edit") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-edit", |el| {
                fill_element(el, create_edit(site, page));
                Ok(())
            }));
    }
    let html = rewrite_str(layout, settings).map_err(|error| fail(error.to_string()))?;
    let base = site.config.read_base();
    match base.is_empty() {
        true => Ok(html),
        false => prefix_urls(&html, &base).map_err(fail),
    }
}

// The page's content is pasted into the layout unparsed, so its own project component tags
// are swapped here, before it goes in.
fn swap_components(content: &str, fragments: &[(&str, &str)]) -> Result<String, String> {
    let mut settings = RewriteStrSettings::new();
    for (tag, html) in fragments {
        settings = settings.append_element_content_handler(element!(tag, |el| {
            el.set_inner_content(html, ContentType::Html);
            Ok(())
        }));
    }
    rewrite_str(content, settings).map_err(|error| error.to_string())
}

// A component's .html is pasted in after parsing, so the tags inside it are read here
// too; one level deep, since pasted markup is never rewritten again.
fn list_tags(
    sources: &[&str],
    fragments: &[(&str, &str)],
) -> Result<BTreeSet<String>, String> {
    let mut tags = BTreeSet::new();
    for source in sources {
        read_tags(source, &mut tags)?;
    }
    for (tag, html) in fragments {
        if tags.contains(*tag) {
            read_tags(html, &mut tags)?;
        }
    }
    Ok(tags)
}

fn read_tags(html: &str, tags: &mut BTreeSet<String>) -> Result<(), String> {
    let found = RefCell::new(BTreeSet::new());
    let settings =
        RewriteStrSettings::new().append_element_content_handler(element!("*", |el| {
            found.borrow_mut().insert(el.tag_name());
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    tags.extend(found.into_inner());
    Ok(())
}

// A placeholder with nothing to show is removed, so a layout can style or hide the
// box around it with `:empty` or `:has()`.
fn fill_element(el: &mut lol_html::html_content::Element, html: Option<String>) {
    match html {
        Some(html) => el.set_inner_content(&html, ContentType::Html),
        None => el.remove(),
    }
}

fn create_head(
    site: &Site,
    page: &Page,
    tags: &BTreeSet<String>,
    reload: bool,
) -> String {
    let title = match page.url.as_str() {
        "/" => escape_html(&site.title),
        _ => format!(
            "{} · {}",
            escape_html(&page.title),
            escape_html(&site.title)
        ),
    };
    let mut head = String::from("<meta charset=\"utf-8\">\n");
    let _ = writeln!(head, "<title>{title}</title>");
    let custom = &site.config.head;
    for (attribute, key, content) in create_metas(site, page) {
        let replaced = custom
            .meta
            .iter()
            .any(|meta| meta.get(attribute).is_some_and(|value| value == key));
        if !replaced {
            let content = escape_html(&content);
            let _ = writeln!(head, "<meta {attribute}=\"{key}\" content=\"{content}\">");
        }
    }
    if let Some(origin) = site.config.read_origin() {
        let url = escape_html(&format!("{origin}{}", page.url));
        let _ = writeln!(head, "<link rel=\"canonical\" href=\"{url}\">");
    }
    if page.path != NOT_FOUND_PAGE {
        let href = escape_html(&create_markdown_path(&page.path));
        let _ = writeln!(
            head,
            "<link rel=\"alternate\" type=\"text/markdown\" href=\"/{href}\">"
        );
    }
    if let Some(favicon) = site.config.favicon.as_ref().or(site.config.logo.as_ref()) {
        let favicon = escape_html(favicon);
        let _ = writeln!(head, "<link rel=\"icon\" href=\"{favicon}\">");
    }
    // Set before the stylesheet applies, so a stored dark theme never flashes light
    head.push_str(
        "<script>try { const theme = localStorage.getItem(\"htmlpress-theme\"); \
         if (theme) document.documentElement.dataset.theme = theme } catch {}</script>\n",
    );
    let _ = writeln!(
        head,
        "<link rel=\"stylesheet\" href=\"/{ASSETS_DIR}/theme.css\">"
    );
    let used = |extension: &str| -> Vec<String> {
        tags.iter()
            .map(|tag| format!("components/{tag}.{extension}"))
            .filter(|path| site.assets.contains_key(path))
            .collect()
    };
    for path in used("css") {
        let _ = writeln!(
            head,
            "<link rel=\"stylesheet\" href=\"/{ASSETS_DIR}/{path}\">"
        );
    }
    let scripts = site
        .assets
        .keys()
        .filter(|path| path.starts_with("scripts/"));
    for path in scripts.cloned().chain(used("js")) {
        let _ = writeln!(
            head,
            "<script type=\"module\" src=\"/{ASSETS_DIR}/{path}\"></script>"
        );
    }
    if reload {
        let _ = writeln!(
            head,
            "<script type=\"module\" src=\"/{ASSETS_DIR}/reload.js\" \
             data-endpoint=\"{RELOAD_PATH}\"></script>"
        );
    }
    for meta in &custom.meta {
        let _ = writeln!(head, "<meta{}>", create_attributes(meta));
    }
    for link in &custom.links {
        let _ = writeln!(head, "<link{}>", create_attributes(link));
    }
    for script in &custom.scripts {
        let source = script.get("children").map_or("", String::as_str);
        let _ = writeln!(
            head,
            "<script{}>{source}</script>",
            create_attributes(script)
        );
    }
    head
}

// Crawlers read og:url and og:image as absolute URLs only, so both need `site`.
fn create_metas(site: &Site, page: &Page) -> Vec<(&'static str, &'static str, String)> {
    let origin = site.config.read_origin();
    let origin = origin.as_deref();
    let mut metas = vec![(
        "name",
        "viewport",
        "width=device-width, initial-scale=1".to_owned(),
    )];
    let section = read_section(site, page);
    let description = (page.description.as_ref())
        .or(section.and_then(|section| section.site_description.as_ref()))
        .or(site.config.description.as_ref());
    if let Some(description) = description {
        metas.push(("name", "description", description.clone()));
    }
    metas.push(("property", "og:type", "website".to_owned()));
    metas.push(("property", "og:site_name", site.title.clone()));
    metas.push(("property", "og:title", page.title.clone()));
    if let Some(description) = description {
        metas.push(("property", "og:description", description.clone()));
    }
    if let Some(origin) = origin {
        metas.push(("property", "og:url", format!("{origin}{}", page.url)));
    }
    let image = page
        .image
        .as_ref()
        .map(|image| match (origin, image.starts_with('/')) {
            (Some(origin), true) => format!("{origin}{image}"),
            _ => image.clone(),
        });
    let card = match image {
        Some(image) => {
            metas.push(("property", "og:image", image));
            "summary_large_image"
        }
        None => "summary",
    };
    metas.push(("name", "twitter:card", card.to_owned()));
    metas
}

fn create_attributes(tag: &std::collections::BTreeMap<String, String>) -> String {
    let mut html = String::new();
    for (name, value) in tag.iter().filter(|(name, _)| *name != "children") {
        let _ = write!(html, " {}=\"{}\"", escape_html(name), escape_html(value));
    }
    html
}

fn create_brand(site: &Site, page: &Page) -> String {
    let (title, link) = read_brand(site, page);
    let logo = site.config.logo.as_ref().map_or(String::new(), |logo| {
        format!("<img src=\"{}\" alt=\"\">", escape_html(logo))
    });
    let (title, link) = (escape_html(title), escape_html(link));
    format!("<a href=\"{link}\">{logo}{title}</a>")
}

// The open page's section may rename the brand and point it at its own home
fn read_brand<'a>(site: &'a Site, page: &Page) -> (&'a str, &'a str) {
    let section = read_section(site, page);
    let title = section.and_then(|section| section.site_title.as_deref());
    let link = section.and_then(|section| section.site_link.as_deref());
    (title.unwrap_or(&site.title), link.unwrap_or("/"))
}

fn read_section<'a>(site: &'a Site, page: &Page) -> Option<&'a Section> {
    let sections = &site.config.sections;
    find_section(sections, &page.url).and_then(|index| sections.get(index))
}

// Every folder above the page, named and linked by its index page when it has one.
fn create_breadcrumbs(site: &Site, page: &Page) -> Option<String> {
    let folders: Vec<&str> = page.path.split('/').collect();
    let mut folders = folders.split_last()?.1.to_vec();
    if page.path.ends_with("/index.html") {
        folders.pop();
    }
    if folders.is_empty() && page.url == "/" {
        return None;
    }
    let mut html = String::from("<nav aria-label=\"Breadcrumbs\"><ol>");
    let (title, link) = read_brand(site, page);
    let _ = write!(
        html,
        "<li><a href=\"{}\">{}</a></li>",
        escape_html(link),
        escape_html(title)
    );
    for depth in 1..=folders.len() {
        let folder = folders[..depth].join("/");
        let index = format!("{folder}/index.html");
        let name = folders[depth - 1];
        let crumb = match site.pages.iter().find(|item| item.path == index) {
            Some(item) => format!(
                "<a href=\"{}\">{}</a>",
                escape_html(&item.url),
                escape_html(&item.title)
            ),
            None => escape_html(name),
        };
        let _ = write!(html, "<li>{crumb}</li>");
    }
    let title = escape_html(&page.title);
    let _ = write!(html, "<li aria-current=\"page\">{title}</li></ol></nav>");
    Some(html)
}

// Pages group by their top folder, titled by that folder's index page, else its name;
// groups keep the order their first page has in the navigation.
// Folders count from the open section's prefix: inside `/api/` the whole sidebar is that
// folder already, so its pages are not wrapped in one more group of the same name
fn create_nav(site: &Site, page: &Page) -> String {
    let prefix = (read_section(site, page))
        .and_then(|section| section.prefix.as_deref())
        .map_or("", |prefix| prefix.trim_start_matches('/'));
    let mut groups: Vec<(Option<&str>, Vec<&Page>)> = vec![];
    for item in list_pages(site, page) {
        let relative = item.path.strip_prefix(prefix).unwrap_or(&item.path);
        let folder = relative.split_once('/').map(|(folder, _)| folder);
        match groups.iter_mut().find(|(key, _)| *key == folder) {
            Some((_, items)) => items.push(item),
            None => groups.push((folder, vec![item])),
        }
    }
    let mut html = String::from("<nav aria-label=\"Pages\">");
    html.push_str(&create_section_list(site, page));
    for (folder, items) in groups {
        let list = create_list(&items, page);
        match folder {
            Some(folder) => {
                let index = format!("{prefix}{folder}/index.html");
                let title = site
                    .pages
                    .iter()
                    .find(|item| item.path == index)
                    .map_or(folder, |item| item.title.as_str());
                let title = escape_html(title);
                let _ = write!(
                    html,
                    "<details open><summary>{title}</summary>{list}</details>"
                );
            }
            None => html.push_str(&list),
        }
    }
    let links: Vec<&Link> = (site.config.links.iter())
        .filter(|link| link.position == Position::Sidebar)
        .filter(|link| {
            (link.prefix.as_deref()).is_none_or(|prefix| page.url.starts_with(prefix))
        })
        .collect();
    if !links.is_empty() {
        html.push_str("<ul data-links>");
        for link in links {
            let url = escape_html(&link.url);
            let title = escape_html(&link.title);
            let icon = create_icon(link.icon.as_deref());
            let _ = write!(html, "<li><a href=\"{url}\">{icon}{title}</a></li>");
        }
        html.push_str("</ul>");
    }
    html.push_str("</nav>");
    html
}

// Only the pages of the open page's section, so each section reads as a site of its own
fn list_pages<'a>(site: &'a Site, page: &Page) -> Vec<&'a Page> {
    let sections = &site.config.sections;
    let section = find_section(sections, &page.url);
    (site.pages.iter())
        .filter(|item| item.path != NOT_FOUND_PAGE)
        .filter(|item| find_section(sections, &item.url) == section)
        .collect()
}

fn create_list(items: &[&Page], page: &Page) -> String {
    let mut html = String::from("<ul>");
    for item in items {
        let current = match item.path == page.path {
            true => " aria-current=\"page\"",
            false => "",
        };
        let url = escape_html(&item.url);
        let title = escape_html(&item.title);
        let _ = write!(html, "<li><a href=\"{url}\"{current}>{title}</a></li>");
    }
    html.push_str("</ul>");
    html
}

const FOOTER: &str = "<p>Created with <span aria-label=\"love\">❤</span> and \
     <a href=\"https://htmlpress.dev\">Htmlpress</a></p>";

fn create_links(site: &Site, page: &Page) -> Option<String> {
    let links: Vec<&Link> = (site.config.links.iter())
        .filter(|link| link.position == Position::Header)
        .filter(|link| {
            (link.prefix.as_deref()).is_none_or(|prefix| page.url.starts_with(prefix))
        })
        .collect();
    let sections: Vec<String> = (site.config.sections.iter().enumerate())
        .filter(|(_, section)| section.position == Position::Header)
        .map(|(index, _)| create_section_link(site, page, index))
        .collect();
    if links.is_empty() && sections.is_empty() {
        return None;
    }
    let mut html = String::from("<nav aria-label=\"Links\">");
    html.push_str(&sections.concat());
    for link in links {
        let url = escape_html(&link.url);
        let title = escape_html(&link.title);
        let icon = create_icon(link.icon.as_deref());
        let _ = write!(html, "<a href=\"{url}\">{icon}{title}</a>");
    }
    html.push_str("</nav>");
    Some(html)
}

// Every section, so the sidebar can switch between them on a phone, where the header
// has no room for its tabs; each item carries its position for the layout to choose
fn create_section_list(site: &Site, page: &Page) -> String {
    if site.config.sections.is_empty() {
        return String::new();
    }
    let mut html = String::from("<ul data-sections>");
    for (index, section) in site.config.sections.iter().enumerate() {
        let position = match section.position {
            Position::Header => "header",
            Position::Sidebar => "sidebar",
        };
        let link = create_section_link(site, page, index);
        let _ = write!(html, "<li data-position=\"{position}\">{link}</li>");
    }
    html.push_str("</ul>");
    html
}

// A section leads to the page at its prefix, else its first page in navigation order
fn create_section_link(site: &Site, page: &Page, index: usize) -> String {
    let sections = &site.config.sections;
    let Some(section) = sections.get(index) else {
        return String::new();
    };
    let (href, current) = match section.kind {
        SectionKind::Custom => {
            let url = section.url.clone().unwrap_or_default();
            let current = is_custom_current(&url, &page.url);
            (url, current)
        }
        SectionKind::Article => {
            let prefix = section.prefix.clone().unwrap_or_default();
            let first = (site.pages.iter())
                .filter(|item| item.path != NOT_FOUND_PAGE)
                .find(|item| find_section(sections, &item.url) == Some(index));
            let href = match site.pages.iter().any(|item| item.url == prefix) {
                true => prefix,
                false => first.map_or(prefix, |item| item.url.clone()),
            };
            (href, find_section(sections, &page.url) == Some(index))
        }
    };
    let current = match current {
        true => " aria-current=\"page\"",
        false => "",
    };
    let (href, title) = (escape_html(&href), escape_html(&section.title));
    let icon = create_icon(section.icon.as_deref());
    format!("<a href=\"{href}\" data-section{current}>{icon}{title}</a>")
}

fn create_pager(site: &Site, page: &Page) -> Option<String> {
    let pages = list_pages(site, page);
    let index = pages.iter().position(|item| item.path == page.path)?;
    let prev = index
        .checked_sub(1)
        .and_then(|index| pages.get(index).copied());
    let next = pages.get(index + 1).copied();
    if prev.is_none() && next.is_none() {
        return None;
    }
    let mut html = String::from("<nav aria-label=\"Previous and next\">");
    for (rel, label, item) in [("prev", "Previous", prev), ("next", "Next", next)] {
        if let Some(item) = item {
            let url = escape_html(&item.url);
            let title = escape_html(&item.title);
            let _ = write!(
                html,
                "<a rel=\"{rel}\" href=\"{url}\"><small>{label}</small>{title}</a>"
            );
        }
    }
    html.push_str("</nav>");
    Some(html)
}

fn create_toc(page: &Page) -> Option<String> {
    if page.toc.is_empty() {
        return None;
    }
    let mut html = String::from("<p>On this page</p><ul>");
    for heading in &page.toc {
        let id = escape_html(&heading.id);
        let title = escape_html(&heading.title);
        let level = heading.level;
        let _ = write!(
            html,
            "<li><a href=\"#{id}\" data-level=\"{level}\">{title}</a></li>"
        );
    }
    html.push_str("</ul>");
    Some(html)
}

// A link that works without the script, which adds copying and opening in a chat
fn create_toolbar(page: &Page) -> Option<String> {
    if page.path == NOT_FOUND_PAGE {
        return None;
    }
    let href = escape_html(&create_markdown_path(&page.path));
    Some(format!(
        "<a href=\"/{href}\" type=\"text/markdown\">View as Markdown</a>"
    ))
}

fn create_edit(site: &Site, page: &Page) -> Option<String> {
    let url = escape_html(&format!("{}{}", site.config.edit.as_ref()?, page.source));
    Some(format!("<a href=\"{url}\">Edit this page</a>"))
}

// Names were checked when the config loaded, so an unknown one cannot reach here
fn create_icon(name: Option<&str>) -> String {
    name.and_then(render_icon).unwrap_or_default()
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for char in text.chars() {
        match char {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(char),
        }
    }
    escaped
}

#[cfg(test)]
#[path = "render_unit.rs"]
mod unit;
