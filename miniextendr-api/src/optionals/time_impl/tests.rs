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
    let parsed = <Date as RDateTimeFormat>::parse("2024-06-20", "[year]-[month]-[day]").unwrap();
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
