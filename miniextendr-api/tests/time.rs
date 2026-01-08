mod r_test_utils;

#[cfg(feature = "time")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "time")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "time")]
use miniextendr_api::{Date, OffsetDateTime};

#[cfg(feature = "time")]
#[test]
fn datetime_roundtrip() {
    r_test_utils::with_r_thread(|| {
        // Use a specific timestamp: 2024-06-15 12:30:45.123 UTC
        let dt = time::macros::datetime!(2024-06-15 12:30:45.123456789 UTC);
        let sexp = dt.into_sexp();
        let back: OffsetDateTime = TryFromSexp::try_from_sexp(sexp).unwrap();

        // Should round-trip (with nanosecond truncation from float conversion)
        assert_eq!(back.year(), 2024);
        assert_eq!(back.month(), time::Month::June);
        assert_eq!(back.day(), 15);
        assert_eq!(back.hour(), 12);
        assert_eq!(back.minute(), 30);
        assert_eq!(back.second(), 45);
    });
}

#[cfg(feature = "time")]
#[test]
fn datetime_epoch() {
    r_test_utils::with_r_thread(|| {
        let epoch = time::macros::datetime!(1970-01-01 0:00:00 UTC);
        let sexp = epoch.into_sexp();
        let back: OffsetDateTime = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.year(), 1970);
        assert_eq!(back.month(), time::Month::January);
        assert_eq!(back.day(), 1);
        assert_eq!(back.hour(), 0);
        assert_eq!(back.minute(), 0);
        assert_eq!(back.second(), 0);
    });
}

#[cfg(feature = "time")]
#[test]
fn datetime_option_none() {
    r_test_utils::with_r_thread(|| {
        let opt: Option<OffsetDateTime> = None;
        let sexp = opt.into_sexp();
        let back: Option<OffsetDateTime> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_none());
    });
}

#[cfg(feature = "time")]
#[test]
fn datetime_option_some() {
    r_test_utils::with_r_thread(|| {
        let dt = time::macros::datetime!(2024-01-01 0:00:00 UTC);
        let opt = Some(dt);
        let sexp = opt.into_sexp();
        let back: Option<OffsetDateTime> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_some());
        assert_eq!(back.unwrap().year(), 2024);
    });
}

#[cfg(feature = "time")]
#[test]
fn datetime_vec_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let dts = vec![
            time::macros::datetime!(2024-01-01 0:00:00 UTC),
            time::macros::datetime!(2024-06-15 12:00:00 UTC),
            time::macros::datetime!(2024-12-31 23:59:59 UTC),
        ];
        let sexp = dts.clone().into_sexp();
        let back: Vec<OffsetDateTime> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back[0].year(), 2024);
        assert_eq!(back[0].month(), time::Month::January);
        assert_eq!(back[1].month(), time::Month::June);
        assert_eq!(back[2].month(), time::Month::December);
    });
}

#[cfg(feature = "time")]
#[test]
fn date_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let d = time::macros::date!(2024 - 06 - 15);
        let sexp = d.into_sexp();
        let back: Date = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.year(), 2024);
        assert_eq!(back.month(), time::Month::June);
        assert_eq!(back.day(), 15);
    });
}

#[cfg(feature = "time")]
#[test]
fn date_epoch() {
    r_test_utils::with_r_thread(|| {
        let d = time::macros::date!(1970 - 01 - 01);
        let sexp = d.into_sexp();
        let back: Date = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.year(), 1970);
        assert_eq!(back.month(), time::Month::January);
        assert_eq!(back.day(), 1);
    });
}

#[cfg(feature = "time")]
#[test]
fn date_option_none() {
    r_test_utils::with_r_thread(|| {
        let opt: Option<Date> = None;
        let sexp = opt.into_sexp();
        let back: Option<Date> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_none());
    });
}

#[cfg(feature = "time")]
#[test]
fn date_option_some() {
    r_test_utils::with_r_thread(|| {
        let d = time::macros::date!(2024 - 01 - 01);
        let opt = Some(d);
        let sexp = opt.into_sexp();
        let back: Option<Date> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_some());
        assert_eq!(back.unwrap().year(), 2024);
    });
}

#[cfg(feature = "time")]
#[test]
fn date_vec_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let dates = vec![
            time::macros::date!(2024 - 01 - 01),
            time::macros::date!(2024 - 06 - 15),
            time::macros::date!(2024 - 12 - 31),
        ];
        let sexp = dates.clone().into_sexp();
        let back: Vec<Date> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back[0].day(), 1);
        assert_eq!(back[1].day(), 15);
        assert_eq!(back[2].day(), 31);
    });
}
