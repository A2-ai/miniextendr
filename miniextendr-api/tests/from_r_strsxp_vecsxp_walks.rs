//! Edge-case coverage for the shared `map_strsxp_with` / `map_vecsxp_with`
//! walk helpers introduced in audit D6 (STRSXP/VECSXP walk dedup).
//!
//! The existing conversion test suite densely covers the happy paths for the
//! consumer impls (`Vec<String>`, `Vec<Option<String>>`, `Vec<Vec<T>>`,
//! `Vec<HashMap<String, V>>`, …) — that density is exactly the point of a pure
//! walk-dedup. What was missing before this refactor: dedicated coverage for
//! the empty-vector and non-matching-SEXPTYPE edges that the two new helpers
//! now centralize (previously duplicated, and untested, in each hand-rolled
//! walk).

mod r_test_utils;

use std::borrow::Cow;
use std::collections::HashMap;

use miniextendr_api::SEXPTYPE;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::prelude::SEXP;
use miniextendr_api::sys::Rf_allocVector;

#[derive(Default)]
struct ProtectCount(i32);

impl ProtectCount {
    unsafe fn protect(&mut self, sexp: SEXP) -> SEXP {
        unsafe { miniextendr_api::sys::Rf_protect(sexp) };
        self.0 += 1;
        sexp
    }
}

impl Drop for ProtectCount {
    fn drop(&mut self) {
        if self.0 > 0 {
            unsafe { miniextendr_api::sys::Rf_unprotect(self.0) };
        }
    }
}

#[test]
fn strsxp_vecsxp_walk_edge_cases() {
    r_test_utils::with_r_thread(|| {
        empty_strsxp_vectors();
        non_strsxp_type_error();
        empty_vecsxp_lists();
        non_vecsxp_type_error();
    });
}

/// Empty STRSXP (`character(0)`) walks to an empty `Vec` for every
/// `map_strsxp_with`-routed representation.
fn empty_strsxp_vectors() {
    let mut guard = ProtectCount::default();
    unsafe {
        let empty = guard.protect(Rf_allocVector(SEXPTYPE::STRSXP, 0));

        let as_string: Vec<String> = TryFromSexp::try_from_sexp(empty).unwrap();
        let as_opt_string: Vec<Option<String>> = TryFromSexp::try_from_sexp(empty).unwrap();
        let as_str: Vec<&'static str> = TryFromSexp::try_from_sexp(empty).unwrap();
        let as_opt_str: Vec<Option<&'static str>> = TryFromSexp::try_from_sexp(empty).unwrap();
        let as_cow: Vec<Cow<'static, str>> = TryFromSexp::try_from_sexp(empty).unwrap();
        let as_opt_cow: Vec<Option<Cow<'static, str>>> = TryFromSexp::try_from_sexp(empty).unwrap();

        assert!(as_string.is_empty());
        assert!(as_opt_string.is_empty());
        assert!(as_str.is_empty());
        assert!(as_opt_str.is_empty());
        assert!(as_cow.is_empty());
        assert!(as_opt_cow.is_empty());
    }
}

/// A non-STRSXP input to a `map_strsxp_with`-routed impl is a `SexpError::Type`
/// naming `STRSXP` (the type-check the helper centralizes).
fn non_strsxp_type_error() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_sexp = guard.protect(SEXP::scalar_integer(1));

        let err = <Vec<String> as TryFromSexp>::try_from_sexp(int_sexp).unwrap_err();
        match err {
            SexpError::Type(e) => assert_eq!(e.expected, SEXPTYPE::STRSXP),
            other => panic!("Vec<String>: expected SexpError::Type, got {other:?}"),
        }

        let err = <Vec<Option<String>> as TryFromSexp>::try_from_sexp(int_sexp).unwrap_err();
        match err {
            SexpError::Type(e) => assert_eq!(e.expected, SEXPTYPE::STRSXP),
            other => panic!("Vec<Option<String>>: expected SexpError::Type, got {other:?}"),
        }

        let err = <Vec<Cow<'static, str>> as TryFromSexp>::try_from_sexp(int_sexp).unwrap_err();
        match err {
            SexpError::Type(e) => assert_eq!(e.expected, SEXPTYPE::STRSXP),
            other => panic!("Vec<Cow<str>>: expected SexpError::Type, got {other:?}"),
        }
    }
}

/// Empty VECSXP (`list()`) walks to an empty `Vec` for `map_vecsxp_with`-routed
/// representations, including the `list_to_vec_of_maps` (collections.rs) path.
fn empty_vecsxp_lists() {
    let mut guard = ProtectCount::default();
    unsafe {
        let empty = guard.protect(Rf_allocVector(SEXPTYPE::VECSXP, 0));

        let as_vec_vec: Vec<Vec<i32>> = TryFromSexp::try_from_sexp(empty).unwrap();
        let as_vec_map: Vec<HashMap<String, i32>> = TryFromSexp::try_from_sexp(empty).unwrap();

        assert!(as_vec_vec.is_empty());
        assert!(as_vec_map.is_empty());
    }
}

/// A non-VECSXP input to a `map_vecsxp_with`-routed impl is a `SexpError::Type`
/// naming `VECSXP`.
fn non_vecsxp_type_error() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_sexp = guard.protect(SEXP::scalar_integer(1));

        let err = <Vec<Vec<i32>> as TryFromSexp>::try_from_sexp(int_sexp).unwrap_err();
        match err {
            SexpError::Type(e) => assert_eq!(e.expected, SEXPTYPE::VECSXP),
            other => panic!("Vec<Vec<i32>>: expected SexpError::Type, got {other:?}"),
        }

        let err = <Vec<HashMap<String, i32>> as TryFromSexp>::try_from_sexp(int_sexp).unwrap_err();
        match err {
            SexpError::Type(e) => assert_eq!(e.expected, SEXPTYPE::VECSXP),
            other => panic!("Vec<HashMap<String, i32>>: expected SexpError::Type, got {other:?}"),
        }
    }
}
