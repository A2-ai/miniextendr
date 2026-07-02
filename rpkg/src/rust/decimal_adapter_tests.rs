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

// region: RDecimalOps adapter trait

/// Drive `Decimal` through the `RDecimalOps` adapter trait (audit A7 — the
/// conversion is tested above; the trait methods were not). Calls are
/// trait-qualified so they cannot resolve to inherent `Decimal` methods.
/// @param s Decimal value as string.
/// @param other Second operand as string (for the *_str arithmetic).
/// @param dp Decimal places for `round`.
#[miniextendr]
pub fn decimal_ops_via_trait(s: &str, other: &str, dp: i32) -> Vec<String> {
    use miniextendr_api::rust_decimal_impl::RDecimalOps;

    let d = Decimal::from_str(s).unwrap_or_default();
    vec![
        RDecimalOps::as_string(&d),
        RDecimalOps::sign(&d).to_string(),
        RDecimalOps::scale(&d).to_string(),
        RDecimalOps::is_integer(&d).to_string(),
        RDecimalOps::as_string(&RDecimalOps::round(&d, dp)),
        RDecimalOps::as_string(&RDecimalOps::floor(&d)),
        RDecimalOps::as_string(&RDecimalOps::ceil(&d)),
        RDecimalOps::as_string(&RDecimalOps::trunc(&d)),
        RDecimalOps::as_string(&RDecimalOps::fract(&d)),
        RDecimalOps::as_string(&RDecimalOps::normalize(&d)),
        RDecimalOps::add_str(&d, other).map_or_else(|e| e, |v| RDecimalOps::as_string(&v)),
        RDecimalOps::div_str(&d, other).map_or_else(|e| e, |v| RDecimalOps::as_string(&v)),
    ]
}

/// Numeric-view methods of `RDecimalOps` (`as_f64`, `as_i64`) plus the
/// predicates, via the trait.
/// @param s Decimal value as string.
#[miniextendr]
pub fn decimal_ops_numeric_views(s: &str) -> Vec<f64> {
    use miniextendr_api::rust_decimal_impl::RDecimalOps;

    let d = Decimal::from_str(s).unwrap_or_default();
    vec![
        RDecimalOps::as_f64(&d),
        RDecimalOps::as_i64(&d).map_or(f64::NAN, |v| v as f64),
        f64::from(u8::from(RDecimalOps::is_zero(&d))),
        f64::from(u8::from(RDecimalOps::is_positive(&d))),
        f64::from(u8::from(RDecimalOps::is_negative(&d))),
    ]
}

// endregion
