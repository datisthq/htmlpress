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
