# Thread Safety in miniextendr

This document explains how to safely call R APIs from threads other than the main R thread.

## The Problem

R's API is designed to be called from a single thread - the main R thread. When you spawn a new thread and try to call R functions, you'll get a segfault. This happens because of R's **stack checking mechanism**.

### How R's Stack Checking Works

R tracks three global variables (defined in `Rinterface.h`, all non-API):

```c
uintptr_t R_CStackStart;  // Top of the main thread's stack
uintptr_t R_CStackLimit;  // Stack size limit
int R_CStackDir;          // Stack growth direction (-1 = down, 1 = up)
```

During initialization (`Rf_initialize_R`), R sets these based on the main thread's stack:

- **Unix**: Uses `getrlimit(RLIMIT_STACK)`, `__libc_stack_end`, or `KERN_USRSTACK`
- **Windows**: Uses `VirtualQuery` to determine stack bounds

Many R API functions call `R_CheckStack()`:

```c
void R_CheckStack(void) {
    int dummy;
    intptr_t usage = R_CStackDir * (R_CStackStart - (uintptr_t)&dummy);

    if (R_CStackLimit != -1 && usage > ((intptr_t) R_CStackLimit))
        R_SignalCStackOverflow(usage);
}
```

When called from a **different thread**, `&dummy` points to a completely different stack, causing:

- `usage` to be a huge negative or positive number
- False stack overflow detection
- Segfault or abort

## The Solution

Setting `R_CStackLimit` to `(uintptr_t)-1` (i.e., `usize::MAX`) **disables stack checking entirely**.

From R source (`src/include/Defn.h`):

```c
if(R_CStackLimit != (uintptr_t)(-1) && usage > ((intptr_t) R_CStackLimit))
```

This is safe because:

1. The OS still enforces real stack limits
2. R functions correctly, just without its own overflow detection

## Using miniextendr's Thread Utilities

All thread utilities require the `nonapi` feature since they access non-API R internals.

```toml
[dependencies]
miniextendr-api = { version = "...", features = ["nonapi"] }
```

### Checked vs Unchecked R FFI

Most `miniextendr_api::ffi::*` functions are **thread-checked** (via `#[r_ffi_checked]`) when
`cfg(debug_assertions)` is enabled. Note: this repo’s workspace sets `debug-assertions = true`
for the release profile, so don’t rely on `--release` to disable these checks.

When you intentionally call R from a non-main thread using this module, use the `*_unchecked`
variants.

### Simple Spawning: `spawn_with_r`

```rust
use miniextendr_api::spawn_with_r;

let handle = spawn_with_r(|| {
    // Safe to call R APIs here!
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42) }
})?;

let result = handle.join().unwrap();
```

This function:

1. Sets stack size to 8 MiB (configurable)
2. Automatically disables R's stack checking
3. Restores stack checking when the thread completes

### Custom Configuration: `RThreadBuilder`

```rust
use miniextendr_api::{RThreadBuilder, WINDOWS_R_STACK_SIZE};

let handle = RThreadBuilder::new()
    .stack_size(WINDOWS_R_STACK_SIZE)  // 64 MiB for heavy workloads
    .name("r-worker".to_string())
    .spawn(|| {
        // R API calls safe here
    })?;
```

### Scoped Threads: `scope_with_r`

For borrowing from the enclosing scope:

```rust
use miniextendr_api::scope_with_r;

let data = vec![1, 2, 3];

std::thread::scope(|s| {
    scope_with_r(s, |_| {
        // Can borrow `data` here!
        println!("len: {}", data.len());
        // R API calls also safe
    });
});
```

Note: Scoped threads use Rust's default stack size (2 MiB). For larger stacks, use `spawn_with_r`.

### Manual Control: `StackCheckGuard`

For existing threads or fine-grained control:

```rust
use miniextendr_api::StackCheckGuard;

std::thread::spawn(|| {
    let _guard = StackCheckGuard::disable();

    // R API calls safe while guard is alive
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42) };

    // Original limit restored when _guard drops
});
```

### One-Time Disable: `with_stack_checking_disabled`

```rust
use miniextendr_api::with_stack_checking_disabled;

let result = with_stack_checking_disabled(|| {
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42) }
});
```

## Stack Size Requirements

### Automatic Configuration

When you enable the `nonapi` feature, miniextendr-api's `build.rs` automatically sets linker flags to configure an 8 MiB stack for binaries, tests, examples, and cdylib crates:

| Platform | Linker Flag |
|----------|-------------|
| Windows MSVC | `/STACK:8388608` |
| Windows GNU | `-Wl,--stack,8388608` |
| macOS | `-Wl,-stack_size,800000` |
| Linux/BSD | `-Wl,-z,stack-size=8388608` |

To override (e.g., for Windows R's 64 MiB), add to `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=/STACK:67108864"]  # 64 MiB
```

### Platform Defaults

R doesn't enforce a specific stack size - it uses whatever the OS provides:

| Platform | Default Stack Size | Source |
|----------|-------------------|--------|
| Linux | ~8 MiB | `ulimit -s` / `getrlimit(RLIMIT_STACK)` |
| macOS | ~8 MiB | `sysctl KERN_USRSTACK` |
| Windows | **64 MiB** | Linker flag (since R 4.2) |

Rust's default thread stack is only **2 MiB**, which may be insufficient for:

- Deep recursion (`lapply` chains, recursive functions)
- Complex formulas
- Large `tryCatch` stacks

### Available Constants

```rust
// Requires feature = "nonapi"

/// 8 MiB - conservative default, matches Unix R
pub const DEFAULT_R_STACK_SIZE: usize = 8 * 1024 * 1024;

/// 64 MiB - matches Windows R for heavy workloads
/// Only available on Windows (gated with #[cfg(windows)])
pub const WINDOWS_R_STACK_SIZE: usize = 64 * 1024 * 1024;
```

## Important Caveats

### R is Still Single-Threaded

Disabling stack checking allows **calling** R from other threads, but R itself is **not thread-safe**. You must ensure:

1. **No concurrent R API calls** - Use mutexes or channels to serialize access
2. **GC safety** - R's garbage collector is not thread-aware
3. **Global state** - R has extensive global state that isn't synchronized

### Recommended Pattern

Use worker threads for Rust computation, marshal R calls to main thread:

```rust
use std::sync::mpsc;

// Channel for R results
let (tx, rx) = mpsc::channel();

// Worker thread does Rust computation
spawn_with_r(move || {
    let rust_result = expensive_rust_computation();

    // Convert to R on this thread (with guard)
    let r_result = unsafe { rust_result.into_sexp() };

    tx.send(r_result).unwrap();
});

// Main thread receives R object
let sexp = rx.recv().unwrap();
```

### ALTREP Callbacks

ALTREP methods are called by R on the main thread, so they don't need `StackCheckGuard`. However, if an ALTREP method spawns threads that call back into R, those threads need the guard.

## Non-API Functions Used

These are gated behind `feature = "nonapi"` and may break with R updates:

| Symbol | Purpose |
|--------|---------|
| `R_CStackStart` | Stack top address |
| `R_CStackLimit` | Stack limit (set to `usize::MAX` to disable) |
| `R_CStackDir` | Stack growth direction |

See [NONAPI.md](NONAPI.md) for the full tracking document.

## References

- R source: `src/main/errors.c` - `R_CheckStack()` implementation
- R source: `src/unix/system.c` - Unix stack initialization
- R source: `src/gnuwin32/system.c` - Windows stack initialization
- R NEWS: "On Windows, the C stack size has been increased to 64MB" (R 4.2)
