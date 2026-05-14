//! Compile-fail and compile-pass (UI) tests for `miniextendr-macros`.
//!
//! Compile-fail tests verify that the macros produce helpful error messages for invalid input.
//! Compile-pass tests verify that valid inputs compile successfully.
//! Run with: `cargo test --test ui`

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
    t.pass("tests/ui/pass/*.rs");
}
