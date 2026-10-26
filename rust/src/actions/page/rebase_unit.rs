use std::collections::BTreeMap;

use super::*;

#[test]
fn resolves_only_relative_urls() {
    assert_eq!(
        resolve_url("shot.png", "/guides/").as_deref(),
        Some("/guides/shot.png")
    );
    assert_eq!(
        resolve_url("../a/b.png?v=1#x", "/guides/cli/").as_deref(),
        Some("/guides/a/b.png?v=1#x")
    );
    assert_eq!(resolve_url("./c.png", "/").as_deref(), Some("/c.png"));
    for url in [
        "/a.png",
        "https://x.dev/a.png",
        "//cdn.dev/a.png",
        "data:image/png;base64,AA",
        "#top",
        "",
    ] {
        assert_eq!(resolve_url(url, "/guides/"), None, "{url}");
    }
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
    let mut page = Page {
        source: "docs/guide.es.html".into(),
        url: "/es/guide/".into(),
        image: Some("cover.png".into()),
        head: r#"<link rel="stylesheet" href="guide.css">"#.into(),
        content: concat!(
            r#"<img src="shot.png" srcset="shot.png 1x, /big.png 2x">"#,
            r#"<a href="data.csv">x</a><video poster="p.jpg"></video>"#,
        )
        .into(),
        ..crate::actions::page::read::read_page("guide.html", "")
            .map_err(|e| e.to_string())?
    };
    rebase_page(&mut page, &["docs/".to_owned()], &BTreeMap::new())?;
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
        let page = crate::actions::page::read::read_page(path, "");
        Ok(Page {
            source: source.into(),
            ..page.map_err(|e| e.to_string())?
        })
    };
    let other = read("es/other.html", "docs/other.es.html")?;
    let index = read("guides/index.html", "docs/guides/index.html")?;
    let bases = ["docs/".to_owned()];
    let links = list_links(&[other, index], &bases);
    let mut page = Page {
        content: concat!(
            r#"<a href="other.es.html#a">x</a><a href="/es/other.html?q=1">x</a>"#,
            r#"<a href="guides/index.html">x</a><a href="missing.html">x</a>"#,
            r#"<img src="other.es.html">"#,
        )
        .into(),
        ..read("guide.html", "docs/guide.html")?
    };
    rebase_page(&mut page, &bases, &links)?;
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
