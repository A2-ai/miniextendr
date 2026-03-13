//! Benchmarks for `#[miniextendr(strict)]` conversion overhead.
//!
//! Compares normal (lossy-tolerant) vs strict (panic-on-overflow) conversion
//! paths for types where strict mode adds range checking: i64, u64, isize, usize
//! and their Vec variants.
//!
//! Normal i64 conversions accept INTSXP/REALSXP/RAWSXP/LGLSXP and silently
//! widen to f64 when the value doesn't fit in i32. Strict conversions reject
//! RAWSXP/LGLSXP and panic instead of widening.

use miniextendr_api::IntoR;
use miniextendr_api::ffi::SEXP;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::strict;

const VEC_SIZES: &[usize] = &[1, 1_000, 10_000];

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Helpers: create R vectors with values that fit in i32 (valid for both paths)

/// Create an INTSXP of given length filled with 0..len.
fn make_intsxp(len: usize) -> SEXP {
    let data: Vec<i32> = (0..len as i32).collect();
    data.into_sexp()
}

/// Create a REALSXP of given length filled with 0.0..len as f64.
fn make_realsxp(len: usize) -> SEXP {
    let data: Vec<f64> = (0..len).map(|i| i as f64).collect();
    data.into_sexp()
}
// endregion

// region: Group 1: Scalar output (Rust → R)

mod scalar_output {
    use super::*;

    /// Normal i64 → R: fits in i32, so produces INTSXP.
    #[divan::bench]
    fn normal_i64_in_range() -> SEXP {
        divan::black_box(42i64.into_sexp())
    }

    /// Strict i64 → R: fits in i32, same result but with range check.
    #[divan::bench]
    fn strict_i64_in_range() -> SEXP {
        divan::black_box(strict::checked_into_sexp_i64(42))
    }

    /// Normal i64 → R: out of i32 range, falls back to REALSXP.
    #[divan::bench]
    fn normal_i64_out_of_range() -> SEXP {
        divan::black_box((i32::MAX as i64 + 1).into_sexp())
    }

    /// Normal u64 → R: fits in i32.
    #[divan::bench]
    fn normal_u64_in_range() -> SEXP {
        divan::black_box(42u64.into_sexp())
    }

    /// Strict u64 → R: fits in i32, with range check.
    #[divan::bench]
    fn strict_u64_in_range() -> SEXP {
        divan::black_box(strict::checked_into_sexp_u64(42))
    }

    /// Normal isize → R: fits in i32.
    #[divan::bench]
    fn normal_isize_in_range() -> SEXP {
        divan::black_box(42isize.into_sexp())
    }

    /// Strict isize → R: fits in i32, with range check.
    #[divan::bench]
    fn strict_isize_in_range() -> SEXP {
        divan::black_box(strict::checked_into_sexp_isize(42))
    }

    /// Normal usize → R: fits in i32.
    #[divan::bench]
    fn normal_usize_in_range() -> SEXP {
        divan::black_box(42usize.into_sexp())
    }

    /// Strict usize → R: fits in i32, with range check.
    #[divan::bench]
    fn strict_usize_in_range() -> SEXP {
        divan::black_box(strict::checked_into_sexp_usize(42))
    }
}
// endregion

// region: Group 2: Scalar input (R → Rust)

mod scalar_input {
    use super::*;

    /// Normal INTSXP → i64 (accepts all 4 SEXP types).
    #[divan::bench]
    fn normal_intsxp_to_i64() {
        let sexp = make_intsxp(1);
        let val: i64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict INTSXP → i64 (rejects RAWSXP/LGLSXP, validates range).
    #[divan::bench]
    fn strict_intsxp_to_i64() {
        let sexp = make_intsxp(1);
        let val = strict::checked_try_from_sexp_i64(sexp, "x");
        divan::black_box(val);
    }

    /// Normal REALSXP → i64 (coerces f64 to i64 via TryCoerce).
    #[divan::bench]
    fn normal_realsxp_to_i64() {
        let sexp = make_realsxp(1);
        let val: i64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict REALSXP → i64 (same coercion but stricter type check).
    #[divan::bench]
    fn strict_realsxp_to_i64() {
        let sexp = make_realsxp(1);
        let val = strict::checked_try_from_sexp_i64(sexp, "x");
        divan::black_box(val);
    }

    /// Normal INTSXP → u64.
    #[divan::bench]
    fn normal_intsxp_to_u64() {
        let sexp = make_intsxp(1);
        let val: u64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict INTSXP → u64.
    #[divan::bench]
    fn strict_intsxp_to_u64() {
        let sexp = make_intsxp(1);
        let val = strict::checked_try_from_sexp_u64(sexp, "x");
        divan::black_box(val);
    }
}
// endregion

// region: Group 3: Vec output (Rust → R)

mod vec_output {
    use super::*;

    /// Normal Vec<i64> → R (elements fit in i32, smart per-element dispatch).
    #[divan::bench(args = VEC_SIZES)]
    fn normal_vec_i64(len: usize) -> SEXP {
        let data: Vec<i64> = (0..len as i64).collect();
        divan::black_box(data.into_sexp())
    }

    /// Strict Vec<i64> → R (range check per element, always produces INTSXP).
    #[divan::bench(args = VEC_SIZES)]
    fn strict_vec_i64(len: usize) -> SEXP {
        let data: Vec<i64> = (0..len as i64).collect();
        divan::black_box(strict::checked_vec_i64_into_sexp(data))
    }

    /// Normal Vec<u64> → R.
    #[divan::bench(args = VEC_SIZES)]
    fn normal_vec_u64(len: usize) -> SEXP {
        let data: Vec<u64> = (0..len as u64).collect();
        divan::black_box(data.into_sexp())
    }

    /// Strict Vec<u64> → R (range check per element).
    #[divan::bench(args = VEC_SIZES)]
    fn strict_vec_u64(len: usize) -> SEXP {
        let data: Vec<u64> = (0..len as u64).collect();
        divan::black_box(strict::checked_vec_u64_into_sexp(data))
    }

    /// Normal Vec<isize> → R.
    #[divan::bench(args = VEC_SIZES)]
    fn normal_vec_isize(len: usize) -> SEXP {
        let data: Vec<isize> = (0..len as isize).collect();
        divan::black_box(data.into_sexp())
    }

    /// Strict Vec<isize> → R (range check per element).
    #[divan::bench(args = VEC_SIZES)]
    fn strict_vec_isize(len: usize) -> SEXP {
        let data: Vec<isize> = (0..len as isize).collect();
        divan::black_box(strict::checked_vec_isize_into_sexp(data))
    }

    /// Normal Vec<usize> → R.
    #[divan::bench(args = VEC_SIZES)]
    fn normal_vec_usize(len: usize) -> SEXP {
        let data: Vec<usize> = (0..len).collect();
        divan::black_box(data.into_sexp())
    }

    /// Strict Vec<usize> → R (range check per element).
    #[divan::bench(args = VEC_SIZES)]
    fn strict_vec_usize(len: usize) -> SEXP {
        let data: Vec<usize> = (0..len).collect();
        divan::black_box(strict::checked_vec_usize_into_sexp(data))
    }
}
// endregion

// region: Group 4: Vec input (R → Rust)

mod vec_input {
    use super::*;

    /// Normal INTSXP → Vec<i64> (accepts 4 SEXP types, coerces each element).
    #[divan::bench(args = VEC_SIZES)]
    fn normal_intsxp_to_vec_i64(len: usize) {
        let sexp = make_intsxp(len);
        let val: Vec<i64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict INTSXP → Vec<i64> (rejects RAWSXP/LGLSXP, validates per element).
    #[divan::bench(args = VEC_SIZES)]
    fn strict_intsxp_to_vec_i64(len: usize) {
        let sexp = make_intsxp(len);
        let val = strict::checked_vec_try_from_sexp_i64(sexp, "x");
        divan::black_box(val);
    }

    /// Normal REALSXP → Vec<i64> (f64 → i64 coercion per element).
    #[divan::bench(args = VEC_SIZES)]
    fn normal_realsxp_to_vec_i64(len: usize) {
        let sexp = make_realsxp(len);
        let val: Vec<i64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict REALSXP → Vec<i64>.
    #[divan::bench(args = VEC_SIZES)]
    fn strict_realsxp_to_vec_i64(len: usize) {
        let sexp = make_realsxp(len);
        let val = strict::checked_vec_try_from_sexp_i64(sexp, "x");
        divan::black_box(val);
    }

    /// Normal INTSXP → Vec<u64>.
    #[divan::bench(args = VEC_SIZES)]
    fn normal_intsxp_to_vec_u64(len: usize) {
        let sexp = make_intsxp(len);
        let val: Vec<u64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict INTSXP → Vec<u64>.
    #[divan::bench(args = VEC_SIZES)]
    fn strict_intsxp_to_vec_u64(len: usize) {
        let sexp = make_intsxp(len);
        let val = strict::checked_vec_try_from_sexp_u64(sexp, "x");
        divan::black_box(val);
    }

    /// Normal INTSXP → Vec<isize>.
    #[divan::bench(args = VEC_SIZES)]
    fn normal_intsxp_to_vec_isize(len: usize) {
        let sexp = make_intsxp(len);
        let val: Vec<isize> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict INTSXP → Vec<isize>.
    #[divan::bench(args = VEC_SIZES)]
    fn strict_intsxp_to_vec_isize(len: usize) {
        let sexp = make_intsxp(len);
        let val = strict::checked_vec_try_from_sexp_isize(sexp, "x");
        divan::black_box(val);
    }

    /// Normal INTSXP → Vec<usize>.
    #[divan::bench(args = VEC_SIZES)]
    fn normal_intsxp_to_vec_usize(len: usize) {
        let sexp = make_intsxp(len);
        let val: Vec<usize> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// Strict INTSXP → Vec<usize>.
    #[divan::bench(args = VEC_SIZES)]
    fn strict_intsxp_to_vec_usize(len: usize) {
        let sexp = make_intsxp(len);
        let val = strict::checked_vec_try_from_sexp_usize(sexp, "x");
        divan::black_box(val);
    }
}
// endregion

// region: Group 5: Option scalar output (Rust → R)

mod option_output {
    use super::*;

    /// Normal Option<i64> Some → R.
    #[divan::bench]
    fn normal_option_i64_some() -> SEXP {
        divan::black_box(Some(42i64).into_sexp())
    }

    /// Strict Option<i64> Some → R (range check).
    #[divan::bench]
    fn strict_option_i64_some() -> SEXP {
        divan::black_box(strict::checked_option_i64_into_sexp(Some(42)))
    }

    /// Normal Option<i64> None → R (produces NA_integer_).
    #[divan::bench]
    fn normal_option_i64_none() -> SEXP {
        divan::black_box(Option::<i64>::None.into_sexp())
    }

    /// Strict Option<i64> None → R (produces NA_integer_, no range check needed).
    #[divan::bench]
    fn strict_option_i64_none() -> SEXP {
        divan::black_box(strict::checked_option_i64_into_sexp(None))
    }
}
// endregion

// region: Group 6: Vec<Option<T>> output (Rust → R)

mod vec_option_output {
    use super::*;

    /// Normal Vec<Option<i64>> → R (all Some, in i32 range).
    #[divan::bench(args = VEC_SIZES)]
    fn normal_vec_option_i64(len: usize) -> SEXP {
        let data: Vec<Option<i64>> = (0..len as i64).map(Some).collect();
        divan::black_box(data.into_sexp())
    }

    /// Strict Vec<Option<i64>> → R (range check per Some element).
    #[divan::bench(args = VEC_SIZES)]
    fn strict_vec_option_i64(len: usize) -> SEXP {
        let data: Vec<Option<i64>> = (0..len as i64).map(Some).collect();
        divan::black_box(strict::checked_vec_option_i64_into_sexp(data))
    }

    /// Normal Vec<Option<i64>> → R (10% NA density).
    #[divan::bench(args = VEC_SIZES)]
    fn normal_vec_option_i64_with_na(len: usize) -> SEXP {
        let data: Vec<Option<i64>> = (0..len as i64)
            .map(|i| if i % 10 == 0 { None } else { Some(i) })
            .collect();
        divan::black_box(data.into_sexp())
    }

    /// Strict Vec<Option<i64>> → R (10% NA density, range check per Some).
    #[divan::bench(args = VEC_SIZES)]
    fn strict_vec_option_i64_with_na(len: usize) -> SEXP {
        let data: Vec<Option<i64>> = (0..len as i64)
            .map(|i| if i % 10 == 0 { None } else { Some(i) })
            .collect();
        divan::black_box(strict::checked_vec_option_i64_into_sexp(data))
    }
}
// endregion
