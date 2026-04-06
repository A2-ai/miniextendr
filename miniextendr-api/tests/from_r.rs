//! Integration tests for TryFromSexp conversions.

mod r_test_utils;

use miniextendr_api::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use miniextendr_api::coerce::Coerced;
use miniextendr_api::ffi::{
    INTEGER, LOGICAL, R_xlen_t, RAW, Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE,
    SexpExt,
};
use miniextendr_api::from_r::{SexpError, TryFromSexp};
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
    let slice: &mut [f64] = unsafe { sexp.as_mut_slice() };
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
            Some(s) => SEXP::charsxp(s),
            None => SEXP::na_string(),
        };
        sexp.set_string_elt(i as R_xlen_t, charsxp);
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
        let int_sexp = guard.protect(SEXP::scalar_integer(42));
        let real_sexp = guard.protect(SEXP::scalar_real(3.5));
        let log_sexp = guard.protect(SEXP::scalar_logical(true));

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
        let na_int = guard.protect(SEXP::scalar_integer(NA_INTEGER));
        let na_real = guard.protect(SEXP::scalar_real(NA_REAL));
        let na_log = guard.protect(SEXP::scalar_logical_raw(NA_LOGICAL));

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
        str_scalar.set_string_elt(0, SEXP::charsxp("hello"));

        let as_str: &'static str = TryFromSexp::try_from_sexp(str_scalar).unwrap();
        let as_string: String = TryFromSexp::try_from_sexp(str_scalar).unwrap();
        assert_eq!(as_str, "hello");
        assert_eq!(as_string, "hello".to_string());

        let na_scalar = guard.protect(Rf_allocVector(SEXPTYPE::STRSXP, 1));
        na_scalar.set_string_elt(0, SEXP::na_string());
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
        let int_scalar = guard.protect(SEXP::scalar_integer(7));
        let coerced: Coerced<i64, i32> = TryFromSexp::try_from_sexp(int_scalar).unwrap();
        assert_eq!(coerced.into_inner(), 7i64);
    }
}

fn test_error_cases() {
    let mut guard = ProtectCount::default();
    unsafe {
        let int_scalar = guard.protect(SEXP::scalar_integer(1));
        let real_scalar = guard.protect(SEXP::scalar_real(1.0));
        let int_vec = make_int_vec(&[1, 2], &mut guard);

        let err = <i32 as TryFromSexp>::try_from_sexp(real_scalar).unwrap_err();
        assert!(matches!(err, miniextendr_api::from_r::SexpError::Type(_)));

        let err = <i32 as TryFromSexp>::try_from_sexp(int_vec).unwrap_err();
        assert!(matches!(err, miniextendr_api::from_r::SexpError::Length(_)));

        let na_log = guard.protect(SEXP::scalar_logical_raw(NA_LOGICAL));
        let err = <bool as TryFromSexp>::try_from_sexp(na_log).unwrap_err();
        assert!(matches!(err, miniextendr_api::from_r::SexpError::Na(_)));

        let coerced_err: Result<Coerced<i8, i32>, SexpError> =
            TryFromSexp::try_from_sexp(int_scalar);
        assert!(coerced_err.is_ok());

        let big_int = guard.protect(SEXP::scalar_integer(1000));
        let coerced_err: Result<Coerced<u8, i32>, SexpError> = TryFromSexp::try_from_sexp(big_int);
        assert!(matches!(coerced_err, Err(SexpError::InvalidValue(_))));
    }
}

// region: Feature-gated tests for macro-based conversions

/// Helper to create a VECSXP (R list) from SEXPs
#[cfg(any(feature = "serde", feature = "aho-corasick"))]
unsafe fn make_list(elements: &[SEXP], guard: &mut ProtectCount) -> SEXP {
    use miniextendr_api::ffi::SexpExt;
    let len = elements.len() as R_xlen_t;
    let sexp = unsafe { guard.protect(Rf_allocVector(SEXPTYPE::VECSXP, len)) };
    for (i, &elem) in elements.iter().enumerate() {
        sexp.set_vector_elt(i as R_xlen_t, elem);
    }
    sexp
}
// endregion

// region: aho-corasick feature tests

#[cfg(feature = "aho-corasick")]
#[test]
fn aho_corasick_option_from_nil() {
    use miniextendr_api::aho_corasick_impl::AhoCorasick;

    r_test_utils::with_r_thread(|| {
        let nil = miniextendr_api::ffi::SEXP::nil();
        let opt: Option<AhoCorasick> = TryFromSexp::try_from_sexp(nil).unwrap();
        assert!(opt.is_none());
    });
}

#[cfg(feature = "aho-corasick")]
#[test]
fn aho_corasick_option_from_patterns() {
    use miniextendr_api::aho_corasick_impl::AhoCorasick;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let patterns = make_str_vec(&[Some("foo"), Some("bar")], &mut guard);
            let opt: Option<AhoCorasick> = TryFromSexp::try_from_sexp(patterns).unwrap();
            assert!(opt.is_some());
            let ac = opt.unwrap();
            assert_eq!(ac.patterns_len(), 2);
        }
    });
}

#[cfg(feature = "aho-corasick")]
#[test]
fn aho_corasick_vec_from_list() {
    use miniextendr_api::aho_corasick_impl::AhoCorasick;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let patterns1 = make_str_vec(&[Some("a"), Some("b")], &mut guard);
            let patterns2 = make_str_vec(&[Some("x"), Some("y"), Some("z")], &mut guard);
            let list = make_list(&[patterns1, patterns2], &mut guard);

            let vec: Vec<AhoCorasick> = TryFromSexp::try_from_sexp(list).unwrap();
            assert_eq!(vec.len(), 2);
            assert_eq!(vec[0].patterns_len(), 2);
            assert_eq!(vec[1].patterns_len(), 3);
        }
    });
}

#[cfg(feature = "aho-corasick")]
#[test]
fn aho_corasick_vec_option_from_list() {
    use miniextendr_api::aho_corasick_impl::AhoCorasick;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let patterns1 = make_str_vec(&[Some("hello")], &mut guard);
            let nil = miniextendr_api::ffi::SEXP::nil();
            let patterns2 = make_str_vec(&[Some("world")], &mut guard);
            let list = make_list(&[patterns1, nil, patterns2], &mut guard);

            let vec: Vec<Option<AhoCorasick>> = TryFromSexp::try_from_sexp(list).unwrap();
            assert_eq!(vec.len(), 3);
            assert!(vec[0].is_some());
            assert!(vec[1].is_none());
            assert!(vec[2].is_some());
        }
    });
}
// endregion

// region: serde (JSON) feature tests

#[cfg(feature = "serde")]
#[test]
fn json_value_option_from_nil() {
    use miniextendr_api::serde_impl::JsonValue;

    r_test_utils::with_r_thread(|| {
        let nil = miniextendr_api::ffi::SEXP::nil();
        let opt: Option<JsonValue> = TryFromSexp::try_from_sexp(nil).unwrap();
        assert!(opt.is_none());
    });
}

#[cfg(feature = "serde")]
#[test]
fn json_value_option_from_sexp() {
    use miniextendr_api::serde_impl::JsonValue;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            // Test with an integer scalar - converts to JSON number
            let int_sexp = guard.protect(SEXP::scalar_integer(42));
            let opt: Option<JsonValue> = TryFromSexp::try_from_sexp(int_sexp).unwrap();
            assert!(opt.is_some());
            let val = opt.unwrap();
            assert!(val.is_number());
            assert_eq!(val.as_i64(), Some(42));
        }
    });
}

#[cfg(feature = "serde")]
#[test]
fn json_value_vec_from_list() {
    use miniextendr_api::serde_impl::JsonValue;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            // Create R objects that convert to different JSON types
            let int_sexp = guard.protect(SEXP::scalar_integer(42)); // -> JSON number
            let str_vec = make_str_vec(&[Some("hello"), Some("world")], &mut guard); // -> JSON array
            let list = make_list(&[int_sexp, str_vec], &mut guard);

            let vec: Vec<JsonValue> = TryFromSexp::try_from_sexp(list).unwrap();
            assert_eq!(vec.len(), 2);
            assert!(vec[0].is_number());
            assert!(vec[1].is_array());
        }
    });
}

#[cfg(feature = "serde")]
#[test]
fn json_value_vec_option_from_list() {
    use miniextendr_api::serde_impl::JsonValue;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            // Create R objects: logical -> JSON bool, NULL -> None, integer -> JSON number
            let bool_sexp = guard.protect(SEXP::scalar_logical(true)); // -> JSON true
            let nil = miniextendr_api::ffi::SEXP::nil();
            let int_sexp = guard.protect(SEXP::scalar_integer(42)); // -> JSON 42
            let list = make_list(&[bool_sexp, nil, int_sexp], &mut guard);

            let vec: Vec<Option<JsonValue>> = TryFromSexp::try_from_sexp(list).unwrap();
            assert_eq!(vec.len(), 3);
            assert!(vec[0].as_ref().unwrap().is_boolean());
            assert!(vec[1].is_none());
            assert!(vec[2].as_ref().unwrap().is_number());
        }
    });
}
// endregion

// region: toml feature tests

#[cfg(feature = "toml")]
#[test]
fn toml_value_option_from_nil() {
    use miniextendr_api::toml_impl::TomlValue;

    r_test_utils::with_r_thread(|| {
        let nil = miniextendr_api::ffi::SEXP::nil();
        let opt: Option<TomlValue> = TryFromSexp::try_from_sexp(nil).unwrap();
        assert!(opt.is_none());
    });
}

#[cfg(feature = "toml")]
#[test]
fn toml_value_option_from_string() {
    use miniextendr_api::into_r::IntoR;
    use miniextendr_api::toml_impl::TomlValue;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let toml_str = r#"key = "value""#.to_string();
            let sexp = guard.protect(toml_str.into_sexp());
            let opt: Option<TomlValue> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert!(opt.is_some());
            let val = opt.unwrap();
            assert!(val.is_table());
        }
    });
}
// endregion

// region: bitvec feature tests

#[cfg(feature = "bitvec")]
#[test]
fn bitvec_option_from_nil() {
    use miniextendr_api::bitvec_impl::RBitVec;

    r_test_utils::with_r_thread(|| {
        let nil = miniextendr_api::ffi::SEXP::nil();
        let opt: Option<RBitVec> = TryFromSexp::try_from_sexp(nil).unwrap();
        assert!(opt.is_none());
    });
}

#[cfg(feature = "bitvec")]
#[test]
fn bitvec_option_from_logical() {
    use miniextendr_api::bitvec_impl::RBitVec;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let log_vec = make_logical_vec(&[1, 0, 1, 1, 0], &mut guard);
            let opt: Option<RBitVec> = TryFromSexp::try_from_sexp(log_vec).unwrap();
            assert!(opt.is_some());
            let bits = opt.unwrap();
            assert_eq!(bits.len(), 5);
            assert!(bits[0]);
            assert!(!bits[1]);
            assert!(bits[2]);
            assert!(bits[3]);
            assert!(!bits[4]);
        }
    });
}

#[cfg(feature = "bitvec")]
#[test]
fn bitvec_msb0_option_from_nil() {
    use miniextendr_api::bitvec_impl::{BitVec, Msb0};

    r_test_utils::with_r_thread(|| {
        let nil = miniextendr_api::ffi::SEXP::nil();
        let opt: Option<BitVec<u8, Msb0>> = TryFromSexp::try_from_sexp(nil).unwrap();
        assert!(opt.is_none());
    });
}

#[cfg(feature = "bitvec")]
#[test]
fn bitvec_msb0_option_from_logical() {
    use miniextendr_api::bitvec_impl::{BitVec, Msb0};

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let log_vec = make_logical_vec(&[0, 1, 0], &mut guard);
            let opt: Option<BitVec<u8, Msb0>> = TryFromSexp::try_from_sexp(log_vec).unwrap();
            assert!(opt.is_some());
            let bits = opt.unwrap();
            assert_eq!(bits.len(), 3);
            assert!(!bits[0]);
            assert!(bits[1]);
            assert!(!bits[2]);
        }
    });
}
// endregion

// region: Test try_from_sexp_unchecked propagation

#[cfg(feature = "aho-corasick")]
#[test]
fn aho_corasick_unchecked_option() {
    use miniextendr_api::aho_corasick_impl::AhoCorasick;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            // Test with patterns
            let patterns = make_str_vec(&[Some("test")], &mut guard);
            let opt: Option<AhoCorasick> = TryFromSexp::try_from_sexp_unchecked(patterns).unwrap();
            assert!(opt.is_some());

            // Test with nil
            let nil = miniextendr_api::ffi::SEXP::nil();
            let opt_nil: Option<AhoCorasick> = TryFromSexp::try_from_sexp_unchecked(nil).unwrap();
            assert!(opt_nil.is_none());
        }
    });
}

#[cfg(feature = "aho-corasick")]
#[test]
fn aho_corasick_unchecked_vec() {
    use miniextendr_api::aho_corasick_impl::AhoCorasick;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let patterns1 = make_str_vec(&[Some("a")], &mut guard);
            let patterns2 = make_str_vec(&[Some("b")], &mut guard);
            let list = make_list(&[patterns1, patterns2], &mut guard);

            let vec: Vec<AhoCorasick> = TryFromSexp::try_from_sexp_unchecked(list).unwrap();
            assert_eq!(vec.len(), 2);
        }
    });
}

#[cfg(feature = "serde")]
#[test]
fn json_value_unchecked_vec_option() {
    use miniextendr_api::serde_impl::JsonValue;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let int_sexp = guard.protect(SEXP::scalar_integer(100));
            let nil = miniextendr_api::ffi::SEXP::nil();
            let list = make_list(&[int_sexp, nil], &mut guard);

            let vec: Vec<Option<JsonValue>> = TryFromSexp::try_from_sexp_unchecked(list).unwrap();
            assert_eq!(vec.len(), 2);
            assert!(vec[0].is_some());
            assert!(vec[1].is_none());
        }
    });
}

#[cfg(feature = "bitvec")]
#[test]
fn bitvec_unchecked_option() {
    use miniextendr_api::bitvec_impl::RBitVec;

    r_test_utils::with_r_thread(|| {
        let mut guard = ProtectCount::default();
        unsafe {
            let log_vec = make_logical_vec(&[1, 0], &mut guard);
            let opt: Option<RBitVec> = TryFromSexp::try_from_sexp_unchecked(log_vec).unwrap();
            assert!(opt.is_some());
            assert_eq!(opt.unwrap().len(), 2);

            let nil = miniextendr_api::ffi::SEXP::nil();
            let opt_nil: Option<RBitVec> = TryFromSexp::try_from_sexp_unchecked(nil).unwrap();
            assert!(opt_nil.is_none());
        }
    });
}
// endregion

// region: Tests for blanket impl with arbitrary lifetimes

#[test]
fn slice_arbitrary_lifetime_i32() {
    // Test that &[T] works with arbitrary lifetimes (not just &'static [T])
    r_test_utils::with_r_thread(|| {
        unsafe {
            let sexp = make_int_vec(&[1, 2, 3], &mut ProtectCount::default());

            // This should work with non-static lifetime
            let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(slice.len(), 3);
            assert_eq!(slice[0], 1);
            assert_eq!(slice[1], 2);
            assert_eq!(slice[2], 3);
        }
    });
}

#[test]
fn slice_arbitrary_lifetime_f64() {
    r_test_utils::with_r_thread(|| unsafe {
        let sexp = make_real_vec(&[1.5, 2.5, 3.5], &mut ProtectCount::default());

        let slice: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 1.5);
        assert_eq!(slice[1], 2.5);
        assert_eq!(slice[2], 3.5);
    });
}

#[test]
fn slice_mut_arbitrary_lifetime() {
    r_test_utils::with_r_thread(|| {
        unsafe {
            let sexp = make_int_vec(&[10, 20, 30], &mut ProtectCount::default());

            // Get mutable slice
            let slice: &mut [i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(slice.len(), 3);

            // Modify in place
            slice[0] = 100;
            slice[1] = 200;

            // Verify modification worked
            assert_eq!(slice[0], 100);
            assert_eq!(slice[1], 200);
            assert_eq!(slice[2], 30);
        }
    });
}

#[test]
fn option_slice_arbitrary_lifetime() {
    r_test_utils::with_r_thread(|| {
        unsafe {
            // Test Some case
            let sexp = make_int_vec(&[1, 2], &mut ProtectCount::default());
            let opt: Option<&[i32]> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert!(opt.is_some());
            let slice = opt.unwrap();
            assert_eq!(slice.len(), 2);

            // Test None case
            let nil = miniextendr_api::ffi::SEXP::nil();
            let opt_nil: Option<&[i32]> = TryFromSexp::try_from_sexp(nil).unwrap();
            assert!(opt_nil.is_none());
        }
    });
}

#[test]
fn option_slice_mut_arbitrary_lifetime() {
    r_test_utils::with_r_thread(|| {
        unsafe {
            // Test Some case with mutation
            let sexp = make_int_vec(&[5, 10], &mut ProtectCount::default());
            let opt: Option<&mut [i32]> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert!(opt.is_some());
            let slice = opt.unwrap();
            slice[0] = 50;
            assert_eq!(slice[0], 50);
            assert_eq!(slice[1], 10);

            // Test None case
            let nil = miniextendr_api::ffi::SEXP::nil();
            let opt_nil: Option<&mut [i32]> = TryFromSexp::try_from_sexp(nil).unwrap();
            assert!(opt_nil.is_none());
        }
    });
}
// endregion
