//! Time adapter tests
use miniextendr_api::time;
use miniextendr_api::time_impl::{Date, OffsetDateTime};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn time_roundtrip_posixct(dt: OffsetDateTime) -> OffsetDateTime {
    dt
}

/// @noRd
#[miniextendr]
pub fn time_roundtrip_date(date: Date) -> Date {
    date
}

/// @noRd
#[miniextendr]
pub fn time_get_year(date: Date) -> i32 {
    date.year()
}

/// @noRd
#[miniextendr]
pub fn time_get_month(date: Date) -> i32 {
    date.month() as i32
}

/// @noRd
#[miniextendr]
pub fn time_get_day(date: Date) -> i32 {
    date.day() as i32
}

/// Unix epoch time (1970-01-01) roundtrip
/// @noRd
#[miniextendr]
pub fn time_epoch_date() -> Date {
    Date::from_calendar_date(1970, time::Month::January, 1).unwrap()
}

/// Epoch datetime (1970-01-01 00:00:00 UTC) roundtrip
/// @noRd
#[miniextendr]
pub fn time_epoch_posixct() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(0).unwrap()
}

/// Date in distant past (1900-01-01)
/// @noRd
#[miniextendr]
pub fn time_distant_past() -> Date {
    Date::from_calendar_date(1900, time::Month::January, 1).unwrap()
}

/// Format a date as YYYY-MM-DD
/// @noRd
#[miniextendr]
pub fn time_format_date(date: Date) -> String {
    let fmt = time::format_description::parse("[year]-[month]-[day]").expect("valid format");
    date.format(&fmt).unwrap_or_else(|e| e.to_string())
}

miniextendr_module! {
    mod time_adapter_tests;
    fn time_roundtrip_posixct;
    fn time_roundtrip_date;
    fn time_get_year;
    fn time_get_month;
    fn time_get_day;
    fn time_epoch_date;
    fn time_epoch_posixct;
    fn time_distant_past;
    fn time_format_date;
}
