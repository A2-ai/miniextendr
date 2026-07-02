//! OrderedFloat adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::ordered_float_impl::OrderedFloat;

/// Test OrderedFloat<f64> scalar roundtrip through R.
/// @param x Numeric scalar.
#[miniextendr]
pub fn ordered_float_roundtrip(x: OrderedFloat<f64>) -> OrderedFloat<f64> {
    x
}

/// Test Vec<OrderedFloat<f64>> roundtrip through R.
/// @param x Numeric vector.
#[miniextendr]
pub fn ordered_float_roundtrip_vec(x: Vec<OrderedFloat<f64>>) -> Vec<OrderedFloat<f64>> {
    x
}

/// Test sorting a numeric vector using OrderedFloat total ordering.
/// @param x Numeric vector to sort.
#[miniextendr]
pub fn ordered_float_sort(x: Vec<f64>) -> Vec<f64> {
    let mut ordered: Vec<OrderedFloat<f64>> = x.into_iter().map(OrderedFloat).collect();
    ordered.sort();
    ordered.into_iter().map(|of| of.0).collect()
}

/// Test whether a value is NaN using OrderedFloat.
/// @param x Numeric scalar.
#[miniextendr]
pub fn ordered_float_is_nan(x: f64) -> bool {
    OrderedFloat(x).is_nan()
}

/// Test whether a value is finite using OrderedFloat.
/// @param x Numeric scalar.
#[miniextendr]
pub fn ordered_float_is_finite(x: f64) -> bool {
    OrderedFloat(x).is_finite()
}

/// Test positive infinity roundtrip through OrderedFloat.
#[miniextendr]
pub fn ordered_float_inf() -> OrderedFloat<f64> {
    OrderedFloat(f64::INFINITY)
}

/// Test negative infinity roundtrip through OrderedFloat.
#[miniextendr]
pub fn ordered_float_neg_inf() -> OrderedFloat<f64> {
    OrderedFloat(f64::NEG_INFINITY)
}

/// Test negative zero roundtrip through OrderedFloat.
#[miniextendr]
pub fn ordered_float_neg_zero() -> OrderedFloat<f64> {
    OrderedFloat(-0.0)
}

/// Test sorting a vector containing Inf, -Inf, NaN, and normal values.
/// @param x Numeric vector with special values.
#[miniextendr]
pub fn ordered_float_sort_special(x: Vec<f64>) -> Vec<f64> {
    let mut ordered: Vec<OrderedFloat<f64>> = x.into_iter().map(OrderedFloat).collect();
    ordered.sort();
    ordered.into_iter().map(|of| of.0).collect()
}

// region: ROrderedFloatOps adapter trait

/// Drive `OrderedFloat<f64>` through the `ROrderedFloatOps` adapter trait
/// (audit A7 — the fixtures above hit inherent/`FloatCore` methods via Deref;
/// the trait was unexercised). Calls are trait-qualified.
/// @param x Numeric scalar.
#[miniextendr]
pub fn ordered_float_ops_via_trait(x: f64) -> Vec<f64> {
    use miniextendr_api::ordered_float_impl::ROrderedFloatOps;

    let v = OrderedFloat(x);
    vec![
        ROrderedFloatOps::inner(&v),
        ROrderedFloatOps::floor(&v),
        ROrderedFloatOps::ceil(&v),
        ROrderedFloatOps::round(&v),
        ROrderedFloatOps::trunc(&v),
        ROrderedFloatOps::fract(&v),
        ROrderedFloatOps::abs(&v),
        ROrderedFloatOps::signum(&v),
        ROrderedFloatOps::min_with(&v, 0.0),
        ROrderedFloatOps::max_with(&v, 0.0),
        ROrderedFloatOps::clamp_to(&v, -1.0, 1.0),
    ]
}

/// Predicate methods of `ROrderedFloatOps`, via the trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn ordered_float_ops_predicates(x: f64) -> Vec<bool> {
    use miniextendr_api::ordered_float_impl::ROrderedFloatOps;

    let v = OrderedFloat(x);
    vec![
        ROrderedFloatOps::is_nan(&v),
        ROrderedFloatOps::is_infinite(&v),
        ROrderedFloatOps::is_finite(&v),
        ROrderedFloatOps::is_positive(&v),
        ROrderedFloatOps::is_negative(&v),
    ]
}

// endregion
