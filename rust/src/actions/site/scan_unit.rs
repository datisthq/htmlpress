use super::*;

fn write_file(root: &Path, path: &str, text: &str) -> crate::Result<()> {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(Error::io(parent))?;
    }
    std::fs::write(&file, text).map_err(Error::io(&file))
}

#[test]
fn orders_pages_index_first_then_by_order_then_path() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(root, "b.html", "<h1>B</h1>")?;
    write_file(root, "a.html", "<h1>A</h1>")?;
    write_file(
        root,
        "z.html",
        r#"<head><meta name="order" content="1"></head><h1>Z</h1>"#,
    )?;
    write_file(root, "index.html", "<h1>Home</h1>")?;
    let site = scan_site(root)?;
    let titles: Vec<&str> = site.pages.iter().map(|page| page.title.as_str()).collect();
    assert_eq!(titles, ["Home", "Z", "A", "B"]);
    Ok(())
}

#[test]
fn puts_negative_orders_last_with_minus_one_at_the_end() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    let order =
        |order: i64| format!(r#"<head><meta name="order" content="{order}"></head>"#);
    write_file(root, "last.html", &format!("{}<h1>Last</h1>", order(-1)))?;
    write_file(root, "late.html", &format!("{}<h1>Late</h1>", order(-2)))?;
    write_file(root, "a.html", "<h1>A</h1>")?;
    write_file(root, "first.html", &format!("{}<h1>First</h1>", order(0)))?;
    let site = scan_site(root)?;
    let titles: Vec<&str> = site.pages.iter().map(|page| page.title.as_str()).collect();
    assert_eq!(titles, ["First", "A", "Late", "Last"]);
    Ok(())
}

#[test]
fn skips_dot_folders_and_the_config_and_keeps_other_files() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(root, "index.html", "<h1>Home</h1>")?;
    write_file(root, "images/logo.svg", "<svg></svg>")?;
    write_file(root, CONFIG_FILE, "title: Docs\n")?;
    write_file(root, ".htmlpress/build/index.html", "<h1>Old</h1>")?;
    write_file(root, "node_modules/x/index.html", "<h1>Dep</h1>")?;
    let site = scan_site(root)?;
    assert_eq!(site.title, "Docs");
    assert_eq!(site.pages.len(), 1);
    assert_eq!(site.files.keys().collect::<Vec<_>>(), ["images/logo.svg"]);
    assert_eq!(site.assets.len(), ASSETS.len());
    Ok(())
}

#[test]
fn lays_the_projects_assets_over_the_built_in_ones() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    let layout = "<main><htmlpress-content></htmlpress-content></main>";
    write_file(root, ".htmlpress/layout.html", layout)?;
    write_file(root, ".htmlpress/components/tabs/tabs.css", "/* mine */")?;
    write_file(
        root,
        ".htmlpress/components/site-footer/site-footer.html",
        "<footer></footer>",
    )?;
    write_file(root, ".htmlpress/notes.md", "kept out")?;
    let site = scan_site(root)?;
    assert_eq!(site.assets["layout.html"], layout);
    assert_eq!(site.assets["components/tabs/tabs.css"], "/* mine */");
    assert_eq!(
        site.assets["components/site-footer/site-footer.html"],
        "<footer></footer>"
    );
    assert!(site.assets.contains_key("components/tabs/tabs.js"));
    assert_eq!(site.assets.len(), ASSETS.len() + 1);
    Ok(())
}

#[test]
fn rejects_an_asset_that_replaces_nothing() -> crate::Result<()> {
    for (path, needle) in [
        (
            ".htmlpress/styles/tokns.css",
            "no built-in styles/tokns.css",
        ),
        (
            ".htmlpress/components/htmlpress-tabs/htmlpress-tabs.css",
            "components/tabs/tabs.html",
        ),
        (
            ".htmlpress/components/htmlpress-tabs.css",
            "components/tabs/tabs.html",
        ),
        (
            ".htmlpress/components/Footer/Footer.html",
            "not a component name",
        ),
        (
            ".htmlpress/components/site-footer.css",
            "components/site-footer/site-footer.html",
        ),
        (
            ".htmlpress/components/site-footer/site-footer.txt",
            ".html, .css or .js",
        ),
        (
            ".htmlpress/layouts/Landing.html",
            "a layout is a .html file",
        ),
        (".htmlpress/layouts/landing.css", "a layout is a .html file"),
    ] {
        let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
        write_file(root.path(), path, "x")?;
        let error = scan_site(root.path()).err().map(|error| error.to_string());
        assert!(
            error
                .as_deref()
                .is_some_and(|message| message.contains(needle)),
            "{path}: {error:?}"
        );
    }
    Ok(())
}

#[test]
fn reads_the_projects_layouts() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(
        root,
        "index.html",
        r#"<meta name="layout" content="landing">"#,
    )?;
    write_file(root, ".htmlpress/layouts/landing.html", "<main></main>")?;
    let site = scan_site(root)?;
    assert_eq!(site.assets["layouts/landing.html"], "<main></main>");
    Ok(())
}

#[test]
fn rejects_a_page_whose_layout_is_missing() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    write_file(
        root.path(),
        "index.html",
        r#"<meta name="layout" content="landing">"#,
    )?;
    let error = scan_site(root.path()).err().map(|error| error.to_string());
    assert!(
        error.as_deref().is_some_and(|message| {
            message.contains("index.html") && message.contains("layouts/landing.html")
        }),
        "{error:?}"
    );
    Ok(())
}

#[test]
fn registers_every_built_in_asset_file() -> crate::Result<()> {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut paths = vec!["layout.html".to_owned(), "favicon.svg".to_owned()];
    for folder in ["styles", "scripts", "components"] {
        let entries =
            std::fs::read_dir(assets.join(folder)).map_err(Error::io(folder))?;
        for entry in entries {
            let entry = entry.map_err(Error::io(folder))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !entry.path().is_dir() {
                paths.push(format!("{folder}/{name}"));
                continue;
            }
            let files = std::fs::read_dir(entry.path()).map_err(Error::io(&name))?;
            for file in files {
                let file = file.map_err(Error::io(&name))?;
                let file = file.file_name().to_string_lossy().into_owned();
                paths.push(format!("{folder}/{name}/{file}"));
            }
        }
    }
    paths.sort();
    let mut registered: Vec<_> =
        ASSETS.iter().map(|asset| asset.path.to_owned()).collect();
    registered.sort();
    assert_eq!(paths, registered);
    Ok(())
}

#[test]
fn puts_a_folders_index_before_its_pages() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    for path in [
        "index.html",
        "api/auth.html",
        "api/index.html",
        "api-v2.html",
    ] {
        write_file(root, path, "<h1>A</h1>")?;
    }
    let site = scan_site(root)?;
    let paths: Vec<&str> = site.pages.iter().map(|page| page.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "index.html",
            "api-v2.html",
            "api/index.html",
            "api/auth.html"
        ]
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn skips_a_base_that_links_outside_the_project() -> crate::Result<()> {
    let outside = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    write_file(outside.path(), "secret.html", "<h1>Secret</h1>")?;
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    std::os::unix::fs::symlink(outside.path(), root.join("docs"))
        .map_err(Error::io("symlink"))?;
    write_file(root, CONFIG_FILE, "include: docs/**/*.html\n")?;
    let site = scan_site(root)?;
    assert!(site.pages.is_empty());
    Ok(())
}

#[test]
fn publishes_a_patched_readme_and_patches_a_page() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(root, "index.html", "<h1>Home</h1>")?;
    write_file(root, "guide.html", "<title>Guide</title><h1>Guide</h1>")?;
    write_file(
        root,
        "README.md",
        "# Readme\n\nSome **text**.\n\n## Setup\n\n| a |\n|---|\n| b |\n",
    )?;
    write_file(
        root,
        CONFIG_FILE,
        "patches:\n  - file: README.md\n    article:\n      title: Introduction\n      path: /introduction/\n      order: 1\n  - file: guide.html\n    article:\n      sidebar: false\n",
    )?;
    let site = scan_site(root)?;
    let intro = site.pages.iter().find(|page| page.url == "/introduction/");
    let intro = intro.ok_or_else(|| Error::Usage("no introduction".to_owned()))?;
    assert_eq!(intro.title, "Introduction");
    assert_eq!(intro.source, "README.md");
    assert!(
        intro.content.contains("<strong>text</strong>")
            && intro.content.contains("<table>")
    );
    assert_eq!(intro.toc.len(), 1);
    let guide = site.pages.iter().find(|page| page.source == "guide.html");
    assert!(guide.is_some_and(|page| !page.sidebar));

    write_file(
        root,
        CONFIG_FILE,
        "patches:\n  - file: guide.html\n    article:\n      path: /\n",
    )?;
    assert!(scan_site(root).is_err());
    write_file(
        root,
        CONFIG_FILE,
        "patches:\n  - file: missing.md\n    article:\n      title: X\n",
    )?;
    assert!(matches!(scan_site(root), Err(Error::Config { .. })));
    Ok(())
}

#[test]
fn refuses_a_patch_or_changelog_that_reads_outside_the_project() -> crate::Result<()> {
    let outer = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    write_file(outer.path(), "secret.md", "# Secret")?;
    let root = outer.path().join("project");
    write_file(&root, "index.html", "<h1>Home</h1>")?;
    write_file(&root, ".env", "TOKEN=1")?;
    write_file(&root, ".secrets/notes.md", "# Token")?;
    for config in [
        "patches:\n  - file: ../secret.md\n    article:\n      title: X\n",
        "patches:\n  - file: .secrets/notes.md\n    article:\n      title: X\n",
        "sections:\n  - title: Log\n    type: changelog\n    prefix: /log/\n    source: ../secret.md\n",
        "sections:\n  - title: Log\n    type: changelog\n    prefix: /log/\n    source: .env\n",
        "skills: ../*.md\n",
    ] {
        write_file(&root, CONFIG_FILE, config)?;
        let error = scan_site(&root)
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();
        assert!(
            error.contains("inside the project") || error.contains("never published"),
            "{config}: {error}"
        );
    }
    Ok(())
}

#[test]
fn colors_code_blocks_with_the_configured_themes() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(
        root,
        "index.html",
        "<pre><code class=\"language-js\">let a = 1</code></pre>",
    )?;
    let site = scan_site(root)?;
    let content = &site.pages[0].content;
    assert!(
        content.contains("color:#8839EF;--shiki-dark:#CBA6F7"),
        "{content}"
    );
    write_file(
        root,
        "htmlpress.yaml",
        "codeThemeLight: github-light\ncodeThemeDark: github-dark\n",
    )?;
    let content = scan_site(root)?.pages[0].content.clone();
    assert!(
        !content.contains("#8839EF") && content.contains("--shiki-dark:"),
        "{content}"
    );
    Ok(())
}

#[test]
fn reads_each_page_into_content_toc_and_markdown() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(
        root,
        "guide.html",
        "<title>Guide</title><h1>Guide</h1><h2>Run :rocket:</h2><p>See $x_1$ and index.html.</p>",
    )?;
    let site = scan_site(root)?;
    let page = &site.pages[0];
    assert_eq!(page.toc.len(), 1);
    assert!(
        page.content.contains("<htmlpress-math data-latex=\"x_1\">"),
        "{}",
        page.content
    );
    assert!(page.markdown.starts_with("# Guide\n"), "{}", page.markdown);
    assert!(page.markdown.contains("$x_1$"), "{}", page.markdown);
    assert!(
        !page.markdown.contains("# Guide\n\n# Guide"),
        "{}",
        page.markdown
    );
    Ok(())
}

#[test]
fn runs_the_config_plugins_and_rejects_a_bad_one() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(root, "index.html", "<h1>Home</h1>")?;
    write_file(
        root,
        "guide.md",
        "# Guide\n\n## Usage\n\n[Out](https://example.com)",
    )?;
    let config = "include: ['**/*.html', '**/*.md']\nplugins:\n  md:\n    - name: md.shiftHeading\n      options: { shift: 1 }\n  html:\n    - html.externalLinks\n";
    write_file(root, CONFIG_FILE, config)?;
    let site = scan_site(root)?;
    let Some(guide) = site.pages.iter().find(|page| page.source == "guide.md") else {
        panic!("no guide page");
    };
    assert!(
        guide.content.contains("<h3 id=\"usage\""),
        "{}",
        guide.content
    );
    assert!(
        guide.content.contains("rel=\"nofollow\""),
        "{}",
        guide.content
    );
    write_file(root, CONFIG_FILE, "plugins:\n  md: [md.toHtml]\n")?;
    match scan_site(root) {
        Err(Error::Config { message, .. }) => {
            assert!(message.starts_with("plugins.md: "), "{message}")
        }
        other => panic!("{other:?}"),
    }
    Ok(())
}

#[test]
fn reads_routes_at_their_path_over_articles_and_out_of_the_nav() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = root.path();
    write_file(root, "index.html", "<h1>Article</h1>")?;
    write_file(root, "guide.html", "<h1>Guide</h1>")?;
    write_file(root, ".htmlpress/routes/index.html", "<h1>Landing</h1>")?;
    write_file(root, ".htmlpress/routes/pricing.md", "# Pricing\n")?;
    write_file(root, ".htmlpress/routes/-parts/hero.html", "<p>Hero</p>")?;
    write_file(root, ".htmlpress/routes/-draft.html", "<h1>Draft</h1>")?;
    let site = scan_site(root)?;
    let pages: Vec<(&str, &str, bool)> = (site.pages.iter())
        .map(|page| (page.url.as_str(), page.title.as_str(), page.sidebar))
        .collect();
    assert_eq!(
        pages,
        [
            ("/", "Landing", false),
            ("/guide/", "Guide", true),
            ("/pricing/", "Pricing", false)
        ]
    );
    assert!(
        site.pages
            .iter()
            .all(|page| page.is_route() != page.sidebar)
    );
    Ok(())
}

#[test]
fn reads_a_route_path_as_a_tanstack_file_route() {
    for (file, path) in [
        ("index.html", Some("index.html")),
        ("about.html", Some("about/index.html")),
        (
            "My Blog/First Post.md",
            Some("My Blog/First Post/index.html"),
        ),
        ("blog.post.html", Some("blog/post/index.html")),
        ("blog/index.html", Some("blog/index.html")),
        ("blog.route.html", Some("blog/index.html")),
        ("(marketing)/pricing.html", Some("pricing/index.html")),
        ("_layout/team.html", Some("team/index.html")),
        ("es/index.html", Some("es/index.html")),
        ("feed[.]xml.html", Some("feed.xml")),
        ("-landing/hero.html", None),
        ("__root.html", None),
        ("posts.$id.html", None),
        ("logo.svg", None),
    ] {
        assert_eq!(read_route_path(file).as_deref(), path, "{file}");
    }
}
