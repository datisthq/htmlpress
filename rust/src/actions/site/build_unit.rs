use super::*;

fn write_source(root: &Path, path: &str, text: &str) -> crate::Result<()> {
    write_file(&root.join(path), text.as_bytes())
}

#[test]
fn builds_pages_files_and_assets() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(root, "guides/build.html", "<h1>Build</h1>")?;
    write_source(root, "images/logo.svg", "<svg></svg>")?;
    let options = BuildOptions::new(root);
    write_source(&options.output, "stale.html", "old")?;

    let report = build_site(&options)?;
    assert_eq!((report.pages, report.files), (2, 1));
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();
    assert!(read("index.html").is_some_and(|html| html.contains("<h1>Home</h1>")));
    assert!(read("guides/build.html").is_some_and(|html| html.contains("theme.css")));
    assert_eq!(read("images/logo.svg").as_deref(), Some("<svg></svg>"));
    assert!(read("_htmlpress/theme.css").is_some());
    assert!(read("_htmlpress/reload.js").is_none());
    assert!(read("stale.html").is_none());
    Ok(())
}

#[test]
fn serves_escapes_and_project_components() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(root, ".htmlpress/styles/tokens.css", ":root { --mine: 1 }")?;
    write_source(root, ".htmlpress/components/htmlpress-tabs.js", "// mine")?;
    write_source(
        root,
        ".htmlpress/components/site-footer.html",
        "<footer></footer>",
    )?;
    write_source(root, ".htmlpress/components/site-footer.css", "footer {}")?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();
    let theme = read("_htmlpress/theme.css").unwrap_or_default();
    assert!(theme.starts_with(":root { --mine: 1 }") && theme.contains("h1"));
    assert_eq!(
        read("_htmlpress/components/htmlpress-tabs.js").as_deref(),
        Some("// mine")
    );
    assert_eq!(
        read("_htmlpress/components/site-footer.css").as_deref(),
        Some("footer {}")
    );
    assert!(read("_htmlpress/components/htmlpress-callout.css").is_some());
    assert!(read("_htmlpress/components/site-footer.html").is_none());
    assert!(read("_htmlpress/layout.html").is_none());
    Ok(())
}

#[test]
fn refuses_an_output_that_holds_the_project() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let project = root.path().join("docs");
    write_source(&project, "index.html", "<h1>Home</h1>")?;
    for output in [root.path().to_owned(), project.clone(), project.join("out")] {
        let options = BuildOptions {
            output,
            ..BuildOptions::new(&project)
        };
        assert!(matches!(build_site(&options), Err(Error::Usage(_))));
    }
    assert!(project.join("index.html").is_file());
    Ok(())
}

#[test]
fn builds_into_a_sibling_folder() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let project = root.path().join("docs");
    write_source(&project, "index.html", "<h1>Home</h1>")?;
    let output = root.path().join("site");
    let report = build_site(&BuildOptions {
        output,
        ..BuildOptions::new(&project)
    })?;
    assert_eq!(report.pages, 1);
    Ok(())
}
