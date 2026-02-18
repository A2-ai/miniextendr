//! Compile-fail (UI) tests for Vctrs derive macros.
//!
//! These tests require the `vctrs` feature to be enabled.
//! Run with: `cargo test --test ui_vctrs --features vctrs`

#[test]
#[cfg(feature = "vctrs")]
fn ui_vctrs() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_vctrs/*.rs");
}
