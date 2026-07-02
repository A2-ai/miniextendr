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

// region: RBigIntOps / RBigUintOps adapter traits

/// Drive `BigInt` through the `RBigIntOps` adapter trait (audit A7 — the
/// conversion is tested above; the trait methods were not). Calls are
/// trait-qualified so they cannot resolve to inherent `BigInt` methods.
/// @param a Big integer as string.
/// @param b Big integer as string (second operand).
#[miniextendr]
pub fn bigint_ops_via_trait(a: &str, b: &str) -> Vec<String> {
    use miniextendr_api::num_bigint_impl::RBigIntOps;

    let x = BigInt::from_str(a).unwrap_or_default();
    vec![
        RBigIntOps::as_string(&x),
        RBigIntOps::sign(&x).to_string(),
        RBigIntOps::bit_length(&x).to_string(),
        RBigIntOps::as_string(&RBigIntOps::abs(&x)),
        RBigIntOps::as_string(&RBigIntOps::neg(&x)),
        RBigIntOps::add_str(&x, b).map_or_else(|e| e, |v| RBigIntOps::as_string(&v)),
        RBigIntOps::mul_str(&x, b).map_or_else(|e| e, |v| RBigIntOps::as_string(&v)),
        RBigIntOps::as_string(&RBigIntOps::pow(&x, 2)),
    ]
}

/// Drive `BigUint` through the `RBigUintOps` adapter trait. Includes the
/// would-be-negative `sub_str` error path.
/// @param a Unsigned big integer as string.
/// @param b Unsigned big integer as string (second operand).
#[miniextendr]
pub fn biguint_ops_via_trait(a: &str, b: &str) -> Vec<String> {
    use miniextendr_api::num_bigint_impl::{BigUint, RBigUintOps};

    let x = a.parse::<BigUint>().unwrap_or_default();
    vec![
        RBigUintOps::as_string(&x),
        RBigUintOps::is_zero(&x).to_string(),
        RBigUintOps::is_one(&x).to_string(),
        RBigUintOps::bit_length(&x).to_string(),
        RBigUintOps::add_str(&x, b).map_or_else(|e| e, |v| RBigUintOps::as_string(&v)),
        RBigUintOps::sub_str(&x, b).map_or_else(|e| e, |v| RBigUintOps::as_string(&v)),
        RBigUintOps::gcd_str(&x, b).map_or_else(|e| e, |v| RBigUintOps::as_string(&v)),
    ]
}

/// Byte-level round-trip through `RBigIntOps::to_bytes_be` / `to_bytes_le`.
/// @param a Big integer as string.
#[miniextendr]
pub fn bigint_ops_bytes(a: &str) -> Vec<i32> {
    use miniextendr_api::num_bigint_impl::RBigIntOps;

    let x = BigInt::from_str(a).unwrap_or_default();
    let be = RBigIntOps::to_bytes_be(&x);
    let le = RBigIntOps::to_bytes_le(&x);
    vec![be.len() as i32, le.len() as i32]
}

// endregion
