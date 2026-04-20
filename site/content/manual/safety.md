+++
title = "Safety Documentation"
weight = 17
description = "This document explains the thread safety invariants and FFI safety requirements for miniextendr. Read this before contributing unsafe code or modifying the worker, thread, or unwind_protect modules."
+++

This document explains the thread safety invariants and FFI safety requirements
for miniextendr. Read this before contributing unsafe code or modifying the
worker, thread, or unwind_protect modules.

## Overview

miniextendr interfaces with R's C API, which has several constraints:

1. **R is single-threaded** - Most R APIs must be called from the main thread
2. **R uses longjmp** - R errors bypass Rust destructors unless handled
3. **R has its own GC** - SEXP objects can be collected if not protected

miniextendr provides abstractions to handle all three safely.

## Thread Model

### Default: Main Thread with R_UnwindProtect

```text
┌─────────────────────────────────────────────────────────────────┐
│  R Main Thread                                                  │
│  ├── R_init_<pkgname>() calls miniextendr_runtime_init()         │
│  ├── .Call() entry points run on this thread                    │
│  ├── User Rust code runs inline via with_r_unwind_protect       │
│  ├── catch_unwind catches Rust panics                           │
│  ├── R_UnwindProtect catches R longjmps                         │
│  └── All R API calls happen here (no thread hop)                │
└─────────────────────────────────────────────────────────────────┘
```

### Optional: Worker Thread (with `worker-thread` feature)

```text
┌─────────────────────────────────────────────────────────────────┐
│  R Main Thread                                                  │
│  ├── .Call() entry points run on this thread                    │
│  └── All R API calls must happen here                           │
│                                                                 │
│  Worker Thread (spawned by miniextendr_runtime_init)             │
│  ├── User Rust code runs here via run_on_worker()               │
│  ├── Panics are caught, converted to R errors                   │
│  └── Uses with_r_thread() to call R APIs                        │
└─────────────────────────────────────────────────────────────────┘
```

### How Panics Are Caught

R's longjmp-based error handling bypasses Rust destructors. miniextendr uses
`R_UnwindProtect` on the main thread to catch both:

1. `catch_unwind` catches Rust panics, allowing destructors to run
2. `R_UnwindProtect` catches R longjmps (e.g., `Rf_error`), runs cleanup
3. Errors are converted to R errors after Rust cleanup completes

With the `worker-thread` feature, the same safety is achieved via bidirectional
channels: user code runs on the worker, `catch_unwind` catches panics, and
`with_r_thread` routes R API calls to the main thread inside `R_UnwindProtect`.

### Thread Identification

```rust
// worker.rs
static R_MAIN_THREAD_ID: OnceLock<thread::ThreadId> = OnceLock::new();

pub fn is_r_main_thread() -> bool {
    R_MAIN_THREAD_ID
        .get()
        .map(|&id| id == std::thread::current().id())
        .unwrap_or(false)  // Safe default: assume NOT main thread
}
```

**Invariant**: `R_MAIN_THREAD_ID` is set exactly once, from the main thread,
during `miniextendr_runtime_init()`. Any call before initialization returns
`false` (safe default - prevents R API calls from wrong thread).

## Sendable Wrappers

### `Sendable<T>`

```rust
// worker.rs
#[repr(transparent)]
pub struct Sendable<T>(pub T);
unsafe impl<T> Send for Sendable<T> {}
```

**Why it's safe**: `Sendable` is used to transfer *owned* data between threads.
The type system ensures:

1. The value is moved into `Sendable` on one thread
2. Transmitted to another thread via channels
3. Extracted and used exclusively on the destination thread

The data is never accessed concurrently - ownership transfers completely.

**Use cases**:
- Sending raw pointers for R API calls (`SendablePtr<T>`)
- Sending allocation results back to callers (`SendableDataPtr`)
- With `worker-thread`: sending closures to the main thread (`MainThreadWork`)

### `SendablePtr<T>` (externalptr.rs)

```rust
type SendablePtr<T> = Sendable<NonNull<T>>;
```

Used to send pointer addresses between threads. The pointed-to data is only
accessed on the main thread after the pointer arrives.

### `SendableDataPtr` (allocator.rs)

```rust
type SendableDataPtr = Sendable<*mut u8>;
```

Similar to `SendablePtr` but allows null (for allocation failures).

## ExternalPtr<T> Thread Safety

`ExternalPtr<T>` is `Send` when `T: Send` (declared as
`unsafe impl<T: TypedExternal + Send> Send for ExternalPtr<T>` in
`miniextendr-api/src/externalptr.rs`). It is **not** `Sync` - there is no
interior synchronization, and R's runtime is single-threaded.

This is sound because the `ExternalPtr` value itself is just an owning handle
over a heap allocation (`Box<Box<dyn Any>>`); transferring the handle to
another thread moves ownership, no shared state is created. What is *not*
allowed from off-thread is calling R API functions on the underlying SEXP -
R's GC, finalizer registration, and pointer dereference all require the main
thread.

**Safe pattern**: Freely move `ExternalPtr<T: Send>` between Rust threads for
compute-only work, but perform all R API calls (including construction from
a SEXP, finalizer registration, and returning to R) on the main thread.
`.Call` entry points always run on the main thread.

## R_UnwindProtect

R errors use `longjmp`, which bypasses Rust destructors. `R_UnwindProtect`
provides a cleanup callback that runs before the longjmp:

```rust
// unwind_protect.rs
R_UnwindProtect_C_unwind(
    Some(trampoline),      // Code to run
    data.cast(),           // Data for trampoline
    Some(cleanup_handler), // Cleanup on longjmp
    data.cast(),           // Data for cleanup
    token,                 // Continuation token
)
```

**miniextendr's approach** (main thread, default):

1. Wrap user code in `catch_unwind` (catches Rust panics)
2. Run via `R_UnwindProtect` (catches R longjmps)
3. If R error: cleanup handler runs, then `R_ContinueUnwind` completes R's error handling
4. If Rust panic: error message is extracted and converted to an R error

**With `worker-thread` feature** (in `run_on_worker`):

1. Wrap user code in `catch_unwind` on the worker thread
2. R API calls route through `with_r_thread` → `R_UnwindProtect` on main thread
3. If R error: cleanup handler sends error message to worker, then panics
4. Worker thread catches the panic, drops resources, sends `Done(Err(...))`
5. Main thread calls `R_ContinueUnwind` to let R complete its error handling

**Key invariant**: The cleanup handler must not block. It sends an error message
and panics immediately so `catch_unwind` can catch it.

## Continuation Token

```rust
// unwind_protect.rs
static R_CONTINUATION_TOKEN: OnceLock<SEXP> = OnceLock::new();
```

A single global token (created via `R_MakeUnwindCont`, preserved with
`R_PreserveObject`) is used for all unwind operations. This avoids leaking
one token per thread.

**Invariant**: The token is created on first use on the main thread and remains
valid for the entire R session.

## Stack Checking (nonapi feature)

R tracks stack bounds to detect overflow:
- `R_CStackStart` - top of main thread's stack
- `R_CStackLimit` - stack size limit
- `R_CStackDir` - growth direction

On non-main threads, these values are invalid. `StackCheckGuard` disables
checking by setting `R_CStackLimit = usize::MAX`:

```rust
// thread.rs
impl StackCheckGuard {
    pub fn disable() -> Self {
        let prev_count = STACK_GUARD_COUNT.fetch_add(1, Ordering::SeqCst);
        if prev_count == 0 {
            let original = get_r_cstack_limit();
            ORIGINAL_STACK_LIMIT.store(original, Ordering::SeqCst);
            unsafe { set_r_cstack_limit(usize::MAX); }
        }
        Self { _private: () }
    }
}
```

**Invariant**: Uses atomic refcounting so multiple concurrent guards work
correctly. Only the last guard to drop restores the original limit.

## Allocator Safety

The R-backed allocator (`allocator.rs`) has special requirements:

1. **Main thread only**: Calls `Rf_allocVector` which must run on main thread
2. **Thread routing**: Uses `with_r_thread_or_inline` - runs inline on main thread,
   routes via `with_r_thread` if worker context exists, panics otherwise
3. **No fallback**: Panics if called from arbitrary thread without worker context

```rust
fn with_r_thread_or_inline<R, F>(f: F) -> R {
    if is_r_main_thread() {
        f()
    } else if has_worker_context() {
        with_r_thread(f)
    } else {
        panic!("R allocator called from non-main thread without worker context");
    }
}
```

**longjmp warning**: `Rf_allocVector` can longjmp on allocation failure. The
allocator is safe when used inside `run_on_worker` (which has unwind protection)
but can cause issues in other contexts.

## FFI Function Categories

All non-variadic functions in `ffi.rs` marked with `#[r_ffi_checked]` behave
identically: they are routed to the main thread via `with_r_thread` when called
from the worker thread. The return value is wrapped in `Sendable` and sent back
to the caller. This applies to both value-returning functions
(`Rf_ScalarInteger`, `Rf_allocVector`) and pointer-returning functions
(`INTEGER`, `REAL`, `DATAPTR`).

Pointer-returning functions are safe to route because the underlying SEXP must
be GC-protected by the caller, and R's GC only runs during R API calls which
are serialized through `with_r_thread`.

By default (main thread execution), all checked wrappers run inline. With the
`worker-thread` feature, they route through `with_r_thread`.

Without the `worker-thread` feature, calling checked wrappers from a non-main
thread panics (there is no routing infrastructure to fall back on).

## Initialization Requirements

`miniextendr_runtime_init()` must be called before any R API use. In practice
this happens via the `miniextendr_init!` macro, which generates
`R_init_<pkgname>` and calls `package_init()` (which runs
`miniextendr_runtime_init()` plus wrapper/registration setup):

```rust
miniextendr_api::miniextendr_init!(pkgname);
```

See [ENTRYPOINT.md](ENTRYPOINT.md) for the full init sequence. `package_init()`:
1. Records `R_MAIN_THREAD_ID` for thread checks
2. With `worker-thread` feature: spawns the worker thread and sets up channels
3. Without `worker-thread`: only records the thread ID (no thread spawned)

**Invariant**: Must be called from the main thread. Calling from another thread
will cause all subsequent thread checks to be incorrect.

## Summary of Invariants

| Component | Invariant |
|-----------|-----------|
| `R_MAIN_THREAD_ID` | Set once, from main thread, during init |
| `Sendable<T>` | Value moved, not shared; accessed only at destination |
| `ExternalPtr<T>` | `Send` when `T: Send`; not `Sync`; R API calls require main thread |
| `AltrepSexp` | !Send + !Sync; materialization on main thread only |
| SEXP (via `TryFromSexp`) | ALTREP auto-materialized before function body runs |
| `R_CONTINUATION_TOKEN` | Created once, preserved for session lifetime |
| `StackCheckGuard` | Atomic refcount; last drop restores limit |
| Allocator | Main thread or worker context only |
| Pointer APIs | Main thread only; panic otherwise |

## Reporting Safety Issues

If you discover a soundness issue in miniextendr, please report it via
[GitHub Issues](https://github.com/miniextendr/miniextendr/issues) with the
`[SAFETY]` tag.
