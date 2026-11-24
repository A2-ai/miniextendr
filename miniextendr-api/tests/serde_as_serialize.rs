//! `AsSerialize<T>`: a `Serialize` value returned through `IntoR` as native R
//! data (a struct as a named list, a `Vec<i32>` as an integer vector, `None`
//! as `NULL`), read back with `from_r`. A value serde refuses is an `Err`
//! of class `miniextendr_serde_error`, never a panic.

#![cfg(feature = "serde")]

mod r_test_utils;

use std::collections::BTreeMap;

use miniextendr_api::condition::RConditionData;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::serde::{AsSerialize, RSerdeError, SERDE_ERROR_CLASS, from_r};
use miniextendr_api::{IntoR, Preserved, SEXP, sys};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct Reading {
    id: i32,
    label: String,
    samples: Vec<f64>,
    note: Option<String>,
    at: Point,
}

#[derive(serde::Serialize)]
struct Wide {
    id: i32,
    count: i128,
}

/// A `Serialize` impl that refuses, as a type's own validation would.
struct Refuses;

impl serde::Serialize for Refuses {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("refused by design"))
    }
}

/// `value` converted through `AsSerialize`, kept rooted.
fn serialized<T: serde::Serialize>(value: T) -> OwnedProtect {
    let sexp = AsSerialize(value)
        .try_into_sexp()
        .expect("the value serializes");
    unsafe { OwnedProtect::new(sexp) }
}

fn assert_identical(value: SEXP, expected: &str) {
    let expected_value = Preserved::new(unsafe {
        miniextendr_api::expression::r_eval_str_global(expected).unwrap()
    });
    let same = bool::from(unsafe { sys::R_compute_identical(value, expected_value.as_sexp(), 0) });
    assert!(same, "not identical to {expected}");
}

#[test]
fn the_wrapper_holds_the_value() {
    let mut wrapped = AsSerialize::new(Point { x: 1.0, y: 2.0 });
    wrapped.as_mut().y = 3.0;
    assert_eq!(wrapped.as_ref(), &Point { x: 1.0, y: 3.0 });
    assert_eq!(AsSerialize::from(Point { x: 1.0, y: 3.0 }), wrapped);
    assert_eq!(
        AsSerialize::<Point>::default().into_inner(),
        Point::default()
    );
    assert_eq!(wrapped.clone().into_inner(), Point { x: 1.0, y: 3.0 });
}

#[test]
fn structs_become_named_lists_and_read_back() {
    r_test_utils::with_r_thread(|| {
        let point = serialized(Point { x: 1.0, y: 2.5 });
        assert_identical(point.get(), "list(x = 1, y = 2.5)");
        assert_eq!(
            from_r::<Point>(point.get()).unwrap(),
            Point { x: 1.0, y: 2.5 }
        );

        let reading = Reading {
            id: 7,
            label: "first".into(),
            samples: vec![0.5, 1.5],
            note: None,
            at: Point { x: 0.0, y: -1.0 },
        };
        let sexp = serialized(&reading);
        assert_identical(
            sexp.get(),
            "list(id = 7L, label = 'first', samples = c(0.5, 1.5), note = NULL, \
             at = list(x = 0, y = -1))",
        );
        assert_eq!(from_r::<Reading>(sexp.get()).unwrap(), reading);
    });
}

#[test]
fn collections_options_and_empty_values() {
    r_test_utils::with_r_thread(|| {
        assert_identical(serialized(vec![1, 2, 3]).get(), "1:3");
        assert_identical(serialized(vec!["a", "b"]).get(), "c('a', 'b')");
        assert_identical(
            serialized(vec![Point { x: 1.0, y: 2.0 }]).get(),
            "list(list(x = 1, y = 2))",
        );
        // An empty `Vec` has no element type to dispatch on.
        assert_identical(serialized(Vec::<i32>::new()).get(), "list()");
        assert_identical(serialized(Option::<Point>::None).get(), "NULL");
        assert_identical(serialized(Some(4)).get(), "4L");
        let map: BTreeMap<String, i32> = [("b".to_string(), 2), ("a".to_string(), 1)].into();
        assert_identical(serialized(map).get(), "list(a = 1L, b = 2L)");
    });
}

#[test]
fn wide_integers_take_the_r_type_that_holds_the_value() {
    r_test_utils::with_r_thread(|| {
        // R has no type for `u32`, `i64` or `u64`: a value an R integer holds
        // is `integer(1)`, any other `numeric(1)`.
        assert_identical(serialized(7u16).get(), "7L");
        assert_identical(serialized(7u32).get(), "7L");
        assert_identical(serialized(-7i64).get(), "-7L");
        assert_identical(serialized(7u64).get(), "7L");
        assert_identical(serialized(u32::MAX).get(), "4294967295");
        assert_identical(serialized(-3_000_000_000i64).get(), "-3e9");
        assert_identical(serialized(3_000_000_000u64).get(), "3e9");
        // `i32::MIN` is `NA_integer_`'s bit pattern, so an `i64` holding it
        // is a double.
        assert_identical(serialized(i64::from(i32::MIN)).get(), "-2147483648");
    });
}

#[test]
fn integers_of_128_bits_are_refused() {
    r_test_utils::with_r_thread(|| {
        // Not widened to a double: writing and reading both refuse them, with
        // serde's own message, which names the type.
        let error = AsSerialize(u128::MAX)
            .try_into_sexp()
            .expect_err("no R type holds a u128");
        assert!(RConditionData::message(&error).contains("u128"), "{error}");
        assert_eq!(RConditionData::class(&error), [SERDE_ERROR_CLASS]);

        // In a struct field too.
        let error = AsSerialize(Wide { id: 7, count: -1 })
            .try_into_sexp()
            .expect_err("no R type holds an i128");
        assert!(RConditionData::message(&error).contains("i128"), "{error}");
        assert_eq!(RConditionData::class(&error), [SERDE_ERROR_CLASS]);

        let seven = serialized(7);
        let error = from_r::<u128>(seven.get()).expect_err("from_r reads no u128");
        assert!(RConditionData::message(&error).contains("u128"), "{error}");
        let error = from_r::<i128>(seven.get()).expect_err("from_r reads no i128");
        assert!(RConditionData::message(&error).contains("i128"), "{error}");
    });
}

#[test]
fn a_refused_value_is_a_classed_error() {
    r_test_utils::with_r_thread(|| {
        let error: RSerdeError = AsSerialize(Refuses)
            .try_into_sexp()
            .expect_err("serde refuses the value");
        assert!(
            RConditionData::message(&error).contains("refused by design"),
            "{error}"
        );
        assert_eq!(RConditionData::class(&error), [SERDE_ERROR_CLASS]);

        // A map whose keys are not strings has no R names.
        let map: BTreeMap<(i32, i32), i32> = [((1, 2), 3)].into();
        let error = AsSerialize(map)
            .try_into_sexp()
            .expect_err("tuple keys are not names");
        assert_eq!(RConditionData::class(&error), [SERDE_ERROR_CLASS]);
    });
}
