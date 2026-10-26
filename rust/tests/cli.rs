use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;
type TestResult = Result<(), Error>;

fn run_cli(args: &[&str]) -> Result<(u8, String), Error> {
    let mut out = vec![];
    let mut err = vec![];
    let argv = std::iter::once("htmlpress").chain(args.iter().copied());
    let program = htmlpress::create_program();
    let code = pollster::block_on(program.try_run_from(argv, &mut out, &mut err))?;
    Ok((code, String::from_utf8(out)?))
}

fn write_page(root: &Path, path: &str, html: &str) -> TestResult {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, html)?;
    Ok(())
}

#[test]
fn builds_a_project_as_json() -> TestResult {
    let root = tempfile::tempdir()?;
    write_page(root.path(), "index.html", "<h1>Home</h1>")?;
    write_page(
        root.path(),
        "guides/build.html",
        "<title>Build</title><p>Run it.</p>",
    )?;
    let path = root.path().to_string_lossy().into_owned();

    let (code, out) = run_cli(&["build", &path, "--json"])?;
    assert_eq!(code, 0);
    let report: serde_json::Value = serde_json::from_str(&out)?;
    assert_eq!(report["pages"], 2);
    assert_eq!(report["files"], 0);
    let page =
        std::fs::read_to_string(root.path().join(".htmlpress/build/guides/build.html"))?;
    assert!(page.contains("<p>Run it.</p>"));
    Ok(())
}

#[test]
fn lists_every_command_for_agents() -> TestResult {
    let (code, out) = run_cli(&["--llms"])?;
    assert_eq!(code, 0);
    let manifest: serde_json::Value = serde_json::from_str(&out)?;
    let names: Vec<&str> = manifest["tools"]
        .as_array()
        .map(|tools| {
            tools
                .iter()
                .filter_map(|tool| tool["name"].as_str())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        ["build", "start", "preview", "clear", "escape"]
            .iter()
            .all(|name| names.contains(name))
    );
    Ok(())
}

#[test]
fn escapes_a_component_and_builds_with_it() -> TestResult {
    let root = tempfile::tempdir()?;
    write_page(
        root.path(),
        "index.html",
        "<htmlpress-badge>New</htmlpress-badge>",
    )?;
    let path = root.path().to_string_lossy().into_owned();

    let (code, out) = run_cli(&["escape", "htmlpress-badge", "--root", &path, "--json"])?;
    assert_eq!(code, 0);
    let report: serde_json::Value = serde_json::from_str(&out)?;
    assert_eq!(
        report["escaped"][0],
        ".htmlpress/components/badge/badge.css"
    );
    let escaped = root.path().join(".htmlpress/components/badge/badge.css");
    std::fs::write(&escaped, "/* mine */")?;

    let (code, _) = run_cli(&["build", &path])?;
    assert_eq!(code, 0);
    let output = root.path().join(".htmlpress/build/_htmlpress/components");
    assert_eq!(
        std::fs::read_to_string(output.join("badge/badge.css"))?,
        "/* mine */"
    );
    let again = run_cli(&["escape", "htmlpress-badge", "--root", &path]);
    assert!(again.is_err_and(|error| error.to_string().contains("already escaped")));
    Ok(())
}

#[test]
fn clears_the_build_as_json() -> TestResult {
    let root = tempfile::tempdir()?;
    write_page(root.path(), "index.html", "<h1>Home</h1>")?;
    let path = root.path().to_string_lossy().into_owned();
    run_cli(&["build", &path])?;
    let (code, out) = run_cli(&["clear", &path, "--json"])?;
    assert_eq!(code, 0);
    let clear: serde_json::Value = serde_json::from_str(&out)?;
    assert_eq!(clear["removed"], true);
    assert!(!root.path().join(".htmlpress/build").exists());
    Ok(())
}

#[test]
fn builds_every_language_with_its_picker_and_fallbacks() -> TestResult {
    let root = tempfile::tempdir()?;
    write_page(
        root.path(),
        "htmlpress.yaml",
        concat!(
            "site: https://docs.example.com\n",
            "title: { en: Docs, es: Documentos }\n",
            "languages:\n",
            "  - { code: en, label: English }\n",
            "  - { code: es, label: Español }\n",
            "  - { code: zh, label: 简体中文 }\n",
            "sections:\n",
            "  - { title: { en: Blog, es: Bitácora }, type: blog, prefix: /blog/ }\n",
        ),
    )?;
    write_page(root.path(), "index.html", "<h1>Home</h1>")?;
    write_page(root.path(), "index.es.html", "<h1>Inicio</h1>")?;
    write_page(
        root.path(),
        "guide.html",
        "<title>Guide</title><h2>Setup</h2>",
    )?;
    write_page(
        root.path(),
        "guide.es.html",
        "<title>Guía</title><h2>Instalación</h2>",
    )?;
    write_page(root.path(), "install.html", "<title>Install</title>")?;
    write_page(
        root.path(),
        "zh/start.html",
        r#"<meta name="language" content="zh"><title>开始</title>"#,
    )?;
    write_page(
        root.path(),
        "blog/hello.es.html",
        r#"<meta name="date" content="2026-03-01"><title>Hola</title>"#,
    )?;
    let path = root.path().to_string_lossy().into_owned();
    let (code, _) = run_cli(&["build", &path])?;
    assert_eq!(code, 0);
    let build = root.path().join(".htmlpress/build");
    let read = |path: &str| std::fs::read_to_string(build.join(path));

    let guide = read("es/guide.html")?;
    assert!(guide.contains(r#"<html lang="es">"#));
    assert!(guide.contains("<title>Guía · Documentos</title>"));
    assert!(guide.contains(r#"<link rel="alternate" hreflang="en" href="https://docs.example.com/guide.html">"#));
    assert!(guide.contains(r#"<link rel="alternate" hreflang="x-default" href="https://docs.example.com/guide.html">"#));
    assert!(guide.contains(r#"<a href="/guide.html" hreflang="en" lang="en">"#));
    assert!(guide.contains(r#"<a href="/zh/" hreflang="zh" lang="zh">"#));
    assert!(guide.contains(r#"<a href="/install.html" hreflang="en" title="Aún sin traducir" data-untranslated>Install<small>en</small></a>"#));
    assert!(!guide.contains(r#"href="/zh/start.html""#));
    assert!(read("es/install.html").is_err());
    assert!(read("guide.html")?.contains("<title>Guide · Docs</title>"));

    assert!(
        read("zh/index.html")?
            .contains(r#"<meta http-equiv="refresh" content="0; url=/zh/start.html">"#)
    );
    assert!(read("zh/start.html")?.contains(r#"<html lang="zh">"#));
    assert!(read("_htmlpress/search.es.json")?.contains("Instalación"));
    assert!(!read("_htmlpress/search.json")?.contains("Instalación"));
    assert!(read("es/llms.txt")?.contains("# Documentos"));
    assert!(read("es/guide.md").is_ok());

    let blog = read("es/blog/index.html")?;
    assert!(blog.contains("<h1>Bitácora</h1>"));
    assert!(blog.contains("1 mar 2026"));
    assert!(read("es/blog/rss.xml")?.contains("<title>Hola</title>"));
    assert!(read("blog/index.html")?.contains("No posts yet."));
    write_page(
        root.path(),
        "blog/news.html",
        r#"<meta name="date" content="2026-04-01"><title>News</title>"#,
    )?;
    let (code, _) = run_cli(&["build", &path])?;
    assert_eq!(code, 0);
    assert!(read("es/blog/index.html")?.contains(r#"<a href="/blog/news.html">"#));
    assert!(
        read("es/blog/rss.xml")?
            .contains("<link>https://docs.example.com/blog/news.html</link>")
    );
    Ok(())
}
