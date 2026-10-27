use sha2::{Digest, Sha256};

use crate::actions::nav::build::build_nav;
use crate::actions::page::convert::{convert_page, create_markdown_path};
use crate::actions::section::find::find_section;
use crate::models::error::Error;
use crate::models::nav::NavNode;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::NOT_FOUND_PAGE;

const SKILLS_SCHEMA: &str = "https://schemas.agentskills.io/discovery/0.2.0/schema.json";
const SKILLS_INDEX: &str = ".well-known/agent-skills/index.json";

/// The files that describe the site to agents, by output path: each page as Markdown,
/// `llms.txt`, the published skills with their index, and `_headers` linking them.
pub(crate) fn describe_site(site: &Site) -> crate::Result<Vec<(String, Vec<u8>)>> {
    let mut files = describe_language(site, "/")?;
    for skill in &site.skills {
        let path = format!(".well-known/agent-skills/{}/SKILL.md", skill.name);
        files.push((path, skill.source.clone()));
    }
    if !site.skills.is_empty() {
        files.push((
            SKILLS_INDEX.to_owned(),
            create_skills_index(site)?.into_bytes(),
        ));
    }
    files.push(("_headers".to_owned(), create_headers(site)?.into_bytes()));
    Ok(files)
}

/// One language's pages as Markdown and its `llms.txt`, under its prefix: `/` for the
/// default language, `/<code>/` for the others.
pub(crate) fn describe_language(
    site: &Site,
    prefix: &str,
) -> crate::Result<Vec<(String, Vec<u8>)>> {
    let pages: Vec<&Page> = (site.pages.iter())
        .filter(|page| page.path != NOT_FOUND_PAGE && !page.untranslated)
        .collect();
    let mut files = vec![];
    for page in &pages {
        let markdown = convert_page(page)?;
        files.push((create_markdown_path(&page.path), markdown.into_bytes()));
    }
    let folder = prefix.trim_start_matches('/');
    let llms = create_llms(site, &pages, folder);
    let home = format!("{folder}index.html");
    if !pages.iter().any(|page| page.path == home) {
        files.push((format!("{folder}index.md"), llms.clone().into_bytes()));
    }
    files.push((format!("{folder}llms.txt"), llms.into_bytes()));
    Ok(files)
}

// Grouped as the navigation groups them: root pages first, then each top folder under
// its index page's title.
fn create_llms(site: &Site, pages: &[&Page], folder: &str) -> String {
    let groups = group_pages(site, pages, folder);
    let mut lines = vec![format!("# {}", site.title)];
    if let Some(description) = &site.config.description {
        lines.extend([String::new(), format!("> {description}")]);
    }
    let origin = (site.config.read_origin()).unwrap_or_else(|| site.config.read_base());
    for (title, items) in groups {
        lines.extend([String::new(), format!("## {title}"), String::new()]);
        for page in items {
            let url = format!("{origin}/{}", create_markdown_path(&page.path));
            let link = format!("[{}]({url})", page.title);
            lines.push(match &page.description {
                Some(description) => format!("- {link}: {description}"),
                None => format!("- {link}"),
            });
        }
    }
    format!("{}\n", lines.join("\n"))
}

// By section, in the config's order with "Other" last, when the site has sections; else
// as the navigation nests them
fn group_pages<'a>(
    site: &'a Site,
    pages: &[&'a Page],
    folder: &str,
) -> Vec<(&'a str, Vec<&'a Page>)> {
    let sections = &site.config.sections;
    if sections.is_empty() {
        return group_folders(pages, folder);
    }
    let mut groups: Vec<(usize, &str, Vec<&Page>)> = vec![];
    for page in pages {
        let (rank, title) = match find_section(sections, &page.url) {
            Some(index) => (index, sections[index].title.text()),
            None => (sections.len(), "Other"),
        };
        match groups.iter_mut().find(|(_, name, _)| *name == title) {
            Some((_, _, items)) => items.push(page),
            None => groups.push((rank, title, vec![page])),
        }
    }
    groups.sort_by_key(|(rank, _, _)| *rank);
    groups
        .into_iter()
        .map(|(_, title, items)| (title, items))
        .collect()
}

// The top pages first, then each top folder under its index page's title, else its name
fn group_folders<'a>(pages: &[&'a Page], folder: &str) -> Vec<(&'a str, Vec<&'a Page>)> {
    let mut groups = vec![("Pages", vec![])];
    for node in build_nav(pages, folder, false) {
        let items = NavNode::flatten(std::slice::from_ref(&node));
        let Some(first) = items.first() else {
            continue;
        };
        if node.children.is_empty() {
            groups[0].1.extend(items);
            continue;
        }
        let relative = first.path.strip_prefix(folder).unwrap_or(&first.path);
        let name = relative.split('/').next().unwrap_or(relative);
        let title = node.page.map_or(name, |page| page.title.as_str());
        groups.push((title, items));
    }
    groups.retain(|(_, items)| !items.is_empty());
    groups
}

fn create_skills_index(site: &Site) -> crate::Result<String> {
    let base = site.config.read_base();
    let skills: Vec<serde_json::Value> = (site.skills.iter())
        .map(|skill| {
            let digest: String = Sha256::digest(&skill.source)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            serde_json::json!({
                "name": skill.name,
                "type": "skill-md",
                "description": skill.description,
                "url": format!("{base}/.well-known/agent-skills/{}/SKILL.md", skill.name),
                "digest": format!("sha256:{digest}"),
            })
        })
        .collect();
    let index = serde_json::json!({ "$schema": SKILLS_SCHEMA, "skills": skills });
    let json = serde_json::to_string_pretty(&index)
        .map_err(|error| Error::Usage(format!("cannot write {SKILLS_INDEX}: {error}")))?;
    Ok(format!("{json}\n"))
}

// Cloudflare and Netlify read _headers; the project's own rules are kept, ours follow.
fn create_headers(site: &Site) -> crate::Result<String> {
    let base = site.config.read_base();
    let mut links = vec![
        format!("<{base}/llms.txt>; rel=\"describedby\"; type=\"text/plain\""),
        format!("<{base}/index.md>; rel=\"alternate\"; type=\"text/markdown\""),
    ];
    if !site.skills.is_empty() {
        links.push(format!(
            "<{base}/.well-known/agent-skills/index.json>; rel=\"describedby\"; \
             type=\"application/json\""
        ));
    }
    let rule = format!("{base}/\n  Link: {}\n", links.join(", "));
    let Some(source) = site.files.get("_headers") else {
        return Ok(rule);
    };
    let path = site.root.join(source);
    let existing = std::fs::read_to_string(&path).map_err(Error::io(&path))?;
    Ok(format!("{}\n\n{rule}", existing.trim_end()))
}

#[cfg(test)]
#[path = "describe_unit.rs"]
mod unit;
