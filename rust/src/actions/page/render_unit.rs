use std::path::PathBuf;

use super::*;
use crate::actions::page::read::read_page;
use crate::models::config::{Config, Link, Position, Section, SectionKind};
use crate::settings::ASSETS;

fn create_site() -> crate::Result<Site> {
    Ok(Site {
        root: PathBuf::from("."),
        config: Config::default(),
        title: "Docs <&>".to_owned(),
        pages: vec![
            read_page("index.html", "<h1>Home</h1>")?,
            read_page(
                "guide.html",
                "<title>Guide</title><p>Read me.</p><h2>Setup</h2>",
            )?,
            read_page("api/index.html", "<title>API</title><p>Reference.</p>")?,
            read_page("api/cli.html", "<title>CLI</title><p>Flags.</p>")?,
        ],
        files: std::collections::BTreeMap::new(),
        assets: ASSETS
            .iter()
            .map(|asset| (asset.path.to_owned(), asset.content.to_owned()))
            .collect(),
        skills: vec![],
    })
}

#[test]
fn renders_the_page_in_the_layout() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<title>Guide · Docs &lt;&amp;&gt;</title>"));
    assert!(html.contains("<p>Read me.</p>"));
    assert!(html.contains(r#"<a href="/guide.html" aria-current="page">Guide</a>"#));
    assert!(html.contains(r#"<a href="/">Home</a>"#));
    assert!(html.contains(
        r#"<htmlpress-brand><a href="/">Docs &lt;&amp;&gt;</a></htmlpress-brand>"#
    ));
    assert!(!html.contains("<htmlpress-content>") && !html.contains(RELOAD_PATH));
    Ok(())
}

#[test]
fn groups_the_navigation_by_folder() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(
        r#"<details open><summary>API</summary><ul><li><a href="/api/">API</a></li><li><a href="/api/cli.html">CLI</a></li></ul></details>"#
    ));
    Ok(())
}

#[test]
fn links_the_previous_and_next_pages() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"<a rel="prev" href="/"><small>Previous</small>Home</a>"#));
    assert!(html.contains(r#"<a rel="next" href="/api/"><small>Next</small>API</a>"#));
    let first = render_page(&site, &site.pages[0], false)?;
    assert!(!first.contains(r#"rel="prev""#));
    Ok(())
}

#[test]
fn fills_the_toc_and_removes_it_without_headings() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r##"<a href="#setup" data-level="2">Setup</a>"##));
    let home = render_page(&site, &site.pages[0], false)?;
    assert!(!home.contains("<htmlpress-toc"));
    Ok(())
}

#[test]
fn adds_the_links_and_the_edit_link_from_the_config() -> crate::Result<()> {
    let mut site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(!html.contains("<htmlpress-links") && !html.contains("<htmlpress-edit"));
    site.config.links = vec![Link {
        title: "GitHub".to_owned(),
        url: "https://github.com/x".to_owned(),
        position: Position::Header,
        icon: None,
        prefix: None,
    }];
    site.config.edit = Some("https://github.com/x/edit/main/docs/".to_owned());
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"<a href="https://github.com/x">GitHub</a>"#));
    assert!(
        html.contains(r#"<a href="https://github.com/x/edit/main/docs/guide.html">"#)
    );
    Ok(())
}

#[test]
fn adds_the_reload_script_for_the_dev_server() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[0], true)?;
    assert!(html.contains(RELOAD_PATH));
    Ok(())
}

#[test]
fn uses_the_layout_escape() -> crate::Result<()> {
    let mut site = create_site()?;
    let layout = "<main><htmlpress-content></htmlpress-content></main>";
    site.assets
        .insert("layout.html".to_owned(), layout.to_owned());
    let html = render_page(&site, &site.pages[0], false)?;
    assert_eq!(html, "<main><h1>Home</h1></main>");
    Ok(())
}

#[test]
fn uses_the_layout_a_page_names() -> crate::Result<()> {
    let mut site = create_site()?;
    let layout = "<header><htmlpress-brand></htmlpress-brand></header>\
                  <main><htmlpress-content></htmlpress-content></main>";
    site.assets
        .insert("layouts/landing.html".to_owned(), layout.to_owned());
    site.pages[0] = read_page(
        "index.html",
        r#"<meta name="layout" content="landing"><h1>Home</h1>"#,
    )?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert_eq!(
        html,
        r#"<header><htmlpress-brand><a href="/">Docs &lt;&amp;&gt;</a></htmlpress-brand></header><main><h1>Home</h1></main>"#
    );
    let other = render_page(&site, &site.pages[1], false)?;
    assert!(other.contains("<htmlpress-nav>"));
    Ok(())
}

#[test]
fn links_only_the_components_a_page_uses() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages[0] = read_page(
        "index.html",
        "<htmlpress-tabs><section></section></htmlpress-tabs>",
    )?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(
        r#"<link rel="stylesheet" href="/_htmlpress/components/tabs/tabs.css">"#
    ));
    assert!(
        html.contains(
            r#"<script type="module" src="/_htmlpress/components/tabs/tabs.js">"#
        )
    );
    assert!(html.contains(r#"href="/_htmlpress/components/nav/nav.css""#));
    assert!(html.contains(r#"href="/_htmlpress/components/header/header.css""#));
    assert!(html.contains(r#"src="/_htmlpress/scripts/code.js""#));
    assert!(
        !html.contains("htmlpress-callout.css") && !html.contains("htmlpress-zoom.js")
    );
    Ok(())
}

#[test]
fn fills_a_project_component_tag_with_its_html() -> crate::Result<()> {
    let mut site = create_site()?;
    let footer = "<footer><htmlpress-badge>v1</htmlpress-badge></footer>";
    site.assets.insert(
        "components/site-footer/site-footer.html".to_owned(),
        footer.to_owned(),
    );
    site.assets.insert(
        "components/site-footer/site-footer.css".to_owned(),
        "footer {}".to_owned(),
    );
    site.assets.insert(
        "components/brand/brand.html".to_owned(),
        "<b>Mine</b>".to_owned(),
    );
    site.pages[0] = read_page(
        "index.html",
        "<h1>Home</h1><htmlpress-site-footer></htmlpress-site-footer>",
    )?;
    site.assets.insert(
        "components/site-header/site-header.html".to_owned(),
        "<header></header>".to_owned(),
    );
    site.assets.insert(
        "components/site-header/site-header.css".to_owned(),
        "header {}".to_owned(),
    );
    site.pages[0]
        .content
        .push_str("<htmlpress-site-header></htmlpress-site-header>");
    let layout = "<head></head><htmlpress-brand></htmlpress-brand><htmlpress-content></htmlpress-content><htmlpress-site-footer></htmlpress-site-footer>";
    site.assets
        .insert("layout.html".to_owned(), layout.to_owned());
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains("<htmlpress-brand><b>Mine</b></htmlpress-brand><h1>Home</h1>"));
    assert!(html.contains("<htmlpress-site-footer><footer>"));
    assert_eq!(html.matches("<footer>").count(), 2);
    assert!(html.contains("href=\"/_htmlpress/components/site-header/site-header.css\""));
    assert!(html.contains("href=\"/_htmlpress/components/site-footer/site-footer.css\""));
    assert!(html.contains("href=\"/_htmlpress/components/badge/badge.css\""));
    Ok(())
}

#[test]
fn adds_the_favicon_social_tags_and_head_from_the_config() -> crate::Result<()> {
    let mut site = create_site()?;
    site.config.site = Some("https://example.com/".to_owned());
    site.config.favicon = Some("/favicon.svg".to_owned());
    site.config.head.scripts = vec![
        [("src", "/stats.js"), ("defer", "")]
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .into(),
    ];
    site.config.head.meta = vec![
        [("property", "og:type"), ("content", "article")]
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .into(),
    ];
    site.pages[1] = read_page(
        "guide.html",
        "<title>Guide</title><meta name=\"image\" content=\"/cover.png\"><p>Hi.</p>",
    )?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"<link rel="icon" href="/favicon.svg">"#));
    assert!(html.contains(r#"<meta property="og:title" content="Guide">"#));
    assert!(html.contains(
        r#"<meta property="og:url" content="https://example.com/guide.html">"#
    ));
    assert!(html.contains(
        r#"<meta property="og:image" content="https://example.com/cover.png">"#
    ));
    assert!(html.contains(r#"<meta name="twitter:card" content="summary_large_image">"#));
    assert!(html.contains(r#"<script defer="" src="/stats.js"></script>"#));
    assert!(html.contains(r#"<meta content="article" property="og:type">"#));
    assert!(!html.contains(r#"content="website""#));
    assert!(!html.contains(r#"<meta name="image""#));
    let home = render_page(&site, &site.pages[0], false)?;
    assert!(home.contains(r#"<meta name="twitter:card" content="summary">"#));
    Ok(())
}

#[test]
fn fills_the_logo_banner_and_footer() -> crate::Result<()> {
    let mut site = create_site()?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(!html.contains("<htmlpress-banner") && !html.contains("rel=\"icon\""));
    assert!(html.contains(r#"<a href="https://htmlpress.dev">Htmlpress</a>"#));
    site.config.logo = Some("/logo.svg".to_owned());
    site.config.banner = Some("<a href=\"/new\">New</a>".to_owned());
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<a href="/"><img src="/logo.svg" alt="">Docs"#));
    assert!(html.contains(r#"<link rel="icon" href="/logo.svg">"#));
    assert!(
        html.contains(r#"<htmlpress-banner><a href="/new">New</a></htmlpress-banner>"#)
    );
    Ok(())
}

#[test]
fn puts_each_link_in_its_position() -> crate::Result<()> {
    let mut site = create_site()?;
    site.config.links = vec![
        Link {
            title: "GitHub".to_owned(),
            url: "https://github.com/x".to_owned(),
            position: Position::Header,
            icon: None,
            prefix: None,
        },
        Link {
            title: "Chat".to_owned(),
            url: "https://chat.x".to_owned(),
            position: Position::Sidebar,
            icon: None,
            prefix: None,
        },
    ];
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<nav aria-label="Links"><a href="https://github.com/x">"#));
    assert!(html.contains(r#"<ul data-links><li><a href="https://chat.x">Chat</a>"#));
    Ok(())
}

#[test]
fn fills_the_breadcrumbs_from_the_folders() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages
        .push(read_page("api/v1/auth.html", "<title>Auth</title>")?);
    let home = render_page(&site, &site.pages[0], false)?;
    assert!(!home.contains("<htmlpress-breadcrumbs"));
    let auth = render_page(&site, &site.pages[4], false)?;
    assert!(auth.contains(
        r#"<ol><li><a href="/">Docs &lt;&amp;&gt;</a></li><li><a href="/api/">API</a></li><li>v1</li><li aria-current="page">Auth</li></ol>"#
    ));
    let index = render_page(&site, &site.pages[2], false)?;
    assert!(index.contains(
        r#"<li><a href="/">Docs &lt;&amp;&gt;</a></li><li aria-current="page">API</li>"#
    ));
    Ok(())
}

#[test]
fn links_the_markdown_from_the_head_and_the_toolbar() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[3], false)?;
    assert!(
        html.contains(
            r#"<link rel="alternate" type="text/markdown" href="/api/cli.md">"#
        )
    );
    assert!(html.contains(r#"<htmlpress-toolbar><a href="/api/cli.md" type="text/markdown">View as Markdown</a></htmlpress-toolbar>"#));
    Ok(())
}

fn create_sectioned_site() -> crate::Result<Site> {
    let mut site = create_site()?;
    site.config.sections = vec![
        Section {
            title: "Docs".to_owned(),
            prefix: Some("/".to_owned()),
            ..Section::default()
        },
        Section {
            title: "API".to_owned(),
            prefix: Some("/api/".to_owned()),
            site_title: Some("API Reference".to_owned()),
            site_link: Some("/api/".to_owned()),
            ..Section::default()
        },
        Section {
            title: "GitHub".to_owned(),
            kind: SectionKind::Custom,
            url: Some("https://github.com/x".to_owned()),
            position: Position::Sidebar,
            ..Section::default()
        },
    ];
    Ok(site)
}

#[test]
fn shows_only_the_open_sections_pages_with_its_brand() -> crate::Result<()> {
    let site = create_sectioned_site()?;
    let cli = render_page(&site, &site.pages[3], false)?;
    assert!(cli.contains(r#"<a href="/api/" data-section aria-current="page">API</a>"#));
    assert!(cli.contains(r#"<a href="/" data-section>Docs</a>"#));
    assert!(cli.contains(r#"<htmlpress-brand><a href="/api/">API Reference</a>"#));
    assert!(!cli.contains(r#"<a href="/guide.html""#), "{cli}");
    assert!(cli.contains(r#"<a rel="prev" href="/api/"><small>Previous</small>API</a>"#));
    assert!(!cli.contains(r#"rel="next""#));
    assert!(!cli.contains("<summary>API</summary>"), "{cli}");

    let guide = render_page(&site, &site.pages[1], false)?;
    assert!(guide.contains(r#"<a href="/" data-section aria-current="page">Docs</a>"#));
    assert!(!guide.contains(r#"href="/api/cli.html""#));
    assert!(guide.contains(
        r#"<li data-position="sidebar"><a href="https://github.com/x" data-section>GitHub</a></li>"#
    ));
    assert!(
        !guide
            .contains(r#"<a href="https://github.com/x" data-section>GitHub</a></nav>"#)
    );
    Ok(())
}

#[test]
fn puts_icons_before_link_and_section_titles() -> crate::Result<()> {
    let mut site = create_sectioned_site()?;
    site.config.sections[1].icon = Some("code".to_owned());
    site.config.links = vec![Link {
        title: "Chat".to_owned(),
        url: "https://chat.x".to_owned(),
        position: Position::Header,
        icon: Some("message-circle".to_owned()),
        prefix: None,
    }];
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<a href="/api/" data-section><svg viewBox="0 0 24 24""#));
    assert!(html.contains(r#"<a href="https://chat.x"><svg viewBox="0 0 24 24""#));
    Ok(())
}

#[test]
fn shows_a_prefixed_link_only_under_its_folder() -> crate::Result<()> {
    let mut site = create_site()?;
    site.config.links = vec![Link {
        title: "Report".to_owned(),
        url: "https://issues.x".to_owned(),
        prefix: Some("/api/".to_owned()),
        ..Link::default()
    }];
    let cli = render_page(&site, &site.pages[3], false)?;
    assert!(cli.contains(r#"<a href="https://issues.x">Report</a>"#));
    let guide = render_page(&site, &site.pages[1], false)?;
    assert!(!guide.contains("issues.x"));
    Ok(())
}

#[test]
fn puts_the_byline_under_the_title() -> crate::Result<()> {
    let mut site = create_site()?;
    let mut page = read_page(
        "post.html",
        "<title>Post</title><meta name=\"date\" content=\"2026-10-01\">\
         <meta name=\"author\" content=\"Ann\"><h1>Post</h1><p>Hi.</p>",
    )?;
    page.updated = Some("2026-10-20".to_owned());
    site.pages.push(page);
    let html = render_page(&site, &site.pages[4], false)?;
    let title = html.find("<h1>Post</h1>").unwrap_or(usize::MAX);
    let byline = html.find("<htmlpress-byline>").unwrap_or(0);
    assert!(title < byline, "{html}");
    assert!(
        html.contains("Published <time datetime=\"2026-10-01\">October 1, 2026</time>")
    );
    assert!(
        html.contains("Updated <time datetime=\"2026-10-20\">October 20, 2026</time>")
    );
    assert!(html.contains("components/byline/byline.css"));
    let home = render_page(&site, &site.pages[0], false)?;
    assert!(!home.contains("<htmlpress-byline>"));
    Ok(())
}

#[test]
fn uses_labels_icons_and_hides_pages_from_the_navigation() -> crate::Result<()> {
    let mut site = create_site()?;
    site.config.edit = Some("https://x.dev/edit/".to_owned());
    site.pages[1].label = Some("Short".to_owned());
    site.pages[1].icon = Some("rocket".to_owned());
    site.pages[3].sidebar = false;
    site.pages[3].source_url = Some("https://x.dev/cli".to_owned());
    site.pages[3].source_action = crate::models::page::SourceAction::View;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<a href="/guide.html"><svg viewBox"#));
    assert!(html.contains("</svg>Short</a>"));
    assert!(!html.contains(r#"href="/api/cli.html""#));
    let cli = render_page(&site, &site.pages[3], false)?;
    assert!(cli.contains(r#"<a href="https://x.dev/cli">View source</a>"#));
    Ok(())
}

#[test]
fn lists_a_folder_with_only_its_index_as_one_page() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages
        .push(read_page("start/index.html", "<title>Start</title>")?);
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(!html.contains("<summary>Start</summary>"));
    assert!(html.contains(r#"<li><a href="/start/">Start</a></li>"#));
    assert!(html.contains("<summary>API</summary>"));
    Ok(())
}

#[test]
fn colors_code_blocks_with_the_configured_themes() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages[1] = read_page(
        "guide.html",
        "<pre><code class=\"language-js\">let a = 1</code></pre>",
    )?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains("light-dark(#8839EF, #CBA6F7)"), "{html}");
    site.config.code_theme_light = Some("github-light".to_owned());
    site.config.code_theme_dark = Some("github-dark".to_owned());
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(!html.contains("#8839EF") && html.contains("light-dark("));
    assert!(!site.pages[1].content.contains("light-dark("));
    Ok(())
}
