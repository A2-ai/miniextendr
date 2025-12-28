#![cfg(feature = "connections")]

use miniextendr_api::connection::{
    EXPECTED_CONNECTIONS_VERSION, RCustomConnection, check_connections_version,
};
use miniextendr_api::ffi::R_CONNECTIONS_VERSION;

#[test]
fn connections_version_constant_matches() {
    assert_eq!(EXPECTED_CONNECTIONS_VERSION, R_CONNECTIONS_VERSION);
    check_connections_version();
}

#[test]
#[ignore = "requires embedded R runtime to call build(); placeholder until integration harness exists"]
fn custom_connection_build_would_need_r_runtime() {
    // This documents the intended integration point even though it can't run in CI yet.
    // A future integration test should initialize embedded R, then call:
    // let conn = RCustomConnection::new().description(\"demo\").build(MyState);
    let _ = RCustomConnection::new();
}

#[test]
fn mode_panics_when_too_long() {
    let result = std::panic::catch_unwind(|| {
        let _ = RCustomConnection::new().mode("abcdef"); // >4 chars should panic
    });
    assert!(result.is_err(), "mode longer than 4 chars should panic");
}
