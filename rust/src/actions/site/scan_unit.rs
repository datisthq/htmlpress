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
    write_file(root, CONFIG_FILE, "title = \"Docs\"\n")?;
    write_file(root, ".htmlpress/build/index.html", "<h1>Old</h1>")?;
    write_file(root, "node_modules/x/index.html", "<h1>Dep</h1>")?;
    let site = scan_site(root)?;
    assert_eq!(site.title, "Docs");
    assert_eq!(site.pages.len(), 1);
    assert_eq!(site.files, ["images/logo.svg"]);
    assert!(site.layout.is_none());
    Ok(())
}

#[test]
fn reads_the_layout_escape() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    write_file(
        root.path(),
        LAYOUT_ESCAPE,
        "<main><hp-content></hp-content></main>",
    )?;
    let site = scan_site(root.path())?;
    assert_eq!(
        site.layout.as_deref(),
        Some("<main><hp-content></hp-content></main>")
    );
    Ok(())
}
