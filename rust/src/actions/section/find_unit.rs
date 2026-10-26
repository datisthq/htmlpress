use super::*;

fn create_section(prefix: &str) -> Section {
    Section {
        title: prefix.into(),
        prefix: Some(prefix.to_owned()),
        ..Section::default()
    }
}

#[test]
fn picks_the_longest_matching_prefix() {
    let sections = [
        create_section("/"),
        create_section("/guides/"),
        create_section("/guides/api/"),
    ];
    assert_eq!(find_section(&sections, "/intro.html"), Some(0));
    assert_eq!(find_section(&sections, "/guides/"), Some(1));
    assert_eq!(find_section(&sections, "/guides/api/auth.html"), Some(2));
    assert_eq!(find_section(&sections[1..], "/intro.html"), None);
}

#[test]
fn matches_a_custom_url_only_on_its_own_pages() {
    assert!(is_custom_current("/", "/"));
    assert!(!is_custom_current("/", "/guide.html"));
    assert!(is_custom_current("/blog/", "/blog/post.html"));
    assert!(is_custom_current("/changelog.html#v1", "/changelog.html"));
    assert!(!is_custom_current("https://github.com/x", "/"));
}
