---
name: miniextendr-connections
description: Use when the user asks about implementing custom R connections in Rust (like file(), url(), gzcon() equivalents), the RConnectionImpl trait, RCustomConnection builder, catch_connection_panic trampolines, the CLOSED-state gotcha with R_new_custom_connection, ABI version checking for the connections API, or reading and writing through R connections from Rust.
---

# miniextendr Connections

R's connection system provides an I/O abstraction used by `file()`, `url()`, `gzcon()`, and other stream-like objects. miniextendr wraps the C connection API with panic-safe trampolines and a builder that lets you implement custom connections entirely in Rust.

## When to use this skill

- "How do I create a custom R connection in Rust?"
- "What is `RConnectionImpl`?"
- "Why does my connection start closed and have to be explicitly opened?"
- "How do panics inside connection callbacks get handled?"
- "What is `catch_connection_panic`?"
- "What is the connections ABI version check?"

## Key concepts

### The connections API is explicitly unstable

R's `R_ext/Connections.h` states: "we do not expect future connection APIs to be backward-compatible … We explicitly reserve the right to change the connection implementation without a compatibility layer."

The `connections` feature in miniextendr is gated for this reason. Before using it in production, verify that `R_CONNECTIONS_VERSION` matches `EXPECTED_CONNECTIONS_VERSION` (both are constants from `connection.rs`). The `check_connections_version()` function asserts this at runtime; call it during `R_init_<pkg>` when the feature is enabled.

The API requires R >= 4.3.0. Use `check_runtime_connections_support()` for a runtime probe.

### Connections start in CLOSED state

`R_new_custom_connection` creates the connection in a closed state (`isopen = FALSE`). After building the connection SEXP, you must explicitly invoke the `open` callback to put it into an open state before returning it to R callers who expect an open connection.

The builder's `.build(impl)` method does not automatically call `open`. If you want an auto-opened connection:

```rust
let conn_sexp = RCustomConnection::new()
    .description("my conn")
    .can_read(true)
    .build(MyConnectionImpl { /* ... */ });
// conn_sexp is CLOSED here. Open it:
unsafe { open_connection(conn_sexp)? };
```

`open_connection` is a helper that calls the connection's registered `open` trampoline through R's internal dispatch. See `miniextendr-api/src/connection.rs` for the implementation.

### `RConnectionImpl` trait

Implement this trait for your connection type:

```rust
trait RConnectionImpl {
    fn open(&mut self) -> bool;
    fn close(&mut self);
    fn read(&mut self, buf: &mut [u8]) -> usize;      // optional: 0 = no bytes available
    fn write(&mut self, buf: &[u8]) -> usize;          // optional: 0 = write failed
    fn seek(&mut self, offset: i64, origin: i32) -> i64; // optional: -1 = not supported
    fn flush(&mut self);                                // optional: no-op default
}
```

Methods you do not override default to safe stubs (returning 0 or false). The trait reflects R's internal connection method table.

### `RCustomConnection` builder

```rust
let conn = RCustomConnection::new()
    .description("description string")
    .mode("r")              // "r", "w", "a", "rb", "wb", etc.
    .class_name("myconn")   // R class name for the connection object
    .can_read(true)
    .can_write(false)
    .can_seek(false)
    .build(my_impl);
```

`.build(impl)` boxes the impl, registers the trampolines, and returns a SEXP. The resulting SEXP is an R connection object of class `c("myconn", "connection")`.

### `catch_connection_panic` trampolines

Every connection callback is wrapped in `catch_connection_panic`. This uses `guarded_ffi_call_with_fallback` from `miniextendr-api/src/ffi_guard.rs`:

- On panic: fires `PanicSource::Connection` telemetry and returns the fallback value (e.g., `0` for read/write, `Rboolean::FALSE` for open, no-op for close/flush).
- Does **not** raise an R error from inside the trampoline — connection callbacks cannot safely longjmp. The error is absorbed and reported via telemetry only.

This means panics inside connection callbacks are silent from R's perspective (the operation appears to have failed with 0 bytes / `FALSE`). Log or record errors in your impl struct if you need to surface them.

## How it works

The builder generates one `extern "C-unwind"` trampoline per connection callback method. Each trampoline:

1. Retrieves `&mut T` from the Rconn's private data pointer.
2. Calls the corresponding `RConnectionImpl` method inside `catch_connection_panic(fallback, || …)`.
3. Returns either the method's return value or the fallback if the method panicked.

R calls these trampolines through its internal connection dispatch table. The implementation pattern mirrors `simple_trampoline!` macros in `miniextendr-api/src/connection.rs` (lines 659–680).

The connection's Rust state is heap-allocated and stored in `Rconn`'s private data pointer. The `close` callback is responsible for freeing it (the builder generates a `close` trampoline that drops the boxed impl).

## Decision trees

### Read-only vs read-write connection?

- Read-only: set `.can_read(true)`, implement `read()`. Leave `write()` as default (returns 0).
- Write-only: set `.can_write(true)`, implement `write()`. Leave `read()` as default.
- Read-write: set both, implement both.
- Seekable: set `.can_seek(true)`, implement `seek()`.

### Where do panics in connection callbacks go?

Panics inside `RConnectionImpl` methods are caught by `catch_connection_panic`. They do NOT propagate to R as errors. The trampoline returns the fallback value (the operation silently fails). Panic telemetry fires, which logs via `PanicSource::Connection`. To surface errors to the R caller, store the error in your impl struct and check it after the connection operation from R.

## Key files

- `miniextendr-api/src/connection.rs` — `RConnectionImpl` trait, `RCustomConnection` builder, `catch_connection_panic`, `check_connections_version`, `check_runtime_connections_support`, trampolines.
- `miniextendr-api/src/ffi_guard.rs` — `guarded_ffi_call_with_fallback` used by `catch_connection_panic`.

## Common pitfalls

- **Connection starts CLOSED**: `R_new_custom_connection` creates in closed state. Always call the open callback explicitly after `build()` if you want an already-open connection. Returning a closed connection to R code that calls `readLines()` on it immediately will produce an "connection is not open" error.

- **Panics are silently absorbed**: unlike `#[miniextendr]` functions (which use the tagged-SEXP error transport to surface Rust panics as R errors), connection trampolines use the fallback-value pattern. A panic in `read()` returns `0` bytes — the R caller sees a short read, not an error. Design your `RConnectionImpl` to return error signals through return values, not panics.

- **ABI version mismatch**: if the connections API changes in a future R version, `check_connections_version()` will panic at package load time. This is intentional — operating with a mismatched ABI is unsafe. Do not suppress this check.

- **`connections` feature must be enabled**: the entire connections module is gated behind `features = ["connections"]` in `Cargo.toml`. Functions and types will not be available without it.

- **Thread safety**: connection callbacks are called by R on the main thread. Do not share connection state with a worker thread without a mutex. The `RConnectionImpl` impl is stored behind a raw pointer from the main thread; accessing it from the worker is unsound.

## Related skills

- `miniextendr-ffi` — `guarded_ffi_call_with_fallback`, panic telemetry, `_unchecked` FFI variants used inside connection trampolines.
- `miniextendr-worker` — threading model; why connection callbacks must stay on the main thread.
