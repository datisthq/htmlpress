use super::*;
use crate::actions::page::read::read_page;

#[test]
fn converts_a_page_with_its_title_and_description() -> crate::Result<()> {
    let page = read_page(
        "guide.html",
        "<title>Guide</title><meta name=\"description\" content=\"How to.\">\
         <h1>Guide</h1><p>Run <code>build</code>.</p><ul><li>One</li><li>Two</li></ul>\
         <pre><code>htmlpress build</code></pre>",
    )?;
    assert_eq!(
        convert_page(&page)?,
        "# Guide\n\n> How to.\n\nRun `build`.\n\n- One\n- Two\n\n```\nhtmlpress build\n```\n"
    );
    Ok(())
}

#[test]
fn reduces_components_to_plain_markdown() -> crate::Result<()> {
    let page = read_page(
        "a.html",
        "<h1>A</h1>\
         <htmlpress-callout type=\"warning\"><p>Careful.</p></htmlpress-callout>\
         <htmlpress-tabs><section title=\"npm\"><p>Use npm.</p></section></htmlpress-tabs>\
         <htmlpress-mermaid><pre>graph LR</pre></htmlpress-mermaid>\
         <htmlpress-terminal><pre><kbd>ls</kbd></pre></htmlpress-terminal>\
         <script>alert(1)</script>",
    )?;
    let markdown = convert_page(&page)?;
    assert!(
        markdown.contains("> **Warning**\n> \n> Careful."),
        "{markdown}"
    );
    assert!(markdown.contains("**npm**\n\nUse npm."), "{markdown}");
    assert!(markdown.contains("```mermaid\ngraph LR\n```"), "{markdown}");
    assert!(markdown.contains("```\n$ ls\n```"), "{markdown}");
    assert!(!markdown.contains("alert") && !markdown.contains("# A\n\n# A"));
    Ok(())
}

#[test]
fn puts_the_markdown_beside_the_page() {
    assert_eq!(create_markdown_path("index.html"), "index.md");
    assert_eq!(
        create_markdown_path("guides/build.html"),
        "guides/build/index.md"
    );
}

#[test]
fn names_a_titled_block_before_its_fence() -> crate::Result<()> {
    let page = read_page(
        "a.html",
        "<h1>A</h1><pre data-title=\"a.yaml\"><code>x: 1</code></pre>",
    )?;
    assert!(convert_page(&page)?.contains("**a.yaml**\n\n```\nx: 1\n```"));
    Ok(())
}

#[test]
fn writes_math_back_as_latex() -> crate::Result<()> {
    let page = read_page(
        "a.html",
        "<h1>A</h1><p>Inline $x_1^2$ here.</p><p>$$\\frac{a}{b}$$</p>",
    )?;
    let markdown = convert_page(&page)?;
    assert!(markdown.contains("Inline $x_1^2$ here."), "{markdown}");
    assert!(markdown.contains("$$\n\\frac{a}{b}\n$$"), "{markdown}");
    Ok(())
}
