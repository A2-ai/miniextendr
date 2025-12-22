//! Compile-fail (UI) tests for `miniextendr-macros`.
//!
//! These tests verify that the macros produce helpful error messages for invalid input.
//! Run with: `cargo test --test ui`

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
