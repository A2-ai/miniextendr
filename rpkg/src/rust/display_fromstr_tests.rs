//! Test fixtures for AsDisplay, AsDisplayVec, AsFromStr, AsFromStrVec wrappers.

use miniextendr_api::convert::{AsDisplay, AsDisplayVec, AsFromStr, AsFromStrVec};
use miniextendr_api::miniextendr;
use std::net::IpAddr;

// region: AsDisplay — T: Display → R character scalar

/// @export
#[miniextendr]
pub fn test_display_ip() -> AsDisplay<IpAddr> {
    AsDisplay("127.0.0.1".parse().unwrap())
}

/// @export
#[miniextendr]
pub fn test_display_number() -> AsDisplay<f64> {
    AsDisplay(std::f64::consts::PI)
}

/// @export
#[miniextendr]
pub fn test_display_bool() -> AsDisplay<bool> {
    AsDisplay(true)
}

// endregion

// region: AsDisplayVec — Vec<T: Display> → R character vector

/// @export
#[miniextendr]
pub fn test_display_vec_ips() -> AsDisplayVec<IpAddr> {
    AsDisplayVec(vec![
        "127.0.0.1".parse().unwrap(),
        "::1".parse().unwrap(),
        "192.168.1.1".parse().unwrap(),
    ])
}

/// @export
#[miniextendr]
pub fn test_display_vec_ints() -> AsDisplayVec<i32> {
    AsDisplayVec(vec![1, 2, 3, 42])
}

// endregion

// region: AsFromStr — R character scalar → T: FromStr

/// @export
#[miniextendr]
pub fn test_fromstr_ip(addr: AsFromStr<IpAddr>) -> bool {
    addr.0.is_loopback()
}

/// @export
#[miniextendr]
pub fn test_fromstr_int(s: AsFromStr<i64>) -> f64 {
    s.0 as f64
}

/// @export
#[miniextendr]
pub fn test_fromstr_bad_input(s: AsFromStr<IpAddr>) -> bool {
    // This should never be reached — parse should fail
    s.0.is_loopback()
}

// endregion

// region: AsFromStrVec — R character vector → Vec<T: FromStr>

/// @export
#[miniextendr]
pub fn test_fromstr_vec_ips(addrs: AsFromStrVec<IpAddr>) -> Vec<bool> {
    addrs.0.into_iter().map(|ip| ip.is_loopback()).collect()
}

/// @export
#[miniextendr]
pub fn test_fromstr_vec_ints(nums: AsFromStrVec<i32>) -> Vec<i32> {
    nums.0
}

// endregion
