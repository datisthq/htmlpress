use std::path::PathBuf;

use super::*;
use crate::actions::page::read::read_page;
use crate::models::config::Config;

fn create_site() -> crate::Result<Site> {
    Ok(Site {
        root: PathBuf::from("."),
        config: Config::default(),
        title: "Docs <&>".to_owned(),
        pages: vec![
            read_page("index.html", "<h1>Home</h1>")?,
            read_page("guide.html", "<title>Guide</title><p>Read me.</p>")?,
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
    assert!(!html.contains(RELOAD_PATH));
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
    site.layout = Some("<main>{{ page.content | safe }}</main>".to_owned());
    let html = render_page(&site, &site.pages[0], false)?;
    assert_eq!(html, "<main><h1>Home</h1></main>");
    Ok(())
}
