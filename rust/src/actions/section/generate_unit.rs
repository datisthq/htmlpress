use super::*;
use crate::actions::language::resolve::resolve_english;
use crate::models::config::Config;

fn create_post(path: &str, date: &str, tags: &str) -> crate::Result<Page> {
    read_page(
        path,
        &format!(
            "<title>{path}</title><meta name=\"date\" content=\"{date}\">\
             <meta name=\"keywords\" content=\"{tags}\"><meta name=\"author\" content=\"Ann\">"
        ),
    )
}

fn create_section(kind: SectionKind, prefix: &str) -> Section {
    Section {
        title: "News".into(),
        kind,
        prefix: Some(prefix.to_owned()),
        source: Some("CHANGELOG.md".to_owned()),
        ..Section::default()
    }
}

#[test]
fn builds_a_blog_index_and_tag_pages() -> crate::Result<()> {
    let pages = vec![
        create_post("blog/old.html", "2026-01-01", "rust")?,
        create_post("blog/new.html", "2026-10-01", "rust, web")?,
    ];
    let config = Config {
        sections: vec![create_section(SectionKind::Blog, "/blog/")],
        ..Config::default()
    };
    let generated =
        generate_pages(Path::new("."), &resolve_english(&config, "Docs"), &pages)?;
    let urls: Vec<&str> = generated.iter().map(|page| page.url.as_str()).collect();
    assert_eq!(urls, ["/blog/", "/blog/tags/rust/", "/blog/tags/web/"]);
    let index = &generated[0].content;
    assert!(
        index.find("/blog/new/") < index.find("/blog/old/"),
        "{index}"
    );
    assert!(index.contains("<p><span>Oct 1, 2026</span><span>Ann</span></p>"));
    assert_eq!(generated[0].updated.as_deref(), Some("2026-10-01"));
    assert!(!index.contains("<htmlpress-badge"));
    assert!(index.contains("<article><h2") && index.contains(r#"<a href="/blog/new/">"#));
    assert!(
        generated[2].content.contains("/blog/new/")
            && !generated[2].content.contains("old")
    );
    assert!(generated[0].source.is_empty());
    Ok(())
}

#[test]
fn keeps_the_projects_own_index() -> crate::Result<()> {
    let pages = vec![read_page("blog/index.html", "<h1>Mine</h1>")?];
    let config = Config {
        sections: vec![create_section(SectionKind::Blog, "/blog/")],
        ..Config::default()
    };
    assert!(
        generate_pages(Path::new("."), &resolve_english(&config, "Docs"), &pages)?
            .is_empty()
    );
    Ok(())
}

#[test]
fn builds_a_page_per_changelog_version() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    std::fs::write(
        root.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.2.0](x) (2026-10-01)\n\n### Features\n\n* b\n\n## 0.1.0 (2026-09-01)\n\n* a\n",
    )
    .map_err(Error::io("write"))?;
    let config = Config {
        sections: vec![create_section(SectionKind::Changelog, "/changelog/")],
        ..Config::default()
    };
    let generated = generate_pages(root.path(), &resolve_english(&config, "Docs"), &[])?;
    let urls: Vec<&str> = generated.iter().map(|page| page.url.as_str()).collect();
    assert_eq!(
        urls,
        ["/changelog/", "/changelog/0-2-0/", "/changelog/0-1-0/"]
    );
    assert_eq!(generated[1].title, "0.2.0");
    assert_eq!(generated[1].date.as_deref(), Some("2026-10-01"));
    assert_eq!(generated[1].order, Some(0));
    assert_eq!(generated[1].source, "CHANGELOG.md");
    assert!(generated[0].content.contains(
        "<a href=\"/changelog/0-2-0/\">0.2.0</a></h2><p><span>Oct 1, 2026</span></p>"
    ));
    assert_eq!(generated[0].updated.as_deref(), Some("2026-10-01"));
    Ok(())
}
