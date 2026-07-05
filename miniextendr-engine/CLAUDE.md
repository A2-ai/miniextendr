# miniextendr-engine

Standalone R embedding for Rust binaries and tests — **not** codegen. This
crate links `libR` (via `build.rs`), initializes R in-process, and hands back
a minimal `REngine` handle. See root `CLAUDE.md` for project rules.
(Wrapper collection/ordering — `collect_r_wrappers()` — lives in
`miniextendr-api/src/registry.rs`, not here.)

## Scope

- `REngineBuilder` / `REngine`: configure args (`with_args`), `interactive`,
  `signal_handlers`; `init()` calls `Rf_initialize_R` directly then
  `setup_Rmainloop()` exactly once (avoids `Rf_initEmbeddedR`'s double setup).
- `ensure_r_home_env()`: resolves `R_HOME` via the env var or `R RHOME`.
- `r_initialized_sentinel()`: checks `R_CStackStart`/`R_CStackDir` markers to
  detect a prior initialization.
- Disables R's C-stack check (`R_CStackLimit = usize::MAX`) before
  `setup_Rmainloop()` — required when R is initialized off the process main
  thread (test harness `r-test-main` thread); see the long comment in `init()`.
- No shutdown: `Rf_endEmbeddedR` is intentionally never called (non-reentrant
  cleanup). The OS reclaims resources at process exit.

## Consumers

- `miniextendr-bench` (harness `init()` boots R once per process).
- `miniextendr-api` dev-dependency (integration tests: `r_test_utils.rs`,
  `thread_nonapi.rs`, `ppsize_limit.rs`).

## Layout

- `lib.rs` — single-file crate (builder, handle, sentinel, `R_HOME` resolution).
- `tests.rs` — sentinel + `ensure_r_home_env` tests (env-var mutation is
  process-global; they serialize on an `ENV_LOCK` mutex).
- `build.rs` — resolves `R_HOME`, emits `-lR` link flags; skipped on wasm32
  targets (checks `CARGO_CFG_TARGET_ARCH`, not `cfg!` — see #482).

## Rules

- **Non-API internals**: uses `Rembedded.h` / `Rinterface.h` symbols. Never
  make an R package's shared library depend on this crate; it is for
  Rust-only executables, integration tests, and benchmarks.
- Init is once-per-process, main-thread(-ish) only; re-initialization returns
  `REngineError::AlreadyInitialized`.
- Verify any change to the initialization sequence against current R sources
  (`background/` has the R source tree) before landing it.
