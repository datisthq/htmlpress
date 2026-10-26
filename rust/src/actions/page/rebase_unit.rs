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
fn rebases_media_of_a_page_served_elsewhere() -> Result<(), String> {
    let mut page = Page {
        source: "docs/guide.es.html".into(),
        url: "/es/guide.html".into(),
        image: Some("cover.png".into()),
        content: concat!(
            r#"<img src="shot.png" srcset="shot.png 1x, /big.png 2x">"#,
            r#"<a href="other.html">x</a><video poster="p.jpg"></video>"#,
        )
        .into(),
        ..crate::actions::page::read::read_page("guide.html", "")
            .map_err(|e| e.to_string())?
    };
    rebase_page(&mut page, &["docs/".to_owned()])?;
    assert_eq!(page.image.as_deref(), Some("/cover.png"));
    assert_eq!(
        page.content,
        concat!(
            r#"<img src="/shot.png" srcset="/shot.png 1x, /big.png 2x">"#,
            r#"<a href="other.html">x</a><video poster="/p.jpg"></video>"#,
        )
    );
    Ok(())
}

#[test]
fn leaves_a_page_served_beside_its_source() -> Result<(), String> {
    let mut page = Page {
        source: "guides/a.html".into(),
        url: "/guides/a.html".into(),
        content: r#"<img src="shot.png">"#.into(),
        ..crate::actions::page::read::read_page("guides/a.html", "")
            .map_err(|e| e.to_string())?
    };
    rebase_page(&mut page, &[])?;
    assert_eq!(page.content, r#"<img src="shot.png">"#);
    Ok(())
}
