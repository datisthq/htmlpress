use super::*;

// The text of every bundle a written page links, whatever the base
fn read_bundled(output: &Path, html: &str) -> String {
    (html.split("_htmlpress/bundles/").skip(1))
        .filter_map(|rest| rest.split('"').next())
        .filter_map(|name| {
            let path = output.join("_htmlpress/bundles").join(name);
            std::fs::read_to_string(path).ok()
        })
        .collect()
}

fn write_source(root: &Path, path: &str, text: &str) -> crate::Result<()> {
    write_file(&root.join(path), text.as_bytes())
}

#[test]
fn swaps_the_output_in_whole_and_reports_what_it_read() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(
        root,
        "htmlpress.yaml",
        "include: docs/**/*.html\nsections:\n  - title: Changelog\n    type: changelog\n    \
         prefix: /changelog/\n    source: CHANGELOG.md\n",
    )?;
    write_source(root, "CHANGELOG.md", "# Changelog\n\n## 1.0.0\n\n- a\n")?;
    write_source(root, "shared/part.md", "part")?;
    write_source(
        root,
        "docs/index.html",
        "<h1>Home</h1><htmlpress-include src=\"../shared/part.md\"></htmlpress-include>",
    )?;
    let options = BuildOptions::new(root);
    let (staging, previous) = create_build_paths(&options.output);
    write_source(&staging, "stale.html", "old")?;
    write_source(&options.output, "stale.html", "old")?;

    let report = build_site(&options)?;
    assert_eq!(report.inputs, ["CHANGELOG.md", "shared/part.md"]);
    assert!(options.output.join("index.html").is_file());
    assert!(!options.output.join("stale.html").exists());
    assert!(!staging.exists() && !previous.exists());
    Ok(())
}

#[test]
fn refuses_an_output_whose_build_folders_hold_the_project() -> crate::Result<()> {
    let parent = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = parent.path().join("site.next");
    write_source(&root, "index.html", "<h1>Home</h1>")?;
    let options = BuildOptions {
        output: parent.path().join("site"),
        ..BuildOptions::new(&root)
    };
    assert!(build_site(&options).is_err());
    assert!(root.join("index.html").is_file());
    Ok(())
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
    let guide = read("guides/build/index.html").unwrap_or_default();
    let bundled = read_bundled(&options.output, &guide);
    assert!(
        bundled.contains("--htmlpress-primary")
            && bundled.contains("@scope (htmlpress-nav)")
    );
    assert!(guide.contains("_htmlpress/components/nav/nav.js"));
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
fn writes_the_logo_in_the_projects_primary() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    write_source(
        root,
        ".htmlpress/styles/tokens.css",
        ":root { --htmlpress-primary: red }",
    )?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let logo = std::fs::read_to_string(options.output.join("_htmlpress/logo.svg"))
        .map_err(Error::io("logo.svg"))?;
    assert!(logo.contains("<style>svg{color:red}</style>"), "{logo}");
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
    assert!(sitemap.contains("<loc>https://example.com/guide/</loc>"));
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
    assert!(
        read_bundled(&options.output, &missing).contains("@scope (htmlpress-button)")
    );
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
    assert_eq!(read("guide/index.md").as_deref(), Some("my own"));
    assert!(read("guide.md").is_none());
    assert!(
        read("llms.txt").is_some_and(|llms| llms.contains("- [Guide](/guide/index.md)"))
    );
    assert!(read(".well-known/agent-skills/one/SKILL.md").is_some());
    assert!(read(".well-known/agent-skills/index.json").is_some());
    let headers = read("_headers").unwrap_or_default();
    assert!(headers.starts_with("/*\n  X-Frame-Options: DENY\n\n/\n  Link: "));
    Ok(())
}

#[test]
fn redirects_a_missing_home_on_the_host_first() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "guide.html", "<h1>Guide</h1>")?;
    write_source(root, "_redirects", "/old/  /guide/  301\n")?;
    write_source(root, "htmlpress.yaml", "base: /repo/\n")?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let read = |path: &str| std::fs::read_to_string(options.output.join(path));

    assert_eq!(
        read("_redirects").map_err(Error::io("_redirects"))?,
        "/old/  /guide/  301\n\n/repo/  /repo/guide/  302\n"
    );
    let stub = read("index.html").map_err(Error::io("index.html"))?;
    assert!(stub.contains(r#"content="0; url=/repo/guide/""#));
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
        "guides/build/index.html",
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
        let up = r#"<a href="/repo/">Up</a>"#;
        assert!(html.contains(up) || page != "guides/build/index.html");
    }
    assert!(
        read("index.html")
            .contains(r#"<link rel="canonical" href="https://x.dev/repo/">"#)
    );
    assert!(
        read("guides/build/index.html")
            .contains(r#"content="https://x.dev/repo/guides/build/""#)
    );
    assert!(read("sitemap.xml").contains("<loc>https://x.dev/repo/guides/</loc>"));
    assert!(read("robots.txt").contains("Sitemap: https://x.dev/repo/sitemap.xml"));
    assert!(read("llms.txt").contains("(https://x.dev/repo/guides/build/index.md)"));
    assert!(read("_headers").starts_with("/repo/\n  Link: </repo/llms.txt>"));
    assert!(
        read("index.html")
            .contains(r#"<htmlpress-search data-index="/repo/_htmlpress/search.json">"#)
    );
    assert!(read("_htmlpress/search.json").contains("\"url\":\"/repo/guides/build/\""));
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
        feed.contains("<link>https://x.dev/blog/hello/</link>"),
        "{feed}"
    );
    assert!(feed.contains("<pubDate>Thu, 01 Oct 2026 00:00:00 +0000</pubDate>"));
    assert!(read("blog/index.html").contains("href=\"/blog/hello/\""));
    assert!(read("blog/tags/news/index.html").contains("Posts tagged ‘news’"));
    let version = read("changelog/0-2-0/index.html");
    assert!(version.contains("<h1>0.2.0</h1>"));
    assert!(
        version.contains("Changelog <small>0.2.0</small>"),
        "{version}"
    );
    assert!(!version.contains("<summary>0.2.0</summary>"), "{version}");
    let nav = &version[version.find("<htmlpress-nav").unwrap_or(0)..];
    let index = nav.find("href=\"/changelog/\"").unwrap_or(usize::MAX);
    assert!(index < nav.find("href=\"/changelog/0-1-0/\"").unwrap_or(0));
    assert!(read("index.html").contains("type=\"application/rss+xml\""));
    assert!(
        read("sitemap.xml").contains(
            "<loc>https://x.dev/blog/hello/</loc><lastmod>2026-10-01</lastmod>"
        )
    );
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

#[test]
fn draws_a_route_bare_filling_its_layout_elements() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "htmlpress.yaml", "title: Docs\nbase: /repo/\n")?;
    write_source(root, "guide.html", "<h1>Guide</h1>")?;
    write_source(
        root,
        ".htmlpress/routes/index.html",
        "<title>Home</title><htmlpress-header><htmlpress-brand></htmlpress-brand>\
         </htmlpress-header><main><a href=\"/guide/\">Guide</a></main>",
    )?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let read = |path: &str| std::fs::read_to_string(options.output.join(path));
    let home = read("index.html").map_err(Error::io("index.html"))?;
    assert!(!home.contains("htmlpress-sidebar"));
    assert!(home.contains("<htmlpress-brand><a"));
    assert!(home.contains("href=\"/repo/guide/\""));
    assert!(home.contains("\"/repo/_htmlpress/bundles/"));
    assert!(read_bundled(&options.output, &home).contains("@scope (htmlpress-header)"));
    let search = read("_htmlpress/search.json").map_err(Error::io("search.json"))?;
    assert!(search.contains("/guide/") && !search.contains("\"url\":\"/\""));
    Ok(())
}

fn read_output(output: &Path, path: &str) -> crate::Result<String> {
    let file = output.join(path);
    std::fs::read_to_string(&file).map_err(Error::io(&file))
}

fn build_reloading(root: &Path, output: &str) -> crate::Result<(BuildOptions, Cache)> {
    let options = BuildOptions {
        output: root.join(output),
        reload: true,
        ..BuildOptions::new(root)
    };
    let (_, cache) = build_cached(&options)?;
    let cache = cache.ok_or_else(|| Error::Usage("no cache".to_owned()))?;
    Ok((options, cache))
}

#[test]
fn rebuilds_a_page_alone_as_a_full_build_would() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<title>Home</title><p>Old</p>")?;
    write_source(root, "guide.html", "<title>Guide</title><p>Guide</p>")?;
    let (options, mut cache) = build_reloading(root, ".out")?;
    write_source(root, "guide.html", "<title>Guide</title><p>New</p>")?;
    let changed = ["guide.html".to_owned(), ".guide.swp".to_owned()];
    let report = rebuild_pages(&options, &mut cache, &changed)?;
    assert_eq!(report.map(|report| report.pages), Some(1));
    let (full, _) = build_reloading(root, ".full")?;
    for path in [
        "guide/index.html",
        "guide/index.md",
        "_htmlpress/search.json",
    ] {
        assert_eq!(
            read_output(&options.output, path)?,
            read_output(&full.output, path)?
        );
    }
    assert!(read_output(&options.output, "guide/index.html")?.contains("New"));
    Ok(())
}

#[test]
fn leaves_an_edit_other_pages_read_to_a_full_build() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<title>Home</title>")?;
    write_source(root, "guide.html", "<title>Guide</title>")?;
    let (options, mut cache) = build_reloading(root, ".out")?;
    write_source(root, "guide.html", "<title>Renamed</title>")?;
    let renamed = rebuild_pages(&options, &mut cache, &["guide.html".to_owned()])?;
    write_source(root, "new.html", "<title>New</title>")?;
    let added = rebuild_pages(&options, &mut cache, &["new.html".to_owned()])?;
    std::fs::remove_file(root.join("guide.html")).map_err(Error::io("guide"))?;
    let removed = rebuild_pages(&options, &mut cache, &["guide.html".to_owned()])?;
    assert_eq!((renamed, added, removed), (None, None, None));
    Ok(())
}

#[test]
fn refuses_to_replace_a_folder_that_is_no_build_output() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    let project = root.join("site");
    std::fs::create_dir_all(&project).map_err(Error::io(&project))?;
    std::fs::write(project.join("index.html"), "<h1>Home</h1>")
        .map_err(Error::io(root))?;
    for folder in [
        root.join("keep"),
        root.join("out.next"),
        project.join(".git"),
    ] {
        std::fs::create_dir_all(&folder).map_err(Error::io(&folder))?;
        std::fs::write(folder.join("file"), "mine").map_err(Error::io(&folder))?;
    }
    for output in [root.join("keep"), root.join("out"), project.join(".git")] {
        let options = BuildOptions {
            output: output.clone(),
            ..BuildOptions::new(&project)
        };
        assert!(
            matches!(build_site(&options), Err(Error::Usage(_))),
            "{output:?}"
        );
    }
    assert!(root.join("keep/file").is_file() && root.join("out.next/file").is_file());
    assert!(project.join(".git/file").is_file());
    let options = BuildOptions {
        output: root.join("built"),
        ..BuildOptions::new(&project)
    };
    build_site(&options)?;
    build_site(&options)?;
    std::fs::create_dir_all(root.join("empty")).map_err(Error::io(root))?;
    build_site(&BuildOptions {
        output: root.join("empty"),
        ..BuildOptions::new(&project)
    })?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn refuses_a_default_output_a_symlink_leads_out_of_the_project() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    let project = root.join("site");
    write_source(&project, "index.html", "<h1>Home</h1>")?;
    write_source(&root.join("home"), "build/file", "mine")?;
    std::os::unix::fs::symlink(root.join("home"), project.join(".htmlpress"))
        .map_err(Error::io(root))?;
    let options = BuildOptions::new(&project);
    assert!(matches!(build_site(&options), Err(Error::Usage(_))));
    assert!(matches!(
        crate::actions::site::clear::clear_site(&options),
        Err(Error::Usage(_))
    ));
    assert!(root.join("home/build/file").is_file());
    Ok(())
}

#[test]
fn copies_a_route_named_for_another_file_type_as_written() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(root, "index.html", "<h1>Home</h1>")?;
    let feed = "<?xml version=\"1.0\"?>\n<rss><channel><link>https://x.dev/</link></channel></rss>\n";
    write_source(root, ".htmlpress/routes/feed[.]xml.html", feed)?;
    write_source(root, "htmlpress.yaml", "site: https://x.dev\n")?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let read = |path: &str| std::fs::read_to_string(options.output.join(path));
    assert_eq!(read("feed.xml").map_err(Error::io(root))?, feed);
    let sitemap = read("sitemap.xml").map_err(Error::io(root))?;
    assert!(!sitemap.contains("feed"), "{sitemap}");
    assert!(!options.output.join("feed.xml.md").exists());
    Ok(())
}

#[test]
fn keeps_an_escaped_dollar_in_markdown_plain() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(
        root,
        "index.md",
        "# Home\n\nplain \\$x\\$ end, and $y$ math\n",
    )?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let html = std::fs::read_to_string(options.output.join("index.html"))
        .map_err(Error::io(root))?;
    assert!(html.contains("plain $x$ end"), "{html}");
    assert_eq!(html.matches("<htmlpress-math").count(), 1);
    Ok(())
}

#[test]
fn keeps_prices_in_markdown_text() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_source(
        root,
        "index.md",
        "# Home\n\nIt costs $5 and $10.\n\nOr $x$ apples.\n",
    )?;
    let options = BuildOptions::new(root);
    build_site(&options)?;
    let html = std::fs::read_to_string(options.output.join("index.html"))
        .map_err(Error::io(root))?;
    assert!(html.contains("<p>It costs $5 and $10.</p>"), "{html}");
    assert_eq!(html.matches("<htmlpress-math").count(), 1);
    Ok(())
}
