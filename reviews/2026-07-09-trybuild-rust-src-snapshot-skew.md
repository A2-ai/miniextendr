# 2026-07-09 — trybuild snapshots: rust-src component skews stderr rendering vs CI

## What was attempted

Diagnosing the fleet-wide "Rust tests" reds that appeared when CI's
`dtolnay/rust-toolchain@stable` moved 1.96.1 → 1.97.0 (2026-07-07): 14 of the
`miniextendr-macros` trybuild UI fixtures mismatched on every branch including
`main`. On PR #1239 (the api-side rayon gate for #1117), regenerate the
snapshots with `TRYBUILD=overwrite` so the branch could be the first green one.

## What went wrong

The overwrite ran under the default dev toolchain and rewrote 6 files, but 5 of
them (the four E0080 collision fixtures plus
`derive_dataframe_enum_struct_field_no_derive`) came out with stdlib-span
*source snippets* (`$crate::panicking::panic_fmt(...)` with caret underlines,
`impl<T: fmt::Debug, ...> for Vec<T, A>`, `pub enum Option<T>`) that CI never
prints. Committing them would have flipped CI red on exactly the files we
"fixed". Earlier sessions had already met this class of skew and misattributed
it twice: once as nightly-vs-stable, once as rustc *version* drift (1.95 vs CI
stable; see the #1117 issue text and PR #1239's original "residual local-only
skew" note).

## Root cause

The **rust-src component**, not the rustc version. When rustlib sources are
installed (typical dev machine; rust-analyzer setups), rustc renders
diagnostics whose spans live in the standard library as full source snippets
with carets. CI installs via `dtolnay/rust-toolchain@stable` (minimal profile,
no rust-src) and renders the bare fallbacks instead: `= note: the failure
occurred here` under `--> $RUST/core/src/panic.rs`, and `help:` lines with a
bare `--> $RUST/...` path and no snippet. Same rustc build (1.97.0 2d8144b78)
on both sides. trybuild normalizes the *paths* but not the snippet-vs-fallback
rendering, so any fixture whose expected stderr touches a `$RUST` span is
un-blessable under a rust-src toolchain.

## Fix / recipe

- Bless only under a CI-equivalent toolchain:

  ```bash
  rustup toolchain install 1.97.0 --profile minimal   # no rust-src
  RUSTUP_TOOLCHAIN=1.97.0 TRYBUILD=overwrite cargo test -p miniextendr-macros --test ui
  RUSTUP_TOOLCHAIN=1.97.0 cargo test -p miniextendr-macros --test ui   # verify green
  ```

  (Match the version to CI's current stable; cross-check a sample against the
  ACTUAL blocks in a failed CI job log — `gh api
  repos/A2-ai/miniextendr/actions/jobs/<job-id>/logs` works while the parent
  run is still in progress.)
- On #1239 the reverted 5 files needed no re-bless at all (their committed
  content already matched CI 1.97 byte for byte once the rayon warnings were
  gone). The only true 1.96→1.97 snapshot skew was
  `fn_implicit_dots_name_conflict.stderr`: rustc promoted
  `varargs_without_pattern` (rust-lang/rust#145544) from warn to deny, so the
  error-path stderr gained `error:`/`#[deny(...)]` in place of
  `warning:`/`#[warn(...)]`.
- Expect 5 fixtures to mismatch locally on any rust-src machine even when CI is
  green. That local red is not actionable; do not "fix" it.
