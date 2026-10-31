use super::parse_meta;

#[test]
fn reads_quoted_bare_and_flag_pairs() {
    assert_eq!(
        parse_meta(r#"title="a b.js" lines=2-3 line-numbers collapse='10'"#),
        [
            ("title".to_owned(), "a b.js".to_owned()),
            ("lines".to_owned(), "2-3".to_owned()),
            ("line-numbers".to_owned(), String::new()),
            ("collapse".to_owned(), "10".to_owned()),
        ]
    );
}
