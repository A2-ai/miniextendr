//! Test fixtures for AsDisplay, AsDisplayVec, AsFromStr, AsFromStrVec wrappers,
//! and the `as.numeric()`-style AsNumeric / AsNumericVec and `as.integer()`-style
//! AsInteger / AsIntegerVec markers.

use miniextendr_api::convert::{
    AsDisplay, AsDisplayVec, AsFromStr, AsFromStrVec, AsInteger, AsIntegerVec, AsNumeric,
    AsNumericVec,
};
use miniextendr_api::miniextendr;
use std::net::IpAddr;

// region: AsDisplay — T: Display → R character scalar

#[miniextendr(noexport)]
pub fn test_display_ip() -> AsDisplay<IpAddr> {
    AsDisplay("127.0.0.1".parse().unwrap())
}

#[miniextendr(noexport)]
pub fn test_display_number() -> AsDisplay<f64> {
    AsDisplay(std::f64::consts::PI)
}

#[miniextendr(noexport)]
pub fn test_display_bool() -> AsDisplay<bool> {
    AsDisplay(true)
}

// endregion

// region: AsDisplayVec — Vec<T: Display> → R character vector

#[miniextendr(noexport)]
pub fn test_display_vec_ips() -> AsDisplayVec<IpAddr> {
    AsDisplayVec(vec![
        "127.0.0.1".parse().unwrap(),
        "::1".parse().unwrap(),
        "192.168.1.1".parse().unwrap(),
    ])
}

#[miniextendr(noexport)]
pub fn test_display_vec_ints() -> AsDisplayVec<i32> {
    AsDisplayVec(vec![1, 2, 3, 42])
}

// endregion

// region: AsFromStr — R character scalar → T: FromStr

#[miniextendr(noexport)]
pub fn test_fromstr_ip(addr: AsFromStr<IpAddr>) -> bool {
    addr.0.is_loopback()
}

#[miniextendr(noexport)]
pub fn test_fromstr_int(s: AsFromStr<i64>) -> f64 {
    s.0 as f64
}

#[miniextendr(noexport)]
pub fn test_fromstr_bad_input(s: AsFromStr<IpAddr>) -> bool {
    // This should never be reached — parse should fail
    s.0.is_loopback()
}

// endregion

// region: AsFromStrVec — R character vector → Vec<T: FromStr>

#[miniextendr(noexport)]
pub fn test_fromstr_vec_ips(addrs: AsFromStrVec<IpAddr>) -> Vec<bool> {
    addrs.0.into_iter().map(|ip| ip.is_loopback()).collect()
}

#[miniextendr(noexport)]
pub fn test_fromstr_vec_ints(nums: AsFromStrVec<i32>) -> Vec<i32> {
    nums.0
}

// endregion

// region: AsNumeric / AsNumericVec — numbers, text, or factor labels → f64 (NA → None)

#[miniextendr(noexport)]
pub fn test_as_numeric_vec(x: AsNumericVec) -> Vec<Option<f64>> {
    x.0
}

#[miniextendr(noexport)]
pub fn test_as_numeric(x: AsNumeric) -> Option<f64> {
    x.0
}

/// `NULL` and `NA` both come back as `NA_real_`.
#[miniextendr(noexport)]
pub fn test_as_numeric_opt(x: Option<AsNumeric>) -> Option<f64> {
    x.and_then(|v| v.0)
}

// endregion

// region: AsInteger / AsIntegerVec — the same inputs → i32, refusing fractions and out-of-range values

#[miniextendr(noexport)]
pub fn test_as_integer_vec(x: AsIntegerVec) -> Vec<Option<i32>> {
    x.0
}

#[miniextendr(noexport)]
pub fn test_as_integer(x: AsInteger) -> Option<i32> {
    x.0
}

/// `NULL` and `NA` both come back as `NA_integer_`.
#[miniextendr(noexport)]
pub fn test_as_integer_opt(x: Option<AsInteger>) -> Option<i32> {
    x.and_then(|v| v.0)
}

/// `no_na` refuses what the marker reads as missing (`"NA"`, `"NaN"`, `""`).
#[miniextendr(noexport)]
pub fn test_as_integer_no_na(#[miniextendr(no_na)] x: AsIntegerVec) -> Vec<Option<i32>> {
    x.0
}

// endregion
