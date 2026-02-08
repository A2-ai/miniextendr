//! Time adapter tests
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
    time::macros::date!(1970 - 01 - 01)
}

/// Epoch datetime (1970-01-01 00:00:00 UTC) roundtrip
/// @noRd
#[miniextendr]
pub fn time_epoch_posixct() -> OffsetDateTime {
    time::macros::datetime!(1970-01-01 0:00 UTC)
}

/// Date in distant past (1900-01-01)
/// @noRd
#[miniextendr]
pub fn time_distant_past() -> Date {
    time::macros::date!(1900 - 01 - 01)
}

/// Format a date with custom format string
/// @noRd
#[miniextendr]
pub fn time_format_date(date: Date) -> String {
    use miniextendr_api::time_impl::RDateTimeFormat;
    date.format("[year]-[month]-[day]").unwrap_or_else(|e| e)
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
