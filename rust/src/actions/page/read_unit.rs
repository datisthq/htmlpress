use super::*;

const DOCUMENT: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Build &amp; deploy</title>
    <meta name="description" content="Ship the site.">
    <meta name="order" content="2">
    <link rel="stylesheet" href="extra.css">
  </head>
  <body>
    <h1>Building</h1>
    <p>Run <code>htmlpress build</code>.</p>
  </body>
</html>
"#;

#[test]
fn reads_a_full_document() -> crate::Result<()> {
    let page = read_page("guides/build.html", DOCUMENT)?;
    assert_eq!(page.title, "Build & deploy");
    assert_eq!(page.description.as_deref(), Some("Ship the site."));
    assert_eq!(page.order, Some(2));
    assert_eq!(page.url, "/guides/build.html");
    assert_eq!(page.head, r#"<link rel="stylesheet" href="extra.css">"#);
    assert!(page.content.starts_with("<h1>Building</h1>"));
    assert!(
        page.content
            .ends_with("<p>Run <code>htmlpress build</code>.</p>")
    );
    assert!(!page.content.contains("<body") && !page.content.contains("<title"));
    Ok(())
}

#[test]
fn reads_a_fragment() -> crate::Result<()> {
    let page = read_page("index.html", "<h1>Hello <em>world</em></h1>\n<p>Hi.</p>\n")?;
    assert_eq!(page.title, "Hello world");
    assert_eq!(page.url, "/");
    assert_eq!(page.head, "");
    assert_eq!(page.content, "<h1>Hello <em>world</em></h1>\n<p>Hi.</p>");
    Ok(())
}

#[test]
fn keeps_a_fragments_title_and_metas_out_of_the_content() -> crate::Result<()> {
    let html =
        "<title>Guide</title>\n<meta name=\"order\" content=\"3\">\n<p>Read me.</p>";
    let page = read_page("guide.html", html)?;
    assert_eq!((page.title.as_str(), page.order), ("Guide", Some(3)));
    assert_eq!(page.content, "<p>Read me.</p>");
    Ok(())
}

#[test]
fn reads_the_layout_and_keeps_it_out_of_the_content() -> crate::Result<()> {
    let html = "<meta name=\"layout\" content=\"landing\">\n<p>Hi.</p>";
    let page = read_page("index.html", html)?;
    assert_eq!(page.layout.as_deref(), Some("landing"));
    assert_eq!(page.content, "<p>Hi.</p>");
    Ok(())
}

#[test]
fn rejects_a_layout_that_is_not_a_name() {
    let html = r#"<meta name="layout" content="../layout">"#;
    let error = read_page("a.html", html)
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("lowercase letters")));
}

#[test]
fn falls_back_to_the_file_name() -> crate::Result<()> {
    let page = read_page("notes/todo.html", "<p>Nothing to see.</p>")?;
    assert_eq!(page.title, "todo");
    Ok(())
}

#[test]
fn rejects_a_non_integer_order() {
    let html = r#"<head><meta name="order" content="first"></head><p>x</p>"#;
    let error = read_page("a.html", html)
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("must be an integer")));
}

#[test]
fn creates_urls() {
    assert_eq!(create_url("index.html"), "/");
    assert_eq!(create_url("guides/index.html"), "/guides/");
    assert_eq!(create_url("guides/build.html"), "/guides/build.html");
    assert_eq!(create_url("myindex.html"), "/myindex.html");
}

#[test]
fn lists_the_headings_and_gives_each_an_id() -> crate::Result<()> {
    let html = "<h1>Guide</h1>\n<h2>Install &amp; run</h2>\n<h3 id=\"flags\">The <code>--port</code> flag</h3>\n<h2>Install &amp; run</h2>\n<h2>Flags</h2>";
    let page = read_page("guide.html", html)?;
    let toc: Vec<_> = page
        .toc
        .iter()
        .map(|heading| (heading.id.as_str(), heading.title.as_str(), heading.level))
        .collect();
    assert_eq!(
        toc,
        [
            ("install-run", "Install & run", 2),
            ("flags", "The --port flag", 3),
            ("install-run-2", "Install & run", 2),
            ("flags-2", "Flags", 2),
        ]
    );
    assert!(page.content.contains("<h2 id=\"install-run\">"));
    assert!(page.content.contains("<h3 id=\"flags\">"));
    assert!(page.content.contains("<h2 id=\"install-run-2\">"));
    Ok(())
}

#[test]
fn keeps_headings_inside_components_out_of_the_toc() -> crate::Result<()> {
    let html = "<h2>Cards</h2>\n<htmlpress-grid><htmlpress-card><h3>One</h3></htmlpress-card></htmlpress-grid>\n<h3>After</h3>";
    let page = read_page("guide.html", html)?;
    let ids: Vec<_> = page.toc.iter().map(|heading| heading.id.as_str()).collect();
    assert_eq!(ids, ["cards", "after"]);
    assert!(page.content.contains("<h3 id=\"one\">One</h3>"));
    Ok(())
}

#[test]
fn reads_the_image_and_keeps_it_out_of_the_head() -> crate::Result<()> {
    let html = "<html><head><title>A</title><meta name=\"image\" content=\"/a.png\"></head><body><p>A</p></body></html>";
    let page = read_page("a.html", html)?;
    assert_eq!(page.image.as_deref(), Some("/a.png"));
    assert!(!page.head.contains("image"));
    Ok(())
}

#[test]
fn expands_package_tabs_into_tabs() -> crate::Result<()> {
    let html = "<htmlpress-package-tabs><pre><code>npm install a &amp;&amp; npx b</code></pre></htmlpress-package-tabs>";
    let page = read_page("a.html", html)?;
    assert_eq!(
        page.content,
        concat!(
            r#"<htmlpress-tabs sync="package-manager">"#,
            r#"<section title="npm"><pre><code>npm install a &amp;&amp; npx b</code></pre></section>"#,
            r#"<section title="pnpm"><pre><code>pnpm add a &amp;&amp; npx b</code></pre></section>"#,
            r#"<section title="yarn"><pre><code>yarn add a &amp;&amp; npx b</code></pre></section>"#,
            r#"<section title="bun"><pre><code>bun add a &amp;&amp; npx b</code></pre></section>"#,
            "</htmlpress-tabs>"
        )
    );
    Ok(())
}

#[test]
fn fills_inline_icons_and_refuses_unknown_ones() -> crate::Result<()> {
    let page = read_page(
        "a.html",
        "<p><htmlpress-icon name=\"rocket\"></htmlpress-icon> Go</p>",
    )?;
    assert!(
        page.content
            .contains("<htmlpress-icon name=\"rocket\"><svg viewBox")
    );
    let error =
        read_page("a.html", "<htmlpress-icon name=\"rockt\"></htmlpress-icon>").err();
    assert!(error.is_some_and(|error| error.to_string().contains("no such icon")));
    Ok(())
}

#[test]
fn reads_the_date_authors_and_tags() -> crate::Result<()> {
    let page = read_page(
        "post.html",
        "<title>Post</title><meta name=\"date\" content=\"2026-10-01T09:00:00Z\">\
         <meta name=\"author\" content=\"Ann\"><meta name=\"author\" content=\"Bo\">\
         <meta name=\"keywords\" content=\"rust, , web \">",
    )?;
    assert_eq!(page.date.as_deref(), Some("2026-10-01"));
    assert_eq!(page.authors, ["Ann", "Bo"]);
    assert_eq!(page.tags, ["rust", "web"]);
    assert!(read_page("a.html", "<meta name=\"date\" content=\"Oct 1\">").is_err());
    Ok(())
}

#[test]
fn reads_the_navigation_and_source_metas() -> crate::Result<()> {
    let page = read_page(
        "guides/long-name.html",
        "<html><head><title>A long title</title><meta name=\"label\" content=\"Short\">\
         <meta name=\"icon\" content=\"rocket\"><meta name=\"path\" content=\"/start/\">\
         <meta name=\"sidebar\" content=\"false\"><meta name=\"toc\" content=\"false\">\
         <meta name=\"source-url\" content=\"https://x.dev/src\">\
         <meta name=\"source-action\" content=\"view\"></head>\
         <body><h2>Part</h2></body></html>",
    )?;
    assert_eq!(page.label.as_deref(), Some("Short"));
    assert_eq!(page.icon.as_deref(), Some("rocket"));
    assert_eq!(
        (page.path.as_str(), page.url.as_str()),
        ("start/index.html", "/start/")
    );
    assert!(!page.sidebar && page.toc.is_empty());
    assert_eq!(page.source_url.as_deref(), Some("https://x.dev/src"));
    assert_eq!(page.source_action, SourceAction::View);
    assert!(page.head.is_empty(), "{}", page.head);
    for bad in [
        "<meta name=\"icon\" content=\"nope\">",
        "<meta name=\"path\" content=\"start\">",
        "<meta name=\"sidebar\" content=\"no\">",
        "<meta name=\"source-action\" content=\"open\">",
    ] {
        assert!(read_page("a.html", bad).is_err(), "{bad}");
    }
    Ok(())
}
