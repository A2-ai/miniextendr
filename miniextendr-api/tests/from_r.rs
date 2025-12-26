//! Integration tests for TryFromSexp conversions.

mod r_test_utils;

use miniextendr_api::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use miniextendr_api::coerce::Coerced;
use miniextendr_api::ffi::{
    INTEGER, LOGICAL, R_NaString, R_xlen_t, RAW, Rf_ScalarInteger, Rf_ScalarLogical, Rf_ScalarReal,
    Rf_allocVector, Rf_mkChar, Rf_protect, Rf_unprotect, SET_STRING_ELT, SEXP, SEXPTYPE,
};
use miniextendr_api::from_r::{CoercedSexpError, TryFromSexp};
use std::collections::{BTreeSet, HashSet};
use std::ffi::CString;

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

unsafe fn make_int_vec(values: &[i32], guard: &mut ProtectCount) -> SEXP {
    let len = values.len() as R_xlen_t;
    let sexp = unsafe { guard.protect(Rf_allocVector(SEXPTYPE::INTSXP, len)) };
    let slice = unsafe { std::slice::from_raw_parts_mut(INTEGER(sexp), values.len()) };
    slice.copy_from_slice(values);
    sexp
}

unsafe fn make_real_vec(values: &[f64], guard: &mut ProtectCount) -> SEXP {
    let len = values.len() as R_xlen_t;
    let sexp = unsafe { guard.protect(Rf_allocVector(SEXPTYPE::REALSXP, len)) };
    let slice =
        unsafe { std::slice::from_raw_parts_mut(miniextendr_api::ffi::REAL(sexp), values.len()) };
    slice.copy_from_slice(values);
    sexp
}

unsafe fn make_logical_vec(values: &[i32], guard: &mut ProtectCount) -> SEXP {
    let len = values.len() as R_xlen_t;
    let sexp = unsafe { guard.protect(Rf_allocVector(SEXPTYPE::LGLSXP, len)) };
    let slice = unsafe { std::slice::from_raw_parts_mut(LOGICAL(sexp), values.len()) };
    slice.copy_from_slice(values);
    sexp
}

unsafe fn make_raw_vec(values: &[u8], guard: &mut ProtectCount) -> SEXP {
    let len = values.len() as R_xlen_t;
    let sexp = unsafe { guard.protect(Rf_allocVector(SEXPTYPE::RAWSXP, len)) };
    let slice = unsafe { std::slice::from_raw_parts_mut(RAW(sexp), values.len()) };
    slice.copy_from_slice(values);
    sexp
}

unsafe fn make_str_vec(values: &[Option<&str>], guard: &mut ProtectCount) -> SEXP {
    let len = values.len() as R_xlen_t;
    let sexp = unsafe { guard.protect(Rf_allocVector(SEXPTYPE::STRSXP, len)) };
    for (i, value) in values.iter().enumerate() {
        let charsxp = match value {
            Some(s) => {
                let c_str = CString::new(*s).expect("CString::new failed");
                unsafe { Rf_mkChar(c_str.as_ptr()) }
            }
            None => unsafe { R_NaString },
        };
        unsafe { SET_STRING_ELT(sexp, i as R_xlen_t, charsxp) };
    }
    sexp
}

#[test]
fn from_r_suite() {
    r_test_utils::with_r_thread(|| {
        test_scalar_conversions();
        test_option_scalars();
        test_slice_and_sets();
        test_vec_option_conversions();
        test_string_conversions();
        test_coerced_conversions();
        test_error_cases();
    });
}

fn test_scalar_conversions() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_sexp = guard.protect(Rf_ScalarInteger(42));
        let real_sexp = guard.protect(Rf_ScalarReal(3.5));
        let log_sexp = guard.protect(Rf_ScalarLogical(1));

        let int_val: i32 = TryFromSexp::try_from_sexp(int_sexp).unwrap();
        let real_val: f64 = TryFromSexp::try_from_sexp(real_sexp).unwrap();
        let bool_val: bool = TryFromSexp::try_from_sexp(log_sexp).unwrap();

        assert_eq!(int_val, 42);
        assert_eq!(real_val, 3.5);
        assert!(bool_val);
    }
}

fn test_option_scalars() {
    let mut guard = ProtectCount::default();
    unsafe {
        let na_int = guard.protect(Rf_ScalarInteger(NA_INTEGER));
        let na_real = guard.protect(Rf_ScalarReal(NA_REAL));
        let na_log = guard.protect(Rf_ScalarLogical(NA_LOGICAL));

        let opt_i32: Option<i32> = TryFromSexp::try_from_sexp(na_int).unwrap();
        let opt_f64: Option<f64> = TryFromSexp::try_from_sexp(na_real).unwrap();
        let opt_bool: Option<bool> = TryFromSexp::try_from_sexp(na_log).unwrap();

        assert!(opt_i32.is_none());
        assert!(opt_f64.is_none());
        assert!(opt_bool.is_none());
    }
}

fn test_slice_and_sets() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_vec = make_int_vec(&[1, 2, 3, 2], &mut guard);
        let real_vec = make_real_vec(&[1.0, 2.5], &mut guard);
        let raw_vec = make_raw_vec(&[1, 2, 3], &mut guard);

        let slice: &[i32] = TryFromSexp::try_from_sexp(int_vec).unwrap();
        assert_eq!(slice, &[1, 2, 3, 2]);

        let real_slice: &[f64] = TryFromSexp::try_from_sexp(real_vec).unwrap();
        assert_eq!(real_slice, &[1.0, 2.5]);

        let raw_slice: &[u8] = TryFromSexp::try_from_sexp(raw_vec).unwrap();
        assert_eq!(raw_slice, &[1, 2, 3]);

        let set: HashSet<i32> = TryFromSexp::try_from_sexp(int_vec).unwrap();
        let btree: BTreeSet<i32> = TryFromSexp::try_from_sexp(int_vec).unwrap();
        assert_eq!(set.len(), 3);
        assert!(set.contains(&1) && set.contains(&2) && set.contains(&3));
        assert_eq!(btree.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    }
}

fn test_vec_option_conversions() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_vec = make_int_vec(&[1, NA_INTEGER, 3], &mut guard);
        let real_vec = make_real_vec(&[1.0, NA_REAL, 3.5], &mut guard);
        let log_vec = make_logical_vec(&[1, NA_LOGICAL, 0], &mut guard);
        let str_vec = make_str_vec(&[Some("a"), None, Some("c")], &mut guard);

        let int_opts: Vec<Option<i32>> = TryFromSexp::try_from_sexp(int_vec).unwrap();
        let real_opts: Vec<Option<f64>> = TryFromSexp::try_from_sexp(real_vec).unwrap();
        let log_opts: Vec<Option<bool>> = TryFromSexp::try_from_sexp(log_vec).unwrap();
        let str_opts: Vec<Option<String>> = TryFromSexp::try_from_sexp(str_vec).unwrap();

        assert_eq!(int_opts, vec![Some(1), None, Some(3)]);
        assert_eq!(real_opts, vec![Some(1.0), None, Some(3.5)]);
        assert_eq!(log_opts, vec![Some(true), None, Some(false)]);
        assert_eq!(
            str_opts,
            vec![Some("a".to_string()), None, Some("c".to_string())]
        );
    }
}

fn test_string_conversions() {
    let mut guard = ProtectCount::default();
    unsafe {
        let str_vec = make_str_vec(&[Some("alpha"), None, Some("")], &mut guard);

        let vec_str: Vec<String> = TryFromSexp::try_from_sexp(str_vec).unwrap();
        assert_eq!(
            vec_str,
            vec!["alpha".to_string(), "".to_string(), "".to_string()]
        );

        let str_scalar = guard.protect(Rf_allocVector(SEXPTYPE::STRSXP, 1));
        let c_str = CString::new("hello").unwrap();
        let charsxp = Rf_mkChar(c_str.as_ptr());
        SET_STRING_ELT(str_scalar, 0, charsxp);

        let as_str: &'static str = TryFromSexp::try_from_sexp(str_scalar).unwrap();
        let as_string: String = TryFromSexp::try_from_sexp(str_scalar).unwrap();
        assert_eq!(as_str, "hello");
        assert_eq!(as_string, "hello".to_string());

        let na_scalar = guard.protect(Rf_allocVector(SEXPTYPE::STRSXP, 1));
        SET_STRING_ELT(na_scalar, 0, R_NaString);
        let na_str: &'static str = TryFromSexp::try_from_sexp(na_scalar).unwrap();
        let na_string: String = TryFromSexp::try_from_sexp(na_scalar).unwrap();
        let opt_string: Option<String> = TryFromSexp::try_from_sexp(na_scalar).unwrap();
        assert_eq!(na_str, "");
        assert_eq!(na_string, "".to_string());
        assert!(opt_string.is_none());

        let set: HashSet<String> = TryFromSexp::try_from_sexp(str_vec).unwrap();
        let btree: BTreeSet<String> = TryFromSexp::try_from_sexp(str_vec).unwrap();
        assert!(set.contains("alpha"));
        assert!(set.contains(""));
        assert_eq!(btree.iter().next().unwrap(), "");
    }
}

fn test_coerced_conversions() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_scalar = guard.protect(Rf_ScalarInteger(7));
        let coerced: Coerced<i64, i32> = TryFromSexp::try_from_sexp(int_scalar).unwrap();
        assert_eq!(coerced.into_inner(), 7i64);
    }
}

fn test_error_cases() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_scalar = guard.protect(Rf_ScalarInteger(1));
        let real_scalar = guard.protect(Rf_ScalarReal(1.0));
        let int_vec = make_int_vec(&[1, 2], &mut guard);

        let err = <i32 as TryFromSexp>::try_from_sexp(real_scalar).unwrap_err();
        assert!(matches!(err, miniextendr_api::from_r::SexpError::Type(_)));

        let err = <i32 as TryFromSexp>::try_from_sexp(int_vec).unwrap_err();
        assert!(matches!(err, miniextendr_api::from_r::SexpError::Length(_)));

        let na_log = guard.protect(Rf_ScalarLogical(NA_LOGICAL));
        let err = <bool as TryFromSexp>::try_from_sexp(na_log).unwrap_err();
        assert!(matches!(err, miniextendr_api::from_r::SexpError::Na(_)));

        let coerced_err: Result<Coerced<i8, i32>, CoercedSexpError> =
            TryFromSexp::try_from_sexp(int_scalar);
        assert!(coerced_err.is_ok());

        let big_int = guard.protect(Rf_ScalarInteger(1000));
        let coerced_err: Result<Coerced<u8, i32>, CoercedSexpError> =
            TryFromSexp::try_from_sexp(big_int);
        assert!(matches!(coerced_err, Err(CoercedSexpError::Coerce(_))));
    }
}
