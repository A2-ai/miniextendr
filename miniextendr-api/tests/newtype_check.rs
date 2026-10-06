//! `TryFromSexpElement::check_sexp`, given by
//! `#[try_from_sexp(validate = ...)]`: the scalar and every container of a
//! newtype run it on the R value before the inner type reads it (#1815).

mod r_test_utils;

use std::sync::atomic::{AtomicUsize, Ordering};

use miniextendr_api::condition::RError;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{RValue, SEXP, SexpExt, r_str};

/// How many times [`plain_number`] ran.
static CHECKS: AtomicUsize = AtomicUsize::new(0);

/// Refuse a number that carries a unit class, with the class and the unit
/// class as a field.
fn plain_number(x: SEXP) -> Result<(), RError> {
    CHECKS.fetch_add(1, Ordering::Relaxed);
    for class in [c"difftime", c"Date", c"POSIXt"] {
        if x.inherits_class(class) {
            let class = class.to_str().unwrap();
            return Err(RError::new(format!("got a {class}; give a plain number"))
                .class(["unit_refused", "test_error"])
                .data("unit_class", class));
        }
    }
    Ok(())
}

#[derive(miniextendr_api::TryFromSexp, Debug, PartialEq)]
#[try_from_sexp(validate = plain_number)]
struct Elapsed(f64);

/// `src` evaluated and converted to `T` (`unchecked`: by
/// `try_from_sexp_unchecked`), with how many times the check ran.
fn convert<T>(src: &str, unchecked: bool) -> (Result<T, SexpError>, usize)
where
    T: TryFromSexp<Error = SexpError>,
{
    let input = unsafe { OwnedProtect::new(r_str!(src).expect("R source should evaluate")) };
    let before = CHECKS.load(Ordering::Relaxed);
    let value = if unchecked {
        unsafe { T::try_from_sexp_unchecked(input.get()) }
    } else {
        T::try_from_sexp(input.get())
    };
    (value, CHECKS.load(Ordering::Relaxed) - before)
}

/// The refusal `plain_number` gives a value of unit class `class`.
fn assert_refused<T: std::fmt::Debug>(result: Result<T, SexpError>, class: &str) {
    match result {
        Err(SexpError::Condition(e)) => {
            assert_eq!(e.message_str(), format!("got a {class}; give a plain number"));
            assert_eq!(e.classes(), ["unit_refused", "test_error"]);
            assert!(
                matches!(e.fields().as_slice(), [(name, RValue::Character(v))]
                    if name == "unit_class" && v == &[Some(class.to_string())]),
                "{:?}",
                e.fields()
            );
        }
        other => panic!("expected the {class} refusal, got {other:?}"),
    }
}

const MINUTES: &str = "as.difftime(5, units = 'mins')";
const DATES: &str = "as.Date(c('2024-01-01', '2024-06-01'))";
const TIMES: &str = "as.POSIXct(c('2024-01-01', NA), tz = 'UTC')";

/// The scalar runs the check before reading the value.
#[test]
fn scalar_runs_the_check() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Elapsed>("5", unchecked);
            assert_eq!((value.unwrap(), checks), (Elapsed(5.0), 1));
            let (value, checks) = convert::<Elapsed>(MINUTES, unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "difftime");
            // The inner type's own refusal comes after the check, as a
            // `SexpError`.
            let (value, checks) = convert::<Elapsed>("'a'", unchecked);
            assert_eq!(checks, 1);
            assert!(matches!(value, Err(SexpError::Type(_))), "{value:?}");
        }
    });
}

/// `Option<T>` runs the check on a value and never on `NULL`, which stays
/// "not given".
#[test]
fn option_runs_the_check_except_on_null() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Option<Elapsed>>("NULL", unchecked);
            assert_eq!((value.unwrap(), checks), (None, 0));
            let (value, checks) = convert::<Option<Elapsed>>("2.5", unchecked);
            assert_eq!((value.unwrap(), checks), (Some(Elapsed(2.5)), 1));
            let (value, checks) = convert::<Option<Elapsed>>("as.Date('2024-01-01')", unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "Date");
        }
    });
}

/// `Vec<T>` runs the check once, on the whole vector, where R keeps the
/// class; an inner type error (`Vec<f64>`'s `SexpTypeError`) still converts
/// into the container's `SexpError`.
#[test]
fn vec_runs_the_check_once() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Vec<Elapsed>>("c(1, 2, 3)", unchecked);
            assert_eq!(
                (value.unwrap(), checks),
                (vec![Elapsed(1.0), Elapsed(2.0), Elapsed(3.0)], 1)
            );
            let (value, checks) = convert::<Vec<Elapsed>>(DATES, unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "Date");
            let (value, checks) = convert::<Vec<Elapsed>>("c('a', 'b')", unchecked);
            assert_eq!(checks, 1);
            assert!(matches!(value, Err(SexpError::Type(_))), "{value:?}");
        }
    });
}

/// `Vec<Option<T>>` runs the check once, on the whole vector; `NA` elements
/// are still `None`.
#[test]
fn vec_option_runs_the_check_once() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Vec<Option<Elapsed>>>("c(1, NA)", unchecked);
            assert_eq!(
                (value.unwrap(), checks),
                (vec![Some(Elapsed(1.0)), None], 1)
            );
            let (value, checks) = convert::<Vec<Option<Elapsed>>>(TIMES, unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "POSIXt");
        }
    });
}

/// The argument-error probe takes the refusal's classes and fields and uses
/// its message as the reason after the wrapper's prefix.
#[test]
fn argument_error_carries_the_refusal() {
    r_test_utils::with_r_thread(|| {
        let (value, _) = convert::<Vec<Elapsed>>(DATES, false);
        let parts = miniextendr_api::__mx_conversion_err_parts!(value.unwrap_err(), true);
        assert_eq!(parts.message, "got a Date; give a plain number");
        assert_eq!(parts.class, ["unit_refused", "test_error"]);
        let data = parts.data.expect("the refusal's fields");
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].0, "unit_class");
    });
}

/// A check refusal inside an `Either` arm: when the other arm refused the
/// kind of value, the argument error reports the refusal with its classes;
/// when both arms refused the kind, it reports the value's type and no
/// refusal classes.
#[cfg(feature = "either")]
#[test]
fn either_arm_reports_the_refusal() {
    use miniextendr_api::either_impl::Either;
    use miniextendr_api::{AsNumeric, DataFrame};

    #[derive(miniextendr_api::TryFromSexp, Debug)]
    #[try_from_sexp(validate = plain_number)]
    struct Tau(AsNumeric);

    type Interval = Either<Option<Tau>, DataFrame>;

    r_test_utils::with_r_thread(|| {
        let (value, _) = convert::<Interval>(MINUTES, false);
        let parts = miniextendr_api::__mx_conversion_err_parts!(value.unwrap_err(), true);
        assert_eq!(parts.message, "got a difftime; give a plain number");
        assert_eq!(parts.class, ["unit_refused", "test_error"]);

        let (value, _) = convert::<Interval>("list(1)", false);
        let parts = miniextendr_api::__mx_conversion_err_parts!(value.unwrap_err(), true);
        assert_eq!(parts.message, "got list");
        assert!(parts.class.is_empty(), "{:?}", parts.class);

        let (value, _) = convert::<Interval>("NULL", false);
        assert!(matches!(value, Ok(Either::Left(None))));
        let (value, _) = convert::<Interval>("3", false);
        assert!(matches!(value, Ok(Either::Left(Some(Tau(AsNumeric(Some(3.0))))))));
    });
}
