use std::path::PathBuf;

use super::*;
use crate::actions::page::read::read_page;
use crate::models::config::{Config, Link, Position, Section, SectionKind};
use crate::settings::ASSETS;

fn create_site() -> crate::Result<Site> {
    Ok(Site {
        root: PathBuf::from("."),
        config: Config::default(),
        title: "Docs <&>".into(),
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
        languages: vec![],
        translations: std::collections::BTreeMap::new(),
    })
}

#[test]
fn renders_the_page_in_the_layout() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<title>Guide · Docs &lt;&amp;&gt;</title>"));
    assert!(html.contains("<p>Read me.</p>"));
    assert!(html.contains(r#"<a href="/guide/" aria-current="page">Guide</a>"#));
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
        r#"<details open><summary><a href="/api/">API</a></summary><ul><li><a href="/api/cli/">CLI</a></li></ul></details>"#
    ));
    Ok(())
}

#[test]
fn leaves_the_byline_off_a_page_with_its_own_layout() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages[1].updated = Some("2026-10-20".to_owned());
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains("<htmlpress-byline>"));
    site.pages[1].layout = Some("landing".to_owned());
    site.assets.insert(
        "layouts/landing.html".to_owned(),
        site.assets["layout.html"].clone(),
    );
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(!html.contains("<htmlpress-byline>"));
    Ok(())
}

#[test]
fn labels_the_navigation_and_its_links() -> crate::Result<()> {
    let mut site = create_site()?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<nav aria-label="Pages"><p data-label>Articles</p><ul>"#));
    site.config.links = vec![Link {
        title: "Chat".into(),
        url: "https://chat.x".to_owned(),
        position: Position::Sidebar,
        ..Link::default()
    }];
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<p data-label>Links</p><ul data-links>"#));
    Ok(())
}

#[test]
fn keeps_a_page_ordered_after_a_folder_below_it() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages
        .push(read_page("zz.html", "<title>Last</title>")?);
    let html = render_page(&site, &site.pages[0], false)?;
    let folder = html.find(r#"<a href="/api/cli/">"#);
    let last = html.find(r#"<a href="/zz/">"#);
    assert!(folder.is_some() && last > folder);
    Ok(())
}

#[test]
fn links_a_group_to_its_index_page() -> crate::Result<()> {
    let mut site = create_site()?;
    let api = render_page(&site, &site.pages[2], false)?;
    assert!(api.contains(r#"<summary><a href="/api/" aria-current="page">API</a>"#));
    site.pages.retain(|item| item.path != "api/index.html");
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(
        html.contains(r#"<details open><summary>api</summary>"#),
        "{html}"
    );
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
    assert!(!home.contains("<htmlpress-toc>"));
    Ok(())
}

#[test]
fn adds_the_links_and_the_edit_link_from_the_config() -> crate::Result<()> {
    let mut site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(!html.contains("<htmlpress-links") && !html.contains("<htmlpress-edit"));
    site.config.links = vec![Link {
        title: "GitHub".into(),
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
fn links_the_sheet_only_with_a_component_that_opens_it() -> crate::Result<()> {
    let mut site = create_site()?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"src="/_htmlpress/components/sheet/sheet.js""#));
    assert!(html.contains(r#"href="/_htmlpress/components/sheet/sheet.css""#));
    site.assets.insert(
        "layout.html".to_owned(),
        "<main><htmlpress-content></main>".to_owned(),
    );
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(!html.contains("sheet.js"));
    Ok(())
}

#[test]
fn renders_the_theme_toggle_as_a_menu() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<summary aria-label="Change theme" aria-keyshortcuts="T""#));
    for choice in ["light", "dark", "system"] {
        assert!(
            html.contains(&format!(r#"<button type="button" data-choice="{choice}">"#))
        );
    }
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
    assert!(
        html.contains(r#"<meta property="og:url" content="https://example.com/guide/">"#)
    );
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
    assert!(!html.contains("<htmlpress-banner"));
    assert!(html.contains(r#"<link rel="icon" href="/_htmlpress/favicon.svg">"#));
    assert!(html.contains(r#"<a href="https://htmlpress.dev">Htmlpress</a>"#));
    site.config.logo = Some("/logo.svg".to_owned());
    site.config.banner = Some("<a href=\"/new\">New</a>".into());
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<a href="/"><img src="/logo.svg" alt="">Docs"#));
    assert!(html.contains(r#"<link rel="icon" href="/logo.svg">"#));
    assert!(
        html.contains(r#"<htmlpress-banner><a href="/new">New</a></htmlpress-banner>"#)
    );
    Ok(())
}

#[test]
fn puts_the_description_under_the_brand_title() -> crate::Result<()> {
    let mut site = create_site()?;
    site.config.description = Some("Notes <&>".into());
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(
        r#"<a href="/"><span>Docs &lt;&amp;&gt;<small>Notes &lt;&amp;&gt;</small></span></a>"#
    ));
    Ok(())
}

#[test]
fn puts_each_link_in_its_position() -> crate::Result<()> {
    let mut site = create_site()?;
    site.config.links = vec![
        Link {
            title: "GitHub".into(),
            url: "https://github.com/x".to_owned(),
            position: Position::Header,
            icon: None,
            prefix: None,
        },
        Link {
            title: "Chat".into(),
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
fn hides_the_breadcrumbs_by_config_unless_the_page_shows_them() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages.push(read_page(
        "api/v1/auth.html",
        "<head><meta name=\"breadcrumbs\" content=\"true\"></head>",
    )?);
    site.pages
        .push(read_page("api/v1/keys.html", "<title>Keys</title>")?);
    site.config.breadcrumbs = Some(false);
    let shown = render_page(&site, &site.pages[4], false)?;
    assert!(shown.contains("<htmlpress-breadcrumbs><nav"));
    let hidden = render_page(&site, &site.pages[5], false)?;
    assert!(!hidden.contains("<htmlpress-breadcrumbs"));
    Ok(())
}

#[test]
fn links_the_markdown_from_the_head_and_the_toolbar() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[3], false)?;
    assert!(html.contains(
        r#"<link rel="alternate" type="text/markdown" href="/api/cli/index.md">"#
    ));
    assert!(html.contains(r#"<htmlpress-toolbar><p>Actions</p><a href="/api/cli/index.md" type="text/markdown"><svg"#));
    assert!(html.contains("</svg>View as Markdown</a></htmlpress-toolbar>"));
    Ok(())
}

fn create_sectioned_site() -> crate::Result<Site> {
    let mut site = create_site()?;
    site.config.sections = vec![
        Section {
            title: "Docs".into(),
            prefix: Some("/".to_owned()),
            ..Section::default()
        },
        Section {
            title: "API".into(),
            prefix: Some("/api/".to_owned()),
            site_title: Some("API Reference".into()),
            site_link: Some("/api/".to_owned()),
            ..Section::default()
        },
        Section {
            title: "GitHub".into(),
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
    assert!(!cli.contains(r#"<a href="/guide/""#), "{cli}");
    assert!(cli.contains(r#"<a rel="prev" href="/api/"><small>Previous</small>API</a>"#));
    assert!(!cli.contains(r#"rel="next""#));
    assert!(!cli.contains("<details open>"), "{cli}");
    assert!(
        cli.contains("<div data-sections><p data-label>Sections</p><details><summary>")
    );
    assert!(cli.contains("<span>API</span></summary>"));

    let guide = render_page(&site, &site.pages[1], false)?;
    assert!(guide.contains(r#"<a href="/" data-section aria-current="page">Docs</a>"#));
    assert!(!guide.contains(r#"href="/api/cli/""#));
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
        title: "Chat".into(),
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
        title: "Report".into(),
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
    assert!(html.contains("Published <time datetime=\"2026-10-01\">Oct 1, 2026</time>"));
    assert!(html.contains("Updated <time datetime=\"2026-10-20\">Oct 20, 2026</time>"));
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
    assert!(html.contains(r#"<a href="/guide/"><svg viewBox"#));
    assert!(html.contains("</svg>Short</a>"));
    assert!(!html.contains(r#"href="/api/cli/""#));
    let cli = render_page(&site, &site.pages[3], false)?;
    assert!(cli.contains(r#"<p>Actions</p><a href="https://x.dev/cli"><svg"#));
    assert!(cli.contains("</svg>View on GitHub</a>"));
    Ok(())
}

#[test]
fn shows_page_icons_only_at_the_top_level() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages[2].icon = Some("code".to_owned());
    site.pages[3].icon = Some("rocket".to_owned());
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<details open><summary><a href="/api/"><svg viewBox"#));
    assert!(html.contains("</svg>API</a></summary>"));
    assert!(html.contains(r#"<li><a href="/api/cli/">CLI</a></li>"#));
    Ok(())
}

#[test]
fn lists_a_folder_with_only_its_index_as_one_page() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages
        .push(read_page("start/index.html", "<title>Start</title>")?);
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(!html.contains(r#"<summary><a href="/start/">"#));
    assert!(html.contains(r#"<li><a href="/start/">Start</a></li>"#));
    assert!(html.contains(r#"<summary><a href="/api/">API</a></summary>"#));
    Ok(())
}

#[test]
fn nests_folders_inside_folders_and_pages_them_in_order() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages
        .push(read_page("api/auth/index.html", "<title>Auth</title>")?);
    site.pages
        .push(read_page("api/auth/login.html", "<title>Login</title>")?);
    site.pages.push(read_page(
        "api/auth/oauth/token.html",
        "<title>Token</title>",
    )?);
    site.pages
        .push(read_page("zed.html", "<title>Zed</title>")?);
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(concat!(
        r#"<li><details open><summary><a href="/api/auth/">Auth</a></summary><ul>"#,
        r#"<li><a href="/api/auth/login/">Login</a></li>"#,
        r#"<li><details open><summary>oauth</summary><ul>"#,
        r#"<li><a href="/api/auth/oauth/token/">Token</a></li></ul></details></li>"#,
    )));
    let token = render_page(&site, &site.pages[6], false)?;
    assert!(token.contains(r#"<a rel="next" href="/zed/">"#));
    Ok(())
}

#[test]
fn writes_the_ui_in_the_pages_language() -> crate::Result<()> {
    let mut site = create_site()?;
    let config: Config = serde_saphyr::from_str(
        "languages:\n  - { code: fa, label: فارسی, messages: { toc.title: فهرست } }\n  - { code: es, label: Español }\n",
    )
    .map_err(|error| Error::Page {
        path: "htmlpress.yaml".into(),
        message: error.to_string(),
    })?;
    site.languages =
        crate::actions::language::resolve::resolve_languages(&config, "Docs");
    site.pages[1].language = "es".to_owned();
    site.pages[1].date = Some("2026-10-01".to_owned());
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"<html lang="es">"#));
    assert!(html.contains(r#"<htmlpress-brand><a href="/es/">"#));
    assert!(html.contains(r##"<a href="#content">Saltar al contenido</a>"##));
    assert!(html.contains("<p>Contenido</p>"));
    assert!(
        html.contains(r#"Publicado el <time datetime="2026-10-01">1 oct 2026</time>"#)
    );
    assert!(html.contains(r#"<a rel="prev" href="/"><small>Anterior</small>Home</a>"#));
    assert!(html.contains(r#""search.label":"Buscar""#));
    site.pages[1].content = r#"<htmlpress-callout type="warning"><p>!</p></htmlpress-callout><htmlpress-callout label="Ojo"></htmlpress-callout>"#.to_owned();
    let callouts = render_page(&site, &site.pages[1], false)?;
    assert!(
        callouts.contains(r#"<htmlpress-callout type="warning" label="Advertencia">"#)
    );
    assert!(callouts.contains(r#"<htmlpress-callout label="Ojo">"#));
    let home = render_page(&site, &site.pages[0], false)?;
    assert!(home.contains(r#"<html lang="fa" dir="rtl">"#));
    assert!(home.contains("<p>Created with"));
    Ok(())
}

#[test]
fn leads_a_custom_section_to_the_pages_translation() -> crate::Result<()> {
    let mut site = create_sectioned_site()?;
    site.config.sections.push(Section {
        title: "Start".into(),
        kind: SectionKind::Custom,
        url: Some("/guide/".to_owned()),
        ..Section::default()
    });
    let mut versions = std::collections::BTreeMap::new();
    versions.insert("es".to_owned(), "/es/guide/".to_owned());
    site.translations.insert("/guide/".to_owned(), versions);
    site.pages[0].language = "es".to_owned();
    let html = render_page(&site, &site.pages[0], false)?;
    assert!(html.contains(r#"<a href="/es/guide/" data-section>Start</a>"#));
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

#[test]
fn fills_an_inline_toc_written_in_the_page() -> crate::Result<()> {
    let mut site = create_site()?;
    site.pages[1] = read_page(
        "guide.html",
        "<title>Guide</title><htmlpress-toc level=\"2\"></htmlpress-toc><h2>One</h2><h3>Sub</h3>",
    )?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r##"<htmlpress-toc level="2" data-inline=""><p>Contents</p><ul><li><a href="#one" data-level="2">One</a></li></ul></htmlpress-toc>"##), "{html}");
    site.pages[1] = read_page(
        "guide.html",
        "<title>Guide</title><htmlpress-toc></htmlpress-toc><p>No headings.</p>",
    )?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(!html.contains("data-inline"));
    site.pages[1] = read_page(
        "guide.html",
        "<htmlpress-toc level=\"5\"></htmlpress-toc><h2>A</h2>",
    )?;
    assert!(render_page(&site, &site.pages[1], false).is_err());
    Ok(())
}

#[test]
fn builds_the_header_buttons_into_the_page() -> crate::Result<()> {
    let site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"<button type="button" aria-label="Search" aria-keyshortcuts="S" title="Search"><svg"#), "{html}");
    assert!(
        html.contains("<span>Search...</span><kbd>S</kbd></button></htmlpress-search>")
    );
    assert!(html.contains(r#"data-icon="system""#));
    assert!(html.contains(r#"aria-controls="htmlpress-sidebar" aria-keyshortcuts="M""#));
    assert!(html.contains(
        r#"<button type="button" data-key="L" data-targeted><span>Next page</span><kbd>L</kbd>"#
    ));
    assert!(html.contains("root.dataset.scripted"));
    Ok(())
}

#[test]
fn keeps_a_project_component_over_a_built_header_button() -> crate::Result<()> {
    let mut site = create_site()?;
    let path = "components/theme-toggle/theme-toggle.html";
    site.assets
        .insert(path.to_owned(), "<a href=\"/dark\">Dark</a>".to_owned());
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(
        html.contains("<htmlpress-theme-toggle><a href=\"/dark\">Dark</a>"),
        "{html}"
    );
    assert!(!html.contains(r#"data-icon="system""#));
    Ok(())
}

#[test]
fn prerenders_hovered_pages_under_the_base() -> crate::Result<()> {
    let mut site = create_site()?;
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(
        html.contains(r#"<script type="speculationrules">"#),
        "{html}"
    );
    assert!(html.contains(r#"{"href_matches":"/*"}"#), "{html}");
    assert!(html.contains(r#"<script type="module" src="/_htmlpress/components/sidebar/sidebar.js" blocking="render">"#));
    site.config.base = Some("docs".to_owned());
    let html = render_page(&site, &site.pages[1], false)?;
    assert!(html.contains(r#"{"href_matches":"/docs/*"}"#), "{html}");
    assert!(html.contains(r#""/docs/_htmlpress/*""#), "{html}");
    Ok(())
}
