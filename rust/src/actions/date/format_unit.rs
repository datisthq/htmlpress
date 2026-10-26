use super::*;

#[test]
fn formats_dates_for_readers_and_feeds() {
    assert_eq!(format_date("2026-10-01"), "October 1, 2026");
    assert_eq!(format_date("soon"), "soon");
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
