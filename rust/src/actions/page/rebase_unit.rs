use std::collections::BTreeMap;

use super::*;
use crate::actions::page::parse::parse_html_source;
use crate::actions::page::read::{finish_page, read_source};
use crate::actions::pipeline::create::create_pipeline;
use crate::models::pipeline::PipelineOptions;

// A page read from `html` at `source` and finished as the scan finishes it under `docs/`
fn finish(
    path: &str,
    source: &str,
    html: &str,
    links: BTreeMap<String, String>,
) -> Result<Page, String> {
    let options = PipelineOptions {
        rebase: Some("docs".into()),
        links,
        themes: None,
        plugins: vec![],
    };
    let mut pipeline = create_pipeline(&options)?;
    let alloc = textlane::Allocator::new();
    let tree = parse_html_source(&mut pipeline, path, alloc.alloc_str(html), &alloc)?;
    let mut page = Page {
        source: source.into(),
        ..read_source(path, &tree).map_err(|e| e.to_string())?
    };
    rebase_image(&mut page, &["docs/".to_owned()]);
    finish_page(&mut page, tree, &mut pipeline, None, &alloc)
        .map_err(|e| e.to_string())?;
    Ok(page)
}

#[test]
fn strips_the_include_base_from_the_source_folder() {
    let bases = vec!["rust/docs/".to_owned()];
    assert_eq!(
        read_source_folder("rust/docs/guides/a.es.html", &bases),
        "/guides/"
    );
    assert_eq!(read_source_folder("README.md", &bases), "/");
}

#[test]
fn rebases_the_urls_of_a_page_served_at_its_folder() -> Result<(), String> {
    let html = concat!(
        r#"<meta name="image" content="cover.png"><link rel="stylesheet" href="guide.css">"#,
        r#"<img src="shot.png" srcset="shot.png 1x, /big.png 2x">"#,
        r#"<a href="data.csv">x</a><video poster="p.jpg"></video>"#,
    );
    let page = finish("guide.es.html", "docs/guide.es.html", html, BTreeMap::new())?;
    assert_eq!(page.image.as_deref(), Some("/cover.png"));
    assert_eq!(page.head, r#"<link rel="stylesheet" href="/guide.css">"#);
    assert_eq!(
        page.content,
        concat!(
            r#"<img src="/shot.png" srcset="/shot.png 1x, /big.png 2x">"#,
            r#"<a href="/data.csv">x</a><video poster="/p.jpg"></video>"#,
        )
    );
    Ok(())
}

#[test]
fn links_a_page_file_to_its_url() -> Result<(), String> {
    let read = |path: &str, source: &str| -> Result<Page, String> {
        let alloc = textlane::Allocator::new();
        let tree = textlane::hast::Node::Root(textlane::hast::Root {
            children: textlane::ArenaVec::new_in(&alloc),
            position: None,
            data: None,
            quirks_mode: None,
        });
        let page = read_source(path, &tree);
        Ok(Page {
            source: source.into(),
            ..page.map_err(|e| e.to_string())?
        })
    };
    let other = read("es/other.html", "docs/other.es.html")?;
    let index = read("guides/index.html", "docs/guides/index.html")?;
    let links = list_links(&[other, index], &["docs/".to_owned()]);
    let html = concat!(
        r#"<a href="other.es.html#a">x</a><a href="/es/other.html?q=1">x</a>"#,
        r#"<a href="guides/index.html">x</a><a href="missing.html">x</a>"#,
        r#"<img src="other.es.html">"#,
    );
    let page = finish("guide.html", "docs/guide.html", html, links)?;
    assert_eq!(
        page.content,
        concat!(
            r#"<a href="/es/other/#a">x</a><a href="/es/other/?q=1">x</a>"#,
            r#"<a href="/guides/">x</a><a href="/missing.html">x</a>"#,
            r#"<img src="/other.es.html">"#,
        )
    );
    Ok(())
}
