# Plan: #945 — `MiniextendrError` trait (classed typed errors for `Result<T,E>`)

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `feat/945-miniextendr-error-trait`.

Decisions baked in:
- Implement the trait per the issue's spec. The issue's gating item 1 (#665
  first) is WAIVED — #665 is PARKED (unhappy-path-only perf); this trait is
  perf-neutral ergonomics and does not depend on it. Gating item 2 (bench) is
  also waived for the same reason; state both waivers in the PR body.
- **Bound change accepted**: blanket `impl<E: std::error::Error>
  MiniextendrError for E` replaces the Err arm's `E: Debug` requirement
  (unreleased project, no backcompat). Any repo fixture whose `E` is
  Debug-only gets `Display + std::error::Error` implemented on it in this PR.
  Flag the bound change prominently in the PR body.
- #670 is already closed; the PR carries `Fixes #945` only.

## Design (from the issue, confirmed against current code)

New trait in `miniextendr-api` (put it in `error_value.rs` or a sibling —
follow where `kind` lives, `error_value.rs:125-131`):

```rust
pub trait MiniextendrError {
    fn message(&self) -> std::borrow::Cow<'_, str>;
    fn class(&self) -> Option<&str> { None }
    fn kind(&self) -> &str { crate::error_value::kind::RESULT_ERR }
}
impl<E: std::error::Error> MiniextendrError for E {
    fn message(&self) -> std::borrow::Cow<'_, str> { self.to_string().into() }
}
```

Coherence constraints (verified live — do not violate):
- `impl<T, E> IntoR for Result<T, E>` (`into_r/result.rs:81`) and the
  `NullOnErr` impl (`:142`) — `MiniextendrError` must stay a macro-only
  trait, never a blanket competing with `IntoR`.
- `AsRError<E: std::error::Error>` (`condition.rs`, module docs `:12,:32`) —
  unrelated value-style path; unchanged.
- The tagged-SEXP return contract and the body's unwind guard stay exactly
  as-is (`with_r_unwind_protect_shim` ABI; MXL300).

## Work items

1. Add the trait + blanket in miniextendr-api; rustdoc with a hand-impl
   example (`tryCatch(my_class = ...)` on the R side); re-export via the
   prelude if `kind` / `error_value` items are prelude-exported (match the
   crate's existing re-export pattern — grep `pub use.*error_value` in
   `lib.rs`/`prelude`).
2. Macro Err arms: inventory with
   `grep -rn "kind::RESULT_ERR" miniextendr-macros/src/` (currently
   `return_type_analysis.rs:267,279` and the corresponding
   `c_wrapper_builder.rs` / method-path sites). Every arm that emits
   `&format!("{:?}", e), kind::RESULT_ERR, None` becomes
   `&e.message(), e.kind(), e.class()` (call attribution argument
   unchanged). All arms must route through `MiniextendrError` — none may
   keep the Debug formatting.
3. Fixtures in rpkg: (a) a fn returning `Result<i32, std::io::Error>`-style
   std error — R error message equals the `Display` text (was `Debug`);
   (b) a custom error type hand-implementing `MiniextendrError` with
   `class() = Some("my_custom_error")` — testthat
   `tryCatch(..., my_custom_error = function(e) ...)` catches it, and
   `conditionMessage` matches `message()`. New exports → ×2 install rule.
4. Message-shape sweep: existing testthat pins on `Result` Err messages used
   the `Debug` rendering (`{:?}` — quoted/struct-ish). Grep
   `rpkg/tests/testthat/` for pins tied to Err-arm messages and update to
   Display text.
5. Snapshots (macros insta) rebaseline; trybuild `.stderr` may move if the
   bound change alters an error-path fixture — if a UI test now fails
   because its `E` is Debug-only, that IS the bound change working: update
   the fixture's expected stderr via CI-authoritative flow, or adjust the
   fixture per the baked decision. Never `TRYBUILD=overwrite` to paper over
   an unexplained diff (#1239).
6. Docs: the error-handling docs page (grep `RESULT_ERR`/`unwrap_in_r` in
   `docs/`) gains a "typed classed errors" subsection with the hand-impl
   example. Cross-package: `just cross-test` must stay green (ABI unchanged).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/945-api.log
cargo test -p miniextendr-macros 2>&1 > /tmp/945-macros.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
just test 2>&1 > /tmp/945-rust.log
just devtools-test 2>&1 > /tmp/945-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/945-devtools.log   # devtools::test always exits 0
just cross-test 2>&1 > /tmp/945-cross.log
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- `IntoR for Result` impls, `AsRError`, the unwind guard, the tagged-SEXP
  slot layout, `error!`/`condition!` macro paths.
- No C-side direct-raise experiments (#665 is parked).

## Done criteria

- Custom classed errors catchable via `tryCatch(<class> = ...)`; std errors
  render via Display; all Err arms routed through the trait; suites,
  snapshots, cross-package, three clippy legs green; `Fixes #945`.

## Escalation rule

If reality diverges from this plan — coherence breaks (E0119) anywhere, an
Err arm's context can't supply the trait bound, the bound change cascades
beyond fixtures — **stop, commit nothing further, and report back. Do not
improvise.**
