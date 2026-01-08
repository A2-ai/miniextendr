//! Time adapter tests
use miniextendr_api::time_impl::{Date, OffsetDateTime};
use miniextendr_api::{miniextendr, miniextendr_module};

#[miniextendr]
pub fn time_roundtrip_posixct(dt: OffsetDateTime) -> OffsetDateTime {
    dt
}

#[miniextendr]
pub fn time_roundtrip_date(date: Date) -> Date {
    date
}

#[miniextendr]
pub fn time_get_year(date: Date) -> i32 {
    date.year()
}

#[miniextendr]
pub fn time_get_month(date: Date) -> i32 {
    date.month() as i32
}

#[miniextendr]
pub fn time_get_day(date: Date) -> i32 {
    date.day() as i32
}

miniextendr_module! {
    mod time_adapter_tests;
    fn time_roundtrip_posixct;
    fn time_roundtrip_date;
    fn time_get_year;
    fn time_get_month;
    fn time_get_day;
}
