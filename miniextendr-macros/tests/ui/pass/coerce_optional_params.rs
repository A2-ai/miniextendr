//! Test: `coerce` on the `Option<T>` / `Vec<Option<T>>` forms (#2274).
//!
//! Per parameter, every mapped type compiles, the non-native numeric ones
//! included; function-wide, a type without a mapping keeps its own
//! conversion instead of being an error.

use miniextendr_api::miniextendr;

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

#[miniextendr(coerce)]
pub fn coerced_function_wide(x: Option<i32>, label: String, flags: Option<Vec<i32>>) -> String {
    format!("{label}{x:?}{flags:?}")
}

fn main() {}
