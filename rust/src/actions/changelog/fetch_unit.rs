use super::*;

#[test]
fn reads_a_repository_from_its_url() {
    let read = |source: &str| read_repository(source);
    assert_eq!(
        read("https://github.com/acme/tool"),
        Some(("acme".into(), "tool".into()))
    );
    assert_eq!(
        read("github.com/acme/tool.git#readme"),
        Some(("acme".into(), "tool".into()))
    );
    assert_eq!(
        read("https://www.github.com/acme/tool/releases"),
        Some(("acme".into(), "tool".into()))
    );
    assert_eq!(read("CHANGELOG.md"), None);
    assert_eq!(read("https://github.com/acme"), None);
    assert_eq!(read("https://gitlab.com/acme/tool"), None);
}

#[test]
fn turns_releases_into_pages_without_drafts() {
    let releases = json!([
        {
            "tag_name": "v1.2.0",
            "published_at": "2026-10-01T12:00:00Z",
            "body": "## v1.2.0\n\n### Features\n\n* fast",
            "html_url": "https://github.com/acme/tool/releases/tag/v1.2.0",
        },
        { "tag_name": "v2.0.0", "draft": true, "body": "" },
    ]);
    let releases = read_releases(&releases);
    assert_eq!(releases.len(), 1);
    let release = &releases[0];
    assert_eq!(
        (release.slug.as_str(), release.date.as_deref()),
        ("v1-2-0", Some("2026-10-01"))
    );
    assert!(release.html.contains("<h2>Features</h2>") && !release.html.contains("<h1"));
    assert!(
        release
            .url
            .as_deref()
            .is_some_and(|url| url.ends_with("v1.2.0"))
    );
}

#[test]
fn shows_release_html_as_text_and_drops_script_links() {
    let releases = json!([{
        "tag_name": "v1",
        "body": "<script>alert(1)</script>\n\n[x](javascript:alert(1)) <img src=x onerror=alert(1)>",
    }]);
    let html = &read_releases(&releases)[0].html;
    assert!(
        !html.contains("<script") && !html.contains("<img"),
        "{html}"
    );
    assert!(
        html.contains("&lt;script&gt;") && html.contains("href=\"#\""),
        "{html}"
    );
}
