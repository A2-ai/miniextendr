//! Benchmarks for safe vs raw data pointer access patterns.
//!
//! Compares `SexpExt::as_slice()` (safe, handles empty vectors) vs raw
//! `ffi::INTEGER()`/`ffi::REAL()` pointer access. Measures the overhead
//! of the safety wrapper that prevents SIGABRT on empty vectors (R returns
//! misaligned 0x1 sentinel for empty vector data pointers).

use miniextendr_api::ffi::{self, SEXP, SexpExt};
use miniextendr_api::IntoR;

const SIZE_INDICES: &[usize] = &[0, 2, 4];

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Group 1: Integer slice access
// =============================================================================

mod integer {
    use super::*;

    /// Safe `as_slice::<i32>()` — empty check + DATAPTR_RO + from_raw_parts.
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_sum(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        let slice: &[i32] = unsafe { sexp.as_slice() };
        let sum: i64 = slice.iter().map(|&x| x as i64).sum();
        divan::black_box(sum);
        assert_eq!(slice.len(), len);
    }

    /// Unchecked `as_slice_unchecked::<i32>()` — no thread safety asserts.
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_unchecked_sum(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        let slice: &[i32] = unsafe { sexp.as_slice_unchecked() };
        let sum: i64 = slice.iter().map(|&x| x as i64).sum();
        divan::black_box(sum);
        assert_eq!(slice.len(), len);
    }

    /// Raw `ffi::INTEGER()` pointer + manual iteration (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn raw_pointer_sum(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        unsafe {
            let ptr = ffi::INTEGER(sexp);
            let mut sum: i64 = 0;
            for i in 0..len {
                sum += *ptr.add(i) as i64;
            }
            divan::black_box(sum);
        }
    }

    /// Safe `as_slice()` — just the slice creation, no iteration.
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_only(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        let slice: &[i32] = unsafe { sexp.as_slice() };
        divan::black_box(slice.as_ptr());
    }

    /// Raw `ffi::INTEGER()` — just the pointer, no iteration.
    #[divan::bench(args = SIZE_INDICES)]
    fn raw_pointer_only(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        unsafe {
            divan::black_box(ffi::INTEGER(sexp));
        }
    }
}

// =============================================================================
// Group 2: Real slice access
// =============================================================================

mod real {
    use super::*;

    /// Safe `as_slice::<f64>()` sum.
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_sum(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
        let slice: &[f64] = unsafe { sexp.as_slice() };
        let sum: f64 = slice.iter().sum();
        divan::black_box(sum);
    }

    /// Unchecked `as_slice_unchecked::<f64>()` sum.
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_unchecked_sum(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
        let slice: &[f64] = unsafe { sexp.as_slice_unchecked() };
        let sum: f64 = slice.iter().sum();
        divan::black_box(sum);
    }

    /// Raw `ffi::REAL()` pointer sum (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn raw_pointer_sum(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
        unsafe {
            let ptr = ffi::REAL(sexp);
            let mut sum: f64 = 0.0;
            for i in 0..len {
                sum += *ptr.add(i);
            }
            divan::black_box(sum);
        }
    }
}

// =============================================================================
// Group 3: Raw (u8) slice access
// =============================================================================

mod raw_bytes {
    use super::*;

    /// Safe `as_slice::<u8>()` sum.
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_sum(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().raw_vec(size_idx);
        let slice: &[u8] = unsafe { sexp.as_slice() };
        let sum: u64 = slice.iter().map(|&x| x as u64).sum();
        divan::black_box(sum);
    }

    /// Raw `ffi::RAW()` pointer sum (baseline).
    #[divan::bench(args = SIZE_INDICES)]
    fn raw_pointer_sum(size_idx: usize) {
        let len = miniextendr_bench::SIZES[size_idx];
        let sexp = miniextendr_bench::fixtures().raw_vec(size_idx);
        unsafe {
            let ptr = ffi::RAW(sexp);
            let mut sum: u64 = 0;
            for i in 0..len {
                sum += *ptr.add(i) as u64;
            }
            divan::black_box(sum);
        }
    }
}

// =============================================================================
// Group 4: Empty vector edge case (where r_slice safety matters most)
// =============================================================================

mod empty_vec {
    use super::*;

    /// Safe `as_slice()` on empty INTSXP — returns &[] without touching data pointer.
    #[divan::bench]
    fn as_slice_empty_int() {
        let sexp: SEXP = Vec::<i32>::new().into_sexp();
        let slice: &[i32] = unsafe { sexp.as_slice() };
        divan::black_box(slice);
    }

    /// Safe `as_slice()` on empty REALSXP.
    #[divan::bench]
    fn as_slice_empty_real() {
        let sexp: SEXP = Vec::<f64>::new().into_sexp();
        let slice: &[f64] = unsafe { sexp.as_slice() };
        divan::black_box(slice);
    }

    /// Safe `as_slice()` on size-1 INTSXP (non-empty, no edge case).
    #[divan::bench]
    fn as_slice_one_int() {
        let sexp: SEXP = vec![42i32].into_sexp();
        let slice: &[i32] = unsafe { sexp.as_slice() };
        divan::black_box(slice);
    }
}

// =============================================================================
// Group 5: TryFromSexp vs as_slice for Vec<T> conversion
// =============================================================================

mod conversion {
    use super::*;
    use miniextendr_api::from_r::TryFromSexp;

    /// `TryFromSexp` for Vec<i32> (full type-check + allocation + copy).
    #[divan::bench(args = SIZE_INDICES)]
    fn try_from_sexp_vec_i32(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        let val: Vec<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// `as_slice` + to_vec (manual conversion: slice creation + allocation + copy).
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_to_vec_i32(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
        let slice: &[i32] = unsafe { sexp.as_slice() };
        let val = slice.to_vec();
        divan::black_box(val);
    }

    /// `TryFromSexp` for Vec<f64>.
    #[divan::bench(args = SIZE_INDICES)]
    fn try_from_sexp_vec_f64(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
        let val: Vec<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }

    /// `as_slice` + to_vec for Vec<f64>.
    #[divan::bench(args = SIZE_INDICES)]
    fn as_slice_to_vec_f64(size_idx: usize) {
        let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
        let slice: &[f64] = unsafe { sexp.as_slice() };
        let val = slice.to_vec();
        divan::black_box(val);
    }
}
