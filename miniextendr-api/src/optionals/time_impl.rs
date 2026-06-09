//! Integration with the `time` crate.
//!
//! Provides conversions between R date/time types and `time` crate types.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | `POSIXct` | `OffsetDateTime` | Seconds since epoch + timezone |
//! | `Date` | `time::Date` | Days since 1970-01-01 |
//!
//! # Features
//!
//! Enable this module with the `time` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["time"] }
//! ```
//!
//! # Fractional Seconds Policy
//!
//! R's POSIXct stores fractional seconds as floating-point. When converting to Rust:
//! - Fractional seconds are **truncated** (not rounded) to nanoseconds
//! - This matches typical timestamp handling expectations
//!
//! # Timezone Handling
//!
//! - POSIXct with `tzone = "UTC"` maps to UTC offset
//! - POSIXct without tzone or with empty tzone defaults to UTC
//! - Other timezones are converted assuming UTC (R doesn't store actual offset)
//!
//! **Note:** R's timezone handling is complex. For reliable timezone support,
//! consider storing timestamps as UTC and converting in R.
//!
//! # Example
//!
//! ```ignore
//! use time::OffsetDateTime;
//!
//! #[miniextendr]
//! fn now() -> OffsetDateTime {
//!     OffsetDateTime::now_utc()
//! }
//!
//! #[miniextendr]
//! fn days_since(date: time::Date) -> i32 {
//!     let today = OffsetDateTime::now_utc().date();
//!     (today - date).whole_days() as i32
//! }
//! ```

pub use time::{Date, OffsetDateTime};

use super::datetime_realsxp::impl_realsxp_datetime;
use crate::cached_class::{date_class_sexp, set_posixct_utc};
use crate::from_r::SexpError;
use crate::{SEXP, SexpExt};

/// Unix epoch as an OffsetDateTime constant.
const UNIX_EPOCH: OffsetDateTime = time::macros::datetime!(1970-01-01 0:00 UTC);

/// Unix epoch as a Date constant.
const UNIX_EPOCH_DATE: Date = time::macros::date!(1970 - 01 - 01);

// region: OffsetDateTime <-> POSIXct
//
// Floor-based split into whole seconds + nanoseconds — correct for negative
// timestamps (-1.2s → floor=-2, fract=0.8 → -2s + 800_000_000ns = -1.2s).

impl_realsxp_datetime!(
    OffsetDateTime,
    "POSIXct",
    |secs: f64| {
        // `whole_secs` saturates on overflow (no UB); `nanos` is bounded to
        // `[0, 1e9) < i32::MAX` because `secs - secs.floor() ∈ [0, 1)`. Any
        // out-of-range timestamp is rejected by `checked_add` below.
        #[allow(clippy::cast_possible_truncation)]
        let whole_secs = secs.floor() as i64;
        #[allow(clippy::cast_possible_truncation)]
        let nanos = ((secs - secs.floor()) * 1_000_000_000.0) as i32;
        let duration = time::Duration::new(whole_secs, nanos);
        UNIX_EPOCH
            .checked_add(duration)
            .ok_or_else(|| SexpError::InvalidValue("timestamp out of range".to_string()))
    },
    |dt: OffsetDateTime| {
        let duration = dt - UNIX_EPOCH;
        duration.whole_seconds() as f64
            + (f64::from(duration.subsec_nanoseconds()) / 1_000_000_000.0)
    },
    |sexp: SEXP| set_posixct_utc(sexp)
);

// endregion

// region: Date <-> R Date

impl_realsxp_datetime!(
    Date,
    "Date",
    |days: f64| {
        // Saturates on overflow (no UB); out-of-range dates are rejected by
        // `checked_add` below.
        #[allow(clippy::cast_possible_truncation)]
        let days_i64 = days.trunc() as i64;
        let duration = time::Duration::days(days_i64);
        UNIX_EPOCH_DATE
            .checked_add(duration)
            .ok_or_else(|| SexpError::InvalidValue("date out of range".to_string()))
    },
    |d: Date| (d - UNIX_EPOCH_DATE).whole_days() as f64,
    |sexp: SEXP| sexp.set_class(date_class_sexp())
);

// endregion

// region: RDuration adapter trait

pub use time::Duration;

/// Adapter trait for [`time::Duration`].
///
/// Provides methods to inspect and manipulate durations from R.
/// Automatically implemented for `time::Duration`.
///
/// # Methods
///
/// - `as_seconds_f64()` - Total duration as floating-point seconds
/// - `as_milliseconds()` - Total duration in milliseconds (i64)
/// - `whole_days()` - Number of whole days
/// - `whole_hours()` - Number of whole hours
/// - `whole_minutes()` - Number of whole minutes
/// - `whole_seconds()` - Number of whole seconds
/// - `is_negative()` - Check if duration is negative
/// - `is_zero()` - Check if duration is zero
/// - `abs()` - Absolute value of duration
///
/// # Example
///
/// ```rust,ignore
/// use time::Duration;
/// use miniextendr_api::time_impl::RDuration;
///
/// #[derive(ExternalPtr)]
/// struct MyDuration(Duration);
///
/// #[miniextendr]
/// impl RDuration for MyDuration {}
/// ```
///
/// In R:
/// ```r
/// d <- MyDuration$new(...)
/// d$as_seconds_f64()  # e.g., 3661.5
/// d$whole_hours()     # e.g., 1
/// d$is_negative()     # FALSE
/// ```
pub trait RDuration {
    /// Get the total duration as floating-point seconds.
    fn as_seconds_f64(&self) -> f64;

    /// Get the total duration in milliseconds.
    fn as_milliseconds(&self) -> i64;

    /// Get the number of whole days in the duration.
    fn whole_days(&self) -> i64;

    /// Get the number of whole hours in the duration.
    fn whole_hours(&self) -> i64;

    /// Get the number of whole minutes in the duration.
    fn whole_minutes(&self) -> i64;

    /// Get the number of whole seconds in the duration.
    fn whole_seconds(&self) -> i64;

    /// Get the subsecond nanoseconds component.
    fn subsec_nanoseconds(&self) -> i32;

    /// Check if the duration is negative.
    fn is_negative(&self) -> bool;

    /// Check if the duration is zero.
    fn is_zero(&self) -> bool;

    /// Get the absolute value of this duration.
    fn abs(&self) -> Duration;
}

impl RDuration for Duration {
    fn as_seconds_f64(&self) -> f64 {
        Duration::as_seconds_f64(*self)
    }

    fn as_milliseconds(&self) -> i64 {
        // whole_milliseconds() returns i128; clamp to i64 range (saturating,
        // not truncating) to mirror the jiff `SignedDuration` behaviour.
        // SAFETY (lint): after clamping into `[i64::MIN, i64::MAX]` the value
        // is representable as `i64`, so the narrowing cannot truncate.
        #[allow(clippy::cast_possible_truncation)]
        let ms = Duration::whole_milliseconds(*self)
            .clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
        ms
    }

    fn whole_days(&self) -> i64 {
        Duration::whole_days(*self)
    }

    fn whole_hours(&self) -> i64 {
        Duration::whole_hours(*self)
    }

    fn whole_minutes(&self) -> i64 {
        Duration::whole_minutes(*self)
    }

    fn whole_seconds(&self) -> i64 {
        Duration::whole_seconds(*self)
    }

    fn subsec_nanoseconds(&self) -> i32 {
        Duration::subsec_nanoseconds(*self)
    }

    fn is_negative(&self) -> bool {
        Duration::is_negative(*self)
    }

    fn is_zero(&self) -> bool {
        Duration::is_zero(*self)
    }

    fn abs(&self) -> Duration {
        Duration::abs(*self)
    }
}
// endregion

// region: RDateTimeFormat adapter trait

/// Adapter trait for formatting and parsing datetime types.
///
/// Provides format/parse operations for `time::OffsetDateTime` and `time::Date`.
/// Uses `time` crate's format description syntax.
///
/// # Format Syntax
///
/// The format string uses bracketed component specifiers:
/// - `[year]` - 4-digit year
/// - `[month]` - Month (01-12)
/// - `[day]` - Day of month (01-31)
/// - `[hour]` - Hour (00-23)
/// - `[minute]` - Minute (00-59)
/// - `[second]` - Second (00-59)
/// - `[subsecond]` - Fractional seconds
/// - `[offset_hour]`, `[offset_minute]` - Timezone offset
///
/// See `time` crate documentation for full format specification.
///
/// # Example
///
/// ```rust,ignore
/// use time::OffsetDateTime;
/// use miniextendr_api::time_impl::RDateTimeFormat;
///
/// let now = OffsetDateTime::now_utc();
/// let formatted = now.format("[year]-[month]-[day] [hour]:[minute]:[second]");
/// // e.g., "2024-01-15 14:30:00"
///
/// let parsed = OffsetDateTime::r_parse(
///     "2024-01-15 14:30:00 +00:00:00",
///     "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
/// );
/// ```
pub trait RDateTimeFormat: Sized {
    /// Format using a format description string.
    ///
    /// Returns the formatted string, or an error message on invalid format.
    fn format(&self, fmt: &str) -> Result<String, String>;

    /// Parse from a string using a format description.
    ///
    /// Returns the parsed value, or an error message on parse failure.
    fn parse(s: &str, fmt: &str) -> Result<Self, String>;
}

impl RDateTimeFormat for OffsetDateTime {
    fn format(&self, fmt: &str) -> Result<String, String> {
        let format = time::format_description::parse(fmt).map_err(|e| e.to_string())?;
        OffsetDateTime::format(*self, &format).map_err(|e| e.to_string())
    }

    fn parse(s: &str, fmt: &str) -> Result<Self, String> {
        let format = time::format_description::parse(fmt).map_err(|e| e.to_string())?;
        OffsetDateTime::parse(s, &format).map_err(|e| e.to_string())
    }
}

impl RDateTimeFormat for Date {
    fn format(&self, fmt: &str) -> Result<String, String> {
        let format = time::format_description::parse(fmt).map_err(|e| e.to_string())?;
        Date::format(*self, &format).map_err(|e| e.to_string())
    }

    fn parse(s: &str, fmt: &str) -> Result<Self, String> {
        let format = time::format_description::parse(fmt).map_err(|e| e.to_string())?;
        Date::parse(s, &format).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_constants_correct() {
        assert_eq!(UNIX_EPOCH.year(), 1970);
        assert_eq!(UNIX_EPOCH.month(), time::Month::January);
        assert_eq!(UNIX_EPOCH.day(), 1);

        assert_eq!(UNIX_EPOCH_DATE.year(), 1970);
        assert_eq!(UNIX_EPOCH_DATE.month(), time::Month::January);
        assert_eq!(UNIX_EPOCH_DATE.day(), 1);
    }

    #[test]
    fn date_arithmetic() {
        let d = UNIX_EPOCH_DATE;
        let d2 = d.checked_add(time::Duration::days(365)).unwrap();
        assert_eq!(d2.year(), 1971);
    }

    #[test]
    fn datetime_arithmetic() {
        let dt = UNIX_EPOCH;
        let dt2 = dt.checked_add(time::Duration::hours(24)).unwrap();
        assert_eq!(dt2.day(), 2);
    }

    #[test]
    fn rduration_seconds() {
        let d = Duration::new(3661, 500_000_000); // 1 hour, 1 minute, 1.5 seconds
        assert!((d.as_seconds_f64() - 3661.5).abs() < 0.001);
        assert_eq!(d.whole_seconds(), 3661);
        assert_eq!(d.subsec_nanoseconds(), 500_000_000);
    }

    #[test]
    fn rduration_components() {
        let d = Duration::days(2) + Duration::hours(3) + Duration::minutes(4);
        assert_eq!(d.whole_days(), 2);
        assert_eq!(d.whole_hours(), 51); // 2*24 + 3
        assert_eq!(d.whole_minutes(), 3064); // 51*60 + 4
    }

    #[test]
    fn rduration_negative() {
        let positive = Duration::hours(1);
        let negative = Duration::hours(-1);
        let zero = Duration::ZERO;

        assert!(!positive.is_negative());
        assert!(negative.is_negative());
        assert!(!zero.is_negative());
        assert!(zero.is_zero());
        assert!(!positive.is_zero());

        assert_eq!(negative.abs(), positive);
    }

    #[test]
    fn rduration_milliseconds() {
        let d = Duration::milliseconds(1500);
        assert_eq!(d.as_milliseconds(), 1500);
        assert_eq!(d.whole_seconds(), 1);
    }

    #[test]
    fn rdatetimeformat_offsetdatetime() {
        let dt = time::macros::datetime!(2024-01-15 14:30:00 UTC);

        // Test formatting
        let formatted =
            <OffsetDateTime as RDateTimeFormat>::format(&dt, "[year]-[month]-[day]").unwrap();
        assert_eq!(formatted, "2024-01-15");

        let formatted_time =
            <OffsetDateTime as RDateTimeFormat>::format(&dt, "[hour]:[minute]:[second]").unwrap();
        assert_eq!(formatted_time, "14:30:00");

        // Test parsing
        let parsed = <OffsetDateTime as RDateTimeFormat>::parse(
            "2024-01-15 14:30:00 +00:00:00",
            "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]",
        )
        .unwrap();
        assert_eq!(parsed.year(), 2024);
        assert_eq!(parsed.month(), time::Month::January);
        assert_eq!(parsed.day(), 15);
        assert_eq!(parsed.hour(), 14);
        assert_eq!(parsed.minute(), 30);
    }

    #[test]
    fn rdatetimeformat_date() {
        let d = time::macros::date!(2024 - 06 - 20);

        // Test formatting
        let formatted = <Date as RDateTimeFormat>::format(&d, "[year]-[month]-[day]").unwrap();
        assert_eq!(formatted, "2024-06-20");

        // Test parsing
        let parsed =
            <Date as RDateTimeFormat>::parse("2024-06-20", "[year]-[month]-[day]").unwrap();
        assert_eq!(parsed.year(), 2024);
        assert_eq!(parsed.month(), time::Month::June);
        assert_eq!(parsed.day(), 20);
    }

    #[test]
    fn rdatetimeformat_errors() {
        // Invalid format string
        let result = <OffsetDateTime as RDateTimeFormat>::format(&UNIX_EPOCH, "[invalid]");
        assert!(result.is_err());

        // Invalid input for format
        let result = <Date as RDateTimeFormat>::parse("not a date", "[year]-[month]-[day]");
        assert!(result.is_err());
    }
}
// endregion
