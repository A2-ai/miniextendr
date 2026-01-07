//! OrderedFloat adapter tests
use miniextendr_api::ordered_float_impl::OrderedFloat;
use miniextendr_api::{miniextendr, miniextendr_module};

#[miniextendr]
pub fn ordered_float_roundtrip(x: OrderedFloat<f64>) -> OrderedFloat<f64> {
    x
}

#[miniextendr]
pub fn ordered_float_roundtrip_vec(x: Vec<OrderedFloat<f64>>) -> Vec<OrderedFloat<f64>> {
    x
}

#[miniextendr]
pub fn ordered_float_sort(x: Vec<f64>) -> Vec<f64> {
    let mut ordered: Vec<OrderedFloat<f64>> = x.into_iter().map(OrderedFloat).collect();
    ordered.sort();
    ordered.into_iter().map(|of| of.0).collect()
}

#[miniextendr]
pub fn ordered_float_is_nan(x: f64) -> bool {
    OrderedFloat(x).is_nan()
}

#[miniextendr]
pub fn ordered_float_is_finite(x: f64) -> bool {
    OrderedFloat(x).is_finite()
}

miniextendr_module! {
    mod ordered_float_adapter_tests;
    fn ordered_float_roundtrip;
    fn ordered_float_roundtrip_vec;
    fn ordered_float_sort;
    fn ordered_float_is_nan;
    fn ordered_float_is_finite;
}
