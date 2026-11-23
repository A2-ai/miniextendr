---
name: miniextendr-connections
description: Use when the user asks about implementing custom R connections in Rust (like file(), url(), gzcon() equivalents), the RConnectionImpl trait, RCustomConnection builder, catch_connection_panic trampolines, the CLOSED-state gotcha with R_new_custom_connection, ABI version checking for the connections API, or reading and writing through R connections from Rust.
---

# miniextendr Connections

R's connection system provides an I/O abstraction used by `file()`, `url()`, `gzcon()`, and other stream-like objects. miniextendr wraps the C connection API with panic-safe trampolines and a builder that lets you implement custom connections entirely in Rust.

## When to use this skill

- "How do I create a custom R connection in Rust?"
- "What is `RConnectionImpl`?"
- "Why is my connection returned already-open, and where does the open callback get invoked?"
- "How do panics inside connection callbacks get handled?"
- "What is `catch_connection_panic`?"
- "What is the connections ABI version check?"

## Key concepts

### The connections API is explicitly unstable

R's `R_ext/Connections.h` states: "we do not expect future connection APIs to be backward-compatible … We explicitly reserve the right to change the connection implementation without a compatibility layer."

The `connections` feature in miniextendr is gated for this reason. Before using it in production, verify that `R_CONNECTIONS_VERSION` matches `EXPECTED_CONNECTIONS_VERSION` (both are constants from `connection.rs`). The `check_connections_version()` function asserts this at runtime; call it during `R_init_<pkg>` when the feature is enabled.

The API requires R >= 4.3.0. Use `check_connections_runtime()` for a runtime probe. Building a custom connection (`RCustomConnection::build`, the `RConnectionIo` `build_*` methods) also needs a build against R >= 4.6, or the `nonapi` feature on older R: `R_new_custom_connection` is non-API before R 4.6, so without `nonapi` it is not declared there and `build` panics.

### Connections from `RCustomConnection::build` are returned open

`R_new_custom_connection` (R's underlying C entry point) returns a connection in the CLOSED state (`isopen = FALSE`) with `text = TRUE` regardless of mode. `RCustomConnection::build` papers over both:

- The mode string is parsed for `'b'` to infer `text`; explicit `.text(...)` still wins.
- The `open` callback is invoked before the SEXP is returned. R's own auto-open path (used by `readLines` / `scan` / etc.) sees `isopen == TRUE` and short-circuits, so there is no double-open.

```rust
let conn_sexp = RCustomConnection::new()
    .description("my conn")
    .mode("r+b")
    .can_read(true)
    .can_write(true)
    .build(MyConnectionImpl { /* ... */ });
// conn_sexp is already OPEN. readLines / writeBin / seek all work.
```

If your `RConnectionImpl::open` returns `false`, the boxed state is dropped via the destroy trampoline and `build` returns `SEXP::nil()`. Make sure `open` returns `true` on success.

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

### Panic-safe trampolines (`connection_callback`, `catch_connection_panic`)

No callback lets a panic unwind into R's C frames. The callbacks fall into two groups, both guarded through `miniextendr-api/src/ffi_guard.rs`:

- `read`, `write`, `fgetc`, `seek`, `truncate` and `flush` run in `connection_callback`, which is `guarded_ffi_call` with `GuardMode::RUnwind`. A panic or an `error!` condition becomes an R error after Rust destructors run. It does not become EOF or a short write.
- `open`, `close` and `destroy` run in `catch_connection_panic`, which is `guarded_ffi_call_with_fallback`. A panic there returns a fallback instead of raising: `FALSE` from `open` (the builder then tears the connection down), and a status of `-1` from `close`, which R reports with its own warning, "Problem closing connection". `close` and `destroy` can run during garbage collection, so they raise nothing themselves.

`PanicSource::Connection` telemetry fires in both groups.

## How it works

The builder generates one `extern "C-unwind"` trampoline per connection callback method. Each trampoline:

1. Retrieves `&mut T` from the Rconn's private data pointer.
2. Calls the corresponding `RConnectionImpl` method inside its guard: `connection_callback(|| …)` for the I/O callbacks, `catch_connection_panic(fallback, || …)` for `open`, `close` and `destroy`.
3. Returns the method's return value. If the method panicked, an I/O callback raises an R error and the other three return their fallback.

R calls these trampolines through its internal connection dispatch table. The `simple_trampoline!` macro in `miniextendr-api/src/connection.rs` generates the `fgetc`, `seek`, `truncate` and `flush` ones.

The connection's Rust state is heap-allocated and stored in `Rconn`'s private data pointer. The `destroy` trampoline frees it: it calls `destroy()` and then drops the boxed impl, even if `destroy()` panics. `close` only calls `close()`.

## Decision trees

### Read-only vs read-write connection?

- Read-only: set `.can_read(true)`, implement `read()`. Leave `write()` as default (returns 0).
- Write-only: set `.can_write(true)`, implement `write()`. Leave `read()` as default.
- Read-write: set both, implement both.
- Seekable: set `.can_seek(true)`, implement `seek()`.

### Where do panics in connection callbacks go?

- In `read`, `write`, `fgetc`, `seek`, `truncate` or `flush`: an R error at the R call that used the connection (`readLines(con)`, `writeBin(x, con)`, ...), raised after Rust destructors run. To fail an operation, panic or use `error!`; the `std::io` adapters raise a failed `io::Result` as the classed condition `miniextendr_connection_io_error`.
- In `open`: the callback returns `FALSE` and the builder tears the connection down.
- In `close`: `close(con)` warns "Problem closing connection" and returns `-1L`.
- In `destroy`: caught, and the Rust state is still dropped.

Panic telemetry (`PanicSource::Connection`) fires in every case.

## Key files

- `miniextendr-api/src/connection.rs` — `RConnectionImpl` trait, `RCustomConnection` builder, `connection_callback`, `catch_connection_panic`, `check_connections_version`, `check_connections_runtime`, trampolines.
- `miniextendr-api/src/ffi_guard.rs` — `guarded_ffi_call` (`GuardMode::RUnwind`) used by `connection_callback`, and `guarded_ffi_call_with_fallback` used by `catch_connection_panic`.

## Common pitfalls

- **Underlying API starts CLOSED with `text = TRUE`**: `R_new_custom_connection` returns a closed connection with `text = TRUE` regardless of mode. `RCustomConnection::build` opens it for you and infers `text` from the mode string before returning, so this is invisible to callers — but if you ever bypass the builder and call `R_new_custom_connection` directly, you need to handle both yourself.

- **A panic is an R error in the I/O callbacks and a fallback in the other three**: a panic in `read`, `write`, `fgetc`, `seek`, `truncate` or `flush` becomes an R error, not EOF or a short write. Only `open` (returns `FALSE`), `close` (status `-1`, R warns) and `destroy` (caught) do not raise. `warning!()` and `message!()` inside a callback become errors too; queue them with `defer_warning!()` / `defer_message!()`.

- **ABI version mismatch**: if the connections API changes in a future R version, `check_connections_version()` will panic at package load time. This is intentional — operating with a mismatched ABI is unsafe. Do not suppress this check.

- **`connections` feature must be enabled**: the entire connections module is gated behind `features = ["connections"]` in `Cargo.toml`. Functions and types will not be available without it.

- **Thread safety**: connection callbacks are called by R on the main thread. The `RConnectionImpl` impl is stored behind a raw pointer from the main thread; accessing it from any other thread is unsound.

## Related skills

- `miniextendr-ffi` — `guarded_ffi_call_with_fallback`, panic telemetry, `_unchecked` FFI variants used inside connection trampolines.
