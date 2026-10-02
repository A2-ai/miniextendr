//! Compile-pass test: `Option<DataFrame>` and `Option<Either<L, R>>` are
//! ordinary parameters. They used to fail with E0275 (overflow evaluating
//! `Vec<HashMap<String, _>>: TryFromSexp`) because only the newtype blanket
//! `Option<T>` impl matched them; `miniextendr-api` now implements both
//! (`NULL` → `None`, anything else converts as the inner type).
//!
//! Covers the generated bindings, not only the trait bound: the argument
//! conversion and its error probe, the native-borrow metadata query, and the
//! `no_na` checks (the R guard plus `__mx_has_na` for `Option<DataFrame>`;
//! `__mx_input_has_na` alone for `Option<Either<..>>`, which has no R guard).

#![allow(dead_code)]

use miniextendr_api::{AsNumericVec, DataFrame, Either, ExternalPtr, TryFromSexp, miniextendr};

#[miniextendr]
pub fn optional_frame(_x: Option<DataFrame>) {}

#[miniextendr]
pub fn optional_choice(_x: Option<Either<f64, String>>) {}

#[miniextendr]
pub fn optional_grid(_x: Option<Either<AsNumericVec, DataFrame>>) {}

#[miniextendr]
pub fn optional_frame_no_na(#[miniextendr(no_na)] x: Option<DataFrame>) -> bool {
    x.is_some()
}

#[miniextendr]
pub fn optional_grid_no_na(
    #[miniextendr(no_na)] x: Option<Either<AsNumericVec, DataFrame>>,
) -> bool {
    x.is_some()
}

/// An `Option<DataFrame>` arm, and an `Option<Either>` beside a borrow.
#[miniextendr]
pub fn optional_frame_arm(
    #[miniextendr(no_na)] x: Either<AsNumericVec, Option<DataFrame>>,
    y: &mut [f64],
    z: Option<Either<i32, String>>,
) -> bool {
    let _ = (y, z);
    x.is_left()
}

// The impls exist for any arms that convert, and forward the inner metadata.
const _: () = assert!(<Option<Either<String, Vec<String>>> as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(!<Option<DataFrame> as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Option<DataFrame> as TryFromSexp>::NATIVE_BORROW.is_none());

#[derive(ExternalPtr)]
pub struct Grid;

#[miniextendr(env)]
impl Grid {
    pub fn new() -> Self {
        Grid
    }

    #[miniextendr(no_na(x))]
    pub fn fill(
        &self,
        x: Option<Either<AsNumericVec, DataFrame>>,
        frame: Option<DataFrame>,
    ) -> bool {
        x.is_some() && frame.is_some()
    }
}

fn main() {}
