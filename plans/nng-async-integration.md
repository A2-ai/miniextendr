# NNG + Async Integration for miniextendr

## Status: Phase 1 Complete

NNG 1.11.0 and mbedtls 3.6.5 are compiled by R's Makevars, linked into the R package, and callable from Rust. Safe Rust wrappers exist for core NNG types. `#[miniextendr(background)]` ships a working proc-macro that spawns background threads and returns async handles to R.

## What's Done

### C library build system (Makevars)
- NNG + mbedtls source trees bundled in `rpkg/src/nng/` and `rpkg/src/mbedtls/`
- Compiled via pattern rules in `Makevars.in` with per-library CPPFLAGS
- Built as static archives (`libnng.a`, `libmbedtls_all.a`)
- Passed to cargo via `-C link-arg=` in RUSTFLAGS (both staticlib and cdylib)
- Platform detection in `configure.ac`: POSIX/Darwin/Linux, kqueue/epoll, arc4random/getrandom
- All NNG protocols (bus, pair, pub/sub, push/pull, req/rep, surveyor) and transports (inproc, IPC, TCP, TLS, WS, FDC) registered via defines

### NNG safe Rust wrappers (`miniextendr-api/src/nng/`)
- `NngSocket` — RAII wrapper, factory methods for all protocols, `dial()`, `listen()`, `send()`, `recv()`, `send_msg()`, `recv_msg()`, timeout setters
- `NngMsg` — owned message, `from_bytes()`, `body()`, `append()`, RAII `Drop`, `into_raw()` for send (avoids double-free)
- `NngError`/`NngResult` — error type wrapping `nng_strerror()`
- FFI declarations for ~40 NNG C functions
- Feature-gated: `nng = []` in Cargo.toml

### `#[miniextendr(background)]` proc-macro
- `MxAsyncHandle` type: `Arc<AtomicBool>` resolved flag, `mpsc::SyncChannel` one-shot receiver
- `AsyncSender`: background thread calls `.send(result)` to deliver
- Proc-macro generates 3 C wrappers per function:
  - `C_fn` — converts args on main thread, spawns `std::thread`, returns `ExternalPtr<MxAsyncHandle>`
  - `C_fn__is_resolved` — non-blocking status check
  - `C_fn__value` — blocks on channel recv, downcasts, `IntoR` conversion
- R wrapper: env-class with `$is_resolved()` and `$value()` methods, class `"mx_async_handle"`
- `Result<T, E>` return types: Ok unwrapped in background thread, Err → R error
- Panics caught via `catch_unwind`, propagated as R errors
- No external dependencies (uses `std::thread` + `std::sync::mpsc`)

### Verified working
- PAIR echo, REQ/REP request-reply over `inproc://`
- `bg_add`, `bg_greeting`, `bg_pi_approx` (1M iterations), `bg_result_ok`, `bg_panicking`

## Known Concessions / Limitations

### Build system
- **Worktree cargo resolution**: Git worktrees cause cargo to find the parent miniextendr workspace for proc-macro crates. Fixed by: (a) configure.ac writes `.cargo/config.toml` with vendor source replacement in dev mode, and (b) `cargo-revendor` resolves workspace inheritance. Manual vendor edits (`{ workspace = true }`) break things — always use `just configure` with `MINIEXTENDR_LOCAL`.
- **PUSH/PULL hangs**: Synchronous `push.send_msg()` + `pull.recv_msg()` on inproc can hang because the connection may not be established before the first send. Needs `NngAio` (async IO) or explicit connection-ready signaling.
- **NNG option names**: Must use NNG's string names (`"recv-timeout"`) not the C macro names (`"NNG_OPT_RECVTIMEO"`).
- **No Windows NNG build**: The Makevars only handles POSIX. Windows would need `src/nng/src/platform/windows/*.c` compiled separately.

### `#[miniextendr(background)]`
- **Thread-per-task**: Each `background` call spawns a new OS thread. Fine for heavy workloads, wasteful for many small tasks. Plan 3 (NNG worker pool) is the upgrade path.
- **No cancellation**: Background threads run to completion. `drop(handle)` in R doesn't abort — the thread finishes and the result is discarded.
- **No Shiny/promises integration** (v1): `$value()` blocks R's main thread. Non-blocking users must poll `$is_resolved()`. Plan: add `as.promise.mx_async_handle` using `later` package.
- **`async` is a Rust keyword**: Can't use `#[miniextendr(async)]` directly. Named `background` instead.
- **`&mut self` methods**: Not supported for `background`. Would require `Clone` to safely move data to the background thread.
- **SEXP return types**: Not supported — SEXP isn't `Send`. Functions must return Rust types that implement `IntoR`.
- **Error message formatting**: The `rust_error_value` class concatenates all fields in the error message. Could be improved with structured condition objects.
- **No `print.mx_async_handle`**: The handle prints as a generic environment. Needs a print method.

## Remaining Work

### Plan 1 Phase B: NNG advanced types
- `NngCtx` — REQ/REP context multiplexing (multiple concurrent requests on one socket)
- `NngAio` — async IO with callbacks (for non-blocking operations)
- `NngDialer`/`NngListener` — explicit control over connection lifecycle
- TCP/IPC transport tests (currently only inproc tested)
- TLS configuration wrappers
- Fix PUSH/PULL with async recv

### Plan 2: mirai interop
- `r_serialize_to_bytes(SEXP)` / `r_unserialize_from_bytes(&[u8])` — R serialization from Rust
- `MiraiHeader` — 8-byte wire protocol header encode/decode
- `MiraiClient` — connect to mirai dispatcher, submit R expressions programmatically
- `RustDaemon` — speak mirai protocol, run registered Rust functions instead of `eval()`
- Custom serialization hooks for Rust types (bypass R serialization)

### Plan 3: NNG worker pool
- `DispatchableFn` + `MX_ASYNC_TASKS` linkme registry
- Dispatcher thread with NNG REQ/REP + PUSH/PULL fan-out
- Worker pool (N threads) connected via `inproc://`
- `#[miniextendr(async_dispatch)]` proc-macro generating trampoline + registry entry
- serde/bincode serialization for Rust↔Rust (no R serialization overhead)

### Plan 4 v2: Polish
- `as.promise.mx_async_handle` for Shiny/promises integration
- `print.mx_async_handle` method
- Thread pool backend (rayon or custom) instead of thread-per-task
- Timeout parameter for `$value(timeout_ms = NULL)`
- UI tests for compile-time error cases
- Formal R test suite in `rpkg/tests/testthat/test-async.R`
