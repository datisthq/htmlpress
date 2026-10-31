use super::*;

fn select(
    include: &[&str],
    exclude: &[&str],
    paths: &[&str],
) -> Result<Selection, String> {
    let config = Config {
        include: include.iter().map(|text| text.to_string()).collect(),
        exclude: exclude.iter().map(|text| text.to_string()).collect(),
        ..Config::default()
    };
    select_paths(&config, paths.iter().map(|path| path.to_string()).collect())
}

fn pairs(map: &BTreeMap<String, String>) -> Vec<(&str, &str)> {
    (map.iter())
        .map(|(served, source)| (served.as_str(), source.as_str()))
        .collect()
}

#[test]
fn makes_every_html_file_a_page_by_default() -> Result<(), String> {
    let selection = select(&[], &[], &["a/b/c.html", "index.html", "logo.svg"])?;
    assert_eq!(
        pairs(&selection.pages),
        [("a/b/c.html", "a/b/c.html"), ("index.html", "index.html")]
    );
    assert_eq!(pairs(&selection.files), [("logo.svg", "logo.svg")]);
    assert_eq!(selection.unused, ["**/*.md"]);
    Ok(())
}

#[test]
fn makes_every_markdown_file_a_page_served_as_html() -> Result<(), String> {
    let paths = [
        "README.md",
        "docs/guide.es.md",
        "docs/img.png",
        "index.html",
    ];
    let selection = select(&[], &[], &paths)?;
    assert_eq!(
        pairs(&selection.pages),
        [
            ("README.html", "README.md"),
            ("docs/guide.es.html", "docs/guide.es.md"),
            ("index.html", "index.html"),
        ]
    );
    assert_eq!(pairs(&selection.files), [("docs/img.png", "docs/img.png")]);
    let selection = select(&["docs/**/*.md"], &[], &paths)?;
    assert_eq!(
        pairs(&selection.pages),
        [("guide.es.html", "docs/guide.es.md")]
    );
    Ok(())
}

#[test]
fn keeps_a_markdown_twin_and_skills_as_files() -> Result<(), String> {
    let config = Config {
        skills: vec!["skills/*/SKILL.md".to_owned()],
        ..Config::default()
    };
    let paths = ["a.html", "a.md", "skills/one/SKILL.md"];
    let selection =
        select_paths(&config, paths.iter().map(|path| path.to_string()).collect())?;
    assert_eq!(pairs(&selection.pages), [("a.html", "a.html")]);
    assert_eq!(
        pairs(&selection.files),
        [
            ("a.md", "a.md"),
            ("skills/one/SKILL.md", "skills/one/SKILL.md")
        ]
    );
    Ok(())
}

#[test]
fn copies_html_outside_include_and_drops_excluded_paths() -> Result<(), String> {
    let selection = select(
        &["guide/**/*.html", "*.html"],
        &["drafts/**", "**/_*.html"],
        &[
            "demo/embed.html",
            "drafts/next.html",
            "drafts/notes.txt",
            "guide/_part.html",
            "guide/a/b.html",
            "index.html",
        ],
    )?;
    assert_eq!(
        pairs(&selection.pages),
        [("a/b.html", "guide/a/b.html"), ("index.html", "index.html")]
    );
    assert_eq!(
        pairs(&selection.files),
        [("demo/embed.html", "demo/embed.html")]
    );
    Ok(())
}

#[test]
fn serves_a_folder_from_the_root_and_skips_what_lies_outside() -> Result<(), String> {
    let selection = select(
        &["docs/**/*.html"],
        &[],
        &[
            "README.md",
            "docs/guide/a.html",
            "docs/index.html",
            "docs/shot.png",
        ],
    )?;
    assert_eq!(
        pairs(&selection.pages),
        [
            ("guide/a.html", "docs/guide/a.html"),
            ("index.html", "docs/index.html")
        ]
    );
    assert_eq!(pairs(&selection.files), [("shot.png", "docs/shot.png")]);
    assert_eq!(read_bases(&Config::default()), [""]);
    Ok(())
}

#[test]
fn refuses_two_paths_served_at_the_same_url() {
    let result = select(&["docs/*.html", "*.html"], &[], &["docs/a.html", "a.html"]);
    assert!(result.is_err_and(|error| error.contains("/a.html")));
}

#[test]
fn reports_patterns_that_match_nothing_and_bad_ones() -> Result<(), String> {
    let selection = select(&["docs/**/*.html"], &["drafts/**"], &["index.html"])?;
    assert_eq!(selection.unused, ["docs/**/*.html", "drafts/**"]);
    assert!(selection.files.is_empty());
    assert!(select(&["[unclosed"], &[], &[]).is_err());
    for outside in [
        "../**/*.html",
        "/etc/*.html",
        "docs/../../*.html",
        "C:/x/*.html",
    ] {
        assert!(select(&[outside], &[], &[]).is_err(), "{outside}");
        assert!(select(&[], &[outside], &[]).is_err(), "{outside}");
    }
    let config = Config {
        include: vec!["../*.html".to_owned()],
        ..Config::default()
    };
    assert!(read_bases(&config).is_empty());
    Ok(())
}

#[test]
fn keeps_a_changelog_source_a_file_and_twins_only_of_pages() -> Result<(), String> {
    let config: Config = serde_saphyr::from_str(
        "include: [\"**/*.md\"]\nsections:\n  - { title: Changes, prefix: /changes/, type: changelog, source: CHANGELOG.md }\n",
    )
    .map_err(|error| error.to_string())?;
    let paths = ["CHANGELOG.md", "a.md", "b.html"];
    let selection =
        select_paths(&config, paths.iter().map(|path| path.to_string()).collect())?;
    assert_eq!(pairs(&selection.pages), [("a.html", "a.md")]);
    assert_eq!(
        pairs(&selection.files),
        [("CHANGELOG.md", "CHANGELOG.md"), ("b.html", "b.html")]
    );
    // An `.html` that is no page is no twin: the `.md` page is built and the clash told
    let paths = ["a.html", "a.md"];
    let clash =
        select_paths(&config, paths.iter().map(|path| path.to_string()).collect());
    assert!(clash.is_err_and(|error| error.contains("both served at /a.html")));
    Ok(())
}
