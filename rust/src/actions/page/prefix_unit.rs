use super::*;

#[test]
fn prefixes_only_root_relative_urls() -> Result<(), String> {
    let html = concat!(
        r#"<a href="/guide.html">a</a><a href="guide.html">b</a>"#,
        r##"<a href="https://x.dev/">c</a><a href="//cdn.dev/x.js">d</a><a href="#top">e</a>"##,
        r#"<img src="/a.png" srcset="/a.png 1x, /a@2x.png 2x, b.png 3x">"#,
        r#"<script data-endpoint="/_htmlpress/reload"></script>"#,
    );
    assert_eq!(
        prefix_urls(html, "/repo")?,
        concat!(
            r#"<a href="/repo/guide.html">a</a><a href="guide.html">b</a>"#,
            r##"<a href="https://x.dev/">c</a><a href="//cdn.dev/x.js">d</a><a href="#top">e</a>"##,
            r#"<img src="/repo/a.png" srcset="/repo/a.png 1x, /repo/a@2x.png 2x, b.png 3x">"#,
            r#"<script data-endpoint="/repo/_htmlpress/reload"></script>"#,
        )
    );
    Ok(())
}
