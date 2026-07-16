//! Compile-fail (UI) tests for Vctrs derive macros.
//!
//! These tests require the `vctrs` feature to be enabled.
//! Run with: `cargo test --test ui_vctrs --features vctrs`

#[test]
#[cfg(feature = "vctrs")]
fn ui_vctrs() {
    // Same skip contract as tests/ui.rs: `just test` sets MINIEXTENDR_SKIP_UI=1
    // on its root-workspace leg so trybuild snapshots run only via `just test-ui`,
    // which isolates them under a CI-matching minimal-profile (no rust-src)
    // toolchain (issues #1239, #1336). Bare `cargo test` (CI included) leaves
    // the var unset and runs normally.
    if std::env::var_os("MINIEXTENDR_SKIP_UI").is_some() {
        eprintln!("MINIEXTENDR_SKIP_UI set: skipping trybuild UI snapshots (run `just test-ui`).");
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_vctrs/*.rs");
}
