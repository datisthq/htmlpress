use sha2::{Digest, Sha256};

use crate::actions::page::convert::{convert_page, create_markdown_path};
use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::NOT_FOUND_PAGE;

const SKILLS_SCHEMA: &str = "https://schemas.agentskills.io/discovery/0.2.0/schema.json";
const SKILLS_INDEX: &str = ".well-known/agent-skills/index.json";

/// The files that describe the site to agents, by output path: each page as Markdown,
/// `llms.txt`, the published skills with their index, and `_headers` linking them.
pub(crate) fn describe_site(site: &Site) -> crate::Result<Vec<(String, Vec<u8>)>> {
    let pages: Vec<&Page> = (site.pages.iter())
        .filter(|page| page.path != NOT_FOUND_PAGE)
        .collect();
    let mut files = vec![];
    for page in &pages {
        let markdown = convert_page(page)?;
        files.push((create_markdown_path(&page.path), markdown.into_bytes()));
    }
    let llms = create_llms(site, &pages);
    if !pages.iter().any(|page| page.path == "index.html") {
        files.push(("index.md".to_owned(), llms.clone().into_bytes()));
    }
    files.push(("llms.txt".to_owned(), llms.into_bytes()));
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

// Grouped as the navigation groups them: root pages first, then each top folder under
// its index page's title.
fn create_llms(site: &Site, pages: &[&Page]) -> String {
    let mut groups: Vec<(Option<&str>, Vec<&Page>)> = vec![];
    for page in pages {
        let folder = page.path.split_once('/').map(|(folder, _)| folder);
        match groups.iter_mut().find(|(key, _)| *key == folder) {
            Some((_, items)) => items.push(page),
            None => groups.push((folder, vec![page])),
        }
    }
    let mut lines = vec![format!("# {}", site.title)];
    if let Some(description) = &site.config.description {
        lines.extend([String::new(), format!("> {description}")]);
    }
    let origin = (site.config.read_origin()).unwrap_or_else(|| site.config.read_base());
    for (folder, items) in groups {
        let title = match folder {
            Some(folder) => (site.pages.iter())
                .find(|page| page.path == format!("{folder}/index.html"))
                .map_or(folder, |page| page.title.as_str()),
            None => "Pages",
        };
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
