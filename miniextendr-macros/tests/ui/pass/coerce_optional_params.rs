//! Test: `coerce` on the `Option<T>` / `Vec<Option<T>>` / `Missing<T>` forms (#2274).
//!
//! Per parameter, every mapped type compiles, the non-native numeric ones
//! included; function-wide, a type without a mapping keeps its own
//! conversion instead of being an error.

use miniextendr_api::{Missing, miniextendr};

#[miniextendr]
pub fn coerced_optional_native(
    #[miniextendr(coerce)] a: Option<i32>,
    #[miniextendr(coerce)] b: Option<f64>,
    #[miniextendr(coerce)] c: Option<bool>,
    #[miniextendr(coerce)] d: Vec<Option<i32>>,
    #[miniextendr(coerce)] e: Vec<Option<f64>>,
    #[miniextendr(coerce)] f: Vec<Option<bool>>,
) -> usize {
    usize::from(a.is_some())
        + usize::from(b.is_some())
        + usize::from(c.is_some())
        + d.len()
        + e.len()
        + f.len()
}

#[miniextendr]
pub fn coerced_optional_numeric(
    #[miniextendr(coerce)] a: Option<u16>,
    #[miniextendr(coerce)] b: Vec<Option<i64>>,
) -> usize {
    usize::from(a.is_some()) + b.len()
}

#[miniextendr]
pub fn coerced_missing(
    #[miniextendr(coerce)] a: Missing<i32>,
    #[miniextendr(coerce)] b: Missing<f64>,
    #[miniextendr(coerce)] c: Missing<bool>,
    #[miniextendr(coerce)] d: Missing<Vec<i32>>,
    #[miniextendr(coerce)] e: Missing<Option<f64>>,
    #[miniextendr(coerce)] f: Missing<Vec<Option<bool>>>,
    #[miniextendr(coerce)] g: Missing<u16>,
) -> usize {
    usize::from(a.is_present())
        + usize::from(b.is_present())
        + usize::from(c.is_present())
        + d.unwrap_or_default().len()
        + usize::from(e.is_present())
        + f.unwrap_or_default().len()
        + usize::from(g.is_present())
}

#[miniextendr]
pub fn coerced_no_na(
    #[miniextendr(coerce, no_na)] a: i32,
    #[miniextendr(coerce, no_na)] b: Option<f64>,
    #[miniextendr(coerce, no_na)] c: Vec<Option<i32>>,
) -> usize {
    usize::try_from(a).unwrap_or(0) + usize::from(b.is_some()) + c.len()
}

#[miniextendr(coerce)]
pub fn coerced_function_wide(x: Option<i32>, label: String, flags: Option<Vec<i32>>) -> String {
    format!("{label}{x:?}{flags:?}")
}

fn main() {}
