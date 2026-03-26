# FFI Guard and Panic Telemetry

How miniextendr catches panics at Rust-R boundaries and provides structured diagnostics.

## Overview

Four independent FFI boundaries exist between Rust and R:

| Boundary | Where | Guard |
|----------|-------|-------|
| Main thread (default) | `unwind_protect.rs` | `R_UnwindProtect` + `catch_unwind` + `error_in_r` tagged values |
| Worker thread (opt-in, requires `worker-thread` feature) | `worker.rs` | `catch_unwind` + `Rf_error` |
| ALTREP trampolines | `altrep_bridge.rs` | Per-type `AltrepGuard` const |
| `R_UnwindProtect` | `unwind_protect.rs` | Catches panics + R longjmps |
| Connection callbacks | `connection.rs` | `catch_unwind` + fallback value |

Each boundary converts Rust panics into R errors (or safe fallback values). The
`ffi_guard` module extracts the common catch-and-convert pattern, and
`panic_telemetry` provides a hook that fires at every conversion site.

```
Rust panic
  |
  v
catch_unwind / R_UnwindProtect
  |
  v
panic_telemetry::fire(message, source)   <-- hook fires here
  |
  v
Rf_error(message)  or  return fallback
```

---

## GuardMode

`GuardMode` selects the catch strategy. Defined in `miniextendr_api::ffi_guard`.

### CatchUnwind

Wraps the closure in `std::panic::catch_unwind`. On panic, fires telemetry and
raises an R error via `Rf_error` (diverges -- never returns).

Use when the closure does **not** call R APIs that might longjmp.

```rust
use miniextendr_api::ffi_guard::{guarded_ffi_call, GuardMode};
use miniextendr_api::panic_telemetry::PanicSource;

let result = guarded_ffi_call(
    || some_pure_rust_work(),
    GuardMode::CatchUnwind,
    PanicSource::Worker,
);
```

### RUnwind

Uses `R_UnwindProtect` to catch **both** Rust panics and R longjmps. Ensures
Rust destructors run even when R errors occur inside the closure.

Use when the closure calls R APIs (e.g., `Rf_allocVector`, `Rf_eval`).

```rust
let result = guarded_ffi_call(
    || call_r_api_that_might_error(),
    GuardMode::RUnwind,
    PanicSource::Altrep,
);
```

### When to Use Each

| Scenario | Mode |
|----------|------|
| Pure Rust computation | `CatchUnwind` |
| Closure calls R APIs | `RUnwind` |
| ALTREP callback (trivial, no-panic) | `AltrepGuard::Unsafe` (stays in `altrep_bridge`) |

---

## guarded_ffi_call

The primary entry point. Executes a closure under the selected guard mode.

```rust
pub fn guarded_ffi_call<F, R>(f: F, mode: GuardMode, source: PanicSource) -> R
```

On panic:
1. Extracts the panic message from the payload.
2. Fires `panic_telemetry::fire()`.
3. `CatchUnwind`: raises R error via `Rf_error` (diverges).
4. `RUnwind`: delegates to `with_r_unwind_protect_sourced()`.

## guarded_ffi_call_with_fallback

A variant that returns a fallback value instead of diverging on panic.

```rust
pub fn guarded_ffi_call_with_fallback<F, R>(f: F, fallback: R, source: PanicSource) -> R
```

Used by connection trampolines where the C caller expects a return value (e.g.,
bytes-read count). Calling `Rf_error` is not an option because the caller would
not handle the longjmp correctly.

```rust
use miniextendr_api::ffi_guard::guarded_ffi_call_with_fallback;
use miniextendr_api::panic_telemetry::PanicSource;

// Connection read trampoline: return 0 bytes on panic
let bytes_read = guarded_ffi_call_with_fallback(
    || do_read(buf, n),
    0,  // fallback: 0 bytes read
    PanicSource::Connection,
);
```

---

## ALTREP Guard Integration

ALTREP trampolines use a compile-time constant `T::GUARD` (from the `Altrep`
trait) to select the guard mode. Since it is a const, the compiler eliminates
dead branches at monomorphization -- zero runtime cost.

```rust
// In altrep_bridge.rs
fn guarded_altrep_call<T: Altrep, F, R>(f: F) -> R {
    match T::GUARD {
        AltrepGuard::Unsafe    => f(),                     // no protection
        AltrepGuard::RustUnwind => guarded_ffi_call(f, CatchUnwind, Altrep),
        AltrepGuard::RUnwind   => guarded_ffi_call(f, RUnwind, Altrep),
    }
}
```

Set the guard per ALTREP type via the derive attribute:

```rust
#[derive(Altrep)]
#[altrep(rust_unwind)]  // default: catch_unwind only
struct MyVec { ... }

#[derive(Altrep)]
#[altrep(r_unwind)]     // R_UnwindProtect (for callbacks that call R APIs)
struct MyLazyVec { ... }

#[derive(Altrep)]
#[altrep(unsafe)]       // no guard (trivial accessors that cannot panic)
struct MyTrivialVec { ... }
```

---

## Panic Telemetry

`panic_telemetry` provides a hook that fires at every panic-to-R-error
conversion site. Defined in `miniextendr_api::panic_telemetry`.

### PanicSource

Identifies which boundary caught the panic:

```rust
pub enum PanicSource {
    Worker,        // worker thread (run_on_worker)
    Altrep,        // ALTREP trampoline (guarded_altrep_call)
    UnwindProtect, // with_r_unwind_protect
    Connection,    // connection callback trampoline
}
```

### PanicReport

The structured report passed to the hook:

```rust
pub struct PanicReport<'a> {
    pub message: &'a str,     // panic message text
    pub source: PanicSource,  // which boundary caught it
}
```

### Setting a Hook

Register a closure that receives every panic report:

```rust
use miniextendr_api::panic_telemetry::{set_panic_telemetry_hook, PanicReport};

set_panic_telemetry_hook(|report| {
    eprintln!("[{:?}] panic: {}", report.source, report.message);
});
```

Only one hook can be active at a time. Calling `set_panic_telemetry_hook` again
replaces the previous hook.

### Clearing the Hook

```rust
use miniextendr_api::panic_telemetry::clear_panic_telemetry_hook;

clear_panic_telemetry_hook();
```

### Thread Safety and Reentrancy

- The hook may fire from **any thread** (worker, main R thread, etc.).
- The hook closure must be `Send + Sync + 'static`.
- The read lock is released before the hook is invoked, so calling
  `set_panic_telemetry_hook` or `clear_panic_telemetry_hook` from within a hook
  is safe (no deadlock).
- Secondary panics inside the hook are caught via `catch_unwind` and silently
  suppressed -- the system is already on a panic-to-error path.

### Performance

`fire()` takes a read lock on an `RwLock<Option<Arc<...>>>`. In the common case
(no hook set, or hook set but no panic), the cost is a single uncontended read
lock -- effectively free. The hook only runs on panic paths, never on hot paths.

---

## Integration Points

All four panic-to-R-error sites call `panic_telemetry::fire()`:

| Site | File | What happens after `fire()` |
|------|------|-----------------------------|
| Worker thread (opt-in) | `worker.rs` | `Rf_error` on main thread (requires `worker-thread` feature) |
| ALTREP bridge | `ffi_guard.rs` (via `guarded_ffi_call`) | `Rf_error` or `R_UnwindProtect` |
| Unwind protect | `unwind_protect.rs` | `R_ContinueUnwind` |
| Connection | `ffi_guard.rs` (via `guarded_ffi_call_with_fallback`) | Returns fallback value |

---

## See Also

- [Error Handling](ERROR_HANDLING.md) -- How panics, Result, and R errors interact
- [ALTREP Guards](ALTREP_GUARDS.md) -- Per-type guard mode selection for ALTREP
- [Threads](THREADS.md) -- Worker thread architecture
- [Connections](CONNECTIONS.md) -- Custom R connections from Rust
