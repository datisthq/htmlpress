use std::fmt::Write;
use std::path::Path;

use crate::actions::changelog::fetch::{fetch_releases, is_github_source};
use crate::actions::changelog::split::split_changelog;
use crate::actions::date::format::format_date;
use crate::actions::language::detect::create_key;
use crate::actions::page::read::read_page;
use crate::actions::path::contain::read_contained;
use crate::models::config::{Section, SectionKind};
use crate::models::error::Error;
use crate::models::language::SiteLanguage;
use crate::models::page::{Page, SourceAction};
use crate::settings::CONFIG_FILE;

/// The pages blog and changelog sections build themselves: a blog's index and one page per
/// tag, a changelog's page per version and its index. A section's own index page wins over
/// the generated one, and no generated page replaces a page the project has.
pub(crate) fn generate_pages(
    root: &Path,
    language: &SiteLanguage,
    pages: &[Page],
) -> crate::Result<Vec<Page>> {
    let mut generated = vec![];
    for section in &language.config.sections {
        let Some(prefix) = section.prefix.as_deref() else {
            continue;
        };
        let created = match section.kind {
            SectionKind::Blog => create_blog(section, prefix, pages, language)?,
            SectionKind::Changelog => create_changelog(root, section, prefix, language)?,
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
    language: &SiteLanguage,
) -> crate::Result<Vec<Page>> {
    let all: Vec<&Page> = pages.iter().collect();
    let mut posts = list_posts(prefix, &all);
    // Another language's blog lists the default posts it has no translation of, as its
    // navigation does, so it never reads "No posts yet" beside a list of posts
    if !language.default {
        let original = list_posts(&create_key(prefix, language), &all);
        let missing: Vec<&Page> = (original.into_iter())
            .filter(|post| !posts.iter().any(|other| other.key == post.key))
            .collect();
        posts.extend(missing);
        posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.url.cmp(&b.url)));
    }
    let mut tags: Vec<&str> = posts
        .iter()
        .flat_map(|post| post.tags.iter().map(String::as_str))
        .collect();
    tags.sort_unstable();
    tags.dedup();
    let folder = prefix.trim_start_matches('/');
    let title = escape_html(&section.title);
    let mut index = format!("<title>{title}</title>\n<h1>{title}</h1>\n");
    index.push_str(&create_post_list(&posts, language));
    if !tags.is_empty() {
        let heading = escape_html(language.read_message("nav.tags"));
        let _ = write!(index, "<h2>{heading}</h2>\n<p>");
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
        let title = escape_html(&language.format_message("blog.tagged", &[("tag", tag)]));
        let html = format!(
            "<title>{title}</title>\n<h1>{title}</h1>\n{}",
            create_post_list(&tagged, language)
        );
        let path = format!(
            "{}index.html",
            create_tag_url(prefix, tag).trim_start_matches('/')
        );
        created.push(create_page(&path, &html, "")?);
    }
    Ok(created)
}

fn create_post_list(posts: &[&Page], language: &SiteLanguage) -> String {
    if posts.is_empty() {
        return format!(
            "<p>{}</p>\n",
            escape_html(language.read_message("blog.empty"))
        );
    }
    // Livemark's blog index: one post after another, each heading a link to it
    let mut html = String::from("<section class=\"htmlpress-posts\">\n");
    for post in posts {
        let _ = write!(
            html,
            "<article><h2><a href=\"{}\">{}</a></h2>",
            escape_html(&post.url),
            escape_html(&post.title)
        );
        let mut byline: Vec<String> = vec![];
        if let Some(date) = &post.date {
            byline.push(format_date(date, language));
        }
        if !post.authors.is_empty() {
            byline.push(post.authors.join(", "));
        }
        if !byline.is_empty() {
            html.push_str("<p>");
            for part in byline {
                let _ = write!(html, "<span>{}</span>", escape_html(&part));
            }
            html.push_str("</p>");
        }
        if let Some(description) = &post.description {
            let _ = write!(html, "<p>{}</p>", escape_html(description));
        }
        html.push_str("</article>\n");
    }
    html.push_str("</section>\n");
    html
}

fn create_changelog(
    root: &Path,
    section: &Section,
    prefix: &str,
    language: &SiteLanguage,
) -> crate::Result<Vec<Page>> {
    let source = section.source.as_deref().unwrap_or_default();
    let fail = |message: String| Error::Config {
        path: root.join(CONFIG_FILE),
        message: format!("section \"{}\": source {message}", section.title),
    };
    let releases = match is_github_source(source) {
        true => fetch_releases(root, source).map_err(fail)?,
        false => split_changelog(&read_contained(root, source, false).map_err(fail)?),
    };
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
        let mut page = create_page(
            &format!("{folder}{}/index.html", release.slug),
            &html,
            source,
        )?;
        if let Some(url) = &release.url {
            page.source_url = Some(url.clone());
            page.source_action = SourceAction::View;
        }
        created.push(page);
        let when = date.map_or(String::new(), |date| {
            format!(" · {}", format_date(date, language))
        });
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
