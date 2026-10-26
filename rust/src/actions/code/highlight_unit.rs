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
        "<p><code>plain inline</code></p>",
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

#[test]
fn marks_diff_severity_focus_and_words() -> Result<(), String> {
    let html = "<pre><code class=\"language-js\" data-add=\"1\" data-remove=\"2\" data-error=\"3\" \
                data-warning=\"4\" data-focus=\"1-2\" data-words=\"greet,&lt;b&gt;\">\
                const greet = 1\nconst b = greet\nlet c = \"&lt;b&gt;\"\nlet d = 4</code></pre>";
    let colored = highlight_code(html, "catppuccin-latte", "catppuccin-mocha")?;
    let lines: Vec<&str> = colored.split("<span class=\"giallo-l\"").skip(1).collect();
    assert_eq!(lines.len(), 4, "{colored}");
    assert!(lines[0].starts_with(" data-diff=\"add\">"), "{}", lines[0]);
    assert!(lines[1].starts_with(" data-diff=\"remove\">"));
    assert!(
        lines[2].starts_with(" data-severity=\"error\" data-dim>"),
        "{}",
        lines[2]
    );
    assert!(lines[3].starts_with(" data-severity=\"warning\" data-dim>"));
    assert!(
        colored.matches("<mark>greet</mark>").count() == 2,
        "{colored}"
    );
    assert!(
        colored.contains("<mark>&lt;b&gt;</mark>") || colored.contains("&lt;"),
        "{colored}"
    );
    assert!(!colored.contains("data-words=\"<mark>"));
    let bad = "<pre><code class=\"language-js\" data-add=\"x\">a</code></pre>";
    let error = highlight_code(bad, "catppuccin-latte", "catppuccin-mocha")
        .err()
        .unwrap_or_default();
    assert!(error.contains("data-add"), "{error}");
    Ok(())
}

#[test]
fn colors_inline_code_that_names_a_language() -> Result<(), String> {
    let html = "<p>Run <code class=\"language-js\">const a = 1</code> now.</p>";
    let colored = highlight_code(html, "catppuccin-latte", "catppuccin-mocha")?;
    assert!(
        colored
            .starts_with("<p>Run <code class=\"language-js\"><span class=\"giallo-l\">"),
        "{colored}"
    );
    assert!(
        colored.contains("light-dark(#8839EF, #CBA6F7)")
            && colored.ends_with("</code> now.</p>")
    );
    Ok(())
}
