use super::*;
use crate::actions::language::resolve::{resolve_english, resolve_languages};
use crate::models::config::Config;
use crate::models::language::Language;

#[test]
fn formats_dates_for_readers_in_their_language() {
    let english = resolve_english(&Config::default(), "Docs");
    assert_eq!(format_date("2026-10-01", &english), "Oct 1, 2026");
    assert_eq!(format_date("soon", &english), "soon");
    let language = |code: &str| Language {
        code: code.to_owned(),
        label: code.to_owned(),
        ..Language::default()
    };
    let config = Config {
        languages: vec![language("de"), language("ja"), language("ru")],
        ..Config::default()
    };
    let languages = resolve_languages(&config, "Docs");
    let dates: Vec<String> = (languages.iter())
        .map(|language| format_date("2026-03-01", language))
        .collect();
    assert_eq!(dates, ["1. März 2026", "2026年3月1日", "1 мар. 2026 г."]);
}

#[test]
fn formats_dates_for_feeds() {
    assert_eq!(
        format_feed_date("2026-10-01").as_deref(),
        Some("Thu, 01 Oct 2026 00:00:00 +0000")
    );
    assert_eq!(
        format_feed_date("2024-02-29").as_deref(),
        Some("Thu, 29 Feb 2024 00:00:00 +0000")
    );
    assert_eq!(
        format_feed_date("1970-01-01").as_deref(),
        Some("Thu, 01 Jan 1970 00:00:00 +0000")
    );
}
