//! Jiff adapter tests — roundtrip and extraction fixtures for all jiff types.
use miniextendr_api::cached_class::set_posixct_utc;
use miniextendr_api::ffi::{Rf_protect, Rf_unprotect, SEXP};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::miniextendr;
use miniextendr_api::{
    AltRealData, AltrepLen, JiffDate, JiffDateTime, JiffTime, JiffTimestampVec, SignedDuration,
    Span, Timestamp, Zoned,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

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

// region: Span ExternalPtr (exercises RSpan adapter trait)

/// An ExternalPtr wrapper around a jiff `Span`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct JiffSpan(pub Span);

type JiffSpanPtr = miniextendr_api::externalptr::ExternalPtr<JiffSpan>;

/// Create a new JiffSpan ExternalPtr from year/month/day components.
/// @param years Number of years (integer).
/// @param months Number of months (integer).
/// @param days Number of days (integer).
#[miniextendr]
pub fn jiff_span_new(years: i32, months: i32, days: i32) -> JiffSpanPtr {
    let span = Span::new()
        .years(years as i64)
        .months(months as i64)
        .days(days as i64);
    miniextendr_api::externalptr::ExternalPtr::new(JiffSpan(span))
}

/// Extract the years component from a JiffSpan.
/// @param span ExternalPtr wrapping a JiffSpan.
#[miniextendr]
pub fn jiff_span_years(span: JiffSpanPtr) -> i32 {
    i32::from(span.0.get_years())
}

/// Extract the months component from a JiffSpan.
/// @param span ExternalPtr wrapping a JiffSpan.
#[miniextendr]
pub fn jiff_span_months(span: JiffSpanPtr) -> i32 {
    span.0.get_months()
}

/// Extract the days component from a JiffSpan.
/// @param span ExternalPtr wrapping a JiffSpan.
#[miniextendr]
pub fn jiff_span_days(span: JiffSpanPtr) -> i32 {
    span.0.get_days()
}

/// Check whether a JiffSpan is zero.
/// @param span ExternalPtr wrapping a JiffSpan.
#[miniextendr]
pub fn jiff_span_is_zero(span: JiffSpanPtr) -> bool {
    span.0.is_zero()
}

/// Check whether a JiffSpan is negative.
/// @param span ExternalPtr wrapping a JiffSpan.
#[miniextendr]
pub fn jiff_span_is_negative(span: JiffSpanPtr) -> bool {
    span.0.is_negative()
}

// endregion

// region: civil::DateTime fixtures (exercises RDateTime adapter trait)

/// An ExternalPtr wrapper around a jiff civil `DateTime`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct JiffDateTimePtr(pub JiffDateTime);

type DateTimePtrType = miniextendr_api::externalptr::ExternalPtr<JiffDateTimePtr>;

/// Build a civil::DateTime ExternalPtr from year/month/day/hour/minute/second components.
/// @param year Year.
/// @param month Month (1–12).
/// @param day Day (1–31).
/// @param hour Hour (0–23).
/// @param minute Minute (0–59).
/// @param second Second (0–59).
#[miniextendr]
pub fn jiff_datetime_new(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
) -> DateTimePtrType {
    let dt = JiffDateTime::constant(
        year as i16,
        month as i8,
        day as i8,
        hour as i8,
        minute as i8,
        second as i8,
        0,
    );
    miniextendr_api::externalptr::ExternalPtr::new(JiffDateTimePtr(dt))
}

/// Extract the year from a civil::DateTime ExternalPtr.
/// @param dt ExternalPtr wrapping a JiffDateTimePtr.
#[miniextendr]
pub fn jiff_datetime_year(dt: DateTimePtrType) -> i32 {
    i32::from(dt.0.year())
}

/// Extract the month from a civil::DateTime ExternalPtr (1–12).
/// @param dt ExternalPtr wrapping a JiffDateTimePtr.
#[miniextendr]
pub fn jiff_datetime_month(dt: DateTimePtrType) -> i32 {
    i32::from(dt.0.month())
}

/// Extract the day from a civil::DateTime ExternalPtr (1–31).
/// @param dt ExternalPtr wrapping a JiffDateTimePtr.
#[miniextendr]
pub fn jiff_datetime_day(dt: DateTimePtrType) -> i32 {
    i32::from(dt.0.day())
}

/// Extract the hour from a civil::DateTime ExternalPtr (0–23).
/// @param dt ExternalPtr wrapping a JiffDateTimePtr.
#[miniextendr]
pub fn jiff_datetime_hour(dt: DateTimePtrType) -> i32 {
    i32::from(dt.0.hour())
}

// endregion

// region: civil::Time fixtures (exercises RTime adapter trait)

/// An ExternalPtr wrapper around a jiff civil `Time`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct JiffTimePtr(pub JiffTime);

type TimePtrType = miniextendr_api::externalptr::ExternalPtr<JiffTimePtr>;

/// Build a civil::Time ExternalPtr from hour/minute/second components.
/// @param hour Hour (0–23).
/// @param minute Minute (0–59).
/// @param second Second (0–59).
#[miniextendr]
pub fn jiff_time_new(hour: i32, minute: i32, second: i32) -> TimePtrType {
    let t = JiffTime::constant(hour as i8, minute as i8, second as i8, 0);
    miniextendr_api::externalptr::ExternalPtr::new(JiffTimePtr(t))
}

/// Extract the hour from a civil::Time ExternalPtr (0–23).
/// @param t ExternalPtr wrapping a JiffTimePtr.
#[miniextendr]
pub fn jiff_time_hour(t: TimePtrType) -> i32 {
    i32::from(t.0.hour())
}

/// Extract the minute from a civil::Time ExternalPtr (0–59).
/// @param t ExternalPtr wrapping a JiffTimePtr.
#[miniextendr]
pub fn jiff_time_minute(t: TimePtrType) -> i32 {
    i32::from(t.0.minute())
}

/// Extract the second from a civil::Time ExternalPtr (0–59).
/// @param t ExternalPtr wrapping a JiffTimePtr.
#[miniextendr]
pub fn jiff_time_second(t: TimePtrType) -> i32 {
    i32::from(t.0.second())
}

// endregion

// region: RDate adapter trait — calendar helpers

use miniextendr_api::{RDate, RTimestamp, RZoned};

/// Return the ISO weekday of a civil::Date (1=Mon … 7=Sun).
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_weekday(date: JiffDate) -> i32 {
    <JiffDate as RDate>::weekday(&date)
}

/// Return the ordinal day-of-year of a civil::Date (1–366).
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_day_of_year(date: JiffDate) -> i32 {
    <JiffDate as RDate>::day_of_year(&date)
}

/// Return the first day of the month for a civil::Date.
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_first_of_month(date: JiffDate) -> JiffDate {
    <JiffDate as RDate>::first_of_month(&date)
}

/// Return the last day of the month for a civil::Date.
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_last_of_month(date: JiffDate) -> JiffDate {
    <JiffDate as RDate>::last_of_month(&date)
}

/// Return the day after a civil::Date, or error if out of range.
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_tomorrow(date: JiffDate) -> Result<JiffDate, String> {
    <JiffDate as RDate>::tomorrow(&date)
}

/// Return the day before a civil::Date, or error if out of range.
/// @param date Date scalar from R.
#[miniextendr]
pub fn jiff_date_yesterday(date: JiffDate) -> Result<JiffDate, String> {
    <JiffDate as RDate>::yesterday(&date)
}

// endregion

// region: RZoned adapter trait — start_of_day + strftime

/// Return the start of day for a Zoned datetime (preserves timezone).
/// @param zdt POSIXct with tzone attribute from R.
#[miniextendr]
pub fn jiff_zoned_start_of_day(zdt: Zoned) -> Result<Zoned, String> {
    <Zoned as RZoned>::start_of_day(&zdt)
}

/// Format a Zoned datetime using a strftime format string.
/// @param zdt POSIXct with tzone attribute from R.
/// @param fmt strftime format string.
#[miniextendr]
pub fn jiff_zoned_strftime(zdt: Zoned, fmt: &str) -> String {
    <Zoned as RZoned>::strftime(&zdt, fmt)
}

// endregion

// region: RTimestamp adapter trait — strftime + as_millisecond

/// Format a Timestamp (UTC) using a strftime format string.
/// @param ts POSIXct (UTC) scalar from R.
/// @param fmt strftime format string.
#[miniextendr]
pub fn jiff_timestamp_strftime(ts: Timestamp, fmt: &str) -> String {
    <Timestamp as RTimestamp>::strftime(&ts, fmt)
}

/// Return the milliseconds-since-Unix-epoch for a Timestamp.
/// @param ts POSIXct (UTC) scalar from R.
#[miniextendr]
pub fn jiff_timestamp_as_millisecond(ts: Timestamp) -> f64 {
    <Timestamp as RTimestamp>::as_millisecond(&ts) as f64
}

// endregion

// region: ALTREP laziness counter (item 9)

static ELT_COUNTER: OnceLock<Arc<AtomicUsize>> = OnceLock::new();

fn elt_counter() -> Arc<AtomicUsize> {
    ELT_COUNTER.get_or_init(|| Arc::new(AtomicUsize::new(0))).clone()
}

#[derive(miniextendr_api::AltrepReal)]
#[altrep(class = "JiffTimestampVecCounted", manual)]
pub struct JiffTimestampVecCounted {
    pub data: Arc<Vec<Timestamp>>,
    pub counter: Arc<AtomicUsize>,
}

impl AltrepLen for JiffTimestampVecCounted {
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl AltRealData for JiffTimestampVecCounted {
    fn elt(&self, i: usize) -> f64 {
        self.counter.fetch_add(1, Ordering::Relaxed);
        let ts = &self.data[i];
        ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0)
    }
}

/// Create a JiffTimestampVecCounted ALTREP and reset the elt call counter.
/// @param n Number of elements (each is i seconds after epoch).
#[miniextendr]
pub fn jiff_counted_altrep(n: i32) -> SEXP {
    let counter = elt_counter();
    counter.store(0, Ordering::Relaxed);
    let data: Vec<Timestamp> = (0..n)
        .map(|i| Timestamp::new(i as i64, 0).expect("valid timestamp"))
        .collect();
    let vec = JiffTimestampVecCounted {
        data: Arc::new(data),
        counter,
    };
    let altrep = vec.into_sexp();
    unsafe {
        Rf_protect(altrep);
        set_posixct_utc(altrep);
        Rf_unprotect(1);
    }
    altrep
}

/// Return the number of times elt() has been called on the last counted ALTREP.
#[miniextendr]
pub fn jiff_counted_altrep_elt_count() -> i32 {
    elt_counter().load(Ordering::Relaxed) as i32
}

// endregion

// region: vctrs rcrd fixtures (exercises span_vec_to_rcrd / zoned_vec_to_rcrd)

/// Build a `jiff_span` vctrs rcrd from three hard-coded spans.
///
/// Spans: 1y2m, 3m15d, zero.
/// @return A vctrs rcrd of class `jiff_span`.
#[cfg(feature = "vctrs")]
#[miniextendr]
pub fn jiff_span_rcrd_demo() -> SEXP {
    let spans = vec![
        Span::new().years(1i64).months(2i64),
        Span::new().months(3i64).days(15i64),
        Span::new(),
    ];
    miniextendr_api::jiff_impl::span_vec_to_rcrd(&spans)
}

/// Build a `jiff_zoned` vctrs rcrd from two zones: UTC and Europe/Paris.
///
/// @return A vctrs rcrd of class `jiff_zoned`.
#[cfg(feature = "vctrs")]
#[miniextendr]
pub fn jiff_zoned_rcrd_demo() -> SEXP {
    use miniextendr_api::jiff::tz::TimeZone;
    let utc_zone = Timestamp::new(1_704_067_200, 0)
        .expect("valid timestamp")
        .to_zoned(TimeZone::UTC);
    let paris_zone = Timestamp::new(1_704_067_200, 0)
        .expect("valid timestamp")
        .in_tz("Europe/Paris")
        .expect("valid tz");
    let zones = vec![utc_zone, paris_zone];
    miniextendr_api::jiff_impl::zoned_vec_to_rcrd(&zones)
}

// endregion
