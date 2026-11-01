use textlane::Allocator;

use super::list_includes;
use crate::actions::node::parse::parse_fragment;
use crate::actions::page::compile::compile_markdown;
use crate::models::pipeline::MarkdownOptions;

#[test]
fn resolves_each_src_from_the_page_folder_or_the_root() {
    let alloc = Allocator::new();
    let html = "<htmlpress-include src=\"parts/a.md\"></htmlpress-include>\
                <htmlpress-include src=\"../b.md\"></htmlpress-include>\
                <htmlpress-include src=\"/c.md\"></htmlpress-include>";
    assert_eq!(
        list_includes("docs/guide.html", &parse_fragment(html, &alloc)),
        ["docs/parts/a.md", "b.md", "c.md"]
    );
    assert!(list_includes("a.html", &parse_fragment("<p>none</p>", &alloc)).is_empty());
}

#[test]
fn reads_markdown_include_directives_outside_fences() -> Result<(), String> {
    let alloc = Allocator::new();
    let markdown = "```md\n::include{file=\"./shown.md\"}\n```\n\n\
                    ::include{file=\"./parts/a.md\"}\n\n<htmlpress-include src=\"b.md\"></htmlpress-include>";
    let compiled = compile_markdown(markdown, &MarkdownOptions::default(), &alloc)?;
    assert_eq!(
        list_includes("docs/guide.md", &compiled.body),
        ["docs/parts/a.md", "docs/b.md"]
    );
    Ok(())
}
