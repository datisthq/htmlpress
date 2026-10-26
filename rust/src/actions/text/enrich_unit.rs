use super::*;

#[test]
fn links_urls_and_emails_in_prose() -> Result<(), String> {
    assert_eq!(
        enrich_text(
            "<p>See https://htmlpress.dev/a?b=1&amp;c=2. Or www.example.com, or (team@example.com).</p>"
        )?,
        "<p>See <a href=\"https://htmlpress.dev/a?b=1&amp;c=2\">https://htmlpress.dev/a?b=1&amp;c=2</a>. \
         Or <a href=\"https://www.example.com\">www.example.com</a>, \
         or (<a href=\"mailto:team@example.com\">team@example.com</a>).</p>"
    );
    Ok(())
}

#[test]
fn replaces_known_shortcodes_only() -> Result<(), String> {
    assert_eq!(
        enrich_text("<p>Ship it :rocket: :tada:</p>")?,
        "<p>Ship it 🚀 🎉</p>"
    );
    for html in [
        "<p>At 10:30:00 sharp</p>",
        "<p>Keep :not-an-emoji: here</p>",
        "<p>a &amp; b, index.html</p>",
    ] {
        assert_eq!(enrich_text(html)?, html);
    }
    Ok(())
}

#[test]
fn leaves_code_links_and_literal_text_alone() -> Result<(), String> {
    for html in [
        "<p><a href=\"/x\">https://already.dev</a></p>",
        "<p><code>:rocket: https://x.dev</code></p>",
        "<pre><code>curl https://x.dev</code></pre>",
        "<htmlpress-terminal><pre><kbd>open https://x.dev</kbd></pre></htmlpress-terminal>",
        "<p data-literal>https://x.dev :rocket:</p>",
        "<title>See https://x.dev</title>",
    ] {
        assert_eq!(enrich_text(html)?, html);
    }
    assert_eq!(
        enrich_text("<p><img data-literal src=\"a.png\"> :tada:</p>")?,
        "<p><img data-literal src=\"a.png\"> 🎉</p>"
    );
    assert_eq!(
        enrich_text("<p><code>x</code> then :rocket:</p>")?,
        "<p><code>x</code> then 🚀</p>"
    );
    Ok(())
}
