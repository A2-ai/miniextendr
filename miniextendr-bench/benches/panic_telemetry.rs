//! Benchmarks for panic telemetry overhead.
//!
//! `panic_telemetry::fire()` acquires a `RwLock` read lock at every panic→R-error
//! boundary (worker, altrep_bridge, unwind_protect, connection). This benchmark
//! measures the cost of that read lock pattern to confirm it's negligible.
//!
//! Since `fire()` is `pub(crate)`, we benchmark the identical RwLock pattern
//! synthetically, plus the public `set`/`clear` API for the write path.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, RwLock};

use miniextendr_api::panic_telemetry;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Synthetic RwLock matching the exact pattern in fire()

type Hook = Arc<dyn Fn(&str) + Send + Sync>;
static BENCH_HOOK: RwLock<Option<Hook>> = RwLock::new(None);

/// Mirrors `panic_telemetry::fire()` exactly: read lock, clone Arc, drop lock,
/// invoke if present.
#[inline(never)]
fn synthetic_fire(msg: &str) {
    let hook = {
        let guard = BENCH_HOOK.read().unwrap_or_else(|e| e.into_inner());
        guard.as_ref().cloned()
    };
    if let Some(hook) = hook {
        let _ = catch_unwind(AssertUnwindSafe(|| hook(msg)));
    }
}
// endregion

// region: Group 1: Read lock cost (the hot-path overhead at panic boundaries)

mod read_lock {
    use super::*;

    /// RwLock read when no hook is installed (Option is None).
    /// This is the fast path: acquire read lock, find None, release.
    #[divan::bench]
    fn no_hook() {
        // Ensure no hook
        {
            let mut g = BENCH_HOOK.write().unwrap();
            *g = None;
        }
        synthetic_fire("bench");
    }

    /// RwLock read when a hook IS installed but no panic is occurring.
    /// This clones the Arc (cheap refcount bump) and invokes the hook.
    #[divan::bench]
    fn with_hook_noop() {
        // Install trivial hook
        {
            let mut g = BENCH_HOOK.write().unwrap();
            *g = Some(Arc::new(|_msg: &str| {}));
        }
        synthetic_fire("bench");
    }

    /// RwLock read + Arc clone + hook invocation with a hook that does
    /// minimal work (writes to a black-boxed variable).
    #[divan::bench]
    fn with_hook_minimal_work() {
        {
            let mut g = BENCH_HOOK.write().unwrap();
            *g = Some(Arc::new(|msg: &str| {
                divan::black_box(msg.len());
            }));
        }
        synthetic_fire("bench panic message");
    }

    /// Bare RwLock read (no Option check, no Arc clone) for reference.
    #[divan::bench]
    fn bare_rwlock_read() {
        let guard = BENCH_HOOK.read().unwrap();
        divan::black_box(&*guard);
    }
}
// endregion

// region: Group 2: Write lock cost (set/clear hook — infrequent operation)

mod write_lock {
    use super::*;

    /// Install a new panic telemetry hook (public API).
    #[divan::bench]
    fn set_hook() {
        panic_telemetry::set_panic_telemetry_hook(|_report| {});
    }

    /// Clear the panic telemetry hook (public API).
    #[divan::bench]
    fn clear_hook() {
        panic_telemetry::clear_panic_telemetry_hook();
    }

    /// Set then immediately clear (full cycle).
    #[divan::bench]
    fn set_then_clear() {
        panic_telemetry::set_panic_telemetry_hook(|_report| {});
        panic_telemetry::clear_panic_telemetry_hook();
    }
}
// endregion
