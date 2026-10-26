use super::*;

#[test]
fn colors_known_languages_in_both_themes() -> Result<(), String> {
    let html = "<pre><code class=\"language-rust\">let x = &quot;&lt;a&gt;&quot;;\n</code></pre>";
    let colored = highlight_code(html, "catppuccin-latte", "catppuccin-mocha")?;
    assert!(
        colored
            .starts_with("<pre><code class=\"language-rust\"><span class=\"giallo-l\">"),
        "{colored}"
    );
    assert!(colored.contains("light-dark(#8839EF, #CBA6F7)"));
    assert!(colored.contains("&lt;a&gt;"));
    assert!(colored.ends_with("</code></pre>"));
    Ok(())
}

#[test]
fn leaves_plain_unknown_and_marked_up_blocks_alone() -> Result<(), String> {
    for html in [
        "<pre><code>plain</code></pre>",
        "<pre><code class=\"language-nope\">a &amp; b</code></pre>",
        "<pre><code class=\"language-rust\">let <mark>x</mark> = 1;</code></pre>",
        "<p><code class=\"language-rust\">inline</code></p>",
    ] {
        assert_eq!(
            highlight_code(html, "catppuccin-latte", "catppuccin-mocha")?,
            html
        );
    }
    assert!(has_theme("catppuccin-mocha") && !has_theme("no-such-theme"));
    Ok(())
}

#[test]
fn highlights_and_numbers_chosen_lines() -> Result<(), String> {
    let html = "<pre><code class=\"language-rust\" data-lines=\"2\" data-line-numbers=\"10\">let a = 1;\nlet b = 2;</code></pre>";
    let colored = highlight_code(html, "catppuccin-latte", "catppuccin-mocha")?;
    assert!(colored.contains("class=\"giallo-ln\""), "{colored}");
    assert!(colored.contains(">10</span>") && colored.contains(">11</span>"));
    assert!(
        colored.contains("background-color: light-dark("),
        "{colored}"
    );
    let bad = "<pre><code class=\"language-rust\" data-lines=\"3-1\">a</code></pre>";
    assert!(highlight_code(bad, "catppuccin-latte", "catppuccin-mocha").is_err());
    Ok(())
}
