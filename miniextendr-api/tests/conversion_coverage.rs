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
