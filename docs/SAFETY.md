# Safety Documentation

This document explains the thread safety invariants and FFI safety requirements
for miniextendr. Read this before contributing unsafe code or modifying the
thread or unwind_protect modules.

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

### How Panics Are Caught

R's longjmp-based error handling bypasses Rust destructors. miniextendr uses
`R_UnwindProtect` on the main thread to catch both:

1. `catch_unwind` catches Rust panics, allowing destructors to run
2. `R_UnwindProtect` catches R longjmps (e.g., `Rf_error`), runs cleanup
3. Errors are converted to R errors after Rust cleanup completes

### Thread Identification

```rust
// thread.rs
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

## ExternalPtr<T> Thread Safety

`ExternalPtr<T>` is `!Send` and `!Sync`. It owns a GC root in a main-thread
pool and an R finalizer, and dropping it releases the root through
`with_r_thread`, so every handle is made, used and dropped on R's main thread.
A `#[miniextendr]` argument of type `ExternalPtr<T>` therefore needs no
`T: Send` (#2027).

**Safe pattern**: to hand the value to another thread, move it out
(`ExternalPtr::into_inner`, on the main thread) and send the plain Rust value.

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

The atomic refcount only coordinates restoration of the process-global value;
it does not make R's global state, garbage collector, or API thread-safe.
Writing R Extensions requires package R API calls to remain on the main thread
and says packages must not change these stack globals for secondary-thread R
calls. Package code must keep every R API call on the main thread instead. Removal or
relocation of these misleading helpers is tracked in #1352.

## Allocator Safety

The R-backed allocator (`allocator.rs`) calls `Rf_allocVector`, which must run
on R's main thread. Every allocator entry point goes through `with_r_thread`,
which runs the allocation inline on the main thread and panics on any other
thread (a `std::thread::spawn` thread, a Rayon worker).

**longjmp warning**: `Rf_allocVector` can longjmp on allocation failure. The
allocator is safe inside a `#[miniextendr]` body (the generated wrapper runs it
under `R_UnwindProtect`) but can cause issues in other contexts.

## FFI Function Categories

All non-variadic functions in `ffi.rs` marked with `#[r_ffi_checked]` behave
identically: the checked variant calls `with_r_thread`, which runs the call
inline on R's main thread and panics on any other thread. This applies to both
value-returning functions (`Rf_ScalarInteger`, `Rf_allocVector`) and
pointer-returning functions (`INTEGER`, `REAL`, `DATAPTR`). The `*_unchecked`
variants skip the check for contexts already known to be on the main thread
(ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread` closures).

## Initialization Requirements

`miniextendr_runtime_init()` must be called before any R API use. In practice
this happens via the `miniextendr_init!` macro, which generates
`R_init_<pkgname>` and calls `package_init()` (which runs
`miniextendr_runtime_init()` plus wrapper/registration setup):

```rust
miniextendr_api::miniextendr_init!(pkgname);
```

See [ENTRYPOINT.md](ENTRYPOINT.md) for the full init sequence. `package_init()`:
1. Records `R_MAIN_THREAD_ID` for thread checks (no thread is spawned)

**Invariant**: Must be called from the main thread. Calling from another thread
will cause all subsequent thread checks to be incorrect.

## Summary of Invariants

| Component | Invariant |
|-----------|-----------|
| `R_MAIN_THREAD_ID` | Set once, from main thread, during init |
| `ExternalPtr<T>` | `!Send` + `!Sync`; made, used and dropped on the main thread |
| `AltrepSexp` | !Send + !Sync; materialization on main thread only |
| SEXP (via `TryFromSexp`) | ALTREP auto-materialized before function body runs |
| `R_CONTINUATION_TOKEN` | Created once, preserved for session lifetime |
| `StackCheckGuard` | Legacy process-global restoration only; not package thread safety (#1352) |
| Allocator | Main thread only; panic otherwise |
| Pointer APIs | Main thread only; panic otherwise |

## Reporting Safety Issues

If you discover a soundness issue in miniextendr, please report it via
[GitHub Issues](https://github.com/A2-ai/miniextendr/issues) with the
`[SAFETY]` tag.
