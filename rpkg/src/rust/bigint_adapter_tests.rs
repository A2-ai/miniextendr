//! num-bigint adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::num_bigint_impl::BigInt;
use std::str::FromStr;

/// Test BigInt string roundtrip via parsing and formatting.
/// @param s String representation of a big integer.
#[miniextendr]
pub fn bigint_roundtrip(s: &str) -> String {
    BigInt::from_str(s)
        .map(|bi| bi.to_string())
        .unwrap_or_else(|_| "PARSE_ERROR".into())
}

/// Test BigInt addition of two string-encoded integers.
/// @param a First big integer as string.
/// @param b Second big integer as string.
#[miniextendr]
pub fn bigint_add(a: &str, b: &str) -> String {
    let a = BigInt::from_str(a).unwrap_or_default();
    let b = BigInt::from_str(b).unwrap_or_default();
    (a + b).to_string()
}

/// Test BigInt multiplication of two string-encoded integers.
/// @param a First big integer as string.
/// @param b Second big integer as string.
#[miniextendr]
pub fn bigint_mul(a: &str, b: &str) -> String {
    let a = BigInt::from_str(a).unwrap_or_default();
    let b = BigInt::from_str(b).unwrap_or_default();
    (a * b).to_string()
}

/// Test BigInt factorial computation.
/// @param n Non-negative integer to compute factorial of.
#[miniextendr]
pub fn bigint_factorial(n: i32) -> String {
    let mut result = BigInt::from(1);
    for i in 2..=n {
        result *= i;
    }
    result.to_string()
}

/// Test whether a BigInt parsed from string is positive.
/// @param s String representation of a big integer.
#[miniextendr]
pub fn bigint_is_positive(s: &str) -> bool {
    BigInt::from_str(s)
        .map(|bi| bi > BigInt::from(0))
        .unwrap_or(false)
}
