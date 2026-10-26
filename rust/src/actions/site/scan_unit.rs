use super::*;

fn write_file(root: &Path, path: &str, text: &str) -> crate::Result<()> {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(Error::io(parent))?;
    }
    std::fs::write(&file, text).map_err(Error::io(&file))
}

#[test]
fn orders_pages_index_first_then_by_order_then_path() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(root, "b.html", "<h1>B</h1>")?;
    write_file(root, "a.html", "<h1>A</h1>")?;
    write_file(
        root,
        "z.html",
        r#"<head><meta name="order" content="1"></head><h1>Z</h1>"#,
    )?;
    write_file(root, "index.html", "<h1>Home</h1>")?;
    let site = scan_site(root)?;
    let titles: Vec<&str> = site.pages.iter().map(|page| page.title.as_str()).collect();
    assert_eq!(titles, ["Home", "Z", "A", "B"]);
    Ok(())
}

#[test]
fn skips_dot_folders_and_the_config_and_keeps_other_files() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(root, "index.html", "<h1>Home</h1>")?;
    write_file(root, "images/logo.svg", "<svg></svg>")?;
    write_file(root, CONFIG_FILE, "title: Docs\n")?;
    write_file(root, ".htmlpress/build/index.html", "<h1>Old</h1>")?;
    write_file(root, "node_modules/x/index.html", "<h1>Dep</h1>")?;
    let site = scan_site(root)?;
    assert_eq!(site.title, "Docs");
    assert_eq!(site.pages.len(), 1);
    assert_eq!(site.files.keys().collect::<Vec<_>>(), ["images/logo.svg"]);
    assert_eq!(site.assets.len(), ASSETS.len());
    Ok(())
}

#[test]
fn lays_the_projects_assets_over_the_built_in_ones() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    let layout = "<main><htmlpress-content></htmlpress-content></main>";
    write_file(root, ".htmlpress/layout.html", layout)?;
    write_file(
        root,
        ".htmlpress/components/htmlpress-tabs.css",
        "/* mine */",
    )?;
    write_file(
        root,
        ".htmlpress/components/site-footer.html",
        "<footer></footer>",
    )?;
    write_file(root, ".htmlpress/notes.md", "kept out")?;
    let site = scan_site(root)?;
    assert_eq!(site.assets["layout.html"], layout);
    assert_eq!(site.assets["components/htmlpress-tabs.css"], "/* mine */");
    assert_eq!(
        site.assets["components/site-footer.html"],
        "<footer></footer>"
    );
    assert!(site.assets.contains_key("components/htmlpress-tabs.js"));
    assert_eq!(site.assets.len(), ASSETS.len() + 1);
    Ok(())
}

#[test]
fn rejects_an_asset_that_replaces_nothing() -> crate::Result<()> {
    for (path, needle) in [
        (
            ".htmlpress/styles/tokns.css",
            "no built-in styles/tokns.css",
        ),
        (
            ".htmlpress/components/htmlpress-calout.css",
            "no built-in htmlpress-calout",
        ),
        (".htmlpress/components/footer.html", "custom element name"),
        (
            ".htmlpress/components/site-footer.txt",
            ".html, .css or .js",
        ),
        (
            ".htmlpress/layouts/Landing.html",
            "a layout is a .html file",
        ),
        (".htmlpress/layouts/landing.css", "a layout is a .html file"),
    ] {
        let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
        write_file(root.path(), path, "x")?;
        let error = scan_site(root.path()).err().map(|error| error.to_string());
        assert!(
            error
                .as_deref()
                .is_some_and(|message| message.contains(needle)),
            "{path}: {error:?}"
        );
    }
    Ok(())
}

#[test]
fn reads_the_projects_layouts() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(
        root,
        "index.html",
        r#"<meta name="layout" content="landing">"#,
    )?;
    write_file(root, ".htmlpress/layouts/landing.html", "<main></main>")?;
    let site = scan_site(root)?;
    assert_eq!(site.assets["layouts/landing.html"], "<main></main>");
    Ok(())
}

#[test]
fn rejects_a_page_whose_layout_is_missing() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    write_file(
        root.path(),
        "index.html",
        r#"<meta name="layout" content="landing">"#,
    )?;
    let error = scan_site(root.path()).err().map(|error| error.to_string());
    assert!(
        error.as_deref().is_some_and(|message| {
            message.contains("index.html") && message.contains("layouts/landing.html")
        }),
        "{error:?}"
    );
    Ok(())
}

#[test]
fn registers_every_built_in_asset_file() -> crate::Result<()> {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets");
    let mut paths = vec!["layout.html".to_owned()];
    for folder in ["styles", "scripts", "components"] {
        let entries =
            std::fs::read_dir(assets.join(folder)).map_err(Error::io(folder))?;
        for entry in entries {
            let entry = entry.map_err(Error::io(folder))?;
            paths.push(format!("{folder}/{}", entry.file_name().to_string_lossy()));
        }
    }
    paths.sort();
    let mut registered: Vec<_> =
        ASSETS.iter().map(|asset| asset.path.to_owned()).collect();
    registered.sort();
    assert_eq!(paths, registered);
    Ok(())
}
