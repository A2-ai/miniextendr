# 2026-05-26 — worker-channel follow-ups

After landing the contention test (#731), three follow-ups fell out.
Per CLAUDE.md, forward-looking work lives in GitHub issues — this entry
just records the roadmap and the implementation notes for #730.

## Issue queue (flat priority)

- **#730 — `dispatch_to_worker` precondition** (this PR). Tiny: add
  `debug_assert!(is_r_main_thread(), …)` at the top of
  `dispatch_to_worker`, document on `run_on_worker` / `with_r_thread`.
- **#733 — testthat coverage for R-longjmp inside `with_r_thread`**.
  Needs an rpkg-side fixture and a testthat assertion. Not in this PR
  because it requires `just rcmdinstall` + docgen turnaround.
- **#734 — `miniextendr_runtime_shutdown` with a job in flight**.
  Probably requires a runtime change (cancellation token or
  in-flight flag), not just a test. Bigger scope.

## #730 — implementation note

### What we're doing

1. `miniextendr-api/src/worker.rs:419` (top of
   `worker_channel::dispatch_to_worker`): add a `debug_assert!` that
   the caller is the R main thread. Cheap in release (no atomic load),
   loud in debug.
2. Rustdoc on `pub fn run_on_worker` (worker.rs:175ish): add a
   `# Panics` entry calling out the precondition.
3. Rustdoc on `pub fn with_r_thread` (worker.rs:120): refine the
   existing "From the worker thread (during `run_on_worker`)" bullet
   to note that the routing target is *the caller of
   `run_on_worker`*, which must be the R main thread.
4. New `#[test]` inside `worker::tests::worker_tests` (already exists
   as the home for worker-thread-feature-gated cases): exercise the
   debug_assert. Use the same `mpsc + std::thread::spawn` pattern as
   `run_on_worker_reentry_panics_not_deadlocks` so we can
   `catch_unwind` it. `#[cfg(debug_assertions)]` because the assert
   only fires in debug.

### What we're NOT doing

- Promoting to a release `assert!`. The atomic load on every dispatch
  is unwarranted overhead given the .Call invariant; #730 explicitly
  lists this as optional. If a future reviewer wants release-mode
  enforcement, that's a separate change.
- Adding a `Result` return for "caller wasn't main". The contract is a
  programming error, not a runtime condition.
- Touching `with_r_thread` runtime behaviour. The TLS-based routing
  inside `route_to_main_thread` already enforces "must be in a
  `run_on_worker` context" via the `.expect(…)` — only the implicit
  "and the caller of `run_on_worker` was main" was unenforced.

### Verification

1. `cargo test -p miniextendr-api --features worker-thread` runs the
   new test in debug; it must catch the `debug_assert!` panic.
2. `cargo build -p miniextendr-api --release` confirms the assert is
   compiled out.
3. `cargo fmt --all -- --check`, `cargo clippy_default`,
   `cargo clippy_all` (curated feature list from `.github/workflows/ci.yml`).
4. Re-run the #731 stress suite to confirm we didn't regress.
