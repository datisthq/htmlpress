use super::*;

#[test]
fn splits_a_release_please_changelog() {
    let markdown = "# Changelog\n\nAll notable changes.\n\n\
        ## [0.2.0](https://github.com/x/y/compare/v0.1.0...v0.2.0) (2026-10-01)\n\n\
        ### Features\n\n* add search ([abc](https://x))\n\n\
        ```sh\n## not a heading\n```\n\n\
        ## 0.1.0 (2026-09-01)\n\n### Bug Fixes\n\n* first\n";
    let releases = split_changelog(markdown);
    assert_eq!(releases.len(), 2);
    assert_eq!(releases[0].version, "0.2.0");
    assert_eq!(releases[0].slug, "0-2-0");
    assert_eq!(releases[0].date.as_deref(), Some("2026-10-01"));
    assert!(
        releases[0].html.starts_with("<h2>Features</h2>"),
        "{}",
        releases[0].html
    );
    assert!(releases[0].html.contains("## not a heading"));
    assert_eq!(releases[1].version, "0.1.0");
    assert!(releases[1].html.contains("<h2>Bug Fixes</h2>"));
}
