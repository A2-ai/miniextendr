//! Regression tests for audit A5: serde_r's `Option<T>` deserialization must
//! accept a typed scalar NA as `None`, matching the macro `TryFromSexp`
//! convention (see `miniextendr-api/src/from_r/logical.rs` and
//! `miniextendr-api/src/from_r/strings.rs`, both of which already treat NA
//! and NULL identically for `Option<T>` fields).
//!
//! `RDeserializer::deserialize_option` (`src/serde/de.rs`) already implements
//! this for all four scalar NA sentinels (logical, integer, real, character)
//! — these tests are the missing regression coverage that the audit's
//! evidence (`test-serde_r.R:67`) never actually exercised: that test only
//! covers the *non*-Option scalar path (`from_r::<i32>`), which correctly
//! continues to reject NA as a genuine missingness error.
//!
//! There's no live-R harness for `#[cfg(test)] mod tests` inside
//! `miniextendr-api/src`, because a real SEXP requires an initialized R
//! runtime (see the comment in `src/strict.rs`). These integration tests use
//! the embedded-R harness in `tests/r_test_utils.rs` instead.
//!
//! Also covers #1166: an NA *element* of an atomic vector must behave like
//! the scalar path — `Vec<T>` errors with `RSerdeError::UnexpectedNa` (not a
//! leaked serde "invalid type: Option value" internal), `Vec<Option<T>>`
//! yields `None`.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::SEXPTYPE;
use miniextendr_api::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::serde::{RSerdeError, from_r};
use miniextendr_api::sys::{Rf_allocVector, Rf_protect, Rf_unprotect};
use serde::Deserialize;

#[derive(Default)]
struct ProtectCount(i32);

impl ProtectCount {
    unsafe fn protect(&mut self, sexp: SEXP) -> SEXP {
        unsafe { Rf_protect(sexp) };
        self.0 += 1;
        sexp
    }
}

impl Drop for ProtectCount {
    fn drop(&mut self) {
        if self.0 > 0 {
            unsafe { Rf_unprotect(self.0) };
        }
    }
}

unsafe fn na_string_scalar(guard: &mut ProtectCount) -> SEXP {
    unsafe {
        let sexp = guard.protect(Rf_allocVector(SEXPTYPE::STRSXP, 1));
        sexp.set_string_elt(0, SEXP::na_string());
        sexp
    }
}

#[derive(Debug, Deserialize)]
struct WithOptionalScalar {
    required: i32,
    optional: Option<i32>,
}

#[test]
fn serde_option_na_suite() {
    r_test_utils::with_r_thread(|| {
        scalar_na_becomes_none_i32();
        scalar_na_becomes_none_f64();
        scalar_na_becomes_none_bool();
        scalar_na_becomes_none_string();
        scalar_null_becomes_none();
        scalar_some_roundtrips();
        bare_scalar_na_still_errors();
        struct_field_na_becomes_none_matching_null();
        struct_required_field_na_still_errors();
        vec_na_element_errors_with_unexpected_na();
        vec_option_na_element_becomes_none();
        vec_without_na_still_roundtrips();
    });
}

fn scalar_na_becomes_none_i32() {
    let mut guard = ProtectCount::default();
    unsafe {
        let na_int = guard.protect(SEXP::scalar_integer(NA_INTEGER));
        let opt: Option<i32> = from_r(na_int).expect("deserialize Option<i32> from NA_integer_");
        assert!(opt.is_none());
    }
}

fn scalar_na_becomes_none_f64() {
    let mut guard = ProtectCount::default();
    unsafe {
        let na_real = guard.protect(SEXP::scalar_real(NA_REAL));
        let opt: Option<f64> = from_r(na_real).expect("deserialize Option<f64> from NA_real_");
        assert!(opt.is_none());
    }
}

fn scalar_na_becomes_none_bool() {
    let mut guard = ProtectCount::default();
    unsafe {
        let na_log = guard.protect(SEXP::scalar_logical_raw(NA_LOGICAL));
        let opt: Option<bool> = from_r(na_log).expect("deserialize Option<bool> from NA");
        assert!(opt.is_none());
    }
}

fn scalar_na_becomes_none_string() {
    let mut guard = ProtectCount::default();
    unsafe {
        let na_str = na_string_scalar(&mut guard);
        let opt: Option<String> =
            from_r(na_str).expect("deserialize Option<String> from NA_character_");
        assert!(opt.is_none());
    }
}

fn scalar_null_becomes_none() {
    // NULL -> None was already correct before audit A5; verify it still is.
    let null_sexp = SEXP::nil();
    let opt_i32: Option<i32> = from_r(null_sexp).expect("deserialize Option<i32> from NULL");
    let opt_f64: Option<f64> = from_r(null_sexp).expect("deserialize Option<f64> from NULL");
    let opt_bool: Option<bool> = from_r(null_sexp).expect("deserialize Option<bool> from NULL");
    let opt_string: Option<String> =
        from_r(null_sexp).expect("deserialize Option<String> from NULL");
    assert!(opt_i32.is_none());
    assert!(opt_f64.is_none());
    assert!(opt_bool.is_none());
    assert!(opt_string.is_none());
}

fn scalar_some_roundtrips() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_sexp = guard.protect(SEXP::scalar_integer(7));
        let opt: Option<i32> = from_r(int_sexp).expect("deserialize Option<i32> from 7L");
        assert_eq!(opt, Some(7));
    }
}

fn bare_scalar_na_still_errors() {
    // The non-Option path is unchanged: a typed NA reaching a bare scalar is
    // still a genuine missingness error, not silently converted.
    let mut guard = ProtectCount::default();
    unsafe {
        let na_int = guard.protect(SEXP::scalar_integer(NA_INTEGER));
        let result: Result<i32, _> = from_r(na_int);
        assert!(result.is_err(), "bare i32 should still reject NA_integer_");

        // Also confirmed on the macro TryFromSexp path for parity.
        let macro_result = i32::try_from_sexp(na_int);
        assert!(
            macro_result.is_err(),
            "macro TryFromSexp<i32> should also reject NA_integer_"
        );
    }
}

/// Build `list(required = <required>, optional = <optional>)`, protecting
/// each transient allocation across its `set_*` call (mirrors the pattern in
/// `tests/dataframe_de.rs::make_factor_dataframe`).
unsafe fn make_with_optional_scalar_list(
    required: i32,
    optional: SEXP,
    guard: &mut ProtectCount,
) -> SEXP {
    unsafe {
        let list = guard.protect(Rf_allocVector(SEXPTYPE::VECSXP, 2));

        let names = guard.protect(Rf_allocVector(SEXPTYPE::STRSXP, 2));
        names.set_string_elt(0, SEXP::charsxp("required"));
        names.set_string_elt(1, SEXP::charsxp("optional"));
        list.set_names(names);

        let required_sexp = guard.protect(SEXP::scalar_integer(required));
        list.set_vector_elt(0, required_sexp);
        list.set_vector_elt(1, optional);

        list
    }
}

fn struct_field_na_becomes_none_matching_null() {
    let mut guard = ProtectCount::default();
    unsafe {
        // required = 1, optional = NA_integer_
        let na_optional = guard.protect(SEXP::scalar_integer(NA_INTEGER));
        let list = make_with_optional_scalar_list(1, na_optional, &mut guard);

        let value: WithOptionalScalar =
            from_r(list).expect("NA in an Option<i32> field should deserialize to None");
        assert_eq!(value.required, 1);
        assert!(value.optional.is_none());

        // Same struct with NULL instead of NA in `optional` -- same result.
        let list2 = make_with_optional_scalar_list(1, SEXP::nil(), &mut guard);

        let value2: WithOptionalScalar =
            from_r(list2).expect("NULL in an Option<i32> field should deserialize to None");
        assert_eq!(value2.required, 1);
        assert!(value2.optional.is_none());
    }
}

fn struct_required_field_na_still_errors() {
    let mut guard = ProtectCount::default();
    unsafe {
        // required = NA_integer_, optional = 5 -- the required (non-Option)
        // field must still error; only Option<T> fields tolerate NA.
        let some_optional = guard.protect(SEXP::scalar_integer(5));
        let list = make_with_optional_scalar_list(NA_INTEGER, some_optional, &mut guard);

        let result: Result<WithOptionalScalar, _> = from_r(list);
        assert!(
            result.is_err(),
            "NA in a non-Option required field should still error"
        );
    }
}

// region: vector-element NA handling (#1166)

/// Assert an error is the clear `UnexpectedNa`, not a leaked serde internal
/// like "invalid type: Option value, expected i32".
#[track_caller]
fn assert_unexpected_na(err: &RSerdeError, context: &str) {
    assert!(
        matches!(err, RSerdeError::UnexpectedNa),
        "{context}: expected RSerdeError::UnexpectedNa, got: {err:?}"
    );
    assert_eq!(err.to_string(), "unexpected NA value");
}

/// An NA element inside a non-`Option` `Vec<T>` is still an error, but with
/// the same clear `UnexpectedNa` message the bare-scalar path produces.
fn vec_na_element_errors_with_unexpected_na() {
    let mut guard = ProtectCount::default();
    unsafe {
        let ints = guard.protect(vec![Some(1i32), None, Some(3)].into_sexp());
        let err = from_r::<Vec<i32>>(ints).expect_err("NA element in Vec<i32> must error");
        assert_unexpected_na(&err, "Vec<i32>");

        let reals = guard.protect(vec![Some(1.5f64), None].into_sexp());
        let err = from_r::<Vec<f64>>(reals).expect_err("NA element in Vec<f64> must error");
        assert_unexpected_na(&err, "Vec<f64>");

        let bools = guard.protect(vec![Some(true), None].into_sexp());
        let err = from_r::<Vec<bool>>(bools).expect_err("NA element in Vec<bool> must error");
        assert_unexpected_na(&err, "Vec<bool>");

        let strs = guard.protect(vec![Some("a".to_string()), None].into_sexp());
        let err = from_r::<Vec<String>>(strs).expect_err("NA element in Vec<String> must error");
        assert_unexpected_na(&err, "Vec<String>");
    }
}

/// `Vec<Option<T>>` still maps NA elements to `None` and non-NA elements to
/// `Some` via the new `VectorElementDeserializer::deserialize_option`.
fn vec_option_na_element_becomes_none() {
    let mut guard = ProtectCount::default();
    unsafe {
        let ints = guard.protect(vec![Some(1i32), None, Some(3)].into_sexp());
        let vals: Vec<Option<i32>> = from_r(ints).expect("Vec<Option<i32>> with NA element");
        assert_eq!(vals, vec![Some(1), None, Some(3)]);

        let reals = guard.protect(vec![Some(1.5f64), None].into_sexp());
        let vals: Vec<Option<f64>> = from_r(reals).expect("Vec<Option<f64>> with NA element");
        assert_eq!(vals, vec![Some(1.5), None]);

        let bools = guard.protect(vec![Some(true), None, Some(false)].into_sexp());
        let vals: Vec<Option<bool>> = from_r(bools).expect("Vec<Option<bool>> with NA element");
        assert_eq!(vals, vec![Some(true), None, Some(false)]);

        let strs = guard.protect(vec![Some("a".to_string()), None].into_sexp());
        let vals: Vec<Option<String>> = from_r(strs).expect("Vec<Option<String>> with NA element");
        assert_eq!(vals, vec![Some("a".to_string()), None]);
    }
}

/// NA-free vectors are unaffected for both `Vec<T>` and `Vec<Option<T>>`.
fn vec_without_na_still_roundtrips() {
    let mut guard = ProtectCount::default();
    unsafe {
        let ints = guard.protect(vec![1i32, 2, 3].into_sexp());
        let vals: Vec<i32> = from_r(ints).expect("Vec<i32> without NA");
        assert_eq!(vals, vec![1, 2, 3]);

        let vals: Vec<Option<i32>> = from_r(ints).expect("Vec<Option<i32>> without NA");
        assert_eq!(vals, vec![Some(1), Some(2), Some(3)]);
    }
}

// endregion
