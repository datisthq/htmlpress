use pulldown_cmark::{Options, Parser, html};

use crate::actions::page::read::read_date;

/// One version of a changelog.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Release {
    /// As the heading names it, `0.2.0`.
    pub version: String,
    /// For its URL, `0-2-0`.
    pub slug: String,
    /// The first `YYYY-MM-DD` in its heading.
    pub date: Option<String>,
    /// Its notes as HTML, each heading one level up so `###` sections become `<h2>`.
    pub html: String,
}

/// Split a Markdown changelog at its `##` headings, one release each, newest first as the
/// file lists them; anything above the first is the file's preamble and dropped. A
/// release-please heading, `## [0.2.0](…) (2026-10-01)`, gives the version and the date.
pub(crate) fn split_changelog(markdown: &str) -> Vec<Release> {
    let mut releases = vec![];
    let mut heading: Option<&str> = None;
    let mut body: Vec<&str> = vec![];
    let mut fenced = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        match line.strip_prefix("## ").filter(|_| !fenced) {
            Some(next) => {
                releases
                    .extend(heading.and_then(|heading| create_release(heading, &body)));
                heading = Some(next);
                body.clear();
            }
            None => body.push(line),
        }
    }
    releases.extend(heading.and_then(|heading| create_release(heading, &body)));
    releases
}

fn create_release(heading: &str, body: &[&str]) -> Option<Release> {
    let text = strip_links(heading);
    let version = text.split_whitespace().next()?.to_owned();
    let date = text
        .split(|char: char| !(char.is_ascii_digit() || char == '-'))
        .find_map(read_date);
    let mut html = String::new();
    html::push_html(
        &mut html,
        Parser::new_ext(&body.join("\n"), Options::ENABLE_TABLES),
    );
    for level in 2..=6 {
        html = html
            .replace(&format!("<h{level}"), &format!("<h{}", level - 1))
            .replace(&format!("</h{level}>"), &format!("</h{}>", level - 1));
    }
    let slug = create_slug(&version);
    Some(Release {
        version,
        slug,
        date,
        html,
    })
}

// `[0.2.0](https://…)` reads as `0.2.0`
fn strip_links(text: &str) -> String {
    let mut plain = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        plain.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find("](").zip(after.find(')')) {
            Some((close, end)) if close < end => {
                plain.push_str(&after[..close]);
                rest = &after[end + 1..];
            }
            _ => {
                plain.push('[');
                rest = after;
            }
        }
    }
    plain.push_str(rest);
    plain
}

fn create_slug(text: &str) -> String {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|char: char| !char.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    words.join("-")
}

#[cfg(test)]
#[path = "split_unit.rs"]
mod unit;
