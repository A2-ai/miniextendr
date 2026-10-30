# Thread Safety in miniextendr

This document explains miniextendr's main-thread contract and the limits of
its legacy non-API stack controls.

## Non-negotiable rule: R API calls stay on R's main thread

R's API, global state, garbage collector, and error signaling are designed for
the main R thread. Calling the API from an arbitrary `std::thread` or Rayon
worker can corrupt memory or segfault even when calls appear serialized.

Writing R Extensions explicitly requires R API entry points to be called from
the main thread. It also says packages must not change R's stack-check globals
to call stack-checking internals on a secondary thread.

## Stack checking is one failure mode, not a safety boundary

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

When called from a different thread, `&dummy` points to a completely different stack, causing:

- `usage` to be a huge negative or positive number
- False stack overflow detection
- Segfault or abort

Setting `R_CStackLimit` to `(uintptr_t)-1` disables this check, but only this
check. It does not make R's allocator, GC, global state, external libraries, or
error longjmps thread-safe. The OS still enforces its real stack limit.

## Where generated code runs

Generated functions and methods run on R's main thread inside
`R_UnwindProtect`. A Rust panic becomes an R condition; an R error unwinds
through the Rust frames, running their destructors.

Code on other threads (Rayon, `std::thread`) stays pure Rust: collect its
results, return to the main thread, and convert them to R objects there. The
Rayon bridge (`with_r_vec`, `par_collect_r`, ...) does exactly this: it
allocates the R vector on the main thread, fills its data pointer in
parallel, and returns. See [RAYON.md](RAYON.md).

`with_r_thread(f)` checks that it runs on R's main thread and runs `f`
inline; anywhere else it panics before `f` reaches R. `is_r_main_thread()`
answers the question without panicking. Neither moves work between threads.

### Checked vs Unchecked R FFI

Most `miniextendr_api::sys::*` functions are **checked** (via `#[r_ffi_checked]`).
They run the call inside `with_r_thread`: directly on the main thread, and a
panic on any other thread.

Use `*_unchecked` only when the surrounding context has already established
that execution is on R's main thread, such as an ALTREP callback, an
`R_UnwindProtect` body, or a `with_r_thread` closure. They are never an escape
hatch for calling R from another thread.

### Legacy non-API stack controls

The `nonapi` feature currently exposes `StackCheckGuard`, `spawn_with_r`,
`scope_with_r`, and `with_stack_checking_disabled`; `RThreadBuilder` is always
available for stack sizing. These APIs can alter R's process-global stack
check, but they do not satisfy R's other threading invariants and must not be
used by packages for off-main R calls. Their removal or relocation to a
narrowly contracted embedded-R surface is tracked in #1352.

## Stack Size Requirements

### Automatic Configuration

`miniextendr-api`'s `build.rs` emits linker flags for an 8 MiB stack for its
binaries, tests, examples, and cdylib targets. This happens independently of
the `nonapi` feature and does not change the R API main-thread rule:

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

Rust's default spawned-thread stack is commonly **2 MiB**, which may be
insufficient for a stack-heavy pure-Rust workload. The comparison explains the
legacy sizing constants; it does not make R evaluation on those threads
supported.

### Available Constants

```rust
/// 8 MiB - conservative default, matches Unix R
/// (Always available, no feature gate required)
pub const DEFAULT_R_STACK_SIZE: usize = 8 * 1024 * 1024;

/// 64 MiB - matches Windows R for heavy workloads
/// Only available on Windows (#[cfg(windows)])
pub const WINDOWS_R_STACK_SIZE: usize = 64 * 1024 * 1024;
```

## Main-thread invariant

Serializing arbitrary spawned-thread calls with a mutex is not sufficient: R
requires the calls themselves, its GC interactions, and error signaling to run
on the main thread.

The supported pattern is pure Rust computation on other threads, joined
before the function returns, with every R API call on the main thread:

```rust
use miniextendr_api::miniextendr;
use rayon::prelude::*;

#[miniextendr]
fn normalized(x: Vec<f64>) -> Vec<f64> {
    let total: f64 = x.par_iter().sum();
    // Return conversion runs on the main thread, at the framework boundary.
    x.par_iter().map(|v| v / total).collect()
}
```

### ALTREP Callbacks

ALTREP methods are called by R on the main thread, so they do not need
`StackCheckGuard`. If an ALTREP method spawns threads, keep those closures
Rust-only, join them, and perform any R API work back in the original callback.

### ALTREP and Thread Safety

When R passes an ALTREP vector (e.g., `1:10`) to a `#[miniextendr]` function,
miniextendr auto-materializes it on the R main thread before the function body
runs. This ensures the data pointer is stable before any SEXP could cross a
thread boundary.

For explicit ALTREP handling, use `AltrepSexp`, a `!Send + !Sync` wrapper
that prevents un-materialized ALTREP vectors from reaching rayon or other
threads at compile time.

See [Receiving ALTREP from R](ALTREP_SEXP.md) for the full guide.

## Unload

`R_unload_<pkg>` (generated by `miniextendr_init!`) calls
`miniextendr_runtime_shutdown()` before `library.dynam.unload` unmaps the
package DLL.

### Why the panic hook must be uninstalled

`miniextendr_panic_hook()` installs a process-global panic hook via
`std::panic::set_hook`. The hook closure is compiled code that lives in the
DLL. After unload, the process-global hook still points at that code. The next
panic from **any** crate (not just miniextendr) would jump to unmapped memory.

`miniextendr_runtime_shutdown` calls `miniextendr_panic_hook_uninstall()`,
which calls `std::panic::take_hook()`: that drops our closure and resets the
process slot to Rust's default hook. On Windows the unmap happens as soon as
the unload hook returns, and a dangling hook fails the next panic with
`fatal runtime error: failed to initiate panic, error 5, aborting`.

### API surface

`miniextendr_runtime_shutdown()`: `extern "C-unwind" fn` with `#[unsafe(no_mangle)]`,
called from the generated `R_unload_<pkg>` hook. Idempotent.

`miniextendr_runtime_init()`: counterpart init, called from `R_init_<pkg>`.
Records R's main thread ID. It does **not** register a libc `atexit` handler:
an `atexit` function pointer into the DLL has the same unmap hazard as the
panic hook.

Source: `miniextendr-api/src/thread.rs` (init and shutdown),
`miniextendr-api/src/backtrace.rs` (panic hook install/uninstall).

## Non-API Functions Used

These are gated behind `feature = "nonapi"` and may break with R updates:

| Symbol | Purpose |
|--------|---------|
| `R_CStackStart` | Stack top address |
| `R_CStackLimit` | Stack limit (set to `usize::MAX` to disable) |
| `R_CStackDir` | Stack growth direction |

See [NONAPI.md](NONAPI.md) for the full tracking document.

## Known Limitations

- **Async/await is not supported.** R's C API is single-threaded and synchronous; use blocking I/O or R-level parallelism (mirai, callr). See [GAPS.md](GAPS.md#43-asyncawait-support).
- **Spawned-thread panics cannot be propagated** through `extern "C-unwind"` functions. Handle thread errors explicitly via `Result` rather than `resume_unwind`. See [GAPS.md](GAPS.md#56-thread-panic-propagation-limitation).
- **Debug-only SEXP thread assertions** mean release builds may not detect SEXP access from wrong threads. Checked FFI wrappers provide runtime checks in all build modes.

See [GAPS.md](GAPS.md) for the full catalog of known limitations.

---

## See Also

- [SAFETY.md](SAFETY.md) -- Safety invariants and the main-thread model
- [ERROR_HANDLING.md](ERROR_HANDLING.md) -- Panic handling and R error propagation
- [FEATURES.md](FEATURES.md#nonapi) -- The `nonapi` feature flag and legacy stack controls
- [RAYON.md](RAYON.md) -- Parallel iteration with Rayon

## References

- Writing R Extensions: "OpenMP support" and "Embedding R in other applications — Threading issues"
- R source: `src/main/errors.c` - `R_CheckStack()` implementation
- R source: `src/unix/system.c` - Unix stack initialization
- R source: `src/gnuwin32/system.c` - Windows stack initialization
- R NEWS: "On Windows, the C stack size has been increased to 64MB" (R 4.2)
