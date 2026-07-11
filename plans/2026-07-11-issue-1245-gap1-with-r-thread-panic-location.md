# Plan: #1245 Gap 1 — real panic location for `with_r_thread` panics from worker fns

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1245-with-r-thread-panic-location`.

## The bug (recap, anchors verified)

A `panic!` inside a `with_r_thread` closure invoked from a worker-dispatched
`#[miniextendr]` fn takes this path:

1. Worker calls `route_to_main_thread` → main thread services the
   `WorkRequest` and catches the user panic; it stringifies via
   `panic_payload_to_string` — **no location** — at
   `miniextendr-api/src/worker.rs:714` (trampoline-caught) and `:731`
   (catch_unwind-caught).
2. The `Err(panic_msg)` string returns to the worker, which re-panics:
   `worker.rs:490` — `Err(panic_msg) => panic!("panic in \`with_r_thread\`: {}", panic_msg)`.
3. The process panic hook (backtrace.rs) records `worker.rs:490` in the
   **worker's** take-once slot; the worker channel fold at `worker.rs:602-606`
   appends `(at …worker.rs:490)` via `panic_message_with_location`.

Result: the R message carries framework glue's location instead of the user's
`panic!` site. The real site IS available in the **main thread's** hook slot
at the two stringify points (the user closure panicked on main).

## Design

Introduce a marker payload so the location folds exactly once, at the point
that knows the truth:

1. **New type** in `miniextendr-api/src/unwind_protect.rs` (beside
   `panic_payload_to_string`, `:220`):
   ```rust
   /// Panic payload whose message is already final (location folded, or
   /// deliberately location-free): downstream folds must use it verbatim and
   /// must NOT append the current thread's recorded panic location (#1245).
   pub(crate) struct PreLocatedPanic(pub(crate) String);
   ```
2. **Main-thread stringify points** (`worker.rs:714` and `:731`): replace
   `panic_payload_to_string(&*payload).into_owned()` with the same
   RCondition-vs-generic branch the worker fold uses at `:602-606`:
   - `payload.is::<crate::condition::RCondition>()` → verbatim
     `panic_payload_to_string(...)` (conditions stay location-free — the
     documented contract in `unwind_protect.rs:243-245`);
   - otherwise `panic_message_with_location(&*payload)` — running on the main
     thread, whose slot holds the user's site.
   Factor the two-armed branch into a small private helper if it keeps the two
   sites identical; do not change the surrounding control flow.
3. **Worker re-panic** (`worker.rs:490`): replace the `panic!(...)` with
   `std::panic::panic_any(crate::unwind_protect::PreLocatedPanic(format!("panic in `with_r_thread`: {panic_msg}")))`.
4. **Worker fold** (`worker.rs:602-606`): add a `PreLocatedPanic` arm that
   (a) uses the inner string verbatim, and (b) **drains the stale slot** —
   call `crate::backtrace::take_last_panic_location()` and discard — because
   the `panic_any` at step 3 fired the hook on the worker thread and recorded
   `worker.rs:<line>`; leaving it would leak into a later unrelated fold on
   the same worker. Add a comment saying exactly that.
5. **`panic_payload_to_string`** (`unwind_protect.rs:220-230`): add a
   `PreLocatedPanic` downcast arm (before the fallback) returning the inner
   string, so the other consumers (`worker.rs:430,824,889`) render it
   correctly if such a payload ever reaches them.
6. **Gap 2: explicitly NOT fixed here** (decision baked in). Add a short
   comment at the `panic_error_handling` construction in
   `miniextendr-macros/src/c_wrapper_builder.rs:648` citing issue #1245 Gap 2
   (outer main-thread catch_unwind stringifies without location; near-no-op
   in practice; needs `panic_message_with_location` made `pub` if ever done).
   No codegen change → no snapshot churn from this item.

## Fixture + tests

7. New fixture in `rpkg/src/rust/panic_location_tests.rs` (mirror the region
   style there; worker dispatch = plain args, NO `SEXP` — see the file's
   main-thread region comment for the dispatch rule):
   ```rust
   /// Panic inside `with_r_thread` from a worker-dispatched fn. The R error
   /// must carry the location of THIS file's `panic!`, not worker.rs (#1245).
   /// @export
   #[miniextendr]
   pub fn panic_location_worker_with_r_thread() -> i32 {
       miniextendr_api::with_r_thread(|| -> i32 { panic!("boom-with-r-thread") })
   }
   ```
   (Match the exact `with_r_thread` import/path used elsewhere in rpkg —
   grep `with_r_thread` in `rpkg/src/rust/gc_stress_fixtures.rs:2560` region
   for the working idiom. If the closure needs a return-type annotation or
   different plumbing, copy that fixture's shape.)
8. testthat in `rpkg/tests/testthat/test-panic-location.R` (mirror existing
   assertions): the condition message from
   `panic_location_worker_with_r_thread()`
   - matches `\(at panic_location_tests\.rs:[0-9]+\)` with the line number of
     the fixture's `panic!` line;
   - does NOT contain `worker.rs`;
   - still starts with the `panic in \`with_r_thread\`:` prefix + the payload
     text `boom-with-r-thread`.
9. Path hygiene while there: assert the folded path in the new test is the
   bare crate-relative filename (`panic_location_tests.rs:NN`), consistent
   with the #1211 pins in that file.

No new SEXP storage across allocations → no gc-stress fixture required
(#430 rule not triggered). The existing stress fixture
`gc_stress_with_r_thread_stop` covers the R-longjmp flavor of this path and
must remain green.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2: panic_location_worker_with_r_thread is a NEW export
just test 2>&1 > /tmp/1245-rust-test.log         # Read the log
just devtools-test 2>&1 > /tmp/1245-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1245-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Commit regenerated `NAMESPACE` + `man/*.Rd` (new export) with the Rust change.
No macro-output change → no insta/trybuild churn expected; if a snapshot
changes, stop and report (never `TRYBUILD=overwrite` locally — #1239).

## Must NOT touch

- Gap 2 beyond the one comment (item 6). No `pub` widening of
  `panic_message_with_location`.
- The RCondition verbatim contract — `error!`/`condition!` payloads must stay
  byte-for-byte unchanged through every path (the branch in item 2 preserves
  this; the tests in `test-panic.R`/condition suites must stay green).
- `error_value.rs` (PROTECT-sensitive) — not in scope.
- Generated files (`wrappers.R`, `wasm_registry.rs`).

## Done criteria

- New fixture's R error ends with the fixture-file location, no `worker.rs`
  anywhere in the message; all existing panic-location pins green.
- Worker slot drained on the PreLocatedPanic arm (no stale-location leak —
  covered by running the full panic-location suite in one session).
- Suites + three clippy legs green; NAMESPACE/man committed in sync.
- PR references #1245 (Gap 1 fixed, Gap 2 stays open — do NOT `Fixes #1245`).

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, `with_r_thread`
cannot be called as sketched from the fixture, the location renders with an
unexpected path shape, any existing panic/condition test breaks — **stop,
commit nothing further, and report back. Do not improvise.**
