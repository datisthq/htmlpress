use std::path::PathBuf;

use super::*;
use crate::actions::page::read::read_page;
use crate::models::config::{Config, Link};

fn create_site() -> crate::Result<Site> {
    Ok(Site {
        root: PathBuf::from("."),
        config: Config::default(),
        title: "Docs <&>".to_owned(),
        pages: vec![
            read_page("index.html", "<h1>Home</h1>")?,
            read_page(
                "guide.html",
                "<title>Guide</title><p>Read me.</p><h2>Setup</h2>",
            )?,
            read_page("api/index.html", "<title>API</title><p>Reference.</p>")?,
            read_page("api/cli.html", "<title>CLI</title><p>Flags.</p>")?,
        ],
        files: vec![],
        layout: None,
    })
}

#[test]
fn renders_the_page_in_the_layout() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<title>Guide · Docs &lt;&amp;&gt;</title>"));
    assert!(html.contains("<p>Read me.</p>"));
    assert!(html.contains(r#"<a href="/guide.html" aria-current="page">Guide</a>"#));
    assert!(html.contains(r#"<a href="/">Home</a>"#));
    assert!(html.contains(r#"<hp-brand><a href="/">Docs &lt;&amp;&gt;</a></hp-brand>"#));
    assert!(!html.contains("<hp-content>") && !html.contains(RELOAD_PATH));
    Ok(())
}

#[test]
fn groups_the_navigation_by_folder() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(
        r#"<details open><summary>API</summary><ul><li><a href="/api/">API</a></li><li><a href="/api/cli.html">CLI</a></li></ul></details>"#
    ));
    Ok(())
}

#[test]
fn links_the_previous_and_next_pages() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"<a rel="prev" href="/"><small>Previous</small>Home</a>"#));
    assert!(html.contains(r#"<a rel="next" href="/api/"><small>Next</small>API</a>"#));
    let first = render_page(&site, &site.pages[0], false)?;
    assert!(!first.contains(r#"rel="prev""#));
    Ok(())
}

#[test]
fn fills_the_toc_and_removes_it_without_headings() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r##"<a href="#setup" data-level="2">Setup</a>"##));
    let home = render_page(&site, &site.pages[0], false)?;
    assert!(!home.contains("<hp-toc"));
    Ok(())
}

#[test]
fn adds_the_links_and_the_edit_link_from_the_config() -> crate::Result<()> {
    let mut site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(!html.contains("<hp-links") && !html.contains("<hp-edit"));
    site.config.links = vec![Link {
        title: "GitHub".to_owned(),
        url: "https://github.com/x".to_owned(),
    }];
    site.config.edit = Some("https://github.com/x/edit/main/docs/".to_owned());
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"<a href="https://github.com/x">GitHub</a>"#));
    assert!(
        html.contains(r#"<a href="https://github.com/x/edit/main/docs/guide.html">"#)
    );
    Ok(())
}

#[test]
fn adds_the_reload_script_for_the_dev_server() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[0], true)?;
    assert!(html.contains(RELOAD_PATH));
    Ok(())
}

#[test]
fn uses_the_layout_escape() -> crate::Result<()> {
    let mut site = create_site()?;
    site.layout = Some("<main><hp-content></hp-content></main>".to_owned());
    let html = render_page(&site, &site.pages[0], false)?;
    assert_eq!(html, "<main><h1>Home</h1></main>");
    Ok(())
}
