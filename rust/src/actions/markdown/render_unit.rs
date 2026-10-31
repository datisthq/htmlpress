use super::{render_markdown, render_untrusted_markdown};

#[test]
fn renders_github_markdown() {
    let html = render_markdown(
        "| a |\n|---|\n| b |\n\n- [x] done\n\n~~gone~~\n\nnote[^1]\n\n[^1]: foot",
    );
    assert!(html.contains("<table>"));
    assert!(html.contains("type=\"checkbox\""));
    assert!(html.contains("<del>gone</del>"));
    assert!(html.contains("footnote"));
}

#[test]
fn drops_frontmatter() {
    assert_eq!(
        render_markdown("---\ntitle: A\n---\n\nhi").trim(),
        "<p>hi</p>"
    );
}

#[test]
fn keeps_raw_html() {
    assert!(
        render_markdown("<htmlpress-badge>x</htmlpress-badge>")
            .contains("<htmlpress-badge>")
    );
}

#[test]
fn drops_scripts_from_untrusted_markdown() {
    let html = render_untrusted_markdown(
        "<script>alert(1)</script>\n\n[a](javascript:alert(1)) [b](java\tscript:x) \
         ![c](data:text/html,x) [d](https://x.dev)",
    );
    assert!(!html.contains("<script"));
    assert!(!html.contains("javascript:"));
    assert!(!html.contains("data:"));
    assert!(html.contains("href=\"https://x.dev\""));
}

#[test]
fn sanitizes_untrusted_html_as_github_does() {
    let html = render_untrusted_markdown(
        "<img src=\"a.png\" onerror=\"alert(1)\" srcset=\"javascript:x 1x\">\n\n\
         <form><button formaction=\"javascript:x\">b</button></form>\n\n\
         <svg><script>x</script></svg><iframe src=\"https://x.dev\"></iframe>\n\n\
         <p style=\"position:fixed\" id=\"main\">p</p>",
    );
    for banned in [
        "onerror",
        "javascript:",
        "<form",
        "formaction",
        "<svg",
        "<script",
        "<iframe",
        "style=",
    ] {
        assert!(!html.contains(banned), "{banned} in {html}");
    }
    assert!(html.contains("id=\"user-content-main\""), "{html}");
}

#[test]
fn raises_release_headings_and_drops_the_title_on_the_tree() {
    let html = render_untrusted_markdown(
        "# v1 [a](x \"</h1><img src=x onerror=alert(1)>\")\n\n## Features\n\n### Fix",
    );
    assert!(!html.contains("v1") && !html.contains("onerror"), "{html}");
    assert!(
        html.contains("<h1>Features</h1>") && html.contains("<h2>Fix</h2>"),
        "{html}"
    );
}
