//! Compile-fail and compile-pass (UI) tests for `miniextendr-macros`.
//!
//! Compile-fail tests verify that the macros produce helpful error messages for invalid input.
//! Compile-pass tests verify that valid inputs compile successfully.
//! Run with: `cargo test --test ui`

#[test]
fn ui() {
    // `just test` sets MINIEXTENDR_SKIP_UI=1 on its root-workspace leg so the
    // trybuild snapshots run only via `just test-ui`, which reruns them under a
    // CI-matching minimal-profile toolchain. Toolchains carrying the `rust-src`
    // component render stdlib source spans into diagnostics that CI's stable
    // (no rust-src) does not, producing false `.stderr` mismatches (issue #1239).
    // Bare `cargo test` (CI included) leaves the var unset and runs normally.
    if std::env::var_os("MINIEXTENDR_SKIP_UI").is_some() {
        eprintln!("MINIEXTENDR_SKIP_UI set: skipping trybuild UI snapshots (run `just test-ui`).");
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
    t.pass("tests/ui/pass/*.rs");
}
