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

fn assert_contains(page: &Page, parts: &[&str]) {
    for part in parts {
        assert!(
            page.content.contains(part),
            "missing {part}\nin {}",
            page.content
        );
    }
}

#[test]
fn reads_livemark_callouts_and_details() -> Result<(), String> {
    let page = compile(
        ":::note\nA note.\n:::\n\n:::warning[Breaking Change]\nGone.\n:::\n\n\
         > [!TIP]\n> GitHub syntax.\n\n:::details[Click to expand]\nHidden **md**.\n:::",
    )?;
    assert_contains(
        &page,
        &[
            r#"<htmlpress-callout type="note"><p>A note.</p></htmlpress-callout>"#,
            r#"<htmlpress-callout type="warning" title="Breaking Change">"#,
            r#"<htmlpress-callout type="tip"><p>GitHub syntax.</p></htmlpress-callout>"#,
            "<details><summary>Click to expand</summary><p>Hidden <strong>md</strong>.</p>",
        ],
    );
    Ok(())
}

#[test]
fn groups_livemark_cards_columns_and_tabs() -> Result<(), String> {
    let page = compile(
        ":::card{title=\"Go\" href=\"/go/\" icon=\"rocket\"}\nLearn.\n:::\n\n\
         :::card{title=\"Plain\"}\nTwo.\n:::\n\n:::column\nLeft\n:::\n\n:::column\nRight\n:::\n\n\
         :::tab{title=\"npm\" sync=\"pm\"}\nUse npm.\n:::\n\n:::tab{title=\"pip\"}\nUse pip.\n:::",
    )?;
    assert_contains(
        &page,
        &[
            r#"<htmlpress-cards><htmlpress-card><h3 id="go"><a href="/go/"><htmlpress-icon name="rocket">"#,
            r#"<htmlpress-card><h3 id="plain">Plain</h3><p>Two.</p></htmlpress-card></htmlpress-cards>"#,
            "<htmlpress-columns><htmlpress-column><p>Left</p></htmlpress-column>",
            r#"<htmlpress-tabs sync="pm"><htmlpress-tab title="npm" sync="pm"><p>Use npm.</p>"#,
        ],
    );
    Ok(())
}

#[test]
fn reads_livemark_inline_and_leaf_directives() -> Result<(), String> {
    let page = compile(
        ":badge[Beta] :badge[Stable]{variant=\"positive\"} \
         :abbr[HTML]{title=\"HyperText Markup Language\"}\n\n\
         ::button[Start]{href=\"/intro/\" variant=\"outline\" size=\"lg\"}\n\n\
         ::toc{maxLevel=\"2\"}\n\n## One\n\n\
         ::video{type=\"youtube\" url=\"https://youtu.be/x\"}\n\n\
         ::audio{type=\"soundcloud\" url=\"https://soundcloud.com/a/b\"}",
    )?;
    assert_contains(
        &page,
        &[
            r#"<htmlpress-badge>Beta</htmlpress-badge> <htmlpress-badge variant="positive">"#,
            r#"<abbr title="HyperText Markup Language">HTML</abbr>"#,
            r#"<htmlpress-button variant="outline" size="lg"><a href="/intro/">Start</a>"#,
            r#"<htmlpress-toc max-level="2"></htmlpress-toc>"#,
            r#"<htmlpress-video type="youtube" url="https://youtu.be/x"><a href="https://youtu.be/x">"#,
            r#"<htmlpress-audio type="soundcloud" url="https://soundcloud.com/a/b"><a "#,
        ],
    );
    Ok(())
}

#[test]
fn reads_livemark_heading_markers_and_definition_lists() -> Result<(), String> {
    let page = compile(
        "## Custom {#custom-id}\n\n### Install [step]\n\n## Hidden [!toc]\n\n\
         ## Only [toc]\n\nTerm\n: Its definition.",
    )?;
    assert_contains(
        &page,
        &[
            r#"<h2 id="custom-id">Custom</h2>"#,
            r#"<h3 data-step="" id="install">Install</h3>"#,
            r#"data-toc="false""#,
            r#"data-toc="only""#,
            "<dt>Term</dt>",
        ],
    );
    let ids: Vec<_> = page.toc.iter().map(|heading| heading.id.as_str()).collect();
    assert_eq!(ids, ["custom-id", "only"]);
    Ok(())
}

#[test]
fn reads_livemark_code_tabs_package_commands_and_inline_code() -> Result<(), String> {
    let page = compile(
        "```js tab=\"JavaScript\" sync=\"lang\"\nconsole.log(1)\n```\n\n\
         ```python tab=\"Python\"\nprint(1)\n```\n\n```npm\nnpm install livemark\n```\n\n\
         ```mermaid\nflowchart LR\n  a --> b\n```\n\nUse `const x = 1{:ts}` and `a{b}`.\n\n\
         ![Dark](https://x.dev/a.png#dark)",
    )?;
    assert_contains(
        &page,
        &[
            r#"<htmlpress-tabs sync="lang"><htmlpress-tab title="JavaScript"><pre"#,
            r#"<htmlpress-tab title="Python"><pre"#,
            r#"<htmlpress-tabs sync="package-manager"><htmlpress-tab title="npm">"#,
            "<htmlpress-mermaid><pre>flowchart LR",
            r#"<code class="language-ts"#,
            "<code>a{b}</code>",
            r#"<img src="https://x.dev/a.png" alt="Dark" data-theme="dark">"#,
        ],
    );
    assert!(!page.content.contains("{:ts}"), "{}", page.content);
    Ok(())
}
