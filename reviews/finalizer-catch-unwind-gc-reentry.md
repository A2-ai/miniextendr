# CI failure: "recursive gc invocation" from catch_unwind in GC finalizer

## What was attempted

PR #272 (`fix/finalizer-panic-safety`) adds panic safety to the
`release_any` ExternalPtr finalizer and macro-generated `__mx_drop_*`
functions by wrapping the user Drop call in `drop_catching_panic`. The
original implementation used `std::panic::catch_unwind`.

## What went wrong

CI (macOS arm64, `R CMD check --as-cran`) produced:

```
 Fatal error: recursive gc invocation
```

during `test-dataframe.R :: "DataFrameRow align works with enum variants and tag column"`.
The test creates `EventRow` enum variants (no custom Drop, no R API calls),
so the objects are finalised by R's GC. The crash is not reproducible
locally — it only manifests on CI where timing and execution order differ.

## Root cause

`std::panic::catch_unwind` lazily initialises LLVM exception-handling
state on the **first call**. That initialisation allocates memory.

Inside R's GC finalizer pass, allocation re-enters the GC allocator and
triggers R's "recursive gc invocation" fatal error.

In local runs, `catch_unwind` is typically called earlier (e.g. by ALTREP
callbacks via `guarded_altrep_call`) before any finalizer runs, so the
lazy init already happened. In CI the finalizer can be the **first**
context to call `catch_unwind` (different execution order, colder state),
triggering the lazy init inside a GC pass.

## Fix

Replace `catch_unwind` with an `AbortIfUnwinding` RAII drop-guard:

```rust
#[must_use]
struct AbortIfUnwinding;

impl Drop for AbortIfUnwinding {
    #[cold]
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("miniextendr: destructor panicked during R finalization; aborting");
            std::process::abort();
        }
    }
}

pub fn drop_catching_panic<F: FnOnce()>(f: F) {
    let _guard = AbortIfUnwinding;
    f();
    // guard dropped here with panicking() == false → no-op
}
```

`std::thread::panicking()` is a cheap TLS read — it does NOT initialise
any exception-handling machinery and cannot allocate. On the happy path
the guard is a zero-cost no-op. On the panic path the guard fires
`process::abort()` before the panic can cross the C-ABI boundary into R.

The soundness guarantee (panics in Drop cannot unwind into R) is fully
preserved. Committed in `287f57c2` on `fix/finalizer-panic-safety`.
