use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt::Write;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

use crate::actions::code::highlight::highlight_code;
use crate::actions::code::title::add_titles;
use crate::actions::date::format::format_date;
use crate::actions::icon::render::render_icon;
use crate::actions::nav::build::build_nav;
use crate::actions::page::convert::create_markdown_path;
use crate::actions::page::prefix::prefix_urls;
use crate::actions::section::find::{find_section, is_custom_current};
use crate::actions::section::generate::list_posts;
use crate::actions::site::index::create_search_path;
use crate::flags::FLAGS;
use crate::models::asset::{create_component_path, read_component_path};
use crate::models::config::{Link, Position, Section, SectionKind};
use crate::models::error::Error;
use crate::models::language::{Direction, SiteLanguage};
use crate::models::nav::NavNode;
use crate::models::page::Page;
use crate::models::page::SourceAction;
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
    let fragments: Vec<(String, &str)> = site
        .assets
        .iter()
        .filter_map(|(path, html)| {
            let (tag, extension) = read_component_path(path)?;
            (extension == "html").then_some((tag, html.as_str()))
        })
        .collect();
    let language = site.read_language(&page.language);
    let content =
        add_byline(&page.content, create_byline(page, language)).map_err(fail)?;
    let content = fill_inline_tocs(&content, page, language).map_err(fail)?;
    let content = label_callouts(&content, language).map_err(fail)?;
    let (light, dark) = site.config.read_code_themes();
    let content = highlight_code(&content, light, dark).map_err(fail)?;
    let content = add_titles(&content).map_err(fail)?;
    let tags = list_tags(&[layout, &content], &fragments).map_err(fail)?;
    let content = swap_components(&content, &fragments).map_err(fail)?;
    let head = create_head(site, page, &tags, reload);
    let nav = create_nav(site, page);
    let replaced = |tag: &str| fragments.iter().any(|(name, _)| *name == tag);

    let mut settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("html", |el| {
            el.set_attribute("lang", &language.locale)?;
            if language.dir == Direction::Rtl {
                el.set_attribute("dir", "rtl")?;
            }
            Ok(())
        }))
        .append_element_content_handler(element!("head", |el| {
            el.prepend(&head, ContentType::Html);
            el.append(&page.head, ContentType::Html);
            Ok(())
        }));
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
    if !replaced("htmlpress-skip") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-skip", |el| {
                let text = escape_html(language.read_message("nav.skip"));
                el.set_inner_content(
                    &format!("<a href=\"#content\">{text}</a>"),
                    ContentType::Html,
                );
                Ok(())
            }));
    }
    if !replaced("htmlpress-banner") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-banner", |el| {
                fill_element(el, site.config.banner.as_deref().map(str::to_owned));
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
                el.set_inner_content(&create_footer(language), ContentType::Html);
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
                fill_element(el, create_toc(page, language));
                Ok(())
            }));
    }
    if !replaced("htmlpress-search") {
        settings =
            settings.append_element_content_handler(element!("htmlpress-search", |el| {
                let index = create_search_path(site.read_language(&page.language));
                el.set_attribute("data-index", &format!("/{ASSETS_DIR}/{index}"))?;
                Ok(())
            }));
    }
    if !replaced("htmlpress-language") {
        settings = settings.append_element_content_handler(element!(
            "htmlpress-language",
            |el| {
                fill_element(el, create_language_picker(site, page));
                Ok(())
            }
        ));
    }
    if !replaced("htmlpress-toolbar") {
        settings = settings.append_element_content_handler(element!(
            "htmlpress-toolbar",
            |el| {
                fill_element(el, create_toolbar(page, language));
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
fn swap_components(
    content: &str,
    fragments: &[(String, &str)],
) -> Result<String, String> {
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
    fragments: &[(String, &str)],
) -> Result<BTreeSet<String>, String> {
    let mut tags = BTreeSet::new();
    for source in sources {
        read_tags(source, &mut tags)?;
    }
    for (tag, html) in fragments {
        if tags.contains(tag) {
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
    head.push_str(&create_alternates(site, page));
    if page.path != NOT_FOUND_PAGE {
        let href = escape_html(&create_markdown_path(&page.path));
        let _ = writeln!(
            head,
            "<link rel=\"alternate\" type=\"text/markdown\" href=\"/{href}\">"
        );
    }
    // A feed's items link absolutely, so a blog has one only with `site`
    if site.config.read_origin().is_some() {
        let blogs = (site.config.sections.iter())
            .filter(|section| section.kind == SectionKind::Blog);
        for section in blogs {
            let prefix = escape_html(section.prefix.as_deref().unwrap_or_default());
            let title = escape_html(&section.title);
            let _ = writeln!(
                head,
                "<link rel=\"alternate\" type=\"application/rss+xml\" title=\"{title}\" \
                 href=\"{prefix}rss.xml\">"
            );
        }
    }
    if let Some(favicon) = site.config.favicon.as_ref().or(site.config.logo.as_ref()) {
        let favicon = escape_html(favicon);
        let _ = writeln!(head, "<link rel=\"icon\" href=\"{favicon}\">");
    }
    // Set before the stylesheet applies, so a stored dark theme never flashes light
    // and a collapsed sidebar never flashes open
    head.push_str(
        "<script>try { const theme = localStorage.getItem(\"htmlpress-theme\"); \
         if (theme) document.documentElement.dataset.theme = theme; \
         if (localStorage.getItem(\"htmlpress-nav\") === \"collapsed\") \
         document.documentElement.dataset.navCollapsed = \"\" } catch {}</script>\n",
    );
    head.push_str(&create_messages(site.read_language(&page.language)));
    let _ = writeln!(
        head,
        "<link rel=\"stylesheet\" href=\"/{ASSETS_DIR}/theme.css\">"
    );
    let used = |extension: &str| -> Vec<String> {
        tags.iter()
            .filter_map(|tag| create_component_path(tag, extension))
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
    let description = (page.description.as_deref())
        .or(section.and_then(|section| section.site_description.as_deref()))
        .or(site.config.description.as_deref());
    if let Some(description) = description {
        metas.push(("name", "description", description.to_owned()));
    }
    metas.push(("property", "og:type", "website".to_owned()));
    metas.push(("property", "og:site_name", site.title.clone()));
    metas.push(("property", "og:title", page.title.clone()));
    if let Some(description) = description {
        metas.push(("property", "og:description", description.to_owned()));
    }
    if let Some(origin) = origin {
        metas.push(("property", "og:url", format!("{origin}{}", page.url)));
    }
    if site.languages.len() > 1 {
        let locale = &site.read_language(&page.language).locale;
        metas.push(("property", "og:locale", locale.replace('-', "_")));
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

// The strings the components' scripts write, which they read from here and fall back to
// English without, so an escaped script keeps working
const SCRIPT_MESSAGES: [&str; 16] = [
    "code.copied",
    "code.copy",
    "code.expand",
    "code.link",
    "nav.backToTop",
    "nav.toggleSidebar",
    "page.copied",
    "page.copyFailed",
    "page.copyMarkdown",
    "page.llmPrompt",
    "page.openChatGpt",
    "page.openClaude",
    "search.button",
    "search.empty",
    "search.label",
    "theme.toggle",
];

fn create_messages(language: &SiteLanguage) -> String {
    let messages: std::collections::BTreeMap<&str, &str> = (SCRIPT_MESSAGES.iter())
        .map(|key| (*key, language.read_message(key)))
        .collect();
    let json = serde_json::to_string(&messages).unwrap_or_default();
    // A message holding `</script>` would end the script early
    let json = json.replace("</", "<\\/");
    format!(
        "<script type=\"application/json\" id=\"htmlpress-messages\">{json}</script>\n"
    )
}

// Each language links to this page's version in it, else to its home
fn create_language_picker(site: &Site, page: &Page) -> Option<String> {
    if site.languages.len() < 2 {
        return None;
    }
    let current = site.read_language(&page.language);
    let versions = site.translations.get(&page.key);
    let label = escape_html(current.read_message("language.choose"));
    let icon = create_icon(Some("languages"));
    let mut html = format!(
        "<details><summary aria-label=\"{label}\" title=\"{label}\">{icon}</summary><ul>"
    );
    for language in &site.languages {
        let url = (versions.and_then(|versions| versions.get(&language.code)))
            .unwrap_or(&language.prefix);
        let flag = (language.flag.as_deref())
            .and_then(|flag| FLAGS.binary_search_by_key(&flag, |(code, _)| code).ok())
            .and_then(|index| FLAGS.get(index))
            .map_or("", |(_, svg)| *svg);
        let selected = match language.code == current.code {
            true => " aria-current=\"true\"",
            false => "",
        };
        let _ = write!(
            html,
            "<li><a href=\"{}\" hreflang=\"{}\" lang=\"{}\"{selected}>{flag}{}</a></li>",
            escape_html(url),
            escape_html(&language.locale),
            escape_html(&language.locale),
            escape_html(&language.label),
        );
    }
    html.push_str("</ul></details>");
    Some(html)
}

// Crawlers read hreflang as absolute URLs, so it needs `site`; one version has none
fn create_alternates(site: &Site, page: &Page) -> String {
    let (Some(origin), Some(versions)) =
        (site.config.read_origin(), site.translations.get(&page.key))
    else {
        return String::new();
    };
    if versions.len() < 2 {
        return String::new();
    }
    let mut html = String::new();
    for language in &site.languages {
        if let Some(url) = versions.get(&language.code) {
            let href = escape_html(&format!("{origin}{url}"));
            let locale = escape_html(&language.locale);
            let _ = writeln!(
                html,
                "<link rel=\"alternate\" hreflang=\"{locale}\" href=\"{href}\">"
            );
            if language.default {
                let _ = writeln!(
                    html,
                    "<link rel=\"alternate\" hreflang=\"x-default\" href=\"{href}\">"
                );
            }
        }
    }
    html
}

fn create_brand(site: &Site, page: &Page) -> String {
    let (title, description, link) = read_brand(site, page);
    let logo = site.config.logo.as_ref().map_or(String::new(), |logo| {
        format!("<img src=\"{}\" alt=\"\">", escape_html(logo))
    });
    let (title, link) = (escape_html(title), escape_html(link));
    let title = match description {
        Some(text) => format!("<span>{title}<small>{}</small></span>", escape_html(text)),
        None => title,
    };
    format!("<a href=\"{link}\">{logo}{title}</a>")
}

// The open page's section may rename the brand and point it at its own home
fn read_brand<'a>(site: &'a Site, page: &Page) -> (&'a str, Option<&'a str>, &'a str) {
    let section = read_section(site, page);
    let title = section.and_then(|section| section.site_title.as_deref());
    let description = section.and_then(|section| section.site_description.as_deref());
    let link = section.and_then(|section| section.site_link.as_deref());
    let description = description.or(site.config.description.as_deref());
    (
        title.unwrap_or(&site.title),
        description,
        link.unwrap_or("/"),
    )
}

fn read_section<'a>(site: &'a Site, page: &Page) -> Option<&'a Section> {
    let sections = &site.config.sections;
    find_section(sections, &page.url).and_then(|index| sections.get(index))
}

// Every folder above the page, named and linked by its index page when it has one.
fn create_breadcrumbs(site: &Site, page: &Page) -> Option<String> {
    // A language's prefix is its home, not a folder to list
    let home = site
        .read_language(&page.language)
        .prefix
        .trim_start_matches('/');
    let path = page.path.strip_prefix(home).unwrap_or(&page.path);
    let folders: Vec<&str> = path.split('/').collect();
    let mut folders = folders.split_last()?.1.to_vec();
    if path.ends_with("/index.html") {
        folders.pop();
    }
    if folders.is_empty() && path == "index.html" {
        return None;
    }
    let label = escape_html(
        site.read_language(&page.language)
            .read_message("nav.breadcrumbs"),
    );
    let mut html = format!("<nav aria-label=\"{label}\"><ol>");
    let (title, _, link) = read_brand(site, page);
    let _ = write!(
        html,
        "<li><a href=\"{}\">{}</a></li>",
        escape_html(link),
        escape_html(title)
    );
    for depth in 1..=folders.len() {
        let folder = folders[..depth].join("/");
        let index = format!("{home}{folder}/index.html");
        let name = folders[depth - 1];
        let crumb = match site.pages.iter().find(|item| item.path == index) {
            Some(item) => format!(
                "<a href=\"{}\">{}</a>",
                escape_html(read_href(item)),
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

// Pages nest under their folders at any depth (`nav/build`), counted from the open
// section's prefix: inside `/api/` the whole sidebar is that folder already, so its pages
// are not wrapped in one more group of the same name
fn create_nav(site: &Site, page: &Page) -> String {
    let label = escape_html(site.read_language(&page.language).read_message("nav.pages"));
    let mut html = format!("<nav aria-label=\"{label}\">");
    html.push_str(&create_section_list(site, page));
    let nav = read_nav(site, page);
    if !nav.is_empty() {
        let key = match read_section(site, page).map(|section| &section.kind) {
            Some(SectionKind::Blog) => "nav.posts",
            Some(SectionKind::Changelog) => "nav.releases",
            _ => "nav.articles",
        };
        let label = escape_html(site.read_language(&page.language).read_message(key));
        let _ = write!(html, "<p data-label>{label}</p>");
    }
    let mut leaves = vec![];
    for node in nav {
        match node.children.is_empty() {
            true => leaves.push(node),
            false => {
                html.push_str(&create_list(site, &leaves, page, true));
                leaves.clear();
                html.push_str(&create_group(site, &node, page, true));
            }
        }
    }
    html.push_str(&create_list(site, &leaves, page, true));
    let links: Vec<&Link> = (site.config.links.iter())
        .filter(|link| link.position == Position::Sidebar)
        .filter(|link| {
            (link.prefix.as_deref()).is_none_or(|prefix| page.url.starts_with(prefix))
        })
        .collect();
    if !links.is_empty() {
        let label =
            escape_html(site.read_language(&page.language).read_message("nav.links"));
        let _ = write!(html, "<p data-label>{label}</p><ul data-links>");
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
// A blog lists its index, then its posts newest first, and no tag page; a changelog its
// index, then its versions in the file's order
fn list_pages<'a>(site: &'a Site, page: &Page) -> Vec<&'a Page> {
    let sections = &site.config.sections;
    let section = find_section(sections, &page.url);
    let pages: Vec<&Page> = (site.pages.iter())
        .filter(|item| item.path != NOT_FOUND_PAGE && item.sidebar)
        .filter(|item| find_section(sections, &item.url) == section)
        .collect();
    let Some(section) = section.and_then(|index| sections.get(index)) else {
        return pages;
    };
    let prefix = section.prefix.as_deref().unwrap_or_default();
    let index = pages.iter().copied().filter(|item| item.url == prefix);
    match section.kind {
        SectionKind::Blog => index.chain(list_posts(prefix, &pages)).collect(),
        SectionKind::Changelog => {
            let versions = pages.iter().copied().filter(|item| item.url != prefix);
            index.chain(versions).collect()
        }
        SectionKind::Article | SectionKind::Custom => pages,
    }
}

fn read_nav<'a>(site: &'a Site, page: &Page) -> Vec<NavNode<'a>> {
    let section = read_section(site, page);
    let prefix = section.and_then(|section| section.prefix.as_deref());
    // A blog's posts and a changelog's versions are one list, each version being a folder
    let flat = section.is_some_and(|section| {
        matches!(section.kind, SectionKind::Blog | SectionKind::Changelog)
    });
    // Without a section, a language's pages still start below its prefix, not inside it
    let language = &site.read_language(&page.language).prefix;
    build_nav(&list_pages(site, page), prefix.unwrap_or(language), flat)
}

// Only top-level entries take their page's icon, as in Livemark: a folder's own pages
// sit under its summary, which carries the index page's icon instead
fn create_list(site: &Site, nodes: &[NavNode], page: &Page, top: bool) -> String {
    if nodes.is_empty() {
        return String::new();
    }
    let mut html = String::from("<ul>");
    for node in nodes {
        match (node.page, node.children.is_empty()) {
            (Some(item), true) => {
                let _ = write!(html, "<li>{}</li>", create_entry(site, item, page, top));
            }
            _ => {
                let _ =
                    write!(html, "<li>{}</li>", create_group(site, node, page, false));
            }
        }
    }
    html.push_str("</ul>");
    html
}

// A listed index is the group's own link, as in Livemark: its title leads to the page
// and only the chevron beside it toggles, as a link in a summary does not toggle its
// details. A folder without one is titled by its index page anyway, else its name
fn create_group(site: &Site, node: &NavNode, page: &Page, top: bool) -> String {
    let path = node.folder.as_deref().unwrap_or_default();
    let label = match node.page {
        Some(item) => create_entry(site, item, page, top),
        None => {
            let index = site.pages.iter().find(|item| item.path == path);
            let name = (path.trim_end_matches("/index.html").rsplit('/').next())
                .unwrap_or_default();
            let title = escape_html(index.map_or(name, |item| item.title.as_str()));
            let icon =
                create_icon(index.and_then(|item| item.icon.as_deref()).filter(|_| top));
            format!("{icon}{title}")
        }
    };
    let list = create_list(site, &node.children, page, false);
    format!("<details open><summary>{label}</summary>{list}</details>")
}

fn create_entry(site: &Site, item: &Page, page: &Page, icon: bool) -> String {
    let current = match item.path == page.path {
        true => " aria-current=\"page\"",
        false => "",
    };
    let url = escape_html(read_href(item));
    let title = escape_html(item.label.as_ref().unwrap_or(&item.title));
    let icon = create_icon(item.icon.as_deref().filter(|_| icon));
    if !item.untranslated {
        return format!("<a href=\"{url}\"{current}>{icon}{title}</a>");
    }
    let (language, default) =
        (site.read_language(&page.language), site.read_language(""));
    let hint = escape_html(language.read_message("language.untranslated"));
    let (locale, code) = (escape_html(&default.locale), escape_html(&default.code));
    format!(
        "<a href=\"{url}\" hreflang=\"{locale}\" title=\"{hint}\" data-untranslated>\
         {icon}{title}<small>{code}</small></a>"
    )
}

// An untranslated page is listed under the open language's prefix but lives at its key
fn read_href(item: &Page) -> &str {
    match item.untranslated {
        true => &item.key,
        false => &item.url,
    }
}

fn create_footer(language: &SiteLanguage) -> String {
    let text = escape_html(language.read_message("footer.createdWith"));
    let text = text
        .replace("{heart}", "<span aria-label=\"love\">❤</span>")
        .replace(
            "{htmlpress}",
            "<a href=\"https://htmlpress.dev\">Htmlpress</a>",
        );
    format!("<p>{text}</p>")
}

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
    let label = escape_html(site.read_language(&page.language).read_message("nav.links"));
    let mut html = format!("<nav aria-label=\"{label}\">");
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
            // An internal page leads to its version in the open page's language, if any
            let url = section.url.clone().unwrap_or_default();
            let url = (site.translations.get(&url))
                .and_then(|versions| versions.get(&page.language))
                .cloned()
                .unwrap_or(url);
            let current = is_custom_current(&url, &page.url);
            (url, current)
        }
        SectionKind::Article | SectionKind::Blog | SectionKind::Changelog => {
            let prefix = section.prefix.clone().unwrap_or_default();
            let first = (site.pages.iter())
                .filter(|item| item.path != NOT_FOUND_PAGE)
                .find(|item| find_section(sections, &item.url) == Some(index));
            let home = site.pages.iter().find(|item| item.url == prefix);
            let href = (home.or(first)).map_or(prefix, |item| read_href(item).to_owned());
            (href, find_section(sections, &page.url) == Some(index))
        }
    };
    let current = match current {
        true => " aria-current=\"page\"",
        false => "",
    };
    let (href, title) = (escape_html(&href), escape_html(&section.title));
    let icon = create_icon(section.icon.as_deref());
    let version = match section.kind == SectionKind::Changelog && section.version {
        true => read_latest(site, index).map_or(String::new(), |version| {
            format!(" <small>{}</small>", escape_html(version))
        }),
        false => String::new(),
    };
    format!("<a href=\"{href}\" data-section{current}>{icon}{title}{version}</a>")
}

// The changelog lists its newest version first, which its generated pages keep as order 0
fn read_latest(site: &Site, index: usize) -> Option<&str> {
    let sections = &site.config.sections;
    let prefix = sections.get(index)?.prefix.as_deref()?;
    (site.pages.iter())
        .filter(|page| {
            find_section(sections, &page.url) == Some(index) && page.url != prefix
        })
        .min_by_key(|page| page.order.unwrap_or(i64::MAX))
        .map(|page| page.title.as_str())
}

fn create_pager(site: &Site, page: &Page) -> Option<String> {
    let pages = NavNode::flatten(&read_nav(site, page));
    let index = pages.iter().position(|item| item.path == page.path)?;
    let prev = index
        .checked_sub(1)
        .and_then(|index| pages.get(index).copied());
    let next = pages.get(index + 1).copied();
    if prev.is_none() && next.is_none() {
        return None;
    }
    let language = site.read_language(&page.language);
    let label = escape_html(language.read_message("nav.pager"));
    let mut html = format!("<nav aria-label=\"{label}\">");
    for (rel, key, item) in [("prev", "nav.previous", prev), ("next", "nav.next", next)] {
        let label = escape_html(language.read_message(key));
        if let Some(item) = item {
            let url = escape_html(read_href(item));
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

// Authors, published and updated dates, as one line under the page's title
fn create_byline(page: &Page, language: &SiteLanguage) -> Option<String> {
    let mut parts = vec![];
    if !page.authors.is_empty() {
        let icon = create_icon(Some("user"));
        parts.push(format!(
            "<span>{icon}{}</span>",
            escape_html(&page.authors.join(", "))
        ));
    }
    if let Some(date) = &page.date {
        let (icon, date) = (create_icon(Some("calendar")), escape_html(date));
        let shown = escape_html(&format_date(&date, language));
        let time = format!("<time datetime=\"{date}\">{shown}</time>");
        let text = escape_html(language.read_message("page.published"));
        let text = text.replace("{date}", &time);
        parts.push(format!("<span>{icon}{text}</span>"));
    }
    if let Some(date) = &page.updated
        && page.date.as_ref() != Some(date)
    {
        let (icon, date) = (create_icon(Some("history")), escape_html(date));
        let shown = escape_html(&format_date(&date, language));
        let time = format!("<time datetime=\"{date}\">{shown}</time>");
        let text = escape_html(language.read_message("page.updated"));
        let text = text.replace("{date}", &time);
        parts.push(format!("<span>{icon}{text}</span>"));
    }
    match parts.is_empty() {
        true => None,
        false => Some(format!(
            "<htmlpress-byline>{}</htmlpress-byline>",
            parts.concat()
        )),
    }
}

// A page's own <htmlpress-toc> goes in unparsed, so the layout's handler never fills it;
// it is filled here, `level="2"` keeping only the <h2>s
fn fill_inline_tocs(
    content: &str,
    page: &Page,
    language: &SiteLanguage,
) -> Result<String, String> {
    if !content.contains("<htmlpress-toc") {
        return Ok(content.to_owned());
    }
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "htmlpress-toc",
        |el| {
            let level = match el.get_attribute("level").as_deref().map(str::trim) {
                None => 3,
                Some("2") => 2,
                Some("3") => 3,
                Some(other) => {
                    return Err(format!(
                        "<htmlpress-toc level> must be 2 or 3, got \"{other}\""
                    )
                    .into());
                }
            };
            match create_toc_up_to(page, level, language) {
                Some(html) => {
                    el.set_attribute("data-inline", "")?;
                    el.set_inner_content(&html, ContentType::Html);
                }
                None => el.remove(),
            }
            Ok(())
        }
    ));
    rewrite_str(content, settings).map_err(|error| error.to_string())
}

// A callout's default label is CSS, which cannot know the page's language; an explicit
// `label` wins over it, so each unlabelled callout gets its type's label in that language
fn label_callouts(content: &str, language: &SiteLanguage) -> Result<String, String> {
    if !content.contains("<htmlpress-callout") {
        return Ok(content.to_owned());
    }
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "htmlpress-callout:not([label])",
        |el| {
            let key = match el.get_attribute("type").as_deref().map(str::trim) {
                Some("success") => "callout.success",
                Some("warning") => "callout.warning",
                Some("danger") => "callout.danger",
                _ => "callout.info",
            };
            el.set_attribute("label", language.read_message(key))?;
            Ok(())
        }
    ));
    rewrite_str(content, settings).map_err(|error| error.to_string())
}

// Right after the first <h1>, where a reader looks for it; first without one
fn add_byline(content: &str, byline: Option<String>) -> Result<String, String> {
    let Some(byline) = byline else {
        return Ok(content.to_owned());
    };
    let placed = std::cell::Cell::new(false);
    let settings =
        RewriteStrSettings::new().append_element_content_handler(element!("h1", |el| {
            if !placed.replace(true) {
                el.after(&byline, ContentType::Html);
            }
            Ok(())
        }));
    let html = rewrite_str(content, settings).map_err(|error| error.to_string())?;
    match placed.get() {
        true => Ok(html),
        false => Ok(format!("{byline}{html}")),
    }
}

fn create_toc(page: &Page, language: &SiteLanguage) -> Option<String> {
    create_toc_up_to(page, 3, language)
}

fn create_toc_up_to(page: &Page, level: u8, language: &SiteLanguage) -> Option<String> {
    let headings: Vec<_> = page
        .toc
        .iter()
        .filter(|heading| heading.level <= level)
        .collect();
    if headings.is_empty() {
        return None;
    }
    let title = escape_html(language.read_message("toc.title"));
    let mut html = format!("<p>{title}</p><ul>");
    for heading in headings {
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
fn create_toolbar(page: &Page, language: &SiteLanguage) -> Option<String> {
    if page.path == NOT_FOUND_PAGE {
        return None;
    }
    let href = escape_html(&create_markdown_path(&page.path));
    Some(format!(
        "<a href=\"/{href}\" type=\"text/markdown\">{}</a>",
        escape_html(language.read_message("page.markdown"))
    ))
}

fn create_edit(site: &Site, page: &Page) -> Option<String> {
    let label = match page.source_action {
        SourceAction::Edit => "page.edit",
        SourceAction::View => "page.view",
    };
    // A page's own source link wins; a page a section built itself has no source to edit
    let url = match &page.source_url {
        Some(url) => url.clone(),
        None if page.source.is_empty() => return None,
        None => format!("{}{}", site.config.edit.as_ref()?, page.source),
    };
    let label = escape_html(site.read_language(&page.language).read_message(label));
    Some(format!("<a href=\"{}\">{label}</a>", escape_html(&url)))
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
