//! Property-based roundtrip tests for Rust ↔ R type conversions.
//!
//! Verifies that `val.into_sexp() → TryFromSexp::try_from_sexp()` preserves values
//! for all core scalar/vector types, catching precision loss, NA confusion, and
//! edge-case failures.
//!
//! All property tests run within a single `with_r_thread` call per test to avoid
//! issues with proptest's forking and R's non-reentrant runtime.
//!
//! Run with: `cargo test -p miniextendr-api --test roundtrip_properties`

mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::into_r::IntoR;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

// =============================================================================
// Helper: run proptest inside with_r_thread
// =============================================================================

/// Run a property test inside the R thread.
/// This avoids issues with proptest forking and R's single-threaded runtime.
fn run_proptest<S>(cases: u32, strategy: S, test_fn: impl Fn(S::Value) + Send + 'static)
where
    S: Strategy + Send + 'static,
    S::Value: Send + std::fmt::Debug,
{
    r_test_utils::with_r_thread(move || {
        let config = Config {
            cases,
            // Disable forking — R cannot be forked safely
            fork: false,
            ..Config::default()
        };
        let mut runner = TestRunner::new(config);
        runner
            .run(&strategy, |val| {
                test_fn(val);
                Ok(())
            })
            .unwrap();
    });
}

/// Protect an SEXP from GC during roundtrip testing.
///
/// Property tests run many iterations inside a single `with_r_thread` call,
/// which can trigger R's garbage collector. SEXPs returned from `into_sexp()`
/// for vector types (INTSXP, REALSXP, LGLSXP, STRSXP) are unprotected and
/// may be collected before `try_from_sexp()` reads them. This wrapper adds
/// `Rf_protect` / `Rf_unprotect` around the roundtrip.
unsafe fn protected_roundtrip<T: TryFromSexp>(sexp: miniextendr_api::ffi::SEXP) -> T
where
    T::Error: std::fmt::Debug,
{
    unsafe {
        miniextendr_api::ffi::Rf_protect(sexp);
        let result = T::try_from_sexp(sexp).unwrap();
        miniextendr_api::ffi::Rf_unprotect(1);
        result
    }
}

// =============================================================================
// Scalar roundtrips
// =============================================================================

#[test]
fn prop_i32_roundtrip() {
    // Exclude i32::MIN which is NA_integer_ in R
    run_proptest(500, (i32::MIN + 1)..=i32::MAX, |val| {
        let sexp = val.into_sexp();
        let recovered: i32 = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

#[test]
fn prop_f64_roundtrip() {
    // Only finite, non-NaN values (NaN/Inf have special meaning in R)
    run_proptest(500, prop::num::f64::NORMAL | prop::num::f64::ZERO, |val| {
        let sexp = val.into_sexp();
        let recovered: f64 = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

#[test]
fn prop_bool_roundtrip() {
    run_proptest(100, any::<bool>(), |val| {
        let sexp = val.into_sexp();
        let recovered: bool = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

#[test]
fn prop_string_roundtrip() {
    // Arbitrary UTF-8 strings excluding NUL (C strings can't contain NUL)
    run_proptest(300, "[^\0]*", |val: String| {
        let sexp = val.clone().into_sexp();
        let recovered: String = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

#[test]
fn prop_u8_roundtrip() {
    run_proptest(256, any::<u8>(), |val| {
        let sexp = val.into_sexp();
        let recovered: u8 = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

// =============================================================================
// Option<T> roundtrips (NA handling)
// =============================================================================

#[test]
fn prop_option_i32_roundtrip() {
    let strategy = prop_oneof![
        3 => ((i32::MIN + 1)..=i32::MAX).prop_map(Some),
        1 => Just(None),
    ];
    run_proptest(300, strategy, |val: Option<i32>| {
        let sexp = val.into_sexp();
        let recovered: Option<i32> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

#[test]
fn prop_option_f64_roundtrip() {
    let strategy = prop_oneof![
        3 => (prop::num::f64::NORMAL | prop::num::f64::ZERO).prop_map(Some),
        1 => Just(None),
    ];
    run_proptest(300, strategy, |val: Option<f64>| {
        let sexp = val.into_sexp();
        let recovered: Option<f64> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

#[test]
fn prop_option_bool_roundtrip() {
    let strategy = prop_oneof![
        3 => any::<bool>().prop_map(Some),
        1 => Just(None),
    ];
    run_proptest(100, strategy, |val: Option<bool>| {
        let sexp = val.into_sexp();
        let recovered: Option<bool> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

#[test]
fn prop_option_string_roundtrip() {
    let strategy = prop_oneof![
        3 => "[^\0]{0,20}".prop_map(Some),
        1 => Just(None),
    ];
    run_proptest(200, strategy, |val: Option<String>| {
        let sexp = val.clone().into_sexp();
        let recovered: Option<String> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

// =============================================================================
// Vec<T> roundtrips
// =============================================================================

#[test]
fn prop_vec_i32_roundtrip() {
    run_proptest(
        200,
        prop::collection::vec((i32::MIN + 1)..=i32::MAX, 0..50),
        |vals: Vec<i32>| {
            let sexp = vals.as_slice().into_sexp();
            let slice: &[i32] = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals.as_slice(), slice);
        },
    );
}

#[test]
fn prop_vec_f64_roundtrip() {
    run_proptest(
        200,
        prop::collection::vec(prop::num::f64::NORMAL | prop::num::f64::ZERO, 0..50),
        |vals: Vec<f64>| {
            let sexp = vals.as_slice().into_sexp();
            let slice: &[f64] = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals.as_slice(), slice);
        },
    );
}

#[test]
fn prop_vec_bool_roundtrip() {
    run_proptest(
        200,
        prop::collection::vec(any::<bool>(), 0..50),
        |vals: Vec<bool>| {
            let sexp = vals.clone().into_sexp();
            let recovered: Vec<bool> = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals, recovered);
        },
    );
}

#[test]
fn prop_vec_string_roundtrip() {
    run_proptest(
        100,
        prop::collection::vec("[^\0]{0,20}", 0..20),
        |vals: Vec<String>| {
            let sexp = vals.as_slice().into_sexp();
            let recovered: Vec<String> = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals, recovered);
        },
    );
}

#[test]
fn prop_vec_u8_roundtrip() {
    run_proptest(
        200,
        prop::collection::vec(any::<u8>(), 0..100),
        |vals: Vec<u8>| {
            let sexp = vals.as_slice().into_sexp();
            let slice: &[u8] = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals.as_slice(), slice);
        },
    );
}

// =============================================================================
// Vec<Option<T>> roundtrips (NA-aware vectors)
// =============================================================================

fn opt_i32_strategy() -> impl Strategy<Value = Option<i32>> {
    prop_oneof![
        3 => ((i32::MIN + 1)..=i32::MAX).prop_map(Some),
        1 => Just(None),
    ]
}

fn opt_f64_strategy() -> impl Strategy<Value = Option<f64>> {
    prop_oneof![
        3 => (prop::num::f64::NORMAL | prop::num::f64::ZERO).prop_map(Some),
        1 => Just(None),
    ]
}

fn opt_bool_strategy() -> impl Strategy<Value = Option<bool>> {
    prop_oneof![
        3 => any::<bool>().prop_map(Some),
        1 => Just(None),
    ]
}

fn opt_string_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        3 => "[^\0]{0,20}".prop_map(Some),
        1 => Just(None),
    ]
}

#[test]
fn prop_vec_option_i32_roundtrip() {
    run_proptest(
        10,
        prop::collection::vec(opt_i32_strategy(), 0..30),
        |vals: Vec<Option<i32>>| {
            let sexp = vals.clone().into_sexp();
            let recovered: Vec<Option<i32>> = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals, recovered);
        },
    );
}

#[test]
fn prop_vec_option_f64_roundtrip() {
    run_proptest(
        10,
        prop::collection::vec(opt_f64_strategy(), 0..30),
        |vals: Vec<Option<f64>>| {
            let sexp = vals.clone().into_sexp();
            let recovered: Vec<Option<f64>> = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals, recovered);
        },
    );
}

#[test]
fn prop_vec_option_bool_roundtrip() {
    run_proptest(
        10,
        prop::collection::vec(opt_bool_strategy(), 0..30),
        |vals: Vec<Option<bool>>| {
            let sexp = vals.clone().into_sexp();
            let recovered: Vec<Option<bool>> = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals, recovered);
        },
    );
}

#[test]
fn prop_vec_option_string_roundtrip() {
    run_proptest(
        10,
        prop::collection::vec(opt_string_strategy(), 0..15),
        |vals: Vec<Option<String>>| {
            let sexp = vals.clone().into_sexp();
            let recovered: Vec<Option<String>> = unsafe { protected_roundtrip(sexp) };
            assert_eq!(vals, recovered);
        },
    );
}

// =============================================================================
// Vec<HashMap<String, T>> roundtrip
// =============================================================================

#[test]
fn prop_vec_hashmap_string_i32_roundtrip() {
    use std::collections::HashMap;
    // Keys: short non-NUL strings; values: valid (non-NA) i32
    let entry_strategy = (
        "[a-z]{1,5}".prop_map(|s: String| s),
        (i32::MIN + 1)..=i32::MAX,
    );
    let map_strategy = prop::collection::hash_map(entry_strategy.0, entry_strategy.1, 0..5);
    let vec_strategy = prop::collection::vec(map_strategy, 0..5);

    run_proptest(10, vec_strategy, |vals: Vec<HashMap<String, i32>>| {
        let sexp = vals.clone().into_sexp();
        let recovered: Vec<HashMap<String, i32>> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(vals, recovered);
    });
}

// =============================================================================
// NamedVector<HashMap<String, T>> roundtrip (named atomic vectors)
// =============================================================================

#[test]
fn prop_named_vector_hashmap_i32_roundtrip() {
    use miniextendr_api::NamedVector;
    use std::collections::HashMap;

    // Keys: unique short lowercase strings; values: valid (non-NA) i32
    let map_strategy = prop::collection::hash_map(
        "[a-z]{1,5}".prop_map(|s: String| s),
        (i32::MIN + 1)..=i32::MAX,
        0..10,
    );

    run_proptest(10, map_strategy, |vals: HashMap<String, i32>| {
        let sexp = NamedVector(vals.clone()).into_sexp();
        let recovered: NamedVector<HashMap<String, i32>> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(vals, recovered.into_inner());
    });
}

#[test]
fn prop_named_vector_hashmap_f64_roundtrip() {
    use miniextendr_api::NamedVector;
    use std::collections::HashMap;

    let map_strategy = prop::collection::hash_map(
        "[a-z]{1,5}".prop_map(|s: String| s),
        prop::num::f64::NORMAL | prop::num::f64::ZERO,
        0..10,
    );

    run_proptest(10, map_strategy, |vals: HashMap<String, f64>| {
        let sexp = NamedVector(vals.clone()).into_sexp();
        let recovered: NamedVector<HashMap<String, f64>> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(vals, recovered.into_inner());
    });
}

#[test]
fn prop_named_vector_btreemap_i32_roundtrip() {
    use miniextendr_api::NamedVector;
    use std::collections::BTreeMap;

    let map_strategy = prop::collection::btree_map(
        "[a-z]{1,5}".prop_map(|s: String| s),
        (i32::MIN + 1)..=i32::MAX,
        0..10,
    );

    run_proptest(10, map_strategy, |vals: BTreeMap<String, i32>| {
        let sexp = NamedVector(vals.clone()).into_sexp();
        let recovered: NamedVector<BTreeMap<String, i32>> = unsafe { protected_roundtrip(sexp) };
        assert_eq!(vals, recovered.into_inner());
    });
}

// =============================================================================
// i64 safe-range roundtrip (via f64 representation)
// =============================================================================

#[test]
fn prop_i64_safe_range_roundtrip() {
    // i64 values within [-2^53, 2^53] can be exactly represented as f64
    run_proptest(300, -(1i64 << 53)..=(1i64 << 53), |val| {
        let sexp = val.into_sexp();
        let recovered: i64 = unsafe { protected_roundtrip(sexp) };
        assert_eq!(val, recovered);
    });
}

// =============================================================================
// Edge case deterministic tests
// =============================================================================

#[test]
fn edge_case_i32_boundaries() {
    r_test_utils::with_r_thread(|| {
        // i32::MIN + 1 is the smallest valid i32 (i32::MIN = NA_integer_)
        let min_valid = i32::MIN + 1;
        let sexp = min_valid.into_sexp();
        let recovered: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, min_valid);

        // i32::MAX
        let sexp = i32::MAX.into_sexp();
        let recovered: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, i32::MAX);

        // Zero
        let sexp = 0i32.into_sexp();
        let recovered: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, 0);

        // -1
        let sexp = (-1i32).into_sexp();
        let recovered: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, -1);
    });
}

#[test]
fn edge_case_f64_special_values() {
    r_test_utils::with_r_thread(|| {
        // Negative zero
        let sexp = (-0.0f64).into_sexp();
        let recovered: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(recovered.is_sign_negative());
        assert_eq!(recovered, 0.0);

        // Very small positive
        let sexp = f64::MIN_POSITIVE.into_sexp();
        let recovered: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, f64::MIN_POSITIVE);

        // Very large
        let sexp = f64::MAX.into_sexp();
        let recovered: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, f64::MAX);

        // Subnormal
        let subnormal = 5e-324_f64;
        let sexp = subnormal.into_sexp();
        let recovered: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, subnormal);
    });
}

#[test]
fn edge_case_empty_vectors() {
    r_test_utils::with_r_thread(|| {
        // Empty i32 vec
        let empty_i32: &[i32] = &[];
        let sexp = empty_i32.into_sexp();
        let recovered: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(recovered.is_empty());

        // Empty f64 vec
        let empty_f64: &[f64] = &[];
        let sexp = empty_f64.into_sexp();
        let recovered: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(recovered.is_empty());

        // Empty string vec
        let empty_str: Vec<String> = vec![];
        let sexp = empty_str.as_slice().into_sexp();
        let recovered: Vec<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(recovered.is_empty());

        // Empty bool vec
        let empty_bool: Vec<bool> = vec![];
        let sexp = empty_bool.clone().into_sexp();
        let recovered: Vec<bool> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(recovered.is_empty());
    });
}

#[test]
fn edge_case_all_na_vector() {
    r_test_utils::with_r_thread(|| {
        // All-NA i32 vector
        let all_na: Vec<Option<i32>> = vec![None, None, None];
        let sexp = all_na.clone().into_sexp();
        let recovered: Vec<Option<i32>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, all_na);

        // All-NA f64 vector
        let all_na: Vec<Option<f64>> = vec![None, None];
        let sexp = all_na.clone().into_sexp();
        let recovered: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, all_na);

        // All-NA string vector
        let all_na: Vec<Option<String>> = vec![None, None, None];
        let sexp = all_na.clone().into_sexp();
        let recovered: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, all_na);

        // All-NA bool vector
        let all_na: Vec<Option<bool>> = vec![None, None, None];
        let sexp = all_na.clone().into_sexp();
        let recovered: Vec<Option<bool>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, all_na);

        // Mixed bool vector
        let mixed: Vec<Option<bool>> = vec![Some(true), None, Some(false)];
        let sexp = mixed.clone().into_sexp();
        let recovered: Vec<Option<bool>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(recovered, mixed);
    });
}

#[test]
fn edge_case_unicode_strings() {
    r_test_utils::with_r_thread(|| {
        let unicode_strings = vec![
            "".to_string(),                 // empty
            " ".to_string(),                // whitespace
            "\t\n".to_string(),             // control chars
            "\u{1F600}".to_string(),        // emoji (grinning face)
            "\u{4E16}\u{754C}".to_string(), // CJK (世界)
            "caf\u{00E9}".to_string(),      // accented Latin (café)
            "\u{0410}\u{0411}".to_string(), // Cyrillic (АБ)
        ];
        let sexp = unicode_strings.as_slice().into_sexp();
        let recovered: Vec<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(unicode_strings, recovered);
    });
}
