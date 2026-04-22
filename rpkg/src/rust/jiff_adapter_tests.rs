//! Jiff adapter tests — roundtrip and extraction fixtures for all jiff types.
use miniextendr_api::cached_class::set_posixct_utc;
use miniextendr_api::ffi::{Rf_protect, Rf_unprotect, SEXP};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::miniextendr;
use miniextendr_api::{JiffDate, JiffTimestampVec, SignedDuration, Timestamp, Zoned};
use std::sync::Arc;

// region: Timestamp (POSIXct UTC)

/// Roundtrip a UTC POSIXct through Timestamp.
/// @param ts POSIXct (UTC) scalar from R.
#[miniextendr]
pub fn jiff_roundtrip_timestamp(ts: Timestamp) -> Timestamp {
    ts
}

/// Roundtrip a vector of UTC POSIXct values through Vec<Timestamp>.
/// @param ts POSIXct (UTC) vector from R.
#[miniextendr]
pub fn jiff_roundtrip_timestamp_vec(ts: Vec<Timestamp>) -> Vec<Timestamp> {
    ts
}

/// Return the Unix epoch as a Timestamp.
#[miniextendr]
pub fn jiff_epoch_timestamp() -> Timestamp {
    Timestamp::UNIX_EPOCH
}

/// Return a negative Timestamp (1960-01-01 00:00:00 UTC) to test floor-based split.
#[miniextendr]
pub fn jiff_negative_timestamp() -> Timestamp {
    // 1960-01-01 00:00:00 UTC = -315619200 seconds
    Timestamp::new(-315_619_200, 0).expect("valid timestamp")
}

/// Return a Timestamp with sub-second precision (1.5 seconds after epoch).
#[miniextendr]
pub fn jiff_fractional_timestamp() -> Timestamp {
    Timestamp::new(1, 500_000_000).expect("valid timestamp")
}

/// Return a Timestamp just before midnight negative (tests floor correctness).
/// -0.5 seconds before epoch = -1 second floor + 0.5 seconds subsec.
#[miniextendr]
pub fn jiff_half_second_before_epoch() -> Timestamp {
    Timestamp::new(-1, 500_000_000).expect("valid timestamp")
}

/// Extract seconds-since-epoch from a Timestamp.
/// @param ts POSIXct (UTC) scalar from R.
#[miniextendr]
pub fn jiff_timestamp_seconds(ts: Timestamp) -> f64 {
    ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0)
}

/// Roundtrip Option<Timestamp> (NULL → NA mapping).
/// @param ts Nullable POSIXct scalar from R.
#[miniextendr]
pub fn jiff_option_timestamp(ts: Option<Timestamp>) -> Option<Timestamp> {
    ts
}

// endregion

// region: Zoned (POSIXct + tzone)

/// Roundtrip a POSIXct with timezone through Zoned.
/// @param zdt POSIXct with tzone attribute from R.
#[miniextendr]
pub fn jiff_roundtrip_zoned(zdt: Zoned) -> Zoned {
    zdt
}

/// Extract the IANA timezone name from a Zoned.
/// @param zdt POSIXct with tzone attribute from R.
#[miniextendr]
pub fn jiff_zoned_tz_name(zdt: Zoned) -> String {
    zdt.time_zone().iana_name().unwrap_or("").to_string()
}

/// Extract the year from a Zoned datetime.
/// @param zdt POSIXct with tzone attribute from R.
#[miniextendr]
pub fn jiff_zoned_year(zdt: Zoned) -> i32 {
    zdt.year() as i32
}

/// Extract the month from a Zoned datetime (1–12).
/// @param zdt POSIXct with tzone attribute from R.
#[miniextendr]
pub fn jiff_zoned_month(zdt: Zoned) -> i32 {
    zdt.month() as i32
}

/// Roundtrip a Vec<Zoned> through R POSIXct.
/// @param zdts POSIXct vector with tzone attribute from R.
#[miniextendr]
pub fn jiff_roundtrip_zoned_vec(zdts: Vec<Zoned>) -> Vec<Zoned> {
    zdts
}

// endregion

// region: civil::Date (R Date)

/// Roundtrip a civil::Date through R Date.
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_roundtrip_date(date: JiffDate) -> JiffDate {
    date
}

/// Roundtrip a Vec<civil::Date> through R Date vector.
/// @param dates Date vector from R.
#[miniextendr]
pub fn jiff_roundtrip_date_vec(dates: Vec<JiffDate>) -> Vec<JiffDate> {
    dates
}

/// Return the Unix epoch date (1970-01-01).
#[miniextendr]
pub fn jiff_epoch_date() -> JiffDate {
    JiffDate::new(1970, 1, 1).expect("valid date")
}

/// Return a date in the distant past (1900-03-01).
#[miniextendr]
pub fn jiff_distant_past_date() -> JiffDate {
    JiffDate::new(1900, 3, 1).expect("valid date")
}

/// Extract the year from a civil::Date.
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_year(date: JiffDate) -> i32 {
    date.year() as i32
}

/// Extract the month from a civil::Date (1–12).
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_month(date: JiffDate) -> i32 {
    date.month() as i32
}

/// Extract the day from a civil::Date (1–31).
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_day(date: JiffDate) -> i32 {
    date.day() as i32
}

// endregion

// region: SignedDuration (difftime)

/// Roundtrip a difftime through SignedDuration.
/// @param dur difftime scalar from R (secs).
#[miniextendr]
pub fn jiff_roundtrip_duration(dur: SignedDuration) -> SignedDuration {
    dur
}

/// Return a positive SignedDuration (3600 seconds = 1 hour).
#[miniextendr]
pub fn jiff_one_hour_duration() -> SignedDuration {
    SignedDuration::from_secs(3600)
}

/// Return a negative SignedDuration (-1.5 seconds).
#[miniextendr]
pub fn jiff_negative_duration() -> SignedDuration {
    SignedDuration::new(-1, 500_000_000)
}

/// Extract seconds from a SignedDuration.
/// @param dur difftime scalar from R.
#[miniextendr]
pub fn jiff_duration_secs(dur: SignedDuration) -> f64 {
    dur.as_secs() as f64 + (dur.subsec_nanos() as f64 / 1_000_000_000.0)
}

// endregion

// region: ALTREP (JiffTimestampVec)

/// Create a JiffTimestampVec ALTREP containing n timestamps (0..n seconds after epoch).
///
/// Returns a POSIXct (UTC) ALTREP REALSXP vector backed by Arc<Vec<Timestamp>>.
/// @param n Number of elements.
#[miniextendr]
pub fn jiff_altrep_timestamps(n: i32) -> SEXP {
    let data: Vec<Timestamp> = (0..n)
        .map(|i| Timestamp::new(i as i64, 0).expect("valid timestamp"))
        .collect();
    let vec = JiffTimestampVec {
        data: Arc::new(data),
    };
    // Create the ALTREP SEXP and set POSIXct class + UTC tzone.
    let altrep = vec.into_sexp();
    unsafe {
        Rf_protect(altrep);
        set_posixct_utc(altrep);
        Rf_unprotect(1);
    }
    altrep
}

/// Return the length of a POSIXct vector materialized from Timestamp.
/// @param x POSIXct vector (may be ALTREP).
#[miniextendr]
pub fn jiff_altrep_len(x: Vec<Timestamp>) -> i32 {
    x.len() as i32
}

/// Check the i-th element (0-based) of a Timestamp POSIXct vector as f64 secs.
/// @param x POSIXct vector (may be ALTREP).
/// @param i 0-based index.
#[miniextendr]
pub fn jiff_altrep_elt(x: Vec<Timestamp>, i: i32) -> f64 {
    let ts = &x[i as usize];
    ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0)
}

// endregion
