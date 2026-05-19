//! Extended coverage for `Vec<Vec<T>>` serde round-tripping across every
//! scalar `T` that has an `IntoR` / `TryFromSexp` impl.
//!
//! PR #649 fixed the round-trip for `Vec<Vec<scalar>>` with all-length-1
//! inner vecs (the `SeqSerializer::end()` coalescing path), with tests for
//! `usize`, `f64`, `String` and triply-nested `i32`. This file sweeps the
//! rest of the scalar matrix so we know every supported leaf type survives
//! the same shape:
//!
//! - signed integers: `i8`, `i16`, `i32`, `i64`
//! - unsigned integers: `u8`, `u16`, `u32`, `u64`
//! - floats: `f32`
//! - booleans: `bool`
//!
//! Each test exercises three shapes: all-length-1 (the bug shape),
//! mixed lengths, and a singleton outer with a singleton inner.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::serde::{from_r, to_r};
use serde::{Deserialize, Serialize};

/// Generate `nested_vec_<T>_*` test trio for a scalar type.
///
/// Produces three `#[test]` functions:
///   - `all_singletons`: every inner vec has length 1 — the #649 bug shape
///   - `mixed_lengths`: mixed inner lengths — the coalescing path doesn't fire
///   - `single_single`: a single outer with a single inner
macro_rules! nested_vec_roundtrip_tests {
    ($t:ty, $mod_name:ident, $singletons:expr, $mixed:expr, $single:expr) => {
        mod $mod_name {
            use super::*;

            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Wrapper {
                nested: Vec<Vec<$t>>,
            }

            #[test]
            fn all_singletons() {
                r_test_utils::with_r_thread(|| {
                    let value = Wrapper {
                        nested: $singletons,
                    };
                    let sexp = to_r(&value).expect("serialize");
                    let back: Wrapper = from_r(sexp).expect("deserialize");
                    assert_eq!(back, value);
                });
            }

            #[test]
            fn mixed_lengths() {
                r_test_utils::with_r_thread(|| {
                    let value = Wrapper { nested: $mixed };
                    let sexp = to_r(&value).expect("serialize");
                    let back: Wrapper = from_r(sexp).expect("deserialize");
                    assert_eq!(back, value);
                });
            }

            #[test]
            fn single_single() {
                r_test_utils::with_r_thread(|| {
                    let value = Wrapper { nested: $single };
                    let sexp = to_r(&value).expect("serialize");
                    let back: Wrapper = from_r(sexp).expect("deserialize");
                    assert_eq!(back, value);
                });
            }
        }
    };
}

nested_vec_roundtrip_tests!(
    i8,
    i8_,
    vec![vec![1i8], vec![-2], vec![127]],
    vec![vec![1i8], vec![2, 3], vec![]],
    vec![vec![42i8]]
);

nested_vec_roundtrip_tests!(
    i16,
    i16_,
    vec![vec![1i16], vec![-2], vec![32767]],
    vec![vec![1i16], vec![2, 3], vec![]],
    vec![vec![42i16]]
);

nested_vec_roundtrip_tests!(
    i32,
    i32_,
    vec![vec![1i32], vec![-2], vec![i32::MAX - 1]],
    vec![vec![1i32], vec![2, 3], vec![]],
    vec![vec![42i32]]
);

// NB: R's native integer width is i32. Values outside [i32::MIN+1, i32::MAX]
// don't round-trip losslessly through INTSXP — they go via REALSXP (f64) and
// lose precision above 2^53. We stay inside i32 range here; lossy-coercion
// behaviour for larger values is a separate concern from #649's coalescing
// fix and warrants its own coverage (see GitHub issue if filed).
nested_vec_roundtrip_tests!(
    i64,
    i64_,
    vec![vec![1i64], vec![-2], vec![i32::MAX as i64]],
    vec![vec![1i64], vec![2, 3], vec![]],
    vec![vec![42i64]]
);

nested_vec_roundtrip_tests!(
    u8,
    u8_,
    vec![vec![1u8], vec![2], vec![255]],
    vec![vec![1u8], vec![2, 3], vec![]],
    vec![vec![42u8]]
);

nested_vec_roundtrip_tests!(
    u16,
    u16_,
    vec![vec![1u16], vec![2], vec![65535]],
    vec![vec![1u16], vec![2, 3], vec![]],
    vec![vec![42u16]]
);

nested_vec_roundtrip_tests!(
    u32,
    u32_,
    vec![vec![1u32], vec![2], vec![i32::MAX as u32]],
    vec![vec![1u32], vec![2, 3], vec![]],
    vec![vec![42u32]]
);

// u64/i64 round-trip is capped at i32::MAX — see comment above i64 tests.
nested_vec_roundtrip_tests!(
    u64,
    u64_,
    vec![vec![1u64], vec![2], vec![i32::MAX as u64]],
    vec![vec![1u64], vec![2, 3], vec![]],
    vec![vec![42u64]]
);

nested_vec_roundtrip_tests!(
    f32,
    f32_,
    vec![vec![1.5f32], vec![-2.5], vec![3.5]],
    vec![vec![1.5f32], vec![2.5, 3.5], vec![]],
    vec![vec![42.0f32]]
);

nested_vec_roundtrip_tests!(
    bool,
    bool_,
    vec![vec![true], vec![false], vec![true]],
    vec![vec![true], vec![false, true], vec![]],
    vec![vec![true]]
);
