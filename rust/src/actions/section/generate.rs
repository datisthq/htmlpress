use std::fmt::Write;
use std::path::Path;

use crate::actions::changelog::split::split_changelog;
use crate::actions::date::format::format_date;
use crate::actions::page::read::read_page;
use crate::models::config::{Config, Section, SectionKind};
use crate::models::error::Error;
use crate::models::page::Page;

/// The pages blog and changelog sections build themselves: a blog's index and one page per
/// tag, a changelog's page per version and its index. A section's own index page wins over
/// the generated one, and no generated page replaces a page the project has.
pub(crate) fn generate_pages(
    root: &Path,
    config: &Config,
    pages: &[Page],
) -> crate::Result<Vec<Page>> {
    let mut generated = vec![];
    for section in &config.sections {
        let Some(prefix) = section.prefix.as_deref() else {
            continue;
        };
        let created = match section.kind {
            SectionKind::Blog => create_blog(section, prefix, pages)?,
            SectionKind::Changelog => create_changelog(root, section, prefix)?,
            SectionKind::Article | SectionKind::Custom => vec![],
        };
        let taken = |page: &Page| {
            (pages.iter().chain(&generated)).any(|other: &Page| other.url == page.url)
        };
        let fresh: Vec<Page> = created.into_iter().filter(|page| !taken(page)).collect();
        generated.extend(fresh);
    }
    Ok(generated)
}

/// A blog's posts, newest first: its pages but the index and the tag pages.
pub(crate) fn list_posts<'a>(prefix: &str, pages: &[&'a Page]) -> Vec<&'a Page> {
    let tags = format!("{prefix}tags/");
    let mut posts: Vec<&Page> = (pages.iter().copied())
        .filter(|page| page.url.starts_with(prefix) && page.url != prefix)
        .filter(|page| !page.url.starts_with(&tags))
        .collect();
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.url.cmp(&b.url)));
    posts
}

/// Where a blog's page for one tag is served.
pub(crate) fn create_tag_url(prefix: &str, tag: &str) -> String {
    format!("{prefix}tags/{}/", create_slug(tag))
}

fn create_blog(
    section: &Section,
    prefix: &str,
    pages: &[Page],
) -> crate::Result<Vec<Page>> {
    let all: Vec<&Page> = pages.iter().collect();
    let posts = list_posts(prefix, &all);
    let mut tags: Vec<&str> = posts
        .iter()
        .flat_map(|post| post.tags.iter().map(String::as_str))
        .collect();
    tags.sort_unstable();
    tags.dedup();
    let folder = prefix.trim_start_matches('/');
    let title = escape_html(&section.title);
    let mut index = format!("<title>{title}</title>\n<h1>{title}</h1>\n");
    index.push_str(&create_post_list(&posts));
    if !tags.is_empty() {
        index.push_str("<h2>Tags</h2>\n<p>");
        for tag in &tags {
            let _ = write!(
                index,
                "<a href=\"{}\"><htmlpress-badge type=\"neutral\">{}</htmlpress-badge></a> ",
                escape_html(&create_tag_url(prefix, tag)),
                escape_html(tag)
            );
        }
        index.push_str("</p>\n");
    }
    let mut created = vec![create_page(&format!("{folder}index.html"), &index, "")?];
    for tag in tags {
        let tagged: Vec<&Page> = posts
            .iter()
            .copied()
            .filter(|post| post.tags.iter().any(|other| other == tag))
            .collect();
        let name = escape_html(tag);
        let html = format!(
            "<title>Tagged “{name}”</title>\n<h1>Tagged “{name}”</h1>\n{}",
            create_post_list(&tagged)
        );
        let path = format!(
            "{}index.html",
            create_tag_url(prefix, tag).trim_start_matches('/')
        );
        created.push(create_page(&path, &html, "")?);
    }
    Ok(created)
}

fn create_post_list(posts: &[&Page]) -> String {
    if posts.is_empty() {
        return "<p>No posts yet.</p>\n".to_owned();
    }
    let mut html = String::from("<htmlpress-grid cols=\"2\">\n");
    for post in posts {
        let mut byline: Vec<String> = vec![];
        if let Some(date) = &post.date {
            byline.push(format_date(date));
        }
        if !post.authors.is_empty() {
            byline.push(post.authors.join(", "));
        }
        let _ = write!(
            html,
            "<htmlpress-card><h3><a href=\"{}\">{}</a></h3>",
            escape_html(&post.url),
            escape_html(&post.title)
        );
        if !byline.is_empty() {
            let _ = write!(
                html,
                "<p><small>{}</small></p>",
                escape_html(&byline.join(" · "))
            );
        }
        if let Some(description) = &post.description {
            let _ = write!(html, "<p>{}</p>", escape_html(description));
        }
        html.push_str("</htmlpress-card>\n");
    }
    html.push_str("</htmlpress-grid>\n");
    html
}

fn create_changelog(
    root: &Path,
    section: &Section,
    prefix: &str,
) -> crate::Result<Vec<Page>> {
    let source = section.source.as_deref().unwrap_or_default();
    let file = root.join(source);
    let markdown = std::fs::read_to_string(&file).map_err(Error::io(&file))?;
    let releases = split_changelog(&markdown);
    let folder = prefix.trim_start_matches('/');
    let title = escape_html(&section.title);
    let mut index = format!("<title>{title}</title>\n<h1>{title}</h1>\n<ul>\n");
    let mut created = vec![];
    for (order, release) in releases.iter().enumerate() {
        let version = escape_html(&release.version);
        let url = format!("{prefix}{}/", release.slug);
        let date = release.date.as_deref();
        let meta = date.map_or(String::new(), |date| {
            format!("<meta name=\"date\" content=\"{date}\">\n")
        });
        let html = format!(
            "<title>{version}</title>\n<meta name=\"order\" content=\"{order}\">\n{meta}<h1>{version}</h1>\n{}",
            release.html
        );
        created.push(create_page(
            &format!("{folder}{}/index.html", release.slug),
            &html,
            source,
        )?);
        let when = date.map_or(String::new(), |date| format!(" · {}", format_date(date)));
        let _ = writeln!(
            index,
            "<li><a href=\"{}\">{version}</a>{when}</li>",
            escape_html(&url)
        );
    }
    index.push_str("</ul>\n");
    created.insert(
        0,
        create_page(&format!("{folder}index.html"), &index, source)?,
    );
    Ok(created)
}

fn create_page(path: &str, html: &str, source: &str) -> crate::Result<Page> {
    let page = read_page(path, html)?;
    Ok(Page {
        source: source.to_owned(),
        ..page
    })
}

fn create_slug(text: &str) -> String {
    let lower = text.to_lowercase();
    let words: Vec<&str> = (lower.split(|char: char| !char.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect();
    words.join("-")
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "generate_unit.rs"]
mod unit;
