//! Functions sharing their file-stem help page, `stem_page_docs` (#1590).
//!
//! Every function here lands on `man/stem_page_docs.Rd`: the first names the
//! page with `@name` + `@rdname stem_page_docs`, the second spells out
//! `@rdname stem_page_docs`, and the third takes the page the wrapper registry
//! injects from the file stem. All three take `values`, which only the first
//! documents. The registry keeps a generated `@param` line only for an
//! argument no function on the page documents: `values` shows the first
//! function's text once (a generated line written later would replace it),
//! and the undocumented `direction` keeps its generated choice list.

use miniextendr_api::miniextendr;

/// @title Values transformed in place
/// @name stem_page_docs
/// @rdname stem_page_docs
/// @description `stem_scale()` multiplies the values by a factor.
/// @param values Numbers to transform.
/// @param factor Multiplier applied to every value.
#[miniextendr]
pub fn stem_scale(values: Vec<f64>, factor: f64) -> Vec<f64> {
    values.into_iter().map(|value| value * factor).collect()
}

/// `stem_shift()` moves the values by an offset.
/// @rdname stem_page_docs
/// @param offset Amount added to every value.
#[miniextendr]
pub fn stem_shift(
    values: Vec<f64>,
    offset: f64,
    #[miniextendr(choices("up", "down"))] direction: &str,
) -> Vec<f64> {
    let offset = if direction == "down" { -offset } else { offset };
    values.into_iter().map(|value| value + offset).collect()
}

/// `stem_floor()` raises the values to at least a floor.
/// @param floor Smallest value kept.
#[miniextendr]
pub fn stem_floor(values: Vec<f64>, floor: f64) -> Vec<f64> {
    values.into_iter().map(|value| value.max(floor)).collect()
}
