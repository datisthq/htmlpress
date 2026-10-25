use std::path::PathBuf;

use super::*;
use crate::actions::page::read::read_page;
use crate::models::config::Config;
use crate::models::skill::Skill;

fn create_site() -> crate::Result<Site> {
    Ok(Site {
        root: PathBuf::from("."),
        config: Config {
            site: Some("https://example.com/".to_owned()),
            description: Some("Docs for agents.".to_owned()),
            ..Config::default()
        },
        title: "Docs".to_owned(),
        pages: vec![
            read_page("index.html", "<h1>Home</h1>")?,
            read_page(
                "guides/index.html",
                "<title>Guides</title><meta name=\"description\" content=\"How to.\">",
            )?,
            read_page("guides/build.html", "<title>Build</title><p>Run it.</p>")?,
            read_page("404.html", "<h1>Lost</h1>")?,
        ],
        files: vec![],
        assets: Default::default(),
        skills: vec![],
    })
}

fn read_file(files: &[(String, Vec<u8>)], path: &str) -> Option<String> {
    let (_, bytes) = files.iter().find(|(name, _)| name == path)?;
    Some(String::from_utf8_lossy(bytes).into_owned())
}

#[test]
fn writes_markdown_pages_and_llms() -> crate::Result<()> {
    let files = describe_site(&create_site()?)?;
    assert_eq!(
        read_file(&files, "guides/build.md").as_deref(),
        Some("# Build\n\nRun it.\n")
    );
    assert!(read_file(&files, "404.md").is_none());
    assert_eq!(
        read_file(&files, "llms.txt").as_deref(),
        Some(concat!(
            "# Docs\n\n> Docs for agents.\n\n",
            "## Pages\n\n- [Home](https://example.com/index.md)\n\n",
            "## Guides\n\n- [Guides](https://example.com/guides/index.md): How to.\n",
            "- [Build](https://example.com/guides/build.md)\n",
        ))
    );
    let headers = read_file(&files, "_headers").unwrap_or_default();
    assert!(headers.starts_with("/\n  Link: </llms.txt>; rel=\"describedby\""));
    assert!(!headers.contains("agent-skills"));
    Ok(())
}

#[test]
fn publishes_skills_with_an_index() -> crate::Result<()> {
    let mut site = create_site()?;
    site.skills = vec![Skill {
        name: "docs-pages".to_owned(),
        description: "Write pages.".to_owned(),
        path: PathBuf::from("skills/docs-pages/SKILL.md"),
        source: b"abc".to_vec(),
    }];
    let files = describe_site(&site)?;
    assert_eq!(
        read_file(&files, ".well-known/agent-skills/docs-pages/SKILL.md").as_deref(),
        Some("abc")
    );
    let index =
        read_file(&files, ".well-known/agent-skills/index.json").unwrap_or_default();
    let index: serde_json::Value =
        serde_json::from_str(&index).map_err(|error| Error::Usage(error.to_string()))?;
    assert_eq!(
        index["skills"][0]["url"],
        "/.well-known/agent-skills/docs-pages/SKILL.md"
    );
    assert_eq!(
        index["skills"][0]["digest"],
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert!(
        read_file(&files, "_headers")
            .is_some_and(|headers| headers.contains("index.json"))
    );
    Ok(())
}
