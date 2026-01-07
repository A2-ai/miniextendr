# Safety Documentation

This document explains the thread safety invariants and FFI safety requirements
for miniextendr. Read this before contributing unsafe code or modifying the
worker/thread modules.

## Overview

miniextendr interfaces with R's C API, which has several constraints:

1. **R is single-threaded** - Most R APIs must be called from the main thread
2. **R uses longjmp** - R errors bypass Rust destructors unless handled
3. **R has its own GC** - SEXP objects can be collected if not protected

miniextendr provides abstractions to handle all three safely.

## Thread Model

### Main Thread vs Worker Thread

```
┌─────────────────────────────────────────────────────────────────┐
│  R Main Thread                                                  │
│  ├── R_init_<pkgname>() calls miniextendr_worker_init()         │
│  ├── .Call() entry points run on this thread                    │
│  └── All R API calls must happen here                           │
│                                                                 │
│  Worker Thread (spawned by miniextendr_worker_init)             │
│  ├── User Rust code runs here via run_on_worker()               │
│  ├── Panics are caught, converted to R errors                   │
│  └── Uses with_r_thread() to call R APIs                        │
└─────────────────────────────────────────────────────────────────┘
```

### Why Use a Worker Thread?

R's longjmp-based error handling bypasses Rust destructors. If a panic or R
error occurs, RAII cleanup won't run. The worker thread pattern solves this:

1. User code runs on a separate worker thread
2. `catch_unwind` catches panics, allowing destructors to run
3. `R_UnwindProtect` catches R errors, routes them through cleanup handlers
4. The main thread converts errors to R errors after cleanup completes

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
during `miniextendr_worker_init()`. Any call before initialization returns
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
- Sending closures to the main thread for execution (`MainThreadWork`)
- Sending raw pointers for R API calls (`SendablePtr<T>`)
- Sending allocation results back to worker (`SendableDataPtr`)

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

`ExternalPtr<T>` is **not** `Send` or `Sync` because:

1. The underlying SEXP is an R object that should only be accessed on the main thread
2. R's finalizer registration (`R_RegisterCFinalizerEx`) must happen on main thread
3. The data pointer can become invalid if R garbage collects the SEXP

**Safe pattern**: Create `ExternalPtr` on main thread, return to R. Access only
via `.Call` entry points (which run on main thread).

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

**miniextendr's approach** (in `run_on_worker`):

1. Wrap user code in `catch_unwind` (catches Rust panics)
2. Run via `R_UnwindProtect` (catches R errors)
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
2. **Worker context**: Uses `with_r_thread_or_inline` to route calls
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

Functions in `ffi.rs` marked with `#[r_ffi_checked]` fall into two categories:

### Value-returning functions

Examples: `Rf_ScalarInteger`, `Rf_allocVector`

These are automatically routed to the main thread via `with_r_thread` when
called from a worker. The result (SEXP) is sent back to the worker.

### Pointer-returning functions

Examples: `INTEGER`, `REAL`, `DATAPTR`

These **must** be called on the main thread and panic otherwise. The returned
pointer could become invalid if:
- R's GC runs on the main thread while worker holds the pointer
- The SEXP is not protected

**Pattern**: Use `with_r_thread` to get data pointers and copy data within
the closure, rather than holding pointers across thread boundaries.

## Initialization Requirements

`miniextendr_worker_init()` must be called before any R API use:

```c
void R_init_pkgname(DllInfo *dll) {
    miniextendr_worker_init();  // First!
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
}
```

This function:
1. Records `R_MAIN_THREAD_ID` for thread checks
2. Spawns the worker thread
3. Sets up the job channel for `run_on_worker`

**Invariant**: Must be called from the main thread. Calling from another thread
will cause all subsequent thread checks to be incorrect.

## Summary of Invariants

| Component | Invariant |
|-----------|-----------|
| `R_MAIN_THREAD_ID` | Set once, from main thread, during init |
| `Sendable<T>` | Value moved, not shared; accessed only at destination |
| `ExternalPtr<T>` | Not Send/Sync; main thread only |
| `R_CONTINUATION_TOKEN` | Created once, preserved for session lifetime |
| `StackCheckGuard` | Atomic refcount; last drop restores limit |
| Allocator | Main thread or worker context only |
| Pointer APIs | Main thread only; panic otherwise |

## Reporting Safety Issues

If you discover a soundness issue in miniextendr, please report it via
[GitHub Issues](https://github.com/miniextendr/miniextendr/issues) with the
`[SAFETY]` tag.
