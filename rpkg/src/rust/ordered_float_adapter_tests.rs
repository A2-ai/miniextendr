//! OrderedFloat adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::ordered_float_impl::OrderedFloat;

/// @noRd
#[miniextendr]
pub fn ordered_float_roundtrip(x: OrderedFloat<f64>) -> OrderedFloat<f64> {
    x
}

/// @noRd
#[miniextendr]
pub fn ordered_float_roundtrip_vec(x: Vec<OrderedFloat<f64>>) -> Vec<OrderedFloat<f64>> {
    x
}

/// @noRd
#[miniextendr]
pub fn ordered_float_sort(x: Vec<f64>) -> Vec<f64> {
    let mut ordered: Vec<OrderedFloat<f64>> = x.into_iter().map(OrderedFloat).collect();
    ordered.sort();
    ordered.into_iter().map(|of| of.0).collect()
}

/// @noRd
#[miniextendr]
pub fn ordered_float_is_nan(x: f64) -> bool {
    OrderedFloat(x).is_nan()
}

/// @noRd
#[miniextendr]
pub fn ordered_float_is_finite(x: f64) -> bool {
    OrderedFloat(x).is_finite()
}

/// Infinity roundtrip
/// @noRd
#[miniextendr]
pub fn ordered_float_inf() -> OrderedFloat<f64> {
    OrderedFloat(f64::INFINITY)
}

/// Negative infinity roundtrip
/// @noRd
#[miniextendr]
pub fn ordered_float_neg_inf() -> OrderedFloat<f64> {
    OrderedFloat(f64::NEG_INFINITY)
}

/// Negative zero roundtrip (should equal positive zero in value)
/// @noRd
#[miniextendr]
pub fn ordered_float_neg_zero() -> OrderedFloat<f64> {
    OrderedFloat(-0.0)
}

/// Sort with Inf, -Inf, NaN, and normal values
/// @noRd
#[miniextendr]
pub fn ordered_float_sort_special(x: Vec<f64>) -> Vec<f64> {
    let mut ordered: Vec<OrderedFloat<f64>> = x.into_iter().map(OrderedFloat).collect();
    ordered.sort();
    ordered.into_iter().map(|of| of.0).collect()
}
