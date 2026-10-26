use super::*;

fn write_source(root: &Path, path: &str, text: &str) -> crate::Result<()> {
    write_file(&root.join(path), text.as_bytes())
}

#[test]
fn builds_pages_files_and_assets() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(root, "guides/build.html", "<h1>Build</h1>")?;
    write_source(root, "images/logo.svg", "<svg></svg>")?;
    let options = BuildOptions::new(root);
    write_source(&options.output, "stale.html", "old")?;

    let report = build_site(&options)?;
    assert_eq!((report.pages, report.files), (2, 1));
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();
    assert!(read("index.html").is_some_and(|html| html.contains("<h1>Home</h1>")));
    assert!(read("guides/build.html").is_some_and(|html| html.contains("theme.css")));
    assert_eq!(read("images/logo.svg").as_deref(), Some("<svg></svg>"));
    assert!(read("_htmlpress/theme.css").is_some());
    assert!(read("_htmlpress/reload.js").is_none());
    assert!(read("stale.html").is_none());
    Ok(())
}

#[test]
fn serves_escapes_and_project_components() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(root, ".htmlpress/styles/tokens.css", ":root { --mine: 1 }")?;
    write_source(root, ".htmlpress/components/tabs/tabs.js", "// mine")?;
    write_source(
        root,
        ".htmlpress/components/site-footer/site-footer.html",
        "<footer></footer>",
    )?;
    write_source(
        root,
        ".htmlpress/components/site-footer/site-footer.css",
        "footer {}",
    )?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();
    let theme = read("_htmlpress/theme.css").unwrap_or_default();
    assert!(theme.starts_with(":root { --mine: 1 }") && theme.contains("h1"));
    assert_eq!(
        read("_htmlpress/components/tabs/tabs.js").as_deref(),
        Some("// mine")
    );
    assert_eq!(
        read("_htmlpress/components/site-footer/site-footer.css").as_deref(),
        Some("footer {}")
    );
    assert!(read("_htmlpress/components/callout/callout.css").is_some());
    assert!(read("_htmlpress/components/site-footer/site-footer.html").is_none());
    assert!(read("_htmlpress/layout.html").is_none());
    Ok(())
}

#[test]
fn refuses_an_output_that_holds_the_project() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let project = root.path().join("docs");
    write_source(&project, "index.html", "<h1>Home</h1>")?;
    for output in [root.path().to_owned(), project.clone(), project.join("out")] {
        let options = BuildOptions {
            output,
            ..BuildOptions::new(&project)
        };
        assert!(matches!(build_site(&options), Err(Error::Usage(_))));
    }
    assert!(project.join("index.html").is_file());
    Ok(())
}

#[test]
fn builds_into_a_sibling_folder() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let project = root.path().join("docs");
    write_source(&project, "index.html", "<h1>Home</h1>")?;
    let output = root.path().join("site");
    let report = build_site(&BuildOptions {
        output,
        ..BuildOptions::new(&project)
    })?;
    assert_eq!(report.pages, 1);
    Ok(())
}

#[test]
fn writes_robots_and_a_sitemap_unless_the_project_ships_them() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(root, "guide.html", "<h1>Guide</h1>")?;
    write_source(root, "404.html", "<h1>Missing</h1>")?;
    let options = BuildOptions::new(root);
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();

    build_site(&options)?;
    assert_eq!(
        read("robots.txt").as_deref(),
        Some(
            "User-agent: *\nContent-Signal: search=yes, ai-input=yes, ai-train=yes\nAllow: /\n"
        )
    );
    assert!(read("sitemap.xml").is_none());

    write_source(
        root,
        "htmlpress.yaml",
        "site: https://example.com/\ncontentSignals:\n  aiTrain: false\n",
    )?;
    build_site(&options)?;
    let robots = read("robots.txt").unwrap_or_default();
    assert!(robots.contains("ai-train=no"));
    assert!(robots.ends_with("\nSitemap: https://example.com/sitemap.xml\n"));
    let sitemap = read("sitemap.xml").unwrap_or_default();
    assert!(sitemap.contains("<loc>https://example.com/</loc>"));
    assert!(sitemap.contains("<loc>https://example.com/guide.html</loc>"));
    assert!(!sitemap.contains("404"));

    write_source(root, "robots.txt", "User-agent: *\nDisallow: /\n")?;
    write_source(root, "sitemap.xml", "<urlset/>")?;
    build_site(&options)?;
    assert_eq!(
        read("robots.txt").as_deref(),
        Some("User-agent: *\nDisallow: /\n")
    );
    assert_eq!(read("sitemap.xml").as_deref(), Some("<urlset/>"));
    Ok(())
}

#[test]
fn writes_a_404_page_unless_the_project_has_one() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    let options = BuildOptions::new(root);
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();

    let report = build_site(&options)?;
    assert_eq!(report.pages, 1);
    let missing = read("404.html").unwrap_or_default();
    assert!(missing.contains("<title>Page not found · "));
    assert!(missing.contains("components/button/button.css"));
    assert!(!read("index.html").unwrap_or_default().contains("404.html"));

    write_source(root, "404.html", "<title>Lost</title><h1>Lost</h1>")?;
    build_site(&options)?;
    assert!(read("404.html").is_some_and(|html| html.contains("<h1>Lost</h1>")));
    assert!(!read("index.html").unwrap_or_default().contains("404.html"));
    Ok(())
}

#[test]
fn writes_agent_files_and_keeps_the_projects_own() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(root, "guide.html", "<title>Guide</title><p>Hi.</p>")?;
    write_source(root, "guide.md", "my own")?;
    write_source(root, "_headers", "/*\n  X-Frame-Options: DENY\n")?;
    write_source(
        root,
        "skills/one/SKILL.md",
        "---\nname: one\ndescription: First.\n---\n",
    )?;
    write_source(root, "htmlpress.yaml", "skills: skills/*/SKILL.md\n")?;
    let options = BuildOptions::new(root);
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();

    build_site(&options)?;
    assert_eq!(read("index.md").as_deref(), Some("# Home\n"));
    assert_eq!(read("guide.md").as_deref(), Some("my own"));
    assert!(read("llms.txt").is_some_and(|llms| llms.contains("- [Guide](/guide.md)")));
    assert!(read(".well-known/agent-skills/one/SKILL.md").is_some());
    assert!(read(".well-known/agent-skills/index.json").is_some());
    let headers = read("_headers").unwrap_or_default();
    assert!(headers.starts_with("/*\n  X-Frame-Options: DENY\n\n/\n  Link: "));
    Ok(())
}

// Every URL the build writes must carry the base, or the site breaks on a sub-path host
#[test]
fn prefixes_every_url_with_the_base() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(
        root,
        "htmlpress.yaml",
        "site: https://x.dev\nbase: /repo/\nlogo: /logo.svg\n\
         banner: '<a href=\"/news.html\">News</a>'\n\
         links:\n  - title: Chat\n    url: /chat.html\n    position: sidebar\n\
         skills: skills/*/SKILL.md\n",
    )?;
    write_source(
        root,
        "index.html",
        "<h1>Home</h1><img src=\"/a.png\" srcset=\"/a.png 2x\">",
    )?;
    write_source(
        root,
        "guides/index.html",
        "<title>Guides</title><a href=\"/\">Home</a>",
    )?;
    write_source(
        root,
        "guides/build.html",
        "<title>Build</title><a href=\"../index.html\">Up</a>",
    )?;
    write_source(
        root,
        "skills/one/SKILL.md",
        "---\nname: one\ndescription: First.\n---\n",
    )?;
    let options = BuildOptions {
        reload: true,
        ..BuildOptions::new(root)
    };
    build_site(&options)?;
    let read = |path: &str| {
        std::fs::read_to_string(options.output.join(path)).unwrap_or_default()
    };

    for page in [
        "index.html",
        "guides/index.html",
        "guides/build.html",
        "404.html",
    ] {
        let html = read(page);
        for attribute in [
            "href=\"/",
            "src=\"/",
            "srcset=\"/",
            "data-endpoint=\"/",
            "data-index=\"/",
        ] {
            for (at, _) in html.match_indices(attribute) {
                let url = &html[at + attribute.len() - 1..];
                assert!(
                    url.starts_with("/repo/"),
                    "{page}: {}",
                    &url[..url.len().min(40)]
                );
            }
        }
        assert!(html.contains("../index.html") || page != "guides/build.html");
    }
    assert!(
        read("index.html")
            .contains(r#"<link rel="canonical" href="https://x.dev/repo/">"#)
    );
    assert!(
        read("guides/build.html")
            .contains(r#"content="https://x.dev/repo/guides/build.html""#)
    );
    assert!(read("sitemap.xml").contains("<loc>https://x.dev/repo/guides/</loc>"));
    assert!(read("robots.txt").contains("Sitemap: https://x.dev/repo/sitemap.xml"));
    assert!(read("llms.txt").contains("(https://x.dev/repo/guides/build.md)"));
    assert!(read("_headers").starts_with("/repo/\n  Link: </repo/llms.txt>"));
    assert!(
        read("index.html")
            .contains(r#"<htmlpress-search data-index="/repo/_htmlpress/search.json">"#)
    );
    assert!(
        read("_htmlpress/search.json").contains("\"url\":\"/repo/guides/build.html\"")
    );
    assert!(
        read(".well-known/agent-skills/index.json")
            .contains("\"url\": \"/repo/.well-known/agent-skills/one/SKILL.md\"")
    );
    Ok(())
}

#[test]
fn builds_only_included_pages_and_leaves_excluded_files_out() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(
        root,
        "htmlpress.yaml",
        "include:\n  - '*.html'\nexclude: drafts/**\n",
    )?;
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(root, "demo/embed.html", "<p>raw</p>")?;
    write_source(root, "drafts/next.html", "<h1>Next</h1>")?;
    write_source(root, "drafts/notes.txt", "notes")?;
    let options = BuildOptions::new(root);
    let read = |path: &str| std::fs::read_to_string(options.output.join(path)).ok();

    let report = build_site(&options)?;
    assert_eq!((report.pages, report.files), (1, 1));
    assert_eq!(read("demo/embed.html").as_deref(), Some("<p>raw</p>"));
    assert!(read("drafts/next.html").is_none() && read("drafts/notes.txt").is_none());
    assert!(!read("index.html").unwrap_or_default().contains("drafts"));
    Ok(())
}

#[test]
fn builds_a_blog_and_a_changelog() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(
        root,
        "htmlpress.yaml",
        concat!(
            "site: https://x.dev\n",
            "sections:\n",
            "  - title: Docs\n",
            "    prefix: /\n",
            "  - title: Blog\n",
            "    type: blog\n",
            "    prefix: /blog/\n",
            "  - title: Changelog\n",
            "    type: changelog\n",
            "    prefix: /changelog/\n",
            "    source: CHANGELOG.md\n",
            "    version: true\n",
        ),
    )?;
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(
        root,
        "blog/hello.html",
        "<title>Hello</title><meta name=\"date\" content=\"2026-10-01\">\
         <meta name=\"keywords\" content=\"news\"><meta name=\"description\" content=\"First.\"><h1>Hello</h1>",
    )?;
    write_source(
        root,
        "CHANGELOG.md",
        "# Changelog\n\n## 0.2.0 (2026-10-02)\n\n* b\n\n## 0.1.0\n\n* a\n",
    )?;
    let options = BuildOptions::new(root);
    let read = |path: &str| {
        std::fs::read_to_string(options.output.join(path)).unwrap_or_default()
    };

    build_site(&options)?;
    let feed = read("blog/rss.xml");
    assert!(
        feed.contains("<link>https://x.dev/blog/hello.html</link>"),
        "{feed}"
    );
    assert!(feed.contains("<pubDate>Thu, 01 Oct 2026 00:00:00 +0000</pubDate>"));
    assert!(read("blog/index.html").contains("href=\"/blog/hello.html\""));
    assert!(read("blog/tags/news/index.html").contains("Posts tagged ‘news’"));
    let version = read("changelog/0-2-0/index.html");
    assert!(version.contains("<h1>0.2.0</h1>"));
    assert!(
        version.contains("Changelog <small>0.2.0</small>"),
        "{version}"
    );
    assert!(!version.contains("<summary>0.2.0</summary>"), "{version}");
    let nav = version.find("href=\"/changelog/\"").unwrap_or(usize::MAX);
    assert!(nav < version.find("href=\"/changelog/0-1-0/\"").unwrap_or(0));
    assert!(read("index.html").contains("type=\"application/rss+xml\""));
    assert!(read("sitemap.xml").contains(
        "<loc>https://x.dev/blog/hello.html</loc><lastmod>2026-10-01</lastmod>"
    ));
    Ok(())
}

#[test]
fn builds_pages_with_their_includes() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "htmlpress.yaml", "exclude: \"**/includes/**\"\n")?;
    write_source(
        root,
        "index.html",
        "<h1>Home</h1>\n<htmlpress-include src=\"includes/intro.md\"></htmlpress-include>",
    )?;
    write_source(root, "includes/intro.md", "## Setup\n\nRun it.\n")?;
    let options = BuildOptions::new(root);
    let report = build_site(&options)?;
    assert_eq!(report.pages, 1);
    let html =
        std::fs::read_to_string(options.output.join("index.html")).unwrap_or_default();
    assert!(
        html.contains("<h2 id=\"setup\">Setup</h2>") && html.contains("<p>Run it.</p>"),
        "{html}"
    );
    assert!(
        html.contains("href=\"#setup\""),
        "the included heading joins the toc"
    );
    write_source(
        root,
        "index.html",
        "<htmlpress-include src=\"../escape.md\"></htmlpress-include>",
    )?;
    assert!(build_site(&options).is_err());
    Ok(())
}
