//! Targeted gap-filling tests for the miniextendr conversion layer.
//!
//! Derived from a `cargo llvm-cov` audit (2026-05-29).  Each test group is
//! labelled with the module it exercises and the specific branch class it hits.
//! Tests that do NOT call into R (pure Rust coerce logic) run directly.
//! Tests that touch R types run inside `with_r_thread`.
//!
//! # Coverage context
//! Before this file, the conversion-layer region coverage was:
//!   from_r/coerced_scalars  ~3 %
//!   strict.rs              ~18 %
//!   into_r/large_integers  ~20 %
//!   from_r/na_vectors      ~23 %
//!   from_r/logical.rs      ~18 %
//!   from_r.rs (root)       ~31 %
//!   coerce.rs              ~57 %
//! into_r_as (IntoRAs)        ~5 %
//!
//! See analysis/conversion-coverage-2026-05-29.md for the full report.

mod r_test_utils;

use miniextendr_api::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::sys::{Rf_allocVector, Rf_protect, Rf_unprotect};
use miniextendr_api::{Rboolean, SEXPTYPE};

// region: helpers

struct Guard(i32);

impl Guard {
    unsafe fn protect(&mut self, s: SEXP) -> SEXP {
        unsafe { Rf_protect(s) };
        self.0 += 1;
        s
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if self.0 > 0 {
            unsafe { Rf_unprotect(self.0) };
        }
    }
}

unsafe fn scalar_int(v: i32, g: &mut Guard) -> SEXP {
    unsafe { g.protect(SEXP::scalar_integer(v)) }
}

unsafe fn scalar_real(v: f64, g: &mut Guard) -> SEXP {
    unsafe { g.protect(SEXP::scalar_real(v)) }
}

unsafe fn scalar_raw(v: u8, g: &mut Guard) -> SEXP {
    unsafe {
        let s = Rf_allocVector(SEXPTYPE::RAWSXP, 1);
        let s = g.protect(s);
        let ptr = s.as_mut_slice::<u8>();
        ptr[0] = v;
        s
    }
}

unsafe fn scalar_logical_raw(v: i32, g: &mut Guard) -> SEXP {
    unsafe { g.protect(SEXP::scalar_logical_raw(v)) }
}

unsafe fn vec_int(values: &[i32], g: &mut Guard) -> SEXP {
    unsafe {
        let s = g.protect(Rf_allocVector(SEXPTYPE::INTSXP, values.len() as isize));
        s.as_mut_slice::<i32>().copy_from_slice(values);
        s
    }
}

unsafe fn vec_real(values: &[f64], g: &mut Guard) -> SEXP {
    unsafe {
        let s = g.protect(Rf_allocVector(SEXPTYPE::REALSXP, values.len() as isize));
        s.as_mut_slice::<f64>().copy_from_slice(values);
        s
    }
}

unsafe fn vec_logical(values: &[i32], g: &mut Guard) -> SEXP {
    unsafe {
        let s = g.protect(Rf_allocVector(SEXPTYPE::LGLSXP, values.len() as isize));
        for (i, &v) in values.iter().enumerate() {
            s.set_logical_elt(i as isize, v);
        }
        s
    }
}

unsafe fn vec_raw(values: &[u8], g: &mut Guard) -> SEXP {
    unsafe {
        let s = g.protect(Rf_allocVector(SEXPTYPE::RAWSXP, values.len() as isize));
        s.as_mut_slice::<u8>().copy_from_slice(values);
        s
    }
}

// endregion

// region: strict.rs — checked_into_sexp_* (outbound strict conversions)

/// `checked_into_sexp_i64` — in-range fits as INTSXP.
#[test]
fn strict_i64_in_range_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_into_sexp_i64;

        let sexp = checked_into_sexp_i64(42i64);
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        assert_eq!(sexp.len(), 1);
        let v: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(v, 42);
    });
}

/// `checked_into_sexp_i64` — i32::MAX boundary is accepted.
#[test]
fn strict_i64_at_i32_max_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_into_sexp_i64;
        let sexp = checked_into_sexp_i64(i32::MAX as i64);
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    });
}

/// `checked_into_sexp_i64` — i32::MIN (NA_integer_ sentinel) panics in strict mode.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_i64_at_i32_min_panics() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_into_sexp_i64;
        let _ = checked_into_sexp_i64(i32::MIN as i64);
    });
}

/// `checked_into_sexp_i64` — value above i32::MAX panics in strict mode.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_i64_above_i32_max_panics() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_into_sexp_i64;
        let _ = checked_into_sexp_i64(i32::MAX as i64 + 1);
    });
}

/// `checked_into_sexp_u64` — in-range value yields INTSXP.
#[test]
fn strict_u64_in_range_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_into_sexp_u64;
        let sexp = checked_into_sexp_u64(100u64);
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    });
}

/// `checked_into_sexp_u64` — value above i32::MAX panics.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_u64_above_i32_max_panics() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_into_sexp_u64;
        let _ = checked_into_sexp_u64(i32::MAX as u64 + 1);
    });
}

/// `checked_vec_i64_into_sexp` — all in range yields INTSXP vector.
#[test]
fn strict_vec_i64_all_in_range_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_i64_into_sexp;
        let sexp = checked_vec_i64_into_sexp(vec![1i64, 2, 3, i32::MAX as i64]);
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        assert_eq!(sexp.len(), 4);
    });
}

/// `checked_vec_i64_into_sexp` — any element out of range panics.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_vec_i64_out_of_range_panics() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_i64_into_sexp;
        let _ = checked_vec_i64_into_sexp(vec![1i64, i64::MAX]);
    });
}

/// `checked_vec_option_i64_into_sexp` — None maps to NA, Some in-range is kept.
#[test]
fn strict_vec_option_i64_none_becomes_na() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_i64_into_sexp;
        let sexp = checked_vec_option_i64_into_sexp(vec![Some(5i64), None, Some(10i64)]);
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        let slice: &[i32] = unsafe { sexp.as_slice() };
        assert_eq!(slice[0], 5);
        assert_eq!(slice[1], NA_INTEGER); // None → NA_integer_
        assert_eq!(slice[2], 10);
    });
}

/// `checked_option_i64_into_sexp` — None yields NA_integer_.
#[test]
fn strict_option_i64_none_yields_na() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_option_i64_into_sexp;
        let sexp = checked_option_i64_into_sexp(None);
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        let v: Option<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(v.is_none());
    });
}

/// Strict INPUT: `checked_try_from_sexp_i64` rejects RAWSXP (panics).
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_i64_rejects_raw() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_try_from_sexp_i64;
        let mut g = Guard(0);
        let raw = unsafe { scalar_raw(1, &mut g) };
        // panics because strict mode rejects RAWSXP
        let _ = checked_try_from_sexp_i64(raw, "x");
    });
}

/// Strict INPUT: `checked_try_from_sexp_i64` rejects LGLSXP (panics).
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_i64_rejects_logical() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_try_from_sexp_i64;
        let mut g = Guard(0);
        let lgl = unsafe { scalar_logical_raw(1, &mut g) };
        // panics because strict mode rejects LGLSXP
        let _ = checked_try_from_sexp_i64(lgl, "x");
    });
}

/// Strict INPUT: `checked_try_from_sexp_i64` accepts INTSXP.
#[test]
fn strict_input_i64_accepts_int() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_try_from_sexp_i64;
        let mut g = Guard(0);
        let s = unsafe { scalar_int(42, &mut g) };
        let val = checked_try_from_sexp_i64(s, "x");
        assert_eq!(val, 42i64);
    });
}

/// Strict INPUT: `checked_try_from_sexp_i64` accepts REALSXP.
#[test]
fn strict_input_i64_accepts_real() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_try_from_sexp_i64;
        let mut g = Guard(0);
        let s = unsafe { scalar_real(7.0, &mut g) };
        let val = checked_try_from_sexp_i64(s, "x");
        assert_eq!(val, 7i64);
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_i64` — audit A6 regression.
/// Must apply the same input-SEXP-type gate as `checked_vec_try_from_sexp_i64`
/// (reject LGLSXP), not silently coerce like the lax `TryFromSexp` path does.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_vec_option_i64_rejects_logical() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_i64;
        let mut g = Guard(0);
        let lgl = unsafe { vec_logical(&[1, 0], &mut g) };
        let _ = checked_vec_option_try_from_sexp_i64(lgl, "x");
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_i64` rejects RAWSXP.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_vec_option_i64_rejects_raw() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_i64;
        let mut g = Guard(0);
        let raw = unsafe { vec_raw(&[1, 2], &mut g) };
        let _ = checked_vec_option_try_from_sexp_i64(raw, "x");
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_i64` accepts INTSXP,
/// mapping `NA_integer_` elements to `None`.
#[test]
fn strict_input_vec_option_i64_accepts_int_with_na() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_i64;
        let mut g = Guard(0);
        let s = unsafe { vec_int(&[1, NA_INTEGER, 3], &mut g) };
        let val = checked_vec_option_try_from_sexp_i64(s, "x");
        assert_eq!(val, vec![Some(1i64), None, Some(3i64)]);
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_i64` accepts REALSXP,
/// mapping `NA_real_` elements to `None`.
#[test]
fn strict_input_vec_option_i64_accepts_real_with_na() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_i64;
        let mut g = Guard(0);
        let s = unsafe { vec_real(&[1.0, NA_REAL, 3.0], &mut g) };
        let val = checked_vec_option_try_from_sexp_i64(s, "x");
        assert_eq!(val, vec![Some(1i64), None, Some(3i64)]);
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_i64` — an out-of-range
/// `Some` element still panics (range checking, not just the type gate).
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_vec_option_i64_out_of_range_panics() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_i64;
        let mut g = Guard(0);
        // f64 above i64 range fails the i64-from-f64 TryCoerce step.
        let s = unsafe { vec_real(&[1.0, 1e300], &mut g) };
        let _ = checked_vec_option_try_from_sexp_i64(s, "x");
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_u64` rejects LGLSXP.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_vec_option_u64_rejects_logical() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_u64;
        let mut g = Guard(0);
        let lgl = unsafe { vec_logical(&[1, 0], &mut g) };
        let _ = checked_vec_option_try_from_sexp_u64(lgl, "x");
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_isize` rejects LGLSXP.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_vec_option_isize_rejects_logical() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_isize;
        let mut g = Guard(0);
        let lgl = unsafe { vec_logical(&[1, 0], &mut g) };
        let _ = checked_vec_option_try_from_sexp_isize(lgl, "x");
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_usize` rejects LGLSXP.
#[test]
#[should_panic(expected = "strict conversion failed")]
fn strict_input_vec_option_usize_rejects_logical() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_usize;
        let mut g = Guard(0);
        let lgl = unsafe { vec_logical(&[1, 0], &mut g) };
        let _ = checked_vec_option_try_from_sexp_usize(lgl, "x");
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_isize` accepts INTSXP with NA.
#[test]
fn strict_input_vec_option_isize_accepts_int_with_na() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_isize;
        let mut g = Guard(0);
        let s = unsafe { vec_int(&[5, NA_INTEGER], &mut g) };
        let val = checked_vec_option_try_from_sexp_isize(s, "x");
        assert_eq!(val, vec![Some(5isize), None]);
    });
}

/// Strict INPUT: `checked_vec_option_try_from_sexp_usize` accepts INTSXP with NA.
#[test]
fn strict_input_vec_option_usize_accepts_int_with_na() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::strict::checked_vec_option_try_from_sexp_usize;
        let mut g = Guard(0);
        let s = unsafe { vec_int(&[5, NA_INTEGER], &mut g) };
        let val = checked_vec_option_try_from_sexp_usize(s, "x");
        assert_eq!(val, vec![Some(5usize), None]);
    });
}

// endregion

// region: into_r/large_integers.rs — i64/u64/isize/usize smart branching

/// i64 below i32::MIN (e.g., i32::MIN - 1) → REALSXP.
#[test]
fn large_int_i64_below_i32_min_yields_realsxp() {
    r_test_utils::with_r_thread(|| {
        let v: i64 = i32::MIN as i64 - 1;
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    });
}

/// i64 at exactly i32::MIN is the NA sentinel → REALSXP, NOT NA.
#[test]
fn large_int_i64_at_i32_min_is_realsxp_not_na() {
    r_test_utils::with_r_thread(|| {
        let v: i64 = i32::MIN as i64;
        let sexp = v.into_sexp();
        // NA_integer_ = i32::MIN in R, so returning it as INTSXP would create an
        // unintended NA.  The impl correctly falls through to REALSXP.
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
        // Should be a finite real, not NA_real_
        let f: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(!f.is_nan());
        assert_eq!(f, i32::MIN as f64);
    });
}

/// i64 above i32::MAX → REALSXP.
#[test]
fn large_int_i64_above_i32_max_yields_realsxp() {
    r_test_utils::with_r_thread(|| {
        let v: i64 = i32::MAX as i64 + 1;
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    });
}

/// i64 in range (i32::MIN, i32::MAX] → INTSXP.
#[test]
fn large_int_i64_in_range_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        let sexp = 100i64.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    });
}

/// u64 small → INTSXP.
#[test]
fn large_int_u64_small_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        let sexp = 5u64.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    });
}

/// u64 above i32::MAX → REALSXP.
#[test]
fn large_int_u64_large_yields_realsxp() {
    r_test_utils::with_r_thread(|| {
        let v: u64 = i32::MAX as u64 + 1;
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    });
}

/// Vec<i64> — all fit → INTSXP.
#[test]
fn large_int_vec_i64_all_fit_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1i64, 2, 3, i32::MAX as i64];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        assert_eq!(sexp.len(), 4);
    });
}

/// Vec<i64> — any out-of-range → REALSXP.
#[test]
fn large_int_vec_i64_any_large_yields_realsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1i64, i64::MAX];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    });
}

/// Vec<i64> with i32::MIN mixed in → REALSXP (guards the NA sentinel).
#[test]
fn large_int_vec_i64_contains_i32_min_yields_realsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![0i64, i32::MIN as i64, 1];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    });
}

/// Vec<u64> — all fit → INTSXP.
#[test]
fn large_int_vec_u64_all_fit_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![0u64, 1, i32::MAX as u64];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    });
}

/// Vec<u64> — any large → REALSXP.
#[test]
fn large_int_vec_u64_any_large_yields_realsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1u64, u64::MAX];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    });
}

/// Vec<u32> — all fit → INTSXP, values round-trip (#973).
#[test]
fn large_int_vec_u32_all_fit_yields_intsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![0u32, 1, i32::MAX as u32];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        let back: Vec<u32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back, vec![0u32, 1, i32::MAX as u32]);
    });
}

/// Vec<u32> — any value above i32::MAX → REALSXP (#973).
#[test]
fn large_int_vec_u32_any_large_yields_realsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1u32, u32::MAX];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
        let back: Vec<u32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back, vec![1u32, u32::MAX]);
    });
}

/// Vec<Rboolean> → LGLSXP, values round-trip (#973).
#[test]
fn vec_rboolean_yields_lglsxp() {
    r_test_utils::with_r_thread(|| {
        let v = vec![Rboolean::TRUE, Rboolean::FALSE, Rboolean::TRUE];
        let sexp = v.into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::LGLSXP);
        let back: Vec<Rboolean> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back, vec![Rboolean::TRUE, Rboolean::FALSE, Rboolean::TRUE]);
    });
}

/// &[Rboolean] → LGLSXP (#973).
#[test]
fn slice_rboolean_yields_lglsxp() {
    r_test_utils::with_r_thread(|| {
        let v = [Rboolean::FALSE, Rboolean::TRUE];
        let sexp = v.as_slice().into_sexp();
        assert_eq!(sexp.type_of(), SEXPTYPE::LGLSXP);
        let back: Vec<Rboolean> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back, vec![Rboolean::FALSE, Rboolean::TRUE]);
    });
}

// endregion

// region: from_r/coerced_scalars.rs — multi-source scalar inbound conversions
//
// These are the INTSXP/REALSXP/RAWSXP/LGLSXP → i8/i16/u16/u32/f32/i64/u64 paths.
// Coverage was ~3% before this file (all the `TryFromSexp for i8` etc. impls).

/// i64 from INTSXP: widen.
#[test]
fn coerced_scalar_i64_from_intsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(7, &mut g) };
        let v: i64 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 7i64);
    });
}

/// i64 from REALSXP: round-trip a whole number.
#[test]
fn coerced_scalar_i64_from_realsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(42.0, &mut g) };
        let v: i64 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 42i64);
    });
}

/// i64 from REALSXP: fractional value is rejected.
#[test]
fn coerced_scalar_i64_from_realsxp_fractional_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(3.5, &mut g) }; // fractional but not pi
        let result: Result<i64, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// i64 from REALSXP: Inf is rejected.
#[test]
fn coerced_scalar_i64_from_realsxp_inf_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(f64::INFINITY, &mut g) };
        let result: Result<i64, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// i64 from REALSXP: NaN is rejected.
#[test]
fn coerced_scalar_i64_from_realsxp_nan_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(f64::NAN, &mut g) };
        let result: Result<i64, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// i64 from REALSXP: NA_real_ is rejected (not None — only Option<i64> maps NA → None).
#[test]
fn coerced_scalar_i64_from_realsxp_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(NA_REAL, &mut g) };
        let result: Result<i64, _> = TryFromSexp::try_from_sexp(s);
        // NA_real_ is a NaN variant, which coerce rejects as CoerceError::NaN
        assert!(result.is_err());
    });
}

/// i64 from RAWSXP: u8 widens to i64.
#[test]
fn coerced_scalar_i64_from_rawsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_raw(200, &mut g) };
        let v: i64 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 200i64);
    });
}

/// i64 from LGLSXP: logical TRUE → 1.
#[test]
fn coerced_scalar_i64_from_lglsxp_true() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(1, &mut g) };
        let v: i64 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 1i64);
    });
}

/// i64 from LGLSXP: NA_logical_ passes through as NA_INTEGER (-2147483648) widened to i64.
///
/// Unlike the i32 path (which has an explicit NA guard), the coerced scalar path
/// for i64 goes through RLogical → i32::MIN (NA_INTEGER) → i64(-2147483648).
/// There is no NA guard for the widening path; the NA sentinel survives as a value.
/// Use Option<i64> to receive NA as None.
#[test]
fn coerced_scalar_i64_from_lglsxp_na_is_na_integer_value() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        // NA_LOGICAL → to_i32() → i32::MIN → widened to i64(-2147483648).
        // This is the documented behavior: use Option<i64> for NA-safe receipt.
        let result: Result<i64, _> = TryFromSexp::try_from_sexp(s);
        // Either Ok(-2147483648) or Err is acceptable per the implementation,
        // but the current code produces Ok (no NA guard on the widening path).
        // This test documents the actual behavior so regressions are caught.
        if let Ok(v) = result {
            assert_eq!(
                v, NA_INTEGER as i64,
                "NA_LOGICAL widened to NA_INTEGER value"
            );
        }
        // If it errors in future (NA guard added), the test should be updated.
    });
}

/// i64 from STRSXP: rejected.
#[test]
fn coerced_scalar_i64_from_strsxp_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::STRSXP, 1);
            let s = g.protect(s);
            s.set_string_elt(0, SEXP::charsxp("hello"));
            s
        };
        let result: Result<i64, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// Option<i64> from NA_integer_ → None.
#[test]
fn coerced_option_i64_from_na_integer_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(NA_INTEGER, &mut g) };
        let v: Option<i64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<i64> from NA_real_ → None.
#[test]
fn coerced_option_i64_from_na_real_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(NA_REAL, &mut g) };
        let v: Option<i64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<i64> from NULL → None.
#[test]
fn coerced_option_i64_from_null_gives_none() {
    r_test_utils::with_r_thread(|| {
        let s = SEXP::nil();
        let v: Option<i64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<i64> from NA_logical_ → None.
#[test]
fn coerced_option_i64_from_na_logical_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        let v: Option<i64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// u64 from INTSXP: negative value is rejected.
#[test]
fn coerced_scalar_u64_from_intsxp_negative_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(-1, &mut g) };
        let result: Result<u64, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// u32 from REALSXP: value with fractional part is rejected.
#[test]
fn coerced_scalar_u32_from_realsxp_fractional_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(1.5, &mut g) };
        let result: Result<u32, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// i8 from INTSXP: overflow (300) is rejected.
#[test]
fn coerced_scalar_i8_from_intsxp_overflow_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(300, &mut g) };
        let result: Result<i8, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// i8 from RAWSXP: small value succeeds.
#[test]
fn coerced_scalar_i8_from_rawsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_raw(50, &mut g) };
        let v: i8 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 50i8);
    });
}

/// f32 from INTSXP: succeeds.
#[test]
fn coerced_scalar_f32_from_intsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(10, &mut g) };
        let v: f32 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 10.0f32);
    });
}

/// f32 from RAWSXP: succeeds.
#[test]
fn coerced_scalar_f32_from_rawsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_raw(5, &mut g) };
        let v: f32 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 5.0f32);
    });
}

/// NA_integer_ in coerced scalar path: i32::MIN passed to i64 coerce → error (not silently 0).
#[test]
fn coerced_scalar_i64_from_na_integer_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(NA_INTEGER, &mut g) };
        // Plain i64 (not Option<i64>) should error on NA
        let result: Result<i64, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

// endregion

// region: from_r/coerced_scalars.rs — f32/usize/isize delegation onto
// try_from_sexp_numeric_scalar/_option (audit D5, #1145-ish)
//
// f32/usize/isize (+ their Options) used to hand-roll the same 4-arm
// INTSXP/REALSXP/RAWSXP/LGLSXP match that i64/u64 already delegate to
// try_from_sexp_numeric_scalar / try_from_sexp_numeric_option. These tests
// pin behavior parity (including the out-of-range/NA error shapes) across
// the collapse to delegating one-liners, plus real _unchecked coverage for
// usize/isize (previously `try_from_sexp_unchecked` just called the checked
// path).

/// f32 from INTSXP: widens cleanly.
#[test]
fn coerced_scalar_f32_from_intsxp_widens() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(7, &mut g) };
        let v: f32 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 7.0f32);
    });
}

/// f32 from REALSXP: widens cleanly.
#[test]
fn coerced_scalar_f32_from_realsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(1.5, &mut g) };
        let v: f32 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 1.5f32);
    });
}

/// f32 from LGLSXP: TRUE → 1.0.
#[test]
fn coerced_scalar_f32_from_lglsxp_true() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(SEXP::scalar_logical(true)) };
        let v: f32 = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 1.0f32);
    });
}

/// f32 from LGLSXP NA: rejected (mirrors the bare i32 NA guard).
#[test]
fn coerced_scalar_f32_from_lglsxp_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        let result: Result<f32, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// f32 from STRSXP: rejected.
#[test]
fn coerced_scalar_f32_from_strsxp_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(SEXP::scalar_string_from_str("nope")) };
        let result: Result<f32, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// Option<f32> from NA_integer_ → None.
#[test]
fn coerced_option_f32_from_na_integer_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(NA_INTEGER, &mut g) };
        let v: Option<f32> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<f32> from NA_real_ → None.
#[test]
fn coerced_option_f32_from_na_real_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(NA_REAL, &mut g) };
        let v: Option<f32> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<f32> from NA_logical_ → None.
#[test]
fn coerced_option_f32_from_na_logical_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        let v: Option<f32> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<f32> from NULL → None.
#[test]
fn coerced_option_f32_from_null_gives_none() {
    r_test_utils::with_r_thread(|| {
        let s = SEXP::nil();
        let v: Option<f32> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<f32> from RAWSXP: raw has no NA, widens directly.
#[test]
fn coerced_option_f32_from_rawsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_raw(9, &mut g) };
        let v: Option<f32> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, Some(9.0f32));
    });
}

/// usize from INTSXP: widens cleanly.
#[test]
fn coerced_scalar_usize_from_intsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(42, &mut g) };
        let v: usize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 42usize);
    });
}

/// usize from INTSXP: negative value is rejected.
#[test]
fn coerced_scalar_usize_from_intsxp_negative_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(-1, &mut g) };
        let result: Result<usize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// usize from REALSXP: in-range whole number succeeds.
#[test]
fn coerced_scalar_usize_from_realsxp_in_range() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(100.0, &mut g) };
        let v: usize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 100usize);
    });
}

/// usize from REALSXP: negative value is rejected (out-of-range low).
#[test]
fn coerced_scalar_usize_from_realsxp_negative_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(-1.0, &mut g) };
        let result: Result<usize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// usize from REALSXP: fractional value is rejected.
#[test]
fn coerced_scalar_usize_from_realsxp_fractional_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(2.5, &mut g) };
        let result: Result<usize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// usize from REALSXP: NA_real_ is rejected (not None — only Option<usize> maps NA → None).
#[test]
fn coerced_scalar_usize_from_realsxp_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(NA_REAL, &mut g) };
        let result: Result<usize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// usize from RAWSXP: widens directly.
#[test]
fn coerced_scalar_usize_from_rawsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_raw(255, &mut g) };
        let v: usize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 255usize);
    });
}

/// usize from LGLSXP: TRUE → 1.
#[test]
fn coerced_scalar_usize_from_lglsxp_true() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(1, &mut g) };
        let v: usize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 1usize);
    });
}

/// usize from LGLSXP NA: rejected.
#[test]
fn coerced_scalar_usize_from_lglsxp_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        let result: Result<usize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// usize `_unchecked` path now real-delegates (not checked-fallback) and stays
/// behavior-identical to the checked path for the happy path.
#[test]
fn coerced_scalar_usize_unchecked_matches_checked() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(17, &mut g) };
        let v: usize = unsafe { TryFromSexp::try_from_sexp_unchecked(s).unwrap() };
        assert_eq!(v, 17usize);
    });
}

/// Option<usize> from NA_integer_ → None.
#[test]
fn coerced_option_usize_from_na_integer_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(NA_INTEGER, &mut g) };
        let v: Option<usize> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<usize> from NA_real_ → None.
#[test]
fn coerced_option_usize_from_na_real_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(NA_REAL, &mut g) };
        let v: Option<usize> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<usize> from NULL → None.
#[test]
fn coerced_option_usize_from_null_gives_none() {
    r_test_utils::with_r_thread(|| {
        let s = SEXP::nil();
        let v: Option<usize> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<usize> from REALSXP: out-of-range (negative) errors, doesn't silently
/// become None or wrap.
#[test]
fn coerced_option_usize_from_realsxp_negative_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(-5.0, &mut g) };
        let result: Result<Option<usize>, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// Option<usize> `_unchecked` path now real-delegates to the numeric-option
/// unchecked helper.
#[test]
fn coerced_option_usize_unchecked_matches_checked() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(NA_INTEGER, &mut g) };
        let v: Option<usize> = unsafe { TryFromSexp::try_from_sexp_unchecked(s).unwrap() };
        assert!(v.is_none());
    });
}

/// isize from INTSXP: negative values are allowed (unlike usize).
#[test]
fn coerced_scalar_isize_from_intsxp_negative() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(-42, &mut g) };
        let v: isize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, -42isize);
    });
}

/// isize from REALSXP: in-range whole number succeeds.
#[test]
fn coerced_scalar_isize_from_realsxp_in_range() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(-100.0, &mut g) };
        let v: isize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, -100isize);
    });
}

/// isize from REALSXP: fractional value is rejected.
#[test]
fn coerced_scalar_isize_from_realsxp_fractional_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(-2.5, &mut g) };
        let result: Result<isize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// isize from REALSXP: NA_real_ is rejected (not None — only Option<isize> maps NA → None).
#[test]
fn coerced_scalar_isize_from_realsxp_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(NA_REAL, &mut g) };
        let result: Result<isize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// isize from RAWSXP: widens directly.
#[test]
fn coerced_scalar_isize_from_rawsxp() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_raw(200, &mut g) };
        let v: isize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 200isize);
    });
}

/// isize from LGLSXP: FALSE → 0.
#[test]
fn coerced_scalar_isize_from_lglsxp_false() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(0, &mut g) };
        let v: isize = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, 0isize);
    });
}

/// isize from LGLSXP NA: rejected.
#[test]
fn coerced_scalar_isize_from_lglsxp_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        let result: Result<isize, _> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err());
    });
}

/// isize `_unchecked` path now real-delegates (not checked-fallback).
#[test]
fn coerced_scalar_isize_unchecked_matches_checked() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(-17, &mut g) };
        let v: isize = unsafe { TryFromSexp::try_from_sexp_unchecked(s).unwrap() };
        assert_eq!(v, -17isize);
    });
}

/// Option<isize> from NA_integer_ → None.
#[test]
fn coerced_option_isize_from_na_integer_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(NA_INTEGER, &mut g) };
        let v: Option<isize> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<isize> from NA_real_ → None.
#[test]
fn coerced_option_isize_from_na_real_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(NA_REAL, &mut g) };
        let v: Option<isize> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<isize> from NULL → None.
#[test]
fn coerced_option_isize_from_null_gives_none() {
    r_test_utils::with_r_thread(|| {
        let s = SEXP::nil();
        let v: Option<isize> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<isize> `_unchecked` path now real-delegates to the numeric-option
/// unchecked helper.
#[test]
fn coerced_option_isize_unchecked_matches_checked() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(-9.0, &mut g) };
        let v: Option<isize> = unsafe { TryFromSexp::try_from_sexp_unchecked(s).unwrap() };
        assert_eq!(v, Some(-9isize));
    });
}

// endregion

// region: from_r/logical.rs — Rboolean and bool conversions

/// Rboolean from LGLSXP TRUE.
#[test]
fn logical_rboolean_from_lglsxp_true() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(SEXP::scalar_logical(true)) };
        let v: Rboolean = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, Rboolean::TRUE);
    });
}

/// Rboolean from LGLSXP FALSE.
#[test]
fn logical_rboolean_from_lglsxp_false() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(SEXP::scalar_logical(false)) };
        let v: Rboolean = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, Rboolean::FALSE);
    });
}

/// Rboolean from LGLSXP NA → SexpError::Na.
#[test]
fn logical_rboolean_from_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        let result: Result<Rboolean, SexpError> = TryFromSexp::try_from_sexp(s);
        assert!(matches!(result, Err(SexpError::Na(_))));
    });
}

/// Option<Rboolean> from LGLSXP NA → None.
#[test]
fn logical_option_rboolean_from_na_gives_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_logical_raw(NA_LOGICAL, &mut g) };
        let v: Option<Rboolean> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<Rboolean> from NULL → None.
#[test]
fn logical_option_rboolean_from_null_gives_none() {
    r_test_utils::with_r_thread(|| {
        let s = SEXP::nil();
        let v: Option<Rboolean> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_none());
    });
}

/// Option<Rboolean> from TRUE → Some(TRUE).
#[test]
fn logical_option_rboolean_from_true_gives_some() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(SEXP::scalar_logical(true)) };
        let v: Option<Rboolean> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, Some(Rboolean::TRUE));
    });
}

// endregion

// region: from_r/na_vectors.rs — Vec<Option<T>> NA handling

/// Vec<Option<i32>> with NA → None at the right index.
#[test]
fn na_vector_option_i32_na_becomes_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::INTSXP, 3);
            let s = g.protect(s);
            let sl: &mut [i32] = s.as_mut_slice();
            sl[0] = 1;
            sl[1] = NA_INTEGER;
            sl[2] = 3;
            s
        };
        let v: Vec<Option<i32>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![Some(1), None, Some(3)]);
    });
}

/// Vec<Option<i32>> — all NA → all None.
#[test]
fn na_vector_option_i32_all_na() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::INTSXP, 2);
            let s = g.protect(s);
            let sl: &mut [i32] = s.as_mut_slice();
            sl[0] = NA_INTEGER;
            sl[1] = NA_INTEGER;
            s
        };
        let v: Vec<Option<i32>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![None, None]);
    });
}

/// Vec<Option<f64>> with NA_real_ → None.
#[test]
fn na_vector_option_f64_na_becomes_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::REALSXP, 3);
            let s = g.protect(s);
            let sl: &mut [f64] = s.as_mut_slice();
            sl[0] = 1.0;
            sl[1] = NA_REAL;
            sl[2] = 3.0;
            s
        };
        let v: Vec<Option<f64>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v[0], Some(1.0));
        assert!(v[1].is_none());
        assert_eq!(v[2], Some(3.0));
    });
}

/// Vec<Option<f64>> with regular NaN → Some(NaN) (NaN ≠ NA_real_).
#[test]
fn na_vector_option_f64_regular_nan_is_some() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            let s = g.protect(s);
            let sl: &mut [f64] = s.as_mut_slice();
            sl[0] = f64::NAN; // Not NA_real_, just an IEEE NaN
            s
        };
        let v: Vec<Option<f64>> = TryFromSexp::try_from_sexp(s).unwrap();
        // f64::NAN is NOT NA_real_ — should round-trip as Some(NaN).
        assert!(v[0].is_some());
        assert!(v[0].unwrap().is_nan());
    });
}

/// Vec<Option<i32>> type mismatch: REALSXP → SexpError::Type.
#[test]
fn na_vector_option_i32_type_mismatch_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(1.0, &mut g) };
        let result: Result<Vec<Option<i32>>, _> = TryFromSexp::try_from_sexp(s);
        assert!(matches!(result, Err(SexpError::Type(_))));
    });
}

/// Vec<Option<f64>> empty vector: succeeds with length-0 vec.
#[test]
fn na_vector_option_f64_empty() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(Rf_allocVector(SEXPTYPE::REALSXP, 0)) };
        let v: Vec<Option<f64>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_empty());
    });
}

/// Vec<Option<i32>> empty vector: the 0x1 sentinel pointer must not crash.
#[test]
fn na_vector_option_i32_empty_zero_sentinel() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        // R returns a 0x1 sentinel for empty vector data pointers (Rust 1.93+
        // validates pointer alignment even for len==0 unless we guard it).
        let s = unsafe { g.protect(Rf_allocVector(SEXPTYPE::INTSXP, 0)) };
        let v: Vec<Option<i32>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_empty(), "empty integer vector should yield empty Vec");
    });
}

// endregion

// region: from_r — coerced multi-source vec shell (from_numeric_vec_with)
//
// The tests above hit the *native* Vec<Option<i32>>/Vec<Option<f64>> impls.
// These exercise the *coerced* path (`try_from_sexp_numeric_{,option_}vec`,
// served for i8/i16/i64/u16/u32/u64/isize/usize/f32) that both route through
// the shared `from_numeric_vec_with` dispatch. They pin the previously
// unverified REALSXP/LGLSXP option cells and the plain-path NA-round-through
// contract that distinguishes the NA-unaware sibling from the NA-aware one.

/// Coerced Vec<Option<i64>> from REALSXP: NA_real_ → None (bit-exact, not NaN).
#[test]
fn coerced_option_vec_i64_realsxp_na_becomes_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = g.protect(Rf_allocVector(SEXPTYPE::REALSXP, 3));
            let sl: &mut [f64] = s.as_mut_slice();
            sl[0] = 1.0;
            sl[1] = NA_REAL;
            sl[2] = 3.0;
            s
        };
        let v: Vec<Option<i64>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![Some(1i64), None, Some(3i64)]);
    });
}

/// Coerced Vec<Option<i64>> from REALSXP: a regular NaN is Some, not None.
#[test]
fn coerced_option_vec_i64_realsxp_regular_nan_errors_not_na() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = g.protect(Rf_allocVector(SEXPTYPE::REALSXP, 1));
            let sl: &mut [f64] = s.as_mut_slice();
            sl[0] = f64::NAN; // plain IEEE NaN, NOT NA_real_
            s
        };
        // f64::NAN is not NA_real_, so the element is not None; coercing a NaN to
        // an integer fails, so the whole conversion errors rather than dropping it.
        let result: Result<Vec<Option<i64>>, SexpError> = TryFromSexp::try_from_sexp(s);
        assert!(
            matches!(result, Err(SexpError::InvalidValue(_))),
            "regular NaN must not be treated as NA"
        );
    });
}

/// Coerced Vec<Option<i64>> from LGLSXP: NA_LOGICAL → None, TRUE → 1, FALSE → 0.
#[test]
fn coerced_option_vec_i64_lglsxp_na_becomes_none() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = g.protect(Rf_allocVector(SEXPTYPE::LGLSXP, 3));
            s.set_logical_elt(0, 1); // TRUE
            s.set_logical_elt(1, NA_LOGICAL);
            s.set_logical_elt(2, 0); // FALSE
            s
        };
        let v: Vec<Option<i64>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![Some(1i64), None, Some(0i64)]);
    });
}

/// Coerced Vec<Option<i64>> from RAWSXP: raw has no NA, every byte is Some.
#[test]
fn coerced_option_vec_i64_rawsxp_all_some() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = g.protect(Rf_allocVector(SEXPTYPE::RAWSXP, 3));
            let sl: &mut [u8] = s.as_mut_slice();
            sl[0] = 1;
            sl[1] = 2;
            sl[2] = 255;
            s
        };
        let v: Vec<Option<i64>> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![Some(1i64), Some(2i64), Some(255i64)]);
    });
}

/// Plain (NA-unaware) coerced Vec<i64> from INTSXP: NA_integer_ rounds THROUGH
/// as the i32::MIN sentinel, it is not dropped. This is the footgun documented
/// on the na_vectors sibling and the contract the Option path avoids.
#[test]
fn coerced_plain_vec_i64_intsxp_na_round_through() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = g.protect(Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let sl: &mut [i32] = s.as_mut_slice();
            sl[0] = 1;
            sl[1] = NA_INTEGER;
            sl[2] = 3;
            s
        };
        let v: Vec<i64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![1i64, i32::MIN as i64, 3i64]);
    });
}

/// Plain (NA-unaware) coerced Vec<i64> from LGLSXP: NA_LOGICAL reads via to_i32()
/// and rounds through as i32::MIN; TRUE → 1, FALSE → 0.
#[test]
fn coerced_plain_vec_i64_lglsxp_na_round_through() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = g.protect(Rf_allocVector(SEXPTYPE::LGLSXP, 3));
            s.set_logical_elt(0, 1); // TRUE
            s.set_logical_elt(1, NA_LOGICAL);
            s.set_logical_elt(2, 0); // FALSE
            s
        };
        let v: Vec<i64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![1i64, i32::MIN as i64, 0i64]);
    });
}

/// Coerced vec shell rejects an unsupported SEXPTYPE with the shared error.
#[test]
fn coerced_vec_i64_wrong_type_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(SEXP::scalar_string_from_str("nope")) };
        let result: Result<Vec<i64>, SexpError> = TryFromSexp::try_from_sexp(s);
        assert!(matches!(result, Err(SexpError::InvalidValue(_))));
    });
}

// endregion

// region: from_r.rs (root) — NA_real_ identity via roundtrip, Vec<bool>, i32 NA guard

/// NA_real_ roundtrip: f64 from an NA scalar returns the NA bit pattern.
///
/// This exercises the NA_real_ → Option<f64>::None branch in na_vectors.rs
/// and confirms the bit-exact check works via the public API.
#[test]
fn from_r_na_real_option_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let sexp = Option::<f64>::None.into_sexp();
        // The scalar NA_real_ must be recognised as NA by the Option<f64> path.
        let v: Option<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(v.is_none(), "NA_real_ REALSXP scalar → Option<f64>::None");
    });
}

/// Regular NaN roundtrip: plain f64 NaN passes through as Some(NaN) in Option<f64>.
#[test]
fn from_r_regular_nan_not_na_in_option_f64() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_real(f64::NAN, &mut g) };
        // f64::NAN is NOT NA_real_, so Option<f64> must give Some(nan).
        let v: Option<f64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_some(), "regular NaN is Some in Option<f64>");
        assert!(v.unwrap().is_nan());
    });
}

/// i32 from INTSXP with NA_integer_ (i32::MIN) → SexpError::Na.
#[test]
fn from_r_i32_na_integer_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(NA_INTEGER, &mut g) };
        let result: Result<i32, SexpError> = TryFromSexp::try_from_sexp(s);
        assert!(
            matches!(result, Err(SexpError::Na(_))),
            "NA_integer_ must be rejected by i32 conversion"
        );
    });
}

/// Vec<bool> with NA → SexpError (NA cannot be represented as bool).
#[test]
fn from_r_vec_bool_with_na_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::LGLSXP, 2);
            let s = g.protect(s);
            s.set_logical_elt(0, 1); // TRUE
            s.set_logical_elt(1, NA_LOGICAL);
            s
        };
        let result: Result<Vec<bool>, SexpError> = TryFromSexp::try_from_sexp(s);
        assert!(result.is_err(), "Vec<bool> with NA must error");
    });
}

/// Vec<bool> with no NA → round-trips.
#[test]
fn from_r_vec_bool_no_na() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::LGLSXP, 3);
            let s = g.protect(s);
            s.set_logical_elt(0, 1); // TRUE
            s.set_logical_elt(1, 0); // FALSE
            s.set_logical_elt(2, 1); // TRUE
            s
        };
        let v: Vec<bool> = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(v, vec![true, false, true]);
    });
}

/// Vec<i32> from empty INTSXP: the 0x1 data-pointer sentinel must not crash.
#[test]
fn from_r_vec_i32_empty_zero_sentinel() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(Rf_allocVector(SEXPTYPE::INTSXP, 0)) };
        let v: Vec<i32> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_empty(), "empty INTSXP → empty Vec<i32>");
    });
}

/// Vec<f64> from empty REALSXP: the 0x1 sentinel must not crash.
#[test]
fn from_r_vec_f64_empty_zero_sentinel() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect(Rf_allocVector(SEXPTYPE::REALSXP, 0)) };
        let v: Vec<f64> = TryFromSexp::try_from_sexp(s).unwrap();
        assert!(v.is_empty(), "empty REALSXP → empty Vec<f64>");
    });
}

// endregion

// region: into_r_as.rs — IntoRAs storage-directed conversions
//
// IntoRAs<Target> uses the trait-level generic, not the method level.
// Syntax: IntoRAs::<Target>::into_r_as(value)

use miniextendr_api::into_r_as::IntoRAs;

/// Vec<i64> IntoRAs<i32> — all fit → INTSXP.
#[test]
fn into_r_as_vec_i64_to_i32_all_fit() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1i64, 2, i32::MAX as i64];
        let sexp = IntoRAs::<i32>::into_r_as(v).unwrap();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        assert_eq!(sexp.len(), 3);
    });
}

/// Vec<i64> IntoRAs<i32> — out-of-range → error.
#[test]
fn into_r_as_vec_i64_to_i32_out_of_range_errors() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1i64, i64::MAX];
        let result = IntoRAs::<i32>::into_r_as(v);
        assert!(result.is_err(), "out-of-range i64 should error in IntoRAs");
    });
}

/// Vec<f64> IntoRAs<i32> — integral floats fit.
#[test]
fn into_r_as_vec_f64_integral_to_i32() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1.0f64, 2.0, 3.0];
        let sexp = IntoRAs::<i32>::into_r_as(v).unwrap();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        assert_eq!(sexp.len(), 3);
        let data: &[i32] = unsafe { sexp.as_slice() };
        assert_eq!(data, &[1, 2, 3]);
    });
}

/// Vec<f64> IntoRAs<i32> — fractional value errors.
#[test]
fn into_r_as_vec_f64_fractional_to_i32_errors() {
    r_test_utils::with_r_thread(|| {
        let v = vec![1.5f64];
        let result = IntoRAs::<i32>::into_r_as(v);
        assert!(result.is_err());
    });
}

/// Vec<f64> IntoRAs<i32> — NaN value errors.
#[test]
fn into_r_as_vec_f64_nan_to_i32_errors() {
    r_test_utils::with_r_thread(|| {
        let v = vec![f64::NAN];
        let result = IntoRAs::<i32>::into_r_as(v);
        assert!(result.is_err());
    });
}

/// i64 IntoRAs<i32> scalar — fits.
#[test]
fn into_r_as_scalar_i64_to_i32_fits() {
    r_test_utils::with_r_thread(|| {
        let sexp = IntoRAs::<i32>::into_r_as(5i64).unwrap();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    });
}

/// i64 IntoRAs<i32> scalar — too large errors.
#[test]
fn into_r_as_scalar_i64_to_i32_too_large_errors() {
    r_test_utils::with_r_thread(|| {
        let result = IntoRAs::<i32>::into_r_as(i64::MAX);
        assert!(result.is_err());
    });
}

/// Vec<i32> IntoRAs<f64> — NA_integer_ (i32::MIN) sentinel must surface
/// MissingValue rather than silently widening to the finite -2147483648.0.
/// Parallels `into_r_as_vec_f64_nan_to_i32_errors` on the NonFinite path.
///
/// Vector conversions now batch element failures (#1097): a single failure
/// still comes back wrapped in `StorageCoerceError::Batched` (one listed
/// entry, `total == 1`) rather than the bare `MissingValue`.
#[test]
fn into_r_as_vec_i32_na_to_f64_missing_value_errors() {
    use miniextendr_api::into_r_as::StorageCoerceError;
    r_test_utils::with_r_thread(|| {
        let v = vec![1i32, NA_INTEGER, 3];
        let result = IntoRAs::<f64>::into_r_as(v);
        assert!(
            matches!(
                &result,
                Err(StorageCoerceError::Batched {
                    container: "Vec<i32>",
                    listed,
                    total: 1,
                }) if matches!(
                    listed.as_slice(),
                    [StorageCoerceError::MissingValue {
                        to: "f64",
                        index: Some(1)
                    }]
                )
            ),
            "NA_integer_ in Vec<i32> must batch as MissingValue at index 1, got {result:?}"
        );
    });
}

/// All-valid Vec<i32> IntoRAs<f64> still widens successfully — guards against
/// the NA check rejecting ordinary negative values.
#[test]
fn into_r_as_vec_i32_to_f64_all_valid() {
    r_test_utils::with_r_thread(|| {
        let v = vec![-5i32, 0, 7];
        let sexp = IntoRAs::<f64>::into_r_as(v).unwrap();
        assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
        let data: &[f64] = unsafe { sexp.as_slice() };
        assert_eq!(data, &[-5.0, 0.0, 7.0]);
    });
}

/// Scalar i32 IntoRAs<f64> — i32::MIN (NA_integer_) must surface MissingValue.
#[test]
fn into_r_as_scalar_i32_na_to_f64_missing_value_errors() {
    use miniextendr_api::into_r_as::StorageCoerceError;
    r_test_utils::with_r_thread(|| {
        let result = IntoRAs::<f64>::into_r_as(i32::MIN);
        assert!(
            matches!(
                result,
                Err(StorageCoerceError::MissingValue {
                    to: "f64",
                    index: None
                })
            ),
            "scalar i32::MIN must error as MissingValue, got {result:?}"
        );
    });
}

// endregion

// region: NA_real_ bit-exact identity — verified via Vec<Option<f64>> roundtrip

/// NA_real_ is the specific R NaN bit pattern, distinct from arithmetic NaN.
///
/// This test verifies the distinction via the Vec<Option<f64>> roundtrip path,
/// which calls is_na_real internally.
#[test]
fn na_real_bit_exact_identity_via_vec_roundtrip() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::altrep_traits::NA_REAL;

        // Build a REALSXP with [NA_REAL, f64::NAN, 1.0].
        let mut g = Guard(0);
        let s = unsafe {
            let s = Rf_allocVector(SEXPTYPE::REALSXP, 3);
            let s = g.protect(s);
            let sl: &mut [f64] = s.as_mut_slice();
            sl[0] = NA_REAL; // R's NA sentinel
            sl[1] = f64::NAN; // arithmetic NaN — NOT the NA bit pattern
            sl[2] = 1.0;
            s
        };

        let v: Vec<Option<f64>> = TryFromSexp::try_from_sexp(s).unwrap();
        // Only the first element (NA_REAL) should be None.
        assert!(v[0].is_none(), "NA_REAL must become None");
        // Regular NaN is Some (it's a valid f64, just not NA).
        assert!(v[1].is_some(), "arithmetic NaN should be Some, not None");
        assert!(v[1].unwrap().is_nan());
        assert_eq!(v[2], Some(1.0));
    });
}

// endregion

// region: from_r/tuples.rs — TryFromSexp for tuples (R list -> (A, B, ...))

/// 2-tuple round-trip: (i32, String) through IntoR then TryFromSexp.
#[test]
fn tuple2_roundtrip_i32_string() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect((42i32, "hello".to_string()).into_sexp()) };
        assert_eq!(s.type_of(), SEXPTYPE::VECSXP);

        let (a, b): (i32, String) = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(a, 42);
        assert_eq!(b, "hello");
    });
}

/// 3-tuple round-trip with a vector element: (f64, bool, Vec<i32>).
#[test]
fn tuple3_roundtrip_f64_bool_vec() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect((1.5f64, true, vec![1i32, 2, 3]).into_sexp()) };

        let (a, b, c): (f64, bool, Vec<i32>) = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(a, 1.5);
        assert!(b);
        assert_eq!(c, vec![1, 2, 3]);
    });
}

/// Arity-8 round-trip exercises the largest generated impl.
#[test]
fn tuple8_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let input = (
            1i32,
            2.0f64,
            true,
            "x".to_string(),
            5i32,
            6.0f64,
            false,
            "y".to_string(),
        );
        let s = unsafe { g.protect(input.into_sexp()) };

        let out: (i32, f64, bool, String, i32, f64, bool, String) =
            TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!(out.0, 1);
        assert_eq!(out.3, "x");
        assert_eq!(out.7, "y");
    });
}

/// Non-list input is a type error naming VECSXP.
#[test]
fn tuple_from_non_list_is_type_error() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { scalar_int(7, &mut g) };

        let r: Result<(i32, i32), SexpError> = TryFromSexp::try_from_sexp(s);
        match r {
            Err(SexpError::Type(e)) => assert_eq!(e.expected, SEXPTYPE::VECSXP),
            other => panic!("expected type error, got {:?}", other.map(|_| ())),
        }
    });
}

/// Length mismatch is a length error naming expected/actual.
#[test]
fn tuple_length_mismatch_is_length_error() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        let s = unsafe { g.protect((1i32, 2i32, 3i32).into_sexp()) };

        let r: Result<(i32, i32), SexpError> = TryFromSexp::try_from_sexp(s);
        match r {
            Err(SexpError::Length(e)) => {
                assert_eq!(e.expected, 2);
                assert_eq!(e.actual, 3);
            }
            other => panic!("expected length error, got {:?}", other.map(|_| ())),
        }
    });
}

/// Two bad elements produce ONE batched diagnostic mentioning both
/// 1-based positions (repo principle: collect all errors in vectorized ops).
#[test]
fn tuple_batches_all_element_errors() {
    r_test_utils::with_r_thread(|| {
        let mut g = Guard(0);
        // list("a", 2L, "c") read as (i32, i32, i32): elements 1 and 3 fail.
        let s = unsafe { g.protect(("a".to_string(), 2i32, "c".to_string()).into_sexp()) };

        let r: Result<(i32, i32, i32), SexpError> = TryFromSexp::try_from_sexp(s);
        match r {
            Err(SexpError::InvalidValue(msg)) => {
                assert!(msg.contains("element 1:"), "missing position 1 in: {msg}");
                assert!(msg.contains("element 3:"), "missing position 3 in: {msg}");
                assert!(!msg.contains("element 2:"), "element 2 is valid: {msg}");
            }
            other => panic!("expected batched InvalidValue, got {:?}", other.map(|_| ())),
        }
    });
}

/// Names on the input list are ignored — conversion is positional.
#[test]
fn tuple_ignores_list_names() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::sys::{Rf_install, Rf_setAttrib};

        let mut g = Guard(0);
        let s = unsafe { g.protect((10i32, 20i32).into_sexp()) };
        unsafe {
            let names = g.protect(vec!["b".to_string(), "a".to_string()].into_sexp());
            Rf_setAttrib(s, Rf_install(c"names".as_ptr()), names);
        }

        let (a, b): (i32, i32) = TryFromSexp::try_from_sexp(s).unwrap();
        assert_eq!((a, b), (10, 20));
    });
}

// endregion
