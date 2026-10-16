//! Strict returns of slices, boxed slices and arrays (#1684), driven by
//! `tests/testthat/test-strict-containers.R`.
//!
//! Each fixture returns in-range values, or values outside R's integer range
//! when `big` is true. Under `strict` the first give integer vectors and the
//! second raise, naming every failing element; the `lax_*` twins show what the
//! same shape does without `strict` (a double vector once a value is too large).

use miniextendr_api::miniextendr;

const BIG: i64 = 3_000_000_000;

/// The `isize` rows' second row when `big`: past `i32::MAX` where `isize` can
/// hold `BIG`. A 32-bit `isize` (wasm32) can't, so there it stays in range and
/// the first row's `i32::MIN` (R's `NA_integer_`) is the only failing value.
#[cfg(target_pointer_width = "64")]
const BIG_ISIZE: isize = 3_000_000_000;
#[cfg(not(target_pointer_width = "64"))]
const BIG_ISIZE: isize = 4;

fn isize_rows(big: bool) -> Vec<[isize; 2]> {
    if big {
        vec![[-2_147_483_648, 2], [3, BIG_ISIZE]]
    } else {
        vec![[1, 2], [3, 4]]
    }
}

fn values(big: bool) -> Vec<i64> {
    if big {
        vec![1, BIG, -BIG]
    } else {
        vec![1, 2, 3]
    }
}

/// @noRd
#[miniextendr(noexport, strict)]
pub fn strict_boxed_i64(big: bool) -> Box<[i64]> {
    values(big).into_boxed_slice()
}

/// @noRd
#[miniextendr(noexport)]
pub fn lax_boxed_i64(big: bool) -> Box<[i64]> {
    values(big).into_boxed_slice()
}

/// @noRd
#[miniextendr(noexport, strict)]
pub fn strict_slice_u64(big: bool) -> &'static [u64] {
    if big { &[1, 3_000_000_000] } else { &[1, 2] }
}

/// @noRd
#[miniextendr(noexport)]
pub fn lax_slice_u64(big: bool) -> &'static [u64] {
    if big { &[1, 3_000_000_000] } else { &[1, 2] }
}

/// @noRd
#[miniextendr(noexport, strict)]
pub fn strict_array_usize(big: bool) -> [usize; 2] {
    if big { [1, 3_000_000_000] } else { [1, 2] }
}

/// @noRd
#[miniextendr(noexport)]
pub fn lax_array_usize(big: bool) -> [usize; 2] {
    if big { [1, 3_000_000_000] } else { [1, 2] }
}

/// @noRd
#[miniextendr(noexport, strict)]
pub fn strict_optional_boxed_i64(big: Option<bool>) -> Option<Box<[i64]>> {
    big.map(|big| values(big).into_boxed_slice())
}

/// @noRd
#[miniextendr(noexport)]
pub fn lax_optional_boxed_i64(big: Option<bool>) -> Option<Box<[i64]>> {
    big.map(|big| values(big).into_boxed_slice())
}

/// @noRd
#[miniextendr(noexport, strict)]
pub fn strict_boxed_rows_i64(big: bool) -> Vec<Box<[i64]>> {
    vec![
        values(false).into_boxed_slice(),
        values(big).into_boxed_slice(),
    ]
}

/// @noRd
#[miniextendr(noexport)]
pub fn lax_boxed_rows_i64(big: bool) -> Vec<Box<[i64]>> {
    vec![
        values(false).into_boxed_slice(),
        values(big).into_boxed_slice(),
    ]
}

/// @noRd
#[miniextendr(noexport, strict)]
pub fn strict_array_rows_isize(big: bool) -> Vec<[isize; 2]> {
    isize_rows(big)
}

/// @noRd
#[miniextendr(noexport)]
pub fn lax_array_rows_isize(big: bool) -> Vec<[isize; 2]> {
    isize_rows(big)
}

/// @noRd
#[miniextendr(noexport, strict)]
pub fn strict_optional_slice_rows_u64(big: bool) -> Vec<Option<&'static [u64]>> {
    let last: &'static [u64] = if big { &[3_000_000_000] } else { &[] };
    vec![Some(&[1, 2]), None, Some(last)]
}
