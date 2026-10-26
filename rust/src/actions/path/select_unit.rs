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
    assert!(selection.unused.is_empty());
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
    Ok(())
}
