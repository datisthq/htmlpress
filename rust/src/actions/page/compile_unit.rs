use super::compile_markdown;
use crate::actions::page::read::read_page;
use crate::models::page::Page;

fn compile(markdown: &str) -> Result<Page, String> {
    let components = ["callout".to_owned(), "badge".to_owned()];
    let html = compile_markdown(markdown, &components, true)?;
    read_page("guide.html", &html).map_err(|error| error.to_string())
}

#[test]
fn reads_every_frontmatter_key_as_the_page_metas() -> Result<(), String> {
    let page = compile(
        "---\ntitle: A & B\ndescription: About it.\norder: 2\nsidebar: false\n\
         author: [Ann, Bo]\nkeywords: [a, b]\nlabel: Short\nicon: rocket\n\
         date: 2026-10-31\nlayout: wide\nhead: <link rel=\"me\" href=\"/x\">\n---\n\n# Heading\n\nBody.",
    )?;
    assert_eq!(page.title, "A & B");
    assert_eq!(page.description.as_deref(), Some("About it."));
    assert_eq!(page.order, Some(2));
    assert!(!page.sidebar);
    assert_eq!(page.authors, ["Ann", "Bo"]);
    assert_eq!(page.tags, ["a", "b"]);
    assert_eq!(page.label.as_deref(), Some("Short"));
    assert_eq!(page.icon.as_deref(), Some("rocket"));
    assert_eq!(page.date.as_deref(), Some("2026-10-31"));
    assert_eq!(page.layout.as_deref(), Some("wide"));
    assert!(
        page.head.contains("<link rel=\"me\" href=\"/x\">"),
        "{}",
        page.head
    );
    assert!(page.content.contains("<p>Body.</p>"), "{}", page.content);
    Ok(())
}

#[test]
fn fails_on_an_unknown_frontmatter_key() {
    let error = compile("---\ntittle: A\n---\n").err().unwrap_or_default();
    assert!(error.contains("unknown key \"tittle\""), "{error}");
}

#[test]
fn builds_empty_and_frontmatter_only_pages() -> Result<(), String> {
    assert_eq!(compile("")?.title, "guide");
    assert_eq!(compile("---\ntitle: Only\n---\n")?.title, "Only");
    Ok(())
}

#[test]
fn turns_directives_into_components() -> Result<(), String> {
    let page =
        compile(":::callout{type=\"warning\"}\nCareful.\n:::\n\nNew :badge[beta].")?;
    assert!(
        page.content.contains(
            "<htmlpress-callout type=\"warning\"><p>Careful.</p></htmlpress-callout>"
        ),
        "{}",
        page.content
    );
    assert!(
        page.content
            .contains("<htmlpress-badge>beta</htmlpress-badge>")
    );
    let error = compile(":::nope\nx\n:::").err().unwrap_or_default();
    assert!(error.contains(":::nope"), "{error}");
    let prose = compile("See note:important at 10:30.")?;
    assert!(
        prose.content.contains("See note:important at 10:30."),
        "{}",
        prose.content
    );
    Ok(())
}

#[test]
fn renders_math_and_fence_meta_as_html_pages_write_them() -> Result<(), String> {
    let page = compile(
        "Inline $a_1$.\n\n$$\n\\frac{a}{b}\n$$\n\n```js title=\"a.js\" lines=\"2\" line-numbers\nlet a\n```",
    )?;
    assert!(
        page.content.contains("<htmlpress-math data-latex=\"a_1\">"),
        "{}",
        page.content
    );
    assert!(
        page.content.contains("<htmlpress-math display=\"\""),
        "{}",
        page.content
    );
    assert!(
        page.content.contains("<pre data-title=\"a.js\">"),
        "{}",
        page.content
    );
    assert!(
        page.content
            .contains("<code class=\"language-js\" data-lines=\"2\" data-line-numbers"),
        "{}",
        page.content
    );
    Ok(())
}

#[test]
fn keeps_raw_html_components_with_markdown_inside() -> Result<(), String> {
    let page = compile("<htmlpress-callout>\n\n**Bold**\n\n</htmlpress-callout>")?;
    assert!(
        page.content.contains("<strong>Bold</strong>"),
        "{}",
        page.content
    );
    assert!(
        page.content.contains("<htmlpress-callout>"),
        "{}",
        page.content
    );
    Ok(())
}
