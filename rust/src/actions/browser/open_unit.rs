use super::*;

#[test]
fn joins_the_path_under_the_base() {
    assert_eq!(
        join_url("http://localhost:8000", "/"),
        "http://localhost:8000/"
    );
    assert_eq!(
        join_url("http://localhost:8000/docs/", "/guides/"),
        "http://localhost:8000/docs/guides/"
    );
    assert_eq!(
        join_url("http://localhost:8000", "faq/"),
        "http://localhost:8000/faq/"
    );
}
