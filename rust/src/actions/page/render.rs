use std::fmt::Write;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str};

use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::{ASSETS_DIR, LAYOUT, RELOAD_PATH};

/// Wrap one page of the site in the layout, the project's escape else the built-in one,
/// filling its `<head>` and every `hp-*` placeholder element the layout holds.
pub(crate) fn render_page(
    site: &Site,
    page: &Page,
    reload: bool,
) -> crate::Result<String> {
    let layout = site.layout.as_deref().unwrap_or(LAYOUT);
    let nav = create_nav(site, page);
    let head = create_head(site, page, reload);
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("head", |el| {
            el.prepend(&head, ContentType::Html);
            el.append(&page.head, ContentType::Html);
            Ok(())
        }))
        .append_element_content_handler(element!("hp-content", |el| {
            el.replace(&page.content, ContentType::Html);
            Ok(())
        }))
        .append_element_content_handler(element!("hp-brand", |el| {
            let title = escape_html(&site.title);
            el.set_inner_content(
                &format!("<a href=\"/\">{title}</a>"),
                ContentType::Html,
            );
            Ok(())
        }))
        .append_element_content_handler(element!("hp-nav", |el| {
            el.set_inner_content(&nav, ContentType::Html);
            Ok(())
        }))
        .append_element_content_handler(element!("hp-links", |el| {
            fill_element(el, create_links(site));
            Ok(())
        }))
        .append_element_content_handler(element!("hp-pager", |el| {
            fill_element(el, create_pager(site, page));
            Ok(())
        }))
        .append_element_content_handler(element!("hp-toc", |el| {
            fill_element(el, create_toc(page));
            Ok(())
        }))
        .append_element_content_handler(element!("hp-edit", |el| {
            fill_element(el, create_edit(site, page));
            Ok(())
        }));
    rewrite_str(layout, settings)
        .map_err(|error| Error::page(&page.path)(error.to_string()))
}

// A placeholder with nothing to show is removed, so a layout can style or hide the
// box around it with `:empty` or `:has()`.
fn fill_element(el: &mut lol_html::html_content::Element, html: Option<String>) {
    match html {
        Some(html) => el.set_inner_content(&html, ContentType::Html),
        None => el.remove(),
    }
}

fn create_head(site: &Site, page: &Page, reload: bool) -> String {
    let title = match page.url.as_str() {
        "/" => escape_html(&site.title),
        _ => format!(
            "{} · {}",
            escape_html(&page.title),
            escape_html(&site.title)
        ),
    };
    let mut head = String::from(
        "<meta charset=\"utf-8\">\n<meta name=\"viewport\" \
         content=\"width=device-width, initial-scale=1\">\n",
    );
    let _ = writeln!(head, "<title>{title}</title>");
    let description = page
        .description
        .as_ref()
        .or(site.config.description.as_ref());
    if let Some(description) = description {
        let description = escape_html(description);
        let _ = writeln!(
            head,
            "<meta name=\"description\" content=\"{description}\">"
        );
    }
    if let Some(url) = &site.config.site {
        let url = escape_html(url.trim_end_matches('/'));
        let path = escape_html(&page.url);
        let _ = writeln!(head, "<link rel=\"canonical\" href=\"{url}{path}\">");
    }
    // Set before the stylesheet applies, so a stored dark theme never flashes light
    head.push_str(
        "<script>try { const theme = localStorage.getItem(\"hp-theme\"); \
         if (theme) document.documentElement.dataset.theme = theme } catch {}</script>\n",
    );
    let _ = writeln!(
        head,
        "<link rel=\"stylesheet\" href=\"/{ASSETS_DIR}/theme.css\">"
    );
    let _ = writeln!(
        head,
        "<script type=\"module\" src=\"/{ASSETS_DIR}/components.js\"></script>"
    );
    if reload {
        let _ = writeln!(
            head,
            "<script type=\"module\" src=\"/{ASSETS_DIR}/reload.js\" \
             data-endpoint=\"{RELOAD_PATH}\"></script>"
        );
    }
    head
}

// Pages group by their top folder, titled by that folder's index page, else its name;
// groups keep the order their first page has in the navigation.
fn create_nav(site: &Site, page: &Page) -> String {
    let mut groups: Vec<(Option<&str>, Vec<&Page>)> = vec![];
    for item in &site.pages {
        let folder = item.path.split_once('/').map(|(folder, _)| folder);
        match groups.iter_mut().find(|(key, _)| *key == folder) {
            Some((_, items)) => items.push(item),
            None => groups.push((folder, vec![item])),
        }
    }
    let mut html = String::from("<nav aria-label=\"Pages\">");
    for (folder, items) in groups {
        let list = create_list(&items, page);
        match folder {
            Some(folder) => {
                let index = format!("{folder}/index.html");
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
    html.push_str("</nav>");
    html
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

fn create_links(site: &Site) -> Option<String> {
    let links = &site.config.links;
    if links.is_empty() {
        return None;
    }
    let mut html = String::from("<nav aria-label=\"Links\">");
    for link in links {
        let url = escape_html(&link.url);
        let title = escape_html(&link.title);
        let _ = write!(html, "<a href=\"{url}\">{title}</a>");
    }
    html.push_str("</nav>");
    Some(html)
}

fn create_pager(site: &Site, page: &Page) -> Option<String> {
    let index = site.pages.iter().position(|item| item.path == page.path)?;
    let prev = index.checked_sub(1).and_then(|index| site.pages.get(index));
    let next = site.pages.get(index + 1);
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

fn create_edit(site: &Site, page: &Page) -> Option<String> {
    let url = escape_html(&format!("{}{}", site.config.edit.as_ref()?, page.path));
    Some(format!("<a href=\"{url}\">Edit this page</a>"))
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
