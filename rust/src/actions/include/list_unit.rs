use super::list_includes;

#[test]
fn resolves_each_src_from_the_page_folder_or_the_root() -> Result<(), String> {
    let html = "<htmlpress-include src=\"parts/a.md\"></htmlpress-include>\
                <htmlpress-include src=\"../b.md\"></htmlpress-include>\
                <htmlpress-include src=\"/c.md\"></htmlpress-include>";
    assert_eq!(
        list_includes("docs/guide.html", html)?,
        ["docs/parts/a.md", "b.md", "c.md"]
    );
    assert!(list_includes("a.html", "<p>none</p>")?.is_empty());
    Ok(())
}
