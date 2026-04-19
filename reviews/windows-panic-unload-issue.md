# Windows: panic runtime corrupted across DLL unload → "failed to initiate panic, error 5"

## Symptom

On Windows-GNU (Rtools45 `x86_64-pc-windows-gnu`), `devtools::test("rpkg")`
aborts the R process partway through the suite with:

```
[Rust] Worker: about to panic
[Rust] Dropped: worker: boxed resource before panic
[Rust] Dropped: worker: resource before panic
fatal runtime error: failed to initiate panic, error 5, aborting
```

`error 5` is Win32 `ERROR_ACCESS_DENIED` surfacing from `RaiseException` —
Rust's panic runtime can't start unwinding, so `catch_unwind` /
`with_r_unwind_protect` never get to run and the process is aborted.

Destructors run before the abort ("Dropped: …" prints), so a panic *is*
being raised successfully; the failure is on the next `RaiseException`
attempt or a follow-up SEH operation.

## Why this is distinct from #190

PR #190 fixes a related but separate Windows issue: rustc emits
`-lgcc_eh -lgcc_s`, but Rtools45's `.static.posix` toolchain ships a
*unified* `libgcc.a` with no separate `libgcc_eh.a` / `libgcc_s.a`. The
PR's CI used empty `ar crs` archives for those names; with `ld.lld`
those empty archives linked fine but left `__gcc_personality_v0` /
`__gcc_personality_seh0` unresolved at runtime, so the *first* panic
aborted immediately. PR #190 (commit `0bfa85a4`) stages the real unified
`libgcc.a` under both names so the personality routines actually resolve.

With PR #190's libgcc fix in place, the first N panics work. This issue
is about what breaks later.

## Reproducer

Confirmed locally on Windows 11 + Rtools45 UCRT64 + rustc 1.94.1 +
R 4.5.3, with PR #190's libgcc fix applied:

- Fresh `library(miniextendr)` + repeat `.Call(C_worker_drop_on_panic)`
  4× in a row — all succeed, panic caught, converted to R error, clean.
- `Rscript -e 'for (f in files) testthat::test_file(f)'` across 40 test
  files including `test-errors-more.R` — all pass, no abort.
- `Rscript -e 'devtools::test("rpkg")'` — aborts with the error above
  when `unsafe_C_worker_drop_on_panic` runs (test-errors-more.R:46).

Differentiator: `devtools::test` uses `devtools::load_all(reset = TRUE)`
which triggers `unloadNamespace` → `library.dynam.unload` →
`R_unload_<pkg>` before the reload. A plain `library(miniextendr)`
never unloads.

## Root cause

Three pieces of the miniextendr runtime hold references into the DLL's
code that outlive `R_unload_<pkg>`:

1. **Worker thread.** `R_unload_<pkg>` calls
   `miniextendr_runtime_shutdown()` which calls
   `worker_channel::shutdown()` (`miniextendr-api/src/worker.rs:332`) —
   an atomic flag the worker polls every 250 ms in its `recv_timeout`
   loop. Shutdown is **signal-only, not join.** On Windows, R's
   `library.dynam.unload` unmaps the DLL's code pages synchronously
   once the unload returns. If the worker thread is still alive
   (mid-poll or about to wake), it resumes executing in freed memory,
   hits an access violation, and corrupts the process's SEH chain. The
   next `RaiseException` anywhere in the process fails.

2. **libc `atexit` handler.** `miniextendr_runtime_init`
   (`worker.rs:195-204`) registers an `atexit_shutdown()` function
   pointer whose address lies in the DLL. If the DLL is unloaded before
   libc's atexit registry fires (normal case for `dyn.unload`), process
   exit jumps to an unmapped address.

3. **Rust panic hook.** `miniextendr_panic_hook()` (`backtrace.rs:7-28`)
   captures a closure via `std::panic::set_hook`. The closure code
   lives in the DLL. After unload the global panic hook still points
   to it; any subsequent panic hook invocation jumps to unmapped
   memory. The project already knows this one — `init.rs:53-55` has an
   explicit *"on Windows, set_hook during DLL init can fail with
   'failed to initiate panic, error 5' because the panic infrastructure
   isn't fully available during DLL loading"* comment and skips
   `set_hook` in the wrapper-generation path.

## Suggested fix

All three hazards share the underlying shape *"DLL code outlives DLL
unload"* — fix them together. No timeouts, no polling, no sleeps.

### (1) Worker: tagged-message channel + blocking join

Replace the atomic-flag + `recv_timeout(250ms)` poll protocol with a
tagged message and a blocking `recv()`:

```rust
enum WorkerMsg {
    Job(AnyJob),
    Shutdown,
}
// JOB_TX: SyncSender<WorkerMsg> wrapped in a Mutex<Option<_>> so
// shutdown can take() it and drop after send, closing the channel.

fn worker_loop(rx: Receiver<WorkerMsg>) {
    while let Ok(msg) = rx.recv() {
        match msg {
            WorkerMsg::Job(job) => job(),
            WorkerMsg::Shutdown => break,
        }
    }
}

pub fn shutdown() {
    // idempotent; only runs the send/join on the first call
    let Some(tx) = JOB_TX.lock().unwrap().take() else { return };
    // Best effort — if the worker already exited the send errors, fine
    let _ = tx.send(WorkerMsg::Shutdown);
    drop(tx); // closing the channel is the other wake-up path
    if let Some(h) = WORKER_JOIN_HANDLE.lock().unwrap().take() {
        h.join().expect("worker thread panicked");
    }
}
```

Why it's rigid:

- No polling interval. No `sleep`. No `is_finished` race.
- `recv()` blocks until either a `Job`, a `Shutdown`, or
  sender-dropped — all three unblock the worker immediately.
- `h.join()` blocks exactly as long as the current in-flight job takes
  to drain plus one `match` — no jitter.
- `R_unload_<pkg>` does not return until the thread is genuinely gone,
  so `library.dynam.unload`'s subsequent `FreeLibrary` sees no live
  code references.
- Drop the `WORKER_SHOULD_STOP` atomic, the 250 ms
  `SHUTDOWN_POLL_INTERVAL`, and the `miniextendr_runtime_join_for_test`
  helper — the new design doesn't need them. The #204 test can assert
  via `miniextendr_runtime_shutdown()` directly; if it ever doesn't
  return, that's the bug to find.

Edge case the current design handles via the atomic and this one
handles via sender-ownership: callers of `dispatch_to_worker` after
shutdown. Now `JOB_TX.lock()` returns `None` and they return a
"worker shut down" error instead of `send` erroring — same outcome,
clearer error source.

### (2) atexit handler

Either:

- Drop the atexit registration entirely on Windows — `R_unload_<pkg>`
  is enough for every normal exit path, and `q("no")` etc. tear down
  the process without needing worker cleanup (the OS reaps the
  thread). The existing comment already notes *"atexit can be flaky on
  Windows"* as a caveat.
- Or register the handler in a host C file that's linked into libR,
  not the rpkg DLL — so the handler code outlives the DLL.

First option is smaller and aligns with the "rigid, no special cases"
spirit of the worker fix.

### (3) Panic hook

In `R_unload_<pkg>`, restore the previous panic hook so the global
hook no longer references DLL code:

```rust
// In miniextendr_panic_hook, save the current hook before replacing
// and expose an "uninstall" that reinstates it.
pub extern "C-unwind" fn miniextendr_panic_hook_uninstall() {
    let _ = std::panic::take_hook();
    // The default hook reinstated by take_hook is the process default,
    // not the closure we stored in DLL code — that's what we want.
}
```

Call it from `miniextendr_runtime_shutdown` after the worker has
joined (order matters: worker might itself panic as it exits and we
don't want the unmapped hook to fire).

## Non-fix: test-level skips

`test-errors-more.R:17` and `:25` already
`skip("thread panic propagation causes runtime errors in extern C-unwind")`
for two sibling tests. Adding one more skip for `worker_drop_on_panic`
on Windows would mask this specific failure but the underlying hazard
remains — any package user whose code panics after
`devtools::load_all(reset = TRUE)` hits it, and `atexit` / `set_hook`
remain time bombs at process exit. Skip is not enough; fix the
lifecycle.

## Scope

Separate from PR #190, which only addresses the libgcc personality-
symbol link-time issue. The libgcc fix is necessary on its own merit
(empty stubs → `_Unwind_RaiseException` can't find a personality even
on the first panic) and is unaffected by this issue's resolution.
Referencing #190 so the two can be read together.
