+++
title = "Trait-ABI + ALTREP: route panics through tagged-SEXP for rust_* class layering (#345)"
description = "Make panics from cross-package trait method calls and ALTREP RUnwind callbacks match tryCatch(rust_error = h, ...)"
+++

# Issue #345 — class layering for trait-ABI + ALTREP

Closes #345 (probably). May also close #347 if approach 1 leaves no remaining
caller of the non-`error_in_r` path.

---

## Problem

PR #344 unified the Rust→R condition pipeline for `#[miniextendr]` fns/methods:
panics produce a tagged SEXP, the generated R wrapper inspects it, raises
`stop(structure(list(message, call, kind, class), class = c("rust_error",
"simpleError", "error", "condition")))`. `tryCatch(rust_error = h, ...)`
matches.

Two callsites still bypass that pipeline and call `Rf_errorcall` directly via
`with_r_unwind_protect`:

1. **Trait-ABI vtable shims**
   - `miniextendr-macros/src/miniextendr_trait.rs:808` — shim for trait
     definition (used by same-package impls)
   - `miniextendr-macros/src/miniextendr_impl_trait/vtable.rs:489` — concrete
     monomorphized shim for generic trait impls
2. **ALTREP `RUnwind` guard**
   - `miniextendr-api/src/ffi_guard.rs:71` → `with_r_unwind_protect_sourced`,
     reached through `guarded_altrep_call` for callbacks marked
     `#[altrep(r_unwind)]`.

A panic from either surfaces as a plain `simpleError`. `tryCatch(rust_error =
h, ...)` does **not** match. This is documented in `docs/CONDITIONS.md`
("Known limitations") and `miniextendr-api/src/unwind_protect.rs:271/291`.

The framework is internally inconsistent until both paths can carry layered
`rust_*` classes (and user-supplied `class = "..."` from `error!()`).

---

## Why it's hard

There is **no R wrapper** between the user's R code and the C entry point in
either context:

- **Trait shims** invoked via vtable — the vtable function pointer is reached
  from another `#[miniextendr]` fn that does `view.method()` in Rust. The
  consumer's outer `#[miniextendr]` fn *does* have an R wrapper, but the
  shim's panic happens inside a nested call that returns through Rust, not
  through R.
- **ALTREP callbacks** — invoked directly by R's runtime (vector dispatch,
  GC). No `.Call` frame, no R wrapper at all.

Without an R wrapper to inspect the tagged SEXP and translate into
`stop(structure(...))`, naive "return tagged SEXP from shim" approaches fail.

---

## Approaches

### Approach 1 — Tagged-SEXP plumbed through the shim → re-panicked at the View boundary (RECOMMENDED for trait shims)

The vtable shim must keep its `SEXP` return type — the View's `from_sexp` is
the consumer-side reconstruction. Insight: **we already have an outer
`error_in_r` context** for cross-package consumption, because the consumer
calls `view.method()` from inside a `#[miniextendr]` fn whose generated C
entry point is wrapped in `with_r_unwind_protect_error_in_r`.

End-to-end flow:

```
Consumer fn (has R wrapper + error_in_r) ──► View::method() ──► vtable shim
                                                                    │
                                                                    │ panic with RCondition::Error{class=["my_class"], ..}
                                                                    ▼
                                            shim's with_r_unwind_protect_*  catches
                                            returns make_rust_condition_value(...) tagged SEXP
                                                                    │
                            View::method() inspects result, finds rust_error_value
                                                                    │
                            re-panic with same RCondition (preserves class vector)
                                                                    │
                                            consumer's outer error_in_r catches re-panic,
                                            returns tagged SEXP through .Call()
                                                                    │
                                            consumer's R wrapper sees rust_error_value,
                                            stop(structure(..., class = c("my_class","rust_error",...)))
                                                                    ▼
                                            tryCatch(rust_error = h, my_class = h2, ...) ✓
```

What changes:

1. **New shim guard variant** `with_r_unwind_protect_shim` (or extend
   `GuardMode` with `ShimTaggedSexp`):
   - On `RCondition::Error` payload → return `make_rust_condition_value(...)`
     SEXP (unprotected — caller protects).
   - On other panic payloads → return `make_rust_error_value(panic_str, "panic", call=None)`.
   - On R longjmp (`R_ContinueUnwind` path) → continue propagating; the outer
     `error_in_r` will catch it (this preserves R-origin error pass-through).
2. **`vtable.rs:489` and `miniextendr_trait.rs:808`** call the new variant.
3. **View method wrapper** (`miniextendr_trait.rs:582-674`, `generate_method_wrapper_for_view`):
   wrap the `vtable_call` with an `inspect_and_repanic` helper:
   ```rust
   let result = { #vtable_call };
   ::miniextendr_api::trait_abi::repanic_if_rust_error(result)?;
   #result_conversion
   ```
   `repanic_if_rust_error` reads the tagged SEXP back into an `RCondition`
   and `panic_any!`s. Lives in `miniextendr-api/src/condition.rs` next to
   `RCondition`.
4. **Reverse-conversion helper** `RCondition::from_tagged_sexp(sexp) -> Option<RCondition>`
   — reads the 4-element list back. Symmetric inverse of
   `make_rust_condition_value`.

Cost: every cross-package trait method invocation gets one extra
`inherits(.val, "rust_error_value")` check on the SEXP return path
(constant time — just a class attribute check). Negligible vs the
`R_GetCCallable` lookup already in the call.

### Approach 2 — Synthesized R closure for trait imports

Generate a tiny R wrapper at the trait-impl import site that wraps the
vtable call in a `.Call`-style entry point with `error_in_r_check_lines`
applied. Requires a new C-callable entry that boxes the SEXP from the shim
plus error inspection on the R side.

Costlier per-invocation (R-side `.Call` overhead, plus the C entry point).
Doesn't help ALTREP (no R-side `.Call` involved). Drop.

### Approach 3 — `Rf_eval` of `stop(structure(...))` from inside the guard

Synthesize an R-side `stop(structure(list(...), class = c(...)))` call
from C and `Rf_eval` it. Works in any context (no R wrapper needed). Cost:
parses + evaluates an R expression on every error path.

This is the **only viable option for ALTREP** — there is no outer
`error_in_r` to catch a re-panic, since R's runtime invokes the callback
directly. For ALTREP we must raise the R error from within the guard.

A pre-built `lang2(install("stop"), structure_call)` cached in a static is
cheap (no parser, no environment lookup beyond `stop` and `structure`).

### Recommended split

- **Trait shims**: Approach 1. Keeps overhead near zero, full class layering,
  works for `error!()`/`warning!()`/`message!()`/`condition!()` modulo
  caveats below.
- **ALTREP**: Approach 3. Build a helper
  `unwind_protect::raise_rust_condition_via_stop(cond, call)` that
  `Rf_eval`s a pre-built `stop(structure(...))` call. Document that
  `warning!()`/`message!()` from ALTREP don't suspend (they degrade to a
  plain panic into `Rf_errorcall`) — this is an existing constraint of
  ALTREP context, not a regression.

---

## Acceptance criteria (mirroring the issue, sharpened)

- [ ] Cross-package trait method panic → `tryCatch(rust_error = h, ...)`
      catches.
- [ ] Cross-package trait method `error!(class = "my_class", "...")` →
      `tryCatch(my_class = h, ...)` catches first; `rust_error = h` second.
- [ ] ALTREP `r_unwind` callback panic → `tryCatch(rust_error = h, ...)`
      catches.
- [ ] ALTREP `r_unwind` callback `error!(class = "my_class", "...")` →
      `tryCatch(my_class = h, ...)` catches.
- [ ] R-origin errors (longjmp from R API calls inside the shim/callback)
      still propagate via `R_ContinueUnwind` — no regression.
- [ ] Bench `unwind_protect.rs` non-panic path within ±5% of pre-#345.
- [ ] `docs/CONDITIONS.md` "Known limitations" section updated:
      trait-ABI/ALTREP error-class layering moves from "limitation" to
      "supported"; `warning!`/`message!` from ALTREP context stays as a
      documented degradation (not a limitation we plan to fix).

---

## Files to modify

| File | Change |
|---|---|
| `miniextendr-api/src/condition.rs` | Add `RCondition::from_tagged_sexp` reverse-conversion. Add `repanic_if_rust_error(sexp) -> ()` (no-op when SEXP isn't tagged; `panic_any!`s the reconstructed RCondition otherwise). |
| `miniextendr-api/src/unwind_protect.rs` | New `with_r_unwind_protect_shim` (returns `SEXP` like `_error_in_r` but used in shim context — or generalize the existing `_error_in_r` and have the shim call it directly). Add `raise_rust_condition_via_stop(cond, call) -> !` helper that `Rf_eval`s `stop(structure(...))`. |
| `miniextendr-api/src/ffi_guard.rs` | `GuardMode::RUnwind` → route through the new tagged-SEXP path when the wrapping context can re-panic; keep direct `Rf_errorcall` path for ALTREP via new `GuardMode::RUnwindAltrep` that calls `raise_rust_condition_via_stop`. |
| `miniextendr-api/src/altrep_traits.rs` | Update `AltrepGuard::RUnwind` codepath to invoke the ALTREP-specific guard. |
| `miniextendr-api/src/altrep_bridge.rs` | `guarded_altrep_call` selects the new ALTREP variant. |
| `miniextendr-api/src/trait_abi.rs` | Re-export `repanic_if_rust_error` for use by macro-generated View method wrappers. |
| `miniextendr-macros/src/miniextendr_trait.rs:582-674` | Insert `repanic_if_rust_error` between the vtable call and the result conversion in the View method wrapper. |
| `miniextendr-macros/src/miniextendr_trait.rs:808` | Call the new shim variant instead of `with_r_unwind_protect`. |
| `miniextendr-macros/src/miniextendr_impl_trait/vtable.rs:489` | Same. |
| `docs/CONDITIONS.md` | Update "Known limitations". Add a section on ALTREP class layering caveats (no warning/message support from callbacks). |
| `miniextendr-api/src/unwind_protect.rs` doc comments at lines 271/291 | Remove the "known limitation" caveat (now handled). |
| `miniextendr-bench/benches/unwind_protect.rs` | Add benches for the new shim path so we can detect regressions. |

---

## Tests

### Rust unit tests

- `miniextendr-api/tests/condition_round_trip.rs` (new):
  `RCondition` → `make_rust_condition_value` → `from_tagged_sexp` round-trip
  preserves message/kind/class. Run inside `with_r_thread`.
- `miniextendr-api/src/unwind_protect.rs` tests:
  `with_r_unwind_protect_shim` returns tagged SEXP for `RCondition::Error`
  panic, plain panic, and (forwards) R longjmp.

### Cross-package R tests (`tests/cross-package/`)

Extend existing producer/consumer fixtures:

- `producer.pkg`: trait method that panics with plain message.
  → `tryCatch(rust_error = handler)` from consumer matches.
- `producer.pkg`: trait method that calls `error!(class = "producer_specific", "...")`.
  → `tryCatch(producer_specific = handler)` from consumer matches; class
  vector reads `c("producer_specific", "rust_error", "simpleError", "error", "condition")`.
- `producer.pkg`: trait method that calls an R API → R errors → producer
  Rf_errorcalls. → consumer sees regular R `simpleError` (no regression on
  the R-origin path).
- `producer.pkg`: trait method that calls `Rf_eval` triggering R-origin
  warning. → consumer sees the warning (no swallowing).

### ALTREP R tests (`rpkg/tests/testthat/`)

Extend `test-altrep-*.R`:

- ALTREP `r_unwind` callback that panics with plain message.
  → `tryCatch(rust_error = handler)` matches.
- ALTREP `r_unwind` callback with `error!(class = "altrep_specific", "...")`.
  → `tryCatch(altrep_specific = handler)` matches.
- ALTREP `r_unwind` callback that `warning!()`s. → degrades to panic →
  plain R error (documented behavior; assert the documented message).

### Bench

- Run `cargo bench --bench unwind_protect -- shim_no_panic` on main and on
  the branch; diff. Target: within ±5% on non-panic path.

---

## Build/test commands

```bash
# Rust
just check
just clippy   # both clippy_default and clippy_all (CLAUDE.md "Reproducing CI clippy")
just test

# rpkg
just configure
just rcmdinstall
just devtools-document   # regenerate R wrappers
just devtools-test

# Cross-package
just cross-install
just cross-test

# Sanity on bench (not committed — local check only)
cargo bench --bench unwind_protect -- shim
```

---

## Commit / PR shape

Single PR `feat/issue-345-trait-altrep-class-layering` against `main`.
Suggested commits:

1. `feat(api): RCondition::from_tagged_sexp + repanic_if_rust_error helper`
2. `feat(api): with_r_unwind_protect_shim for tagged-SEXP shim returns`
3. `feat(api): raise_rust_condition_via_stop for ALTREP RUnwind path`
4. `feat(macros): wire trait shims and View method wrappers through new path`
5. `feat(altrep): route RUnwind guard through Rf_eval(stop(...)) path`
6. `test(cross-package): rust_error class layering across the trait-ABI boundary`
7. `test(rpkg): rust_error class layering across ALTREP RUnwind callbacks`
8. `docs: update CONDITIONS.md known limitations + unwind_protect doc comments`

---

## Gotchas

- **PROTECT discipline**: `make_rust_condition_value` already needs careful
  PROTECT (see `MEMORY.md` "Common gotchas"). The shim path adds **another**
  return through PROTECT-sensitive code — when the shim returns a tagged
  SEXP, the View method wrapper's `repanic_if_rust_error` runs Rust code
  before the SEXP is consumed by `from_sexp`. Need to confirm whether the
  shim's tagged SEXP needs to be PROTECTed across the View's
  `inspect_and_repanic` call. R-devel CI is the failure mode if we get this
  wrong (tests pass on R-release).
- **`R_ContinueUnwind` semantics**: the shim must still let R-origin
  longjmps propagate. `with_r_unwind_protect_error_in_r` already handles
  this — replicate the structure in `_shim`. Don't accidentally convert
  R-origin errors into tagged SEXPs.
- **No `match.call()` available in the shim**: the shim has no R-side
  context. The `call` slot in the tagged SEXP is `None`; the consumer's
  outer R wrapper supplies `sys.call()` via the existing
  `%||% sys.call()` fallback in `error_in_r_check_lines`.
- **ALTREP `Rf_eval(stop(...))` cost**: the stop-call AST should be cached
  in a static `OnceLock<SEXP>` and PROTECTed once at package init via
  `R_PreserveObject`. Each call only fills in the `structure(...)` argument
  with the message/class — fast path, no parsing.
- **Generic trait shims**: `miniextendr_impl_trait/vtable.rs:489` is the
  monomorphized path for `RExtend<i32>` etc. Make sure both shim sites
  pass the same `Option<call>` (i.e., `None`) to the new variant.
- **`unwind_protect_sourced` non-shim callers**: `with_r_unwind_protect`
  and `with_r_unwind_protect_sourced` are still used by other code paths
  (e.g., ALTREP guard). Don't break them — add the new variant alongside,
  do not refactor the existing function in place.
- **`error_in_r_check_lines` switch already handles user classes**: no
  R-side changes needed on the consumer wrapper. The existing
  `condition_switch_lines` reads `.val$class` and prepends user classes
  ahead of `rust_*`.
- **ALTREP `warning!`/`message!`**: the issue says "out of scope". Confirm
  that `RCondition::Warning`/`Message`/`Condition` payloads in the new
  ALTREP guard route through `panic_payload_to_r_error` with a clear
  diagnostic message, not silently swallowed.
- **`docs/CONDITIONS.md` regenerated to `site/`**: after editing `docs/`,
  run `just site-docs && git add docs/ site/content/manual/ site/data/plans.json`
  per CLAUDE.md.
- **MXL300 lint**: do NOT add direct `Rf_error()` calls — the new ALTREP
  helper goes through `Rf_eval(stop(...))` which is the `r_stop`-equivalent
  pattern. `MXL300` should still flag any direct `Rf_error`/`Rf_errorcall`.

---

## Open questions / things to confirm in the spike

1. **Does `from_tagged_sexp` need a PROTECT around the input SEXP?** The
   View method wrapper has just received it from the vtable shim; it is
   reachable through nothing on the R side. Probably yes — wrap the
   `inspect_and_repanic` body in an `Rf_protect`/`Rf_unprotect` pair, or
   structure as `let _guard = OwnedProtect::new(sexp);`.
2. **Does `Rf_eval(stop(...))` from ALTREP context risk re-entering ALTREP
   dispatch?** Stop should longjmp before that's a problem, but worth a
   stress test (lots of ALTREP elt accesses in tight loops + intermittent
   panics).
3. **Approach 1 viability when `view.method()` is called outside any
   `#[miniextendr]` fn** — e.g., from a test harness, from an init
   callback. There's no outer `error_in_r` to catch the re-panic. Does the
   panic propagate to a `catch_unwind` in worker.rs? If not, we abort.
   Likely fix: have `repanic_if_rust_error` fall back to
   `raise_rust_condition_via_stop` when no outer handler exists. Hard to
   detect — may need to always use the `Rf_eval(stop(...))` path for the
   View boundary too. Flag during the spike.

---

## Out of scope

- `warning!()` / `message!()` / `condition!()` from ALTREP callbacks — the
  R-runtime-driven invocation context can't cleanly suspend execution to
  call back into R for non-fatal signals. Document, don't fix.
- Same-package trait calls — those go through normal C wrapper + R wrapper
  with `error_in_r` already. Already supported.
- Refactoring `with_r_unwind_protect` away (#347). If approach 1 leaves no
  callers of `with_r_unwind_protect_sourced` outside the new ALTREP
  variant, follow-up #347 becomes trivial — but do that as a separate PR
  to keep #345's diff focused.
