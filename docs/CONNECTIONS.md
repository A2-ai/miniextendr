# Custom R Connections

R connections are the standard abstraction for I/O in R -- `readLines()`, `writeLines()`, `readBin()`, `writeBin()`, `scan()`, and many other functions all operate on connections. miniextendr lets you create custom R connections backed by Rust types, enabling you to expose any Rust I/O source or sink to R's connection infrastructure.

**Feature flag:** `connections` (opt-in, experimental)

```toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["connections"] }
```

> **Warning:** R explicitly reserves the right to change the connection C API without backward compatibility. This module performs a compile-time ABI version check, but future R releases may break it. See [Safety & Version Checking](#safety-version-checking).

## Table of Contents

- [Quick Start](#quick-start)
- [The RConnectionImpl Trait](#the-rconnectionimpl-trait)
- [Builder Pattern](#builder-pattern)
- [Connection Lifecycle](#connection-lifecycle)
- [std::io Adapters](#std-io-adapters)
- [Safety & Version Checking](#safety-version-checking)
- [Capability Probing](#capability-probing)
- [Error Handling](#error-handling)
- [Trampoline Architecture](#trampoline-architecture)
- [Complete Examples](#complete-examples)
- [Helper Functions](#helper-functions)
- [Standard Streams](#standard-streams)
- [Limitations](#limitations)

## Quick Start

Here is a minimal read-only connection that serves an in-memory string:

```rust
use miniextendr_api::connection::{RConnectionImpl, RCustomConnection};
use miniextendr_api::sys::SEXP;
use miniextendr_api::miniextendr;

struct StringSource {
    data: Vec<u8>,
    pos: usize,
}

impl RConnectionImpl for StringSource {
    fn open(&mut self) -> bool {
        self.pos = 0;
        true
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        let remaining = self.data.len().saturating_sub(self.pos);
        let n = buf.len().min(remaining);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        n
    }
}

#[miniextendr]
pub fn my_string_connection(text: &str) -> SEXP {
    RCustomConnection::new()
        .description("string source")
        .mode("r")
        .class_name("stringSource")
        .can_read(true)
        .text(true)
        .build(StringSource {
            data: text.as_bytes().to_vec(),
            pos: 0,
        })
}
```

From R:

```r
conn <- my_string_connection("hello\nworld")
readLines(conn)
#> [1] "hello" "world"
close(conn)
```

## The RConnectionImpl Trait

Implement `RConnectionImpl` on your type to define connection behavior. All methods have sensible defaults -- you only override what you need.

```rust
pub trait RConnectionImpl: Sized + 'static {
    fn open(&mut self) -> bool { true }
    fn close(&mut self) {}
    fn destroy(&mut self) {}
    fn read(&mut self, buf: &mut [u8]) -> usize { 0 }
    fn write(&mut self, buf: &[u8]) -> usize { 0 }
    fn fgetc(&mut self) -> i32 { /* reads one byte via read() */ }
    fn seek(&mut self, where_: f64, origin: i32, rw: i32) -> f64 { /* raises */ }
    fn truncate(&mut self) { /* raises */ }
    fn flush(&mut self) -> i32 { 0 }
    fn vfprintf(&mut self, fmt: *const c_char, ap: *mut c_void) -> i32 { -1 }
}
```

### Method Reference

| Method | When Called | Return Value | Notes |
|--------|-----------|-------------|-------|
| `open` | `open(conn)` in R or auto-open on first use | `true` = success, `false` = failure | |
| `close` | `close(conn)` in R | (none) | Called before `destroy` |
| `destroy` | Connection is garbage collected | (none) | Your type is dropped automatically after this |
| `read` | `readBin()`, `readLines()`, etc. | Number of bytes read (0 = EOF) | R takes a short count as the end of the stream |
| `write` | `writeBin()`, `writeLines()`, etc. | Number of bytes written | Write all of `buf`: R warns about a short count, or ignores it |
| `fgetc` | Character-by-character reading | Byte as `i32`, or -1 on EOF | Default delegates to `read` |
| `seek` | `seek(conn, where, origin)` | Previous position | See origin codes below; the default raises "'seek' not enabled for this connection" |
| `truncate` | `truncate(conn)` | (none) | The default raises "truncation not enabled for this connection" |
| `flush` | `flush(conn)` | 0 | R ignores the value |
| `vfprintf` | Formatted print (rarely needed) | Characters written, or -1 | R typically uses `write` |

### Seek Origin Codes

| Value | Meaning | R Equivalent |
|-------|---------|-------------|
| 1 | From start | `"start"` |
| 2 | From current position | `"current"` |
| 3 | From end | `"end"` |

When `where_` is `NaN`, the caller is querying the current position -- return it without moving.

R hands every returned value to the R caller as it is, so report a failed
operation by raising an error (`error!` or a panic, see [Error Handling](#error-handling)),
not by returning -1 or a short count.

### Memory Management

Your type is `Box`-ed and stored in the connection's `private` field. When the connection is garbage collected, `destroy()` is called first, then your type is automatically dropped. You do not need to free memory manually.

## Builder Pattern

`RCustomConnection` configures and creates the R connection object:

```rust
let conn_sexp = RCustomConnection::new()
    .description("my data source")   // Shown in summary(conn)
    .mode("rb")                      // Open mode (see table below)
    .class_name("myConnection")      // R class (along with "connection")
    .text(false)                     // Binary mode
    .can_read(true)                  // Supports reading
    .can_write(false)                // Does not support writing
    .can_seek(true)                  // Supports seeking
    .blocking(true)                  // Blocking I/O (default)
    .build(my_state);                // Consumes the state
```

### Builder Methods

| Method | Default | Description |
|--------|---------|-------------|
| `description(s)` | `"custom connection"` | Description shown in `summary()` |
| `mode(s)` | `"r"` | Open mode string |
| `class_name(s)` | `"customConnection"` | First element of the R class vector |
| `text(bool)` | Inferred from mode | Text vs binary mode |
| `can_read(bool)` | Inferred from mode | Whether reading is supported |
| `can_write(bool)` | Inferred from mode | Whether writing is supported |
| `can_seek(bool)` | `false` | Whether seeking is supported |
| `blocking(bool)` | `true` | Whether I/O is blocking |

### Mode Strings

| Mode | Meaning |
|------|---------|
| `"r"` | Read text |
| `"w"` | Write text |
| `"a"` | Append text |
| `"rb"` | Read binary |
| `"wb"` | Write binary |
| `"ab"` | Append binary |
| `"r+"` | Read/write text |
| `"r+b"` or `"rb+"` | Read/write binary |

The mode string must be at most 4 characters.

### Return Value

`build()` returns a `SEXP` -- an R connection object that can be returned directly from a `#[miniextendr]` function.

## Connection Lifecycle

```text
build()
  |
  v
[Connection created in CLOSED state]
  |
  v
open() callback  <-- triggered by open(conn) or auto-open
  |
  v
[OPEN state]
  |  read() / write() / seek() / flush() / fgetc()
  v
close() callback  <-- triggered by close(conn) or auto-close
  |
  v
[CLOSED state]
  |
  v
destroy() callback  <-- triggered by garbage collection
  |
  v
[Rust type dropped (Box<T> freed)]
```

Key points:

- `R_new_custom_connection` creates connections in the **CLOSED** state. R opens them automatically on first use.
- `close()` is called before `destroy()`. Your close handler should release resources; destroy is for final cleanup.
- `destroy()` is always called, even if the connection was never opened.
- Your Rust type is dropped after `destroy()` returns, even if `destroy()` panics.

## std::io Adapters

If your type already implements `std::io::Read`, `std::io::Write`, or `std::io::Seek`, you can skip implementing `RConnectionImpl` entirely. The `RConnectionIo` builder wraps any `std::io` type automatically:

```rust
use miniextendr_api::connection::RConnectionIo;
use std::io::Cursor;

#[miniextendr]
pub fn cursor_connection(data: Vec<u8>) -> SEXP {
    RConnectionIo::new(Cursor::new(data))
        .description("Rust Cursor")
        .mode("r+b")
        .build_read_write_seek()
}
```

### Available Build Methods

Choose the build method matching your type's trait bounds:

| Method | Requires | Capabilities |
|--------|----------|-------------|
| `build_read()` | `Read` | Read-only |
| `build_write()` | `Write` | Write-only |
| `build_read_write()` | `Read + Write` | Read and write |
| `build_read_seek()` | `Read + Seek` | Read with seeking |
| `build_write_seek()` | `Write + Seek` | Write with seeking |
| `build_read_write_seek()` | `Read + Write + Seek` | Full capabilities |
| `build_bufread()` | `BufRead` | Buffered read (optimized `fgetc`) |

Capabilities (can_read, can_write, can_seek) are auto-detected from the adapter type. You can override them:

```rust
RConnectionIo::new(my_reader)
    .can_write(false)  // Override auto-detection
    .build_read()
```

### Adapter Types

The adapters are also available directly if you need finer control:

| Adapter | Wraps | Capabilities |
|---------|-------|-------------|
| `IoRead<T>` | `Read` | Read |
| `IoWrite<T>` | `Write` | Write, flush |
| `IoReadWrite<T>` | `Read + Write` | Read, write, flush |
| `IoReadSeek<T>` | `Read + Seek` | Read, seek |
| `IoWriteSeek<T>` | `Write + Seek` | Write, seek, flush |
| `IoReadWriteSeek<T>` | `Read + Write + Seek` | Read, write, seek, flush |
| `IoBufRead<T>` | `BufRead` | Read, optimized fgetc |

```rust
use miniextendr_api::connection::{IoRead, RCustomConnection};

let adapter = IoRead::new(my_reader);
let conn = RCustomConnection::new()
    .description("adapted reader")
    .mode("rb")
    .can_read(true)
    .build(adapter);
```

### Boxed I/O trait objects

The existing adapters accept owned trait objects directly:

```rust
use miniextendr_api::connection::RConnectionIo;
use std::io::{Cursor, Read};

let reader: Box<dyn Read> = Box::new(Cursor::new(vec![1_u8, 2, 3]));
let connection = RConnectionIo::new(reader).mode("rb").build_read();
// R now owns the reader; close(connection) releases it.
```

`Box<dyn BufRead>` goes with `build_bufread()`, and `Box<dyn Write>` with
`build_write()`. The stream must own what it reads from or writes to (or borrow
only `'static` data), because the connection can outlive the call that made it.

## Safety & Version Checking

R's connection C API is explicitly unstable. From R's `R_ext/Connections.h`:

> "We explicitly reserve the right to change the connection implementation without a compatibility layer."

### R versions before 4.6

R 4.6 lists `R_GetConnection`, `R_ReadConnection`, `R_WriteConnection` and
`R_new_custom_connection` as experimental API. R 4.4 and 4.5 export the same
entry points, but `R CMD check` notes them there as non-API calls. So a
package built against R < 4.6 (`miniextendr-api`'s `r_4_6` cfg unset; see
`NONAPI.md`, "R-version cfgs") reaches an R connection through R functions
instead (#1905):

- **State** (`ConnectionCapabilities::from_sexp`, `connection_mode`,
  `connection_description`, `is_binary_mode`, the `RStdout` / `RStderr` /
  `RNullConnection` conversions): from `summary.connection()` and
  `isSeekable()`. R has no R-level getter for the blocking flag, so
  `is_blocking` is `None`.
- **Bytes** (`read_connection` / `write_connection`, `RNullConnection`, the
  indicatif connection target): `writeBin()`
  on a binary connection; `writeChar(useBytes = TRUE)` on a text-mode one,
  which takes the longest prefix of the bytes that is UTF-8 without a NUL.
  Reads use `readBin()` / `readChar(useBytes = TRUE)`. Each calls the
  connection's own `read` / `write` callback, as the C entry points do, and
  each refuses a connection that is not open.
- **The raw handle** (`get_connection`, `ConnectionCapabilities::from_handle`)
  exists only on R >= 4.6. `read_connection` / `write_connection` take the
  connection SEXP and work on every R.
- **Custom connections** (`RCustomConnection`, `RConnectionIo`) still need
  `R_new_custom_connection`, which has no R-level alternative, so a package
  that uses them carries that one NOTE on R < 4.6.

miniextendr mitigates this with **two layers of defense**:

### Compile-Time ABI Check

```rust
pub const EXPECTED_CONNECTIONS_VERSION: c_int = 1;

pub fn check_connections_version() {
    assert_eq!(
        R_CONNECTIONS_VERSION, EXPECTED_CONNECTIONS_VERSION,
        "R_CONNECTIONS_VERSION mismatch"
    );
}
```

This is called automatically by `RCustomConnection::build()`. If the R headers used during compilation have a different `R_CONNECTIONS_VERSION`, the assertion fails at compile time (both values are const), catching ABI mismatches before any unsafe code runs.

### Runtime Version Check

For additional safety, `check_connections_runtime()` probes the running R version:

```rust
use miniextendr_api::connection::check_connections_runtime;

unsafe {
    match check_connections_runtime() {
        Ok(()) => { /* R >= 4.3.0, connections supported */ }
        Err(msg) => panic!("Connections not available: {msg}"),
    }
}
```

This calls `R.Version()` at runtime and verifies R >= 4.3.0 (when `R_CONNECTIONS_VERSION = 1` was stabilized). Use this in package initialization or before creating your first connection to fail early with a clear error message.

The `Rconn` struct layout (`#[repr(C)]`) mirrors R's `struct Rconn` exactly. Any field reordering or resizing in a future R version would cause memory corruption. The version checks are the first line of defense.

## Capability Probing

You can query the capabilities and state of a connection at runtime using `ConnectionCapabilities`:

```rust
use miniextendr_api::connection::ConnectionCapabilities;

unsafe {
    let caps = ConnectionCapabilities::from_sexp(conn_sexp);
    println!("read={}, write={}, seek={}, text={}, open={}, blocking={:?}",
        caps.can_read, caps.can_write, caps.can_seek,
        caps.is_text, caps.is_open, caps.is_blocking);
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `can_read` | `bool` | Connection supports reading |
| `can_write` | `bool` | Connection supports writing |
| `can_seek` | `bool` | Connection supports seeking |
| `is_text` | `bool` | Text mode (`true`) vs binary mode (`false`) |
| `is_open` | `bool` | Connection is currently open |
| `is_blocking` | `Option<bool>` | Connection uses blocking I/O; `None` from `from_sexp` on R < 4.6 |

### Construction

| Method | Input | Notes |
|--------|-------|-------|
| `from_sexp(conn_sexp)` | R connection SEXP | `R_GetConnection` on R >= 4.6; `summary()` / `isSeekable()` before |
| `from_handle(handle)` | `Rconnection` handle | R >= 4.6 only, like `get_connection` |

### Binary Mode Helper

```rust
use miniextendr_api::connection::{is_binary_mode, connection_mode};

unsafe {
    if is_binary_mode(conn_sexp) {
        // Binary mode: "rb", "wb", "r+b", etc.
    }

    let mode = connection_mode(conn_sexp);  // e.g., "rb+"
}
```

`is_binary_mode()` checks if the connection's mode string contains `'b'`. `connection_mode()` returns the full mode string.

### Connection Description

```rust
use miniextendr_api::connection::connection_description;

unsafe {
    let desc = connection_description(conn_sexp); // e.g., "my data source"
}
```

## Error Handling

### Panic Safety

Ordinary I/O callbacks (`read`, `write`, `fgetc`, `seek`, `truncate`, `flush`,
`vfprintf`) run under `R_UnwindProtect`. Rust panics and `error!` conditions
become R errors after Rust destructors run. They do not become EOF or a
successful short write. The connection can still be closed after an error;
the framework does not roll back changes already made by the stream.

`warning!()` and `message!()` cannot suspend a callback, so inside one they
become an R error too. To report a warning or a message, queue it with
`defer_warning!()` / `defer_message!()`: it is signalled when the callback
returns.

`open` retains a `FALSE` fallback so the builder tears down failed opens.
`close` and `destroy` can run during garbage collection, so they suppress
failures and deferred conditions; `destroy` still drops the Rust owner.

### Destroy Always Runs

The `destroy` trampoline has special handling: even if `destroy()` panics, the `Box<T>` is still freed and the private pointer is set to null:

```rust
// Always drop the boxed state, even if destroy() panicked
let _ = unsafe { Box::from_raw(private as *mut T) };
(*conn).private = std::ptr::null_mut();
```

### std::io Error Handling

The adapters raise `miniextendr_connection_io_error` with the original message
and fields `operation`, `io_kind`, and `os_error` (the OS code when available,
`NA` otherwise). This applies to reads, buffered character reads, writes,
flushes, and seeks. A call that fails with `Interrupted` is repeated, as
`read_exact` and `write_all` do.

```r
tryCatch(
  readBin(con, "raw", 16L),
  miniextendr_connection_io_error = function(e) c(e$operation, e$io_kind)
)
```

R reads a short count from the `read` callback as the end of the stream, but
`Read::read` may return fewer bytes at any time. So a read fills the buffer
until the stream ends (`Ok(0)`), or until a non-blocking stream has nothing
more for now (`WouldBlock`); R's socket connections stop at the same two
points. A write is `write_all`, so a writer that stops mid-way (`WouldBlock`
included) raises: R would drop the rest of a short formatted write silently.

Seek returns the position **before** the move, as R requires. A position query
does not move. As in R, a fractional offset is truncated. A position before the
start leaves the stream where it is and returns that position, as `file()`
connections do: `readLines()` and `scan()` re-position a non-blocking seekable
connection with `seek(con, seek(con, -1, 1, 1), 1, 1)`. Other offsets the stream
refuses (an infinite one, or before the start relative to the current position
or the end) raise. The std::io adapters use one shared read/write cursor, so
`rw` does not select separate positions.

## Trampoline Architecture

R's connection system expects C function pointers for each callback. miniextendr bridges these to Rust trait methods using monomorphized trampolines:

```text
R calls C function pointer
  |
  v
trampoline<T>(conn: *mut Rconn) -> ReturnType
  |
  +--> connection_callback(|| {
  |      let state = get_state::<T>(conn);  // Extract &mut T from conn.private
  |      state.method(args)                  // Call trait method
  |    })
  |
  v
Return value (or an R error after Rust unwinding)
```

Each trampoline is a generic `unsafe extern "C-unwind" fn` parameterized by `T: RConnectionImpl`. When you call `build::<T>(state)`, the compiler generates concrete function pointers for your specific type. The guard catches Rust panics and R exits before returning to C.

The `private` field of the `Rconn` struct stores a `Box::into_raw(Box::new(state))` pointer. Trampolines cast this back to `&mut T` via `get_state()`.

## Complete Examples

### In-Memory Read/Write Buffer

A buffer that supports reading, writing, and seeking -- similar to `textConnection()` but in binary mode:

```rust
use miniextendr_api::connection::{RConnectionImpl, RCustomConnection};
use miniextendr_api::sys::SEXP;
use miniextendr_api::miniextendr;

struct MemoryBuffer {
    data: Vec<u8>,
    position: usize,
}

impl RConnectionImpl for MemoryBuffer {
    fn open(&mut self) -> bool {
        self.position = 0;
        true
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        let available = self.data.len().saturating_sub(self.position);
        let to_read = buf.len().min(available);
        if to_read > 0 {
            buf[..to_read].copy_from_slice(
                &self.data[self.position..self.position + to_read]
            );
            self.position += to_read;
        }
        to_read
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        let end_pos = self.position + buf.len();
        if end_pos > self.data.len() {
            self.data.resize(end_pos, 0);
        }
        self.data[self.position..end_pos].copy_from_slice(buf);
        self.position = end_pos;
        buf.len()
    }

    fn seek(&mut self, where_: f64, origin: i32, _rw: i32) -> f64 {
        if where_.is_nan() {
            return self.position as f64;
        }
        let new_pos = match origin {
            1 => where_.max(0.0) as usize,                        // Start
            2 => (self.position as isize + where_ as isize) as usize, // Current
            3 => (self.data.len() as isize + where_ as isize) as usize, // End
            _ => panic!("unknown seek origin {origin}"),
        };
        let previous = miniextendr_api::coerce::TryCoerce::<f64>::try_coerce(self.position)
            .expect("connection position is representable in R");
        self.position = new_pos.min(self.data.len());
        previous
    }

    fn flush(&mut self) -> i32 { 0 }
}

#[miniextendr]
pub fn memory_connection() -> SEXP {
    RCustomConnection::new()
        .description("memory buffer")
        .mode("r+b")
        .class_name("memoryConnection")
        .can_read(true)
        .can_write(true)
        .can_seek(true)
        .text(false)
        .build(MemoryBuffer { data: Vec::new(), position: 0 })
}
```

```r
conn <- memory_connection()
writeBin(charToRaw("Hello, World!"), conn)
seek(conn, 0)
rawToChar(readBin(conn, "raw", 13))
#> [1] "Hello, World!"
close(conn)
```

### Streaming Generator

A read-only connection that generates data on-the-fly:

```rust
struct CounterConnection {
    current: i32,
    max: i32,
    buffer: Vec<u8>,
    buffer_pos: usize,
}

impl RConnectionImpl for CounterConnection {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        // Refill internal buffer when exhausted
        if self.buffer_pos >= self.buffer.len() {
            if self.current > self.max {
                return 0; // EOF
            }
            self.buffer = format!("{}\n", self.current).into_bytes();
            self.buffer_pos = 0;
            self.current += 1;
        }

        let available = self.buffer.len() - self.buffer_pos;
        let n = buf.len().min(available);
        buf[..n].copy_from_slice(&self.buffer[self.buffer_pos..self.buffer_pos + n]);
        self.buffer_pos += n;
        n
    }
}

#[miniextendr]
pub fn counter_connection(start: i32, end: i32) -> SEXP {
    RCustomConnection::new()
        .description(&format!("counter {}:{}", start, end))
        .mode("r")
        .class_name("counterConnection")
        .can_read(true)
        .text(true)
        .build(CounterConnection {
            current: start, max: end,
            buffer: Vec::new(), buffer_pos: 0,
        })
}
```

```r
conn <- counter_connection(1L, 5L)
readLines(conn)
#> [1] "1" "2" "3" "4" "5"
close(conn)
```

### std::io Cursor Adapter

Wrap Rust's `std::io::Cursor` with zero boilerplate:

```rust
use miniextendr_api::connection::RConnectionIo;
use std::io::Cursor;

#[miniextendr]
pub fn cursor_connection(data: Vec<u8>) -> SEXP {
    RConnectionIo::new(Cursor::new(data))
        .description("Rust Cursor")
        .mode("r+b")
        .class_name("cursorConnection")
        .build_read_write_seek()
}
```

### Transform Connection

Apply a byte-level transformation while reading:

```rust
struct TransformConnection<F: Fn(u8) -> u8 + 'static> {
    data: Vec<u8>,
    position: usize,
    transform: F,
}

impl<F: Fn(u8) -> u8 + 'static> RConnectionImpl for TransformConnection<F> {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        let available = self.data.len().saturating_sub(self.position);
        let n = buf.len().min(available);
        for i in 0..n {
            buf[i] = (self.transform)(self.data[self.position + i]);
        }
        self.position += n;
        n
    }
}

#[miniextendr]
pub fn uppercase_connection(text: &str) -> SEXP {
    RCustomConnection::new()
        .description("uppercase transform")
        .mode("r")
        .class_name("uppercaseConnection")
        .can_read(true)
        .text(true)
        .build(TransformConnection {
            data: text.as_bytes().to_vec(),
            position: 0,
            transform: |b| if b.is_ascii_lowercase() { b.to_ascii_uppercase() } else { b },
        })
}
```

## Helper Functions

For working with *existing* R connections from Rust (e.g., when a user passes a connection as an argument):

```rust
use miniextendr_api::connection::{read_connection, write_connection};

// Read from it
let mut buf = [0u8; 1024];
let n = unsafe { read_connection(conn_sexp, &mut buf) };

// Write to it; the count can be short before R 4.6 (see below)
let data = b"output data\n";
let written = unsafe { write_connection(conn_sexp, data) };
```

Both work on every supported R. Built against R >= 4.6 they are
`R_ReadConnection` / `R_WriteConnection`; before, `readBin()` / `writeBin()`
or `readChar()` / `writeChar()` (see "R versions before 4.6"), which can write
less than the whole buffer to a text-mode connection. Both refuse a connection
that is not open.

`get_connection(conn_sexp)` returns the raw `Rconnection` handle, for reading
the `Rconn` struct directly (`ConnectionCapabilities::from_handle` takes it).
It is built only against R >= 4.6: R 4.4 and 4.5 offer no R-level route to
the handle, and `R CMD check` notes `R_GetConnection` there as non-API.

## Standard Streams

`RStdin`, `RStdout` and `RStderr` are zero-sized handles for R's terminal
connections `stdin()`, `stdout()` and `stderr()`. They convert to and from
those connection objects, so a `#[miniextendr]` function can take or return
one.

`RStdout` and `RStderr` implement `std::io::Write` through R's console
(`Rprintf` / `REprintf`), so `sink()` and `capture.output()` see what they
write.

`RStdin` reads R's console input, the way R does: one line at a time with
`readLines(stdin(), n = 1L, warn = FALSE)`. R's terminal `stdin()` has no
`read` callback, so `R_ReadConnection` can't read it. The API follows
`std::io::Stdin`:

- `RStdin` implements `std::io::Read`. Every handle reads the same stream
  through one line buffer.
- `RStdin.lock()` returns an `RStdinLock`, which implements `std::io::BufRead`
  (`fill_buf()` is the unread rest of the current line) and `Read`.
- `RStdin.read_line(&mut s)` and `RStdin.lines()` are shortcuts through a lock.

```rust
use miniextendr_api::connection::RStdin;

#[miniextendr]
pub fn read_numbers(n: usize) -> Vec<f64> {
    RStdin
        .lines()
        .take(n)
        .map(|line| line.expect("read from stdin").trim().parse().expect("a number"))
        .collect()
}
```

What a read returns:

- Every line ends in `"\n"`. `readLines()` drops the terminator (`"\n"`,
  `"\r\n"` or `"\r"`), and `RStdin` adds `"\n"`, also to a last line that had
  none.
- The bytes are the ones `readLines()` returns, in the native encoding.
- At the end of the input, `read()` returns `Ok(0)`.
- Rust holds at most one line. An R-side read of `stdin()` while part of a
  line is still unread in Rust gets the next line.

**`stdin()` is R's console, not always the process's standard input.** In an
interactive session it is what the user types. Under `Rscript` or
`R --file`, it is the script being run: a read gets the lines after the code
being evaluated, and input piped into `Rscript` never reaches it. For the
process's standard input, open R's `file("stdin")` and read that connection.

**R's main thread only.** Off it, a read that needs a new line returns an
`io::Error`. A second `RStdinLock` on one thread panics ("RStdin is already
locked on this thread"), including a read through another `RStdin` handle
while a lock is held.

## Iterator and random streams

`IteratorReader<I>` adapts any `Iterator<Item = u8>` to `std::io::Read`.
It works with the existing `RConnectionIo` builder. Construction and empty
reads consume nothing; EOF is permanent. A connection reads sequentially, so it
does not need ALTREP's finite vector length or cache for repeated indexing. For
a vector R can index, build ALTREP data from the iterator instead:
[ALTREP.md](ALTREP.md#constructing-from-any-iterator).

For raw RNG bytes, `rand` already provides the adapter:

```rust,ignore
use miniextendr_api::connection::RConnectionIo;
use miniextendr_api::rand::{RngReader, SeedableRng, rngs::StdRng};

let connection = RConnectionIo::new(RngReader(StdRng::seed_from_u64(42)))
    .description("random bytes")
    .build_read();
```

Enable `connections` and `rand`. This is an unbounded stream; the R caller
chooses how many bytes to read with `readBin(con, "raw", n)`.

For `rand_distr` samples, use its existing `Distribution::sample_iter` and
choose the byte encoding explicitly. Enable `connections` and `rand_distr`:

```rust,ignore
use miniextendr_api::connection::{IteratorReader, RConnectionIo};
use miniextendr_api::rand::{SeedableRng, rngs::StdRng};
use miniextendr_api::rand_distr::{Distribution, Normal};

let samples = Normal::new(0.0_f64, 1.0).unwrap()
    .sample_iter(StdRng::seed_from_u64(42));
let connection = RConnectionIo::new(
    IteratorReader::new(samples.flat_map(f64::to_le_bytes)),
)
.description("normal samples")
.build_read();
```

Read with `readBin(con, "double", n, size = 8L, endian = "little")`. A read
can stop partway through a sample: `FlatMap` retains the remaining bytes, so
changing read sizes does not discard or redraw samples. Add `take(n)` to the
sample iterator before encoding to make a finite stream.

These examples own an independent Rust RNG. `RRng` instead uses R's global
RNG state; a guard around the function that creates the connection does not
cover reads performed later. A custom `RConnectionImpl::read` using `RRng`
must acquire and release R's RNG state inside each guarded read callback.
Never retain an RNG-state guard for the connection's entire lifetime.

## Limitations

- **Experimental API.** R may change the connection ABI in any release. The compile-time version check catches this, but you may need to update miniextendr when upgrading R.
- **No `vfprintf` by default.** The `vfprintf` callback receives raw C varargs (`va_list`), which are not portable in Rust. The default returns -1. R rarely calls this -- it prefers the `write` callback.
- **Not `Send`/`Sync`.** Connections run on the main R thread. Your `RConnectionImpl` type does not need to be thread-safe.
- **GC protection.** The SEXP returned by `build()` must be protected from R's garbage collector if you store it. Returning it directly from a `#[miniextendr]` function handles this automatically.
- **No stat support.** R's connection C API does not include a `stat` callback (file size, modification time, etc.). If you need stat-like information, track it in your `RConnectionImpl` type or query it through a separate `#[miniextendr]` function.

## See Also

- [FEATURES.md](FEATURES.md) -- Feature flags reference (`connections`)
- [THREADS.md](THREADS.md) -- Thread safety and the worker thread model
- [ERROR_HANDLING.md](ERROR_HANDLING.md) -- Panic handling across the FFI boundary
- Test fixtures: `rpkg/src/rust/connection_tests.rs`
