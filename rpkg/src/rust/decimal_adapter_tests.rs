//! rust_decimal adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::rust_decimal_impl::Decimal;
use std::str::FromStr;

/// Test Decimal string roundtrip via parsing and formatting.
/// @param s Decimal value as string.
#[miniextendr]
pub fn decimal_roundtrip(s: &str) -> String {
    Decimal::from_str(s)
        .map(|d| d.to_string())
        .unwrap_or_else(|_| "PARSE_ERROR".into())
}

/// Test Decimal addition of two string-encoded values.
/// @param a First decimal as string.
/// @param b Second decimal as string.
#[miniextendr]
pub fn decimal_add(a: &str, b: &str) -> String {
    let a = Decimal::from_str(a).unwrap_or_default();
    let b = Decimal::from_str(b).unwrap_or_default();
    (a + b).to_string()
}

/// Test Decimal multiplication of two string-encoded values.
/// @param a First decimal as string.
/// @param b Second decimal as string.
#[miniextendr]
pub fn decimal_mul(a: &str, b: &str) -> String {
    let a = Decimal::from_str(a).unwrap_or_default();
    let b = Decimal::from_str(b).unwrap_or_default();
    (a * b).to_string()
}

/// Test rounding a Decimal to a given number of decimal places.
/// @param s Decimal value as string.
/// @param dp Number of decimal places.
#[miniextendr]
pub fn decimal_round(s: &str, dp: i32) -> String {
    Decimal::from_str(s)
        .map(|d| d.round_dp(dp as u32).to_string())
        .unwrap_or_else(|_| "PARSE_ERROR".into())
}

/// Test extracting the scale (number of decimal digits) of a Decimal.
/// @param s Decimal value as string.
#[miniextendr]
pub fn decimal_scale(s: &str) -> i32 {
    Decimal::from_str(s).map(|d| d.scale() as i32).unwrap_or(-1)
}

/// Test whether a Decimal value parsed from string is zero.
/// @param s Decimal value as string.
#[miniextendr]
pub fn decimal_is_zero(s: &str) -> bool {
    Decimal::from_str(s).map(|d| d.is_zero()).unwrap_or(false)
}
