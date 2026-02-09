# Plan: Panic Handler and Guard-Safety Migration

## 1. Goal

Audit all production panic/catch boundaries and decide where we can replace ad-hoc panic handling with existing guard safety mechanisms:

1. `with_r_unwind_protect`
2. `with_r_thread`
3. `r_stop` / error helpers
4. shared guard wrappers for FFI boundaries

This plan focuses on runtime code paths (not unit-test panic assertions).

## 2. Scope and Inventory

### Core panic/catch boundaries currently in production

1. `miniextendr-api/src/unwind_protect.rs:95` (`with_r_unwind_protect`) and internals at `miniextendr-api/src/unwind_protect.rs:115`, `miniextendr-api/src/unwind_protect.rs:143`
2. `miniextendr-api/src/worker.rs:271`, `miniextendr-api/src/worker.rs:318`, `miniextendr-api/src/worker.rs:373`
3. `miniextendr-api/src/altrep_bridge.rs:25` (`catch_altrep_panic`)
4. `miniextendr-api/src/connection.rs:399` (`catch_connection_panic`)
5. `miniextendr-macros/src/miniextendr_trait.rs:597` (generated trait shim `catch_unwind`)
6. `miniextendr-macros/src/c_wrapper_builder.rs:280` and `miniextendr-macros/src/c_wrapper_builder.rs:359`
7. `miniextendr-macros/src/lib.rs:947` and `miniextendr-macros/src/lib.rs:1011` (parallel codegen path)
8. Panic-to-R converters:
   - `miniextendr-api/src/worker.rs:141`
   - `miniextendr-api/src/worker.rs:150`
   - `miniextendr-api/src/unwind_protect.rs:41`
9. Panic hook surface:
   - `miniextendr-api/src/backtrace.rs:7`
10. Panic-on-misuse init/thread checks:
   - `miniextendr-api/src/trait_abi/ccall.rs:107`
   - `miniextendr-api/src/trait_abi/ccall.rs:117`
   - `miniextendr-api/src/trait_abi/ccall.rs:128`
   - `miniextendr-api/src/trait_abi/ccall.rs:139`
   - `miniextendr-api/src/trait_abi/ccall.rs:187`
   - `miniextendr-api/src/trait_abi/ccall.rs:220`
   - `miniextendr-api/src/trait_abi/ccall.rs:266`
   - `miniextendr-api/src/ffi.rs:742`, `miniextendr-api/src/ffi.rs:761`, `miniextendr-api/src/ffi.rs:780`, `miniextendr-api/src/ffi.rs:795`, `miniextendr-api/src/ffi.rs:810`

## 3. Decision Matrix (Replace vs Keep)

## A. Keep (foundational, not replaceable)

1. `with_r_unwind_protect` internals (`miniextendr-api/src/unwind_protect.rs`)
Reason: this is the primitive that already combines `R_UnwindProtect` and panic capture.

2. `run_on_worker` internal catch points (`miniextendr-api/src/worker.rs`)
Reason: required for cross-thread panic capture and conversion before returning to R.

3. panic hook registration (`miniextendr-api/src/backtrace.rs:7`)
Reason: diagnostic hook, not an error boundary candidate.

## B. High-confidence migration candidates

1. Generated trait ABI shims (`miniextendr-macros/src/miniextendr_trait.rs:597`)
Current: `catch_unwind` + `panic_message_to_r_error`.
Plan: switch shim body to `with_r_unwind_protect(|| {...}, None)` so panic and R longjmp both run destructor-safe path.

2. Generated wrapper return-error control flow (`miniextendr-macros/src/c_wrapper_builder.rs:425`, `miniextendr-macros/src/c_wrapper_builder.rs:461`, `miniextendr-macros/src/c_wrapper_builder.rs:603`)
Current: expected `Option::None` / `Result::Err` cases are promoted to `panic!` and then converted later.
Plan: replace panic-as-control-flow with direct error path (`r_stop` equivalent) in generated code where on main thread, and structured error transfer from worker path.

3. Trait ABI callable init and accessors (`miniextendr-api/src/trait_abi/ccall.rs`)
Current: panic on missing init/missing symbols.
Plan: introduce `Result`-returning internal APIs plus extern wrapper behavior that converts to R errors at R entrypoints.

## C. Medium-confidence candidates (needs prototype/benchmark)

1. ALTREP trampoline guard (`miniextendr-api/src/altrep_bridge.rs:25`)
Current: `catch_unwind` + `r_stop`.
Plan: prototype `with_r_unwind_protect` wrapping for one callback family and measure overhead/safety benefit (R longjmp cleanup consistency).

2. Macro wrapper paths duplicated in two generators (`miniextendr-macros/src/c_wrapper_builder.rs` and `miniextendr-macros/src/lib.rs`)
Current: duplicated `catch_unwind`/RNG patterns.
Plan: converge to one shared guard-generation path so behavior stays consistent.

## D. Explicitly not replaceable with `with_r_*` as-is

1. Connection callback panic wrapper (`miniextendr-api/src/connection.rs:399`)
Reason: connection callbacks intentionally return fallback values (`FALSE`, `0`, `-1`) on panic; `with_r_unwind_protect` is longjmp-based and does not naturally provide fallback-return semantics.
Action: keep fallback model; only refactor for shared message extraction/telemetry and add explicit docs.

2. Checked FFI wrappers panicking on wrong thread (`miniextendr-api/src/ffi.rs:742` etc.)
Reason: these wrappers intentionally fail-fast for misuse; generic automatic routing is not always safe or possible.
Action: migrate internal call sites toward `error.rs` helpers where routing is appropriate; keep panic checks as guardrails.

## 4. Implementation Phases

## Phase 1: Introduce unified guard helpers ✅ DONE

1. `panic_telemetry` module centralized with `PanicSource` enum and `fire()`.
2. `guarded_altrep_call` dispatches via `T::GUARD` const.
3. `with_r_unwind_protect_sourced` threads PanicSource through unwind-protect.
4. `catch_connection_panic` wraps connection callbacks with fallback returns.

## Phase 2: Trait shim migration to unwind-protect ✅ DONE

1. Changed generated trait shims to use `with_r_unwind_protect` instead of `catch_unwind`.
2. Arity checks preserved (before unwind-protect, uses r_stop).
3. All macro tests (212), cross-package, and rpkg compilation verified.

## Phase 3: Wrapper generation cleanup

1. Remove panic-as-control-flow for expected `Option`/`Result` failure in generated wrapper paths.
2. Keep panic capture only for actual unexpected panics.
3. Consolidate duplicated logic between `c_wrapper_builder` and `lib.rs` wrapper generation.

## Phase 4: Trait ABI ccall init hardening

1. Add `try_init_ccallables()` and `try_mx_*()` APIs returning `Result`.
2. Keep panicking convenience methods only where explicitly desired.
3. Update C-exposed init entrypoint to produce R-facing error behavior instead of Rust panic where feasible.

## Phase 5: ALTREP guard prototype ✅ DONE

1. All 41 trampolines migrated to `guarded_altrep_call` with zero-cost const dispatch.
2. Three guard modes: `#[altrep(unsafe)]`, `#[altrep(rust_unwind)]`, `#[altrep(r_unwind)]`.
3. Full test and benchmark coverage verified.

## 5. Validation Plan

1. `cargo test -p miniextendr-api`
2. `cargo test -p miniextendr-macros`
3. Cross-package interop tests:
   - `tests/cross-package/consumer.pkg/tests/testthat/test-interop.R`
   - `tests/cross-package/consumer.pkg/tests/testthat/test-edge-cases.R`
4. Worker/unwind regressions:
   - `rpkg/tests/testthat/test-worker.R`
   - `rpkg/tests/testthat/test-unwind.R`
5. Performance checks:
   - `miniextendr-bench/benches/wrappers.rs`
   - `miniextendr-bench/benches/trait_abi.rs`
   - add ALTREP callback microbench if needed

## 6. Acceptance Criteria

1. No generated trait shim relies on ad-hoc `catch_unwind` for primary R-entry safety; it uses `with_r_unwind_protect`.
2. Expected `Option`/`Result` error returns in generated wrappers no longer depend on synthetic panics.
3. Panic conversion logic is centralized (no repeated payload parsing implementations).
4. Connection callbacks retain explicit fallback semantics and are documented as intentional exceptions.
5. All regression tests above pass with no new skipped safety cases.

## 7. Open Questions

1. Should trait ABI ccall init failures during package load be hard errors (`r_stop`) or hard panics in debug builds only?
2. Is ALTREP callback overhead acceptable if every trampoline moves to unwind-protect?
3. Do we want a public guard API for downstream custom trampolines, or keep it internal-only?
