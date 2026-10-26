use super::*;

fn create_project(files: &[(&str, &str)]) -> Result<tempfile::TempDir, String> {
    let dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    for (path, text) in files {
        let file = dir.path().join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(file, text).map_err(|error| error.to_string())?;
    }
    Ok(dir)
}

#[test]
fn inlines_html_markdown_and_code_files() -> Result<(), String> {
    let dir = create_project(&[
        (
            "docs/includes/note.html",
            "<!doctype html><html><head><title>x</title></head><body><p>Note</p></body></html>",
        ),
        (
            "docs/includes/list.md",
            "---\ntitle: x\n---\n\n- one\n- two\n",
        ),
        ("examples/server.ts", "const a = 1 < 2\n"),
    ])?;
    let html = "<htmlpress-include src=\"includes/note.html\"></htmlpress-include>\
        <htmlpress-include src=\"includes/list.md\"></htmlpress-include>\
        <htmlpress-include src=\"/examples/server.ts\" data-lines=\"1\" data-title=\"server\"></htmlpress-include>";
    let resolved = resolve_includes(dir.path(), "docs/page.html", html)?;
    assert!(resolved.starts_with("<p>Note</p>"), "{resolved}");
    assert!(resolved.contains("<li>one</li>") && !resolved.contains("title: x"));
    assert!(resolved.contains(
        "<pre data-title=\"server\"><code class=\"language-typescript\" data-lines=\"1\">const a = 1 &lt; 2</code></pre>"
    ), "{resolved}");
    assert!(!resolved.contains("htmlpress-include"));
    Ok(())
}

#[test]
fn nests_and_refuses_cycles_escapes_and_missing_files() -> Result<(), String> {
    let dir = create_project(&[
        (
            "a.html",
            "<p>A</p><htmlpress-include src=\"b.html\"></htmlpress-include>",
        ),
        ("b.html", "<p>B</p>"),
        (
            "loop.html",
            "<htmlpress-include src=\"loop.html\"></htmlpress-include>",
        ),
        (".env", "TOKEN=1"),
    ])?;
    let nested = resolve_includes(
        dir.path(),
        "page.html",
        "<htmlpress-include src=\"a.html\"></htmlpress-include>",
    )?;
    assert_eq!(nested, "<p>A</p><p>B</p>");
    for src in ["loop.html", "missing.html", "../outside.html", ".env", ""] {
        let html = format!("<htmlpress-include src=\"{src}\"></htmlpress-include>");
        assert!(
            resolve_includes(dir.path(), "page.html", &html).is_err(),
            "{src}"
        );
    }
    let unclosed = resolve_includes(
        dir.path(),
        "page.html",
        "<htmlpress-include src=\"b.html\"><p>After</p>",
    )?;
    assert!(
        unclosed.contains("<p>B</p>") && unclosed.contains("<p>After</p>"),
        "{unclosed}"
    );
    Ok(())
}

#[test]
fn refuses_includes_that_fan_out_past_the_budget() -> Result<(), String> {
    let ten = |src: &str| {
        format!("<htmlpress-include src=\"{src}\"></htmlpress-include>").repeat(10)
    };
    let dir = create_project(&[
        ("a.html", &ten("b.html")),
        ("b.html", &ten("c.html")),
        ("c.html", &ten("d.html")),
        ("d.html", "<p>leaf</p>"),
    ])?;
    let error = resolve_includes(dir.path(), "page.html", &ten("a.html"))
        .err()
        .unwrap_or_default();
    assert!(error.contains("expand past 1000"), "{error}");
    Ok(())
}
