const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// A `YYYY-MM-DD` date as readers see it, `October 1, 2026`; the input as is when it is
/// not one.
pub(crate) fn format_date(date: &str) -> String {
    match read_parts(date) {
        Some((year, month, day)) => {
            format!("{} {day}, {year}", MONTHS[(month - 1) as usize])
        }
        None => date.to_owned(),
    }
}

/// A `YYYY-MM-DD` date as RSS wants it, `Thu, 01 Oct 2026 00:00:00 +0000`.
pub(crate) fn format_feed_date(date: &str) -> Option<String> {
    let (year, month, day) = read_parts(date)?;
    let weekday = WEEKDAYS[count_days(year, month, day).rem_euclid(7) as usize];
    let month = &MONTHS[(month - 1) as usize][..3];
    Some(format!("{weekday}, {day:02} {month} {year} 00:00:00 +0000"))
}

fn read_parts(date: &str) -> Option<(i64, i64, i64)> {
    let mut parts = date
        .get(..10)?
        .split('-')
        .map(|part| part.parse::<i64>().ok());
    let (year, month, day) = (parts.next()??, parts.next()??, parts.next()??);
    ((1..=12).contains(&month) && (1..=31).contains(&day)).then_some((year, month, day))
}

// Days since 1970-01-01, a Thursday, shifted so 0 is a Monday (Howard Hinnant's algorithm)
fn count_days(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    era * 146097 + of_cycle - 719468 + 3
}

#[cfg(test)]
#[path = "format_unit.rs"]
mod unit;
