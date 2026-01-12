//! rust_decimal adapter tests
use miniextendr_api::rust_decimal_impl::Decimal;
use miniextendr_api::{miniextendr, miniextendr_module};
use std::str::FromStr;

/// @noRd
#[miniextendr]
pub fn decimal_roundtrip(s: &str) -> String {
    Decimal::from_str(s)
        .map(|d| d.to_string())
        .unwrap_or_else(|_| "PARSE_ERROR".into())
}

/// @noRd
#[miniextendr]
pub fn decimal_add(a: &str, b: &str) -> String {
    let a = Decimal::from_str(a).unwrap_or_default();
    let b = Decimal::from_str(b).unwrap_or_default();
    (a + b).to_string()
}

/// @noRd
#[miniextendr]
pub fn decimal_mul(a: &str, b: &str) -> String {
    let a = Decimal::from_str(a).unwrap_or_default();
    let b = Decimal::from_str(b).unwrap_or_default();
    (a * b).to_string()
}

/// @noRd
#[miniextendr]
pub fn decimal_round(s: &str, dp: i32) -> String {
    Decimal::from_str(s)
        .map(|d| d.round_dp(dp as u32).to_string())
        .unwrap_or_else(|_| "PARSE_ERROR".into())
}

/// @noRd
#[miniextendr]
pub fn decimal_scale(s: &str) -> i32 {
    Decimal::from_str(s).map(|d| d.scale() as i32).unwrap_or(-1)
}

/// @noRd
#[miniextendr]
pub fn decimal_is_zero(s: &str) -> bool {
    Decimal::from_str(s).map(|d| d.is_zero()).unwrap_or(false)
}

miniextendr_module! {
    mod decimal_adapter_tests;
    fn decimal_roundtrip;
    fn decimal_add;
    fn decimal_mul;
    fn decimal_round;
    fn decimal_scale;
    fn decimal_is_zero;
}
