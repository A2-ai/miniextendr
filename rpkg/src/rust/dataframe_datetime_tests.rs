//! Datetime columns in `#[derive(DataFrameRow)]` rows.
//!
//! A `jiff` or `time` type spelled by its crate path (`jiff::Timestamp`,
//! `time::Date`, or behind `miniextendr_api::`) is one column, `Option<T>` an
//! NA-able one: POSIXct for instants, Date for civil dates, difftime for
//! `jiff::SignedDuration`. The writer and the generated reader agree, so the
//! frames round-trip.

use miniextendr_api::dataframe::{BuiltDataFrame, DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, miniextendr};
use miniextendr_api::{jiff, time};

/// One reading with a column per datetime type.
#[derive(DataFrameRow)]
pub struct Reading {
    pub id: i32,
    pub at: jiff::Timestamp,
    pub seen: Option<jiff::Timestamp>,
    pub zoned: Option<jiff::Zoned>,
    pub day: Option<jiff::civil::Date>,
    pub lag: Option<jiff::SignedDuration>,
    pub stamp: Option<time::OffsetDateTime>,
    pub due: Option<::miniextendr_api::time::Date>,
}

fn readings() -> Vec<Reading> {
    let at = jiff::Timestamp::from_second(0).expect("epoch");
    vec![
        Reading {
            id: 1,
            at,
            seen: Some(jiff::Timestamp::from_second(86_400).expect("one day")),
            zoned: Some(at.in_tz("Europe/Oslo").expect("bundled zone")),
            day: Some(jiff::civil::date(2024, 2, 29)),
            lag: Some(jiff::SignedDuration::from_secs(90)),
            stamp: Some(time::OffsetDateTime::from_unix_timestamp(3_600).expect("in range")),
            due: Some(
                time::Date::from_calendar_date(2024, time::Month::March, 1).expect("valid date"),
            ),
        },
        Reading {
            id: 2,
            at: jiff::Timestamp::from_second(60).expect("a minute"),
            seen: None,
            zoned: None,
            day: None,
            lag: None,
            stamp: None,
            due: None,
        },
    ]
}

/// Two readings; the second is `None` in every optional column.
#[miniextendr(noexport)]
pub fn df_datetime_rows() -> BuiltDataFrame {
    readings().into_dataframe().unwrap()
}

/// Read `df` into `Vec<Reading>` with the generated reader and write it back.
///
/// @param df A data.frame shaped like `df_datetime_rows()`.
#[miniextendr(noexport)]
pub fn df_datetime_roundtrip(df: DataFrame) -> BuiltDataFrame {
    Vec::<Reading>::from_dataframe(&df)
        .unwrap()
        .into_dataframe()
        .unwrap()
}

/// Variants spelling one datetime type two ways share one column.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum Shift {
    Start {
        at: jiff::Timestamp,
    },
    Stop {
        at: ::miniextendr_api::jiff::Timestamp,
        day: time::Date,
    },
}

fn shifts() -> Vec<Shift> {
    vec![
        Shift::Start {
            at: jiff::Timestamp::from_second(0).expect("epoch"),
        },
        Shift::Stop {
            at: jiff::Timestamp::from_second(7_200).expect("two hours"),
            day: time::Date::from_calendar_date(1970, time::Month::January, 2).expect("valid date"),
        },
    ]
}

/// A start and a stop; only the stop has a `day`.
#[miniextendr(noexport)]
pub fn df_datetime_enum_rows() -> BuiltDataFrame {
    shifts().into_dataframe().unwrap()
}

/// Read `df` into `Vec<Shift>` and write it back.
///
/// @param df A data.frame shaped like `df_datetime_enum_rows()`.
#[miniextendr(noexport)]
pub fn df_datetime_enum_roundtrip(df: DataFrame) -> BuiltDataFrame {
    Vec::<Shift>::from_dataframe(&df)
        .unwrap()
        .into_dataframe()
        .unwrap()
}
