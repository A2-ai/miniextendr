//! num-bigint adapter tests
use miniextendr_api::num_bigint_impl::BigInt;
use miniextendr_api::{miniextendr, miniextendr_module};
use std::str::FromStr;

#[miniextendr]
pub fn bigint_roundtrip(s: &str) -> String {
    BigInt::from_str(s).map(|bi| bi.to_string()).unwrap_or_else(|_| "PARSE_ERROR".into())
}

#[miniextendr]
pub fn bigint_add(a: &str, b: &str) -> String {
    let a = BigInt::from_str(a).unwrap_or_default();
    let b = BigInt::from_str(b).unwrap_or_default();
    (a + b).to_string()
}

#[miniextendr]
pub fn bigint_mul(a: &str, b: &str) -> String {
    let a = BigInt::from_str(a).unwrap_or_default();
    let b = BigInt::from_str(b).unwrap_or_default();
    (a * b).to_string()
}

#[miniextendr]
pub fn bigint_factorial(n: i32) -> String {
    let mut result = BigInt::from(1);
    for i in 2..=n {
        result *= i;
    }
    result.to_string()
}

#[miniextendr]
pub fn bigint_is_positive(s: &str) -> bool {
    BigInt::from_str(s).map(|bi| bi > BigInt::from(0)).unwrap_or(false)
}

miniextendr_module! {
    mod bigint_adapter_tests;
    fn bigint_roundtrip;
    fn bigint_add;
    fn bigint_mul;
    fn bigint_factorial;
    fn bigint_is_positive;
}
