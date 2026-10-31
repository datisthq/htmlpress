use super::{has_theme, highlight_code};

const THEMES: (&str, &str) = ("catppuccin-latte", "catppuccin-mocha");

#[test]
fn colors_known_languages_in_both_themes() -> Result<(), String> {
    let html = "<pre data-title=\"a.rs\"><code class=\"language-rust\">let x = &quot;&lt;a&gt;&quot;;\n</code></pre>";
    let colored = highlight_code(html, THEMES.0, THEMES.1)?;
    assert!(
        colored.starts_with("<pre class=\"shiki shiki-themes"),
        "{colored}"
    );
    assert!(colored.contains("data-title=\"a.rs\""), "{colored}");
    assert!(colored.contains("--shiki-dark:"), "{colored}");
    assert!(colored.contains("\"&lt;a>\""), "{colored}");
    Ok(())
}

#[test]
fn marks_lines_words_focus_and_numbers() -> Result<(), String> {
    let html = "<pre><code class=\"language-rust\" data-lines=\"1\" data-add=\"2\" \
                data-remove=\"3\" data-error=\"4\" data-warning=\"5\" data-focus=\"1\" \
                data-line-numbers=\"10\">let a = 1;\nlet b = 2;\nc\nd\ne</code></pre>\
                <pre><code class=\"language-rust\" data-words=\"b\">let b = 2;</code></pre>";
    let colored = highlight_code(html, THEMES.0, THEMES.1)?;
    for class in [
        "line focused highlighted",
        "line diff add",
        "line diff remove",
        "line severity error",
        "line severity warning",
        "has-focused",
        "highlighted-word",
    ] {
        assert!(colored.contains(class), "{class} in {colored}");
    }
    assert!(
        colored.contains("<span class=\"line-number\">10</span>"),
        "{colored}"
    );
    assert!(
        colored.contains("<span class=\"line-number\">14</span>"),
        "{colored}"
    );
    Ok(())
}

#[test]
fn colors_ansi_and_inline_code_but_leaves_marked_up_blocks() -> Result<(), String> {
    let ansi = highlight_code(
        "<pre><code class=\"language-ansi\">\u{1b}[1;32mok\u{1b}[0m</code></pre>",
        THEMES.0,
        THEMES.1,
    )?;
    assert!(ansi.contains("ok") && !ansi.contains('\u{1b}'), "{ansi}");
    let inline = highlight_code(
        "<p><code class=\"language-js\">let a</code></p>",
        THEMES.0,
        THEMES.1,
    )?;
    assert!(inline.starts_with("<p><span class=\"shiki"), "{inline}");
    let marked =
        "<pre><code class=\"language-rust\">let <mark>x</mark> = 1;</code></pre>";
    assert_eq!(highlight_code(marked, THEMES.0, THEMES.1)?, marked);
    Ok(())
}

#[test]
fn knows_the_built_in_themes() {
    assert!(has_theme("catppuccin-mocha") && !has_theme("no-such-theme"));
}
