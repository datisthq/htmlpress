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

#[test]
fn makes_every_html_file_a_page_by_default() -> Result<(), String> {
    let selection = select(&[], &[], &["a/b/c.html", "index.html", "logo.svg"])?;
    assert_eq!(selection.pages, ["a/b/c.html", "index.html"]);
    assert_eq!(selection.files, ["logo.svg"]);
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
    assert_eq!(selection.pages, ["guide/a/b.html", "index.html"]);
    assert_eq!(selection.files, ["demo/embed.html"]);
    Ok(())
}

#[test]
fn reports_patterns_that_match_nothing_and_bad_ones() -> Result<(), String> {
    let selection = select(&["docs/**/*.html"], &["drafts/**"], &["index.html"])?;
    assert_eq!(selection.unused, ["docs/**/*.html", "drafts/**"]);
    assert_eq!(selection.files, ["index.html"]);
    assert!(select(&["[unclosed"], &[], &[]).is_err());
    Ok(())
}
