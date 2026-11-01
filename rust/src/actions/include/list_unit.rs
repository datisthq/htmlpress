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

#[test]
fn reads_markdown_include_directives_outside_fences() -> Result<(), String> {
    let markdown = "```md\n::include{file=\"./shown.md\"}\n```\n\n\
                    ::include{file=\"./parts/a.md\"}\n\n<htmlpress-include src=\"b.md\"></htmlpress-include>";
    assert_eq!(
        list_includes("docs/guide.md", markdown)?,
        ["docs/parts/a.md", "docs/b.md"]
    );
    assert!(list_includes("docs/guide.html", "::include{file=\"a.md\"}")?.is_empty());
    Ok(())
}
