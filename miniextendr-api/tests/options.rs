//! Integration tests for `get_option` (reading `getOption()` from Rust).

mod r_test_utils;

use miniextendr_api::condition::RError;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::options::OPTION_ERROR_CLASS;
use miniextendr_api::{RValue, SEXP, get_option, r_str};

/// Evaluate R source for its side effect (setting options).
fn run(code: &str) {
    r_str!(code).expect("R code evaluates");
}

/// An `i32` that refuses negatives with its own classed condition.
#[derive(Debug)]
struct NonNegative(i32);

impl TryFromSexp for NonNegative {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let n = i32::try_from_sexp(sexp)?;
        if n < 0 {
            return Err(RError::new("must not be negative")
                .class("pkg_negative")
                .data("value", n)
                .into());
        }
        Ok(NonNegative(n))
    }
}

#[test]
fn unset_and_null_options_read_none() {
    r_test_utils::with_r_thread(|| {
        run("options(mx.test.unset = NULL)");
        assert_eq!(get_option::<i32>("mx.test.unset").unwrap(), None);
        assert_eq!(get_option::<String>("mx.test.never.set").unwrap(), None);
    });
}

#[test]
fn set_options_convert() {
    r_test_utils::with_r_thread(|| {
        run(r#"options(mx.test.int = 3L, mx.test.str = "wide", mx.test.vec = c(1, 2))"#);
        assert_eq!(get_option::<i32>("mx.test.int").unwrap(), Some(3));
        assert_eq!(
            get_option::<String>("mx.test.str").unwrap().as_deref(),
            Some("wide")
        );
        assert_eq!(
            get_option::<Vec<f64>>("mx.test.vec").unwrap(),
            Some(vec![1.0, 2.0])
        );
        run("options(mx.test.int = NULL, mx.test.str = NULL, mx.test.vec = NULL)");
    });
}

#[test]
fn a_value_that_does_not_convert_names_the_option() {
    r_test_utils::with_r_thread(|| {
        run(r#"options(mx.test.bad = "wide")"#);
        let err = get_option::<i32>("mx.test.bad").unwrap_err();
        assert_eq!(
            err.message_str(),
            r#"options("mx.test.bad") is invalid: expected integer, got character"#
        );
        assert_eq!(err.classes(), [OPTION_ERROR_CLASS]);
        assert!(matches!(
            err.fields().as_slice(),
            [(field, RValue::Character(name))]
                if field == "option" && name.as_slice() == [Some("mx.test.bad".to_string())]
        ));

        // `?` carries it into a `SexpError`, classes included.
        let via_sexp: Result<Option<i32>, SexpError> = (|| Ok(get_option("mx.test.bad")?))();
        match via_sexp {
            Err(SexpError::Condition(e)) => assert_eq!(e.classes(), [OPTION_ERROR_CLASS]),
            other => panic!("expected a condition, got {other:?}"),
        }
        run("options(mx.test.bad = NULL)");
    });
}

#[test]
fn a_classed_refusal_keeps_its_classes_and_fields() {
    r_test_utils::with_r_thread(|| {
        run("options(mx.test.neg = -2L)");
        let err = get_option::<NonNegative>("mx.test.neg").unwrap_err();
        assert_eq!(
            err.message_str(),
            r#"options("mx.test.neg") is invalid: must not be negative"#
        );
        assert_eq!(err.classes(), ["pkg_negative", OPTION_ERROR_CLASS]);
        let names: Vec<&str> = err.fields().iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["value", "option"]);

        run("options(mx.test.neg = 4L)");
        assert_eq!(
            get_option::<NonNegative>("mx.test.neg").unwrap().unwrap().0,
            4
        );
        run("options(mx.test.neg = NULL)");
    });
}

#[test]
fn get_option_panics_off_the_main_thread() {
    r_test_utils::with_r_thread(|| {});
    let joined = std::thread::spawn(|| get_option::<i32>("digits")).join();
    let payload = joined.expect_err("get_option off the main thread panics");
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .unwrap_or_default();
    assert!(message.contains("R's main thread"), "{message}");
}
