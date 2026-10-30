//! Benchmarks for panic telemetry overhead.
//!
//! `panic_telemetry::fire()` reads a thread-local hook slot at every
//! panic→R-error boundary (altrep_bridge, unwind_protect, connection) and, with
//! a hook set, clones its `Rc`. This benchmark measures that pattern to confirm
//! it's negligible.
//!
//! Since `fire()` is `pub(crate)`, we benchmark the identical slot pattern
//! synthetically, plus the public `set`/`clear` API. Each benchmark sets and
//! fires on its own thread, as the runtime does on R's main thread.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use miniextendr_api::panic_telemetry;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Synthetic slot matching the exact pattern in fire()

type Hook = Rc<dyn Fn(&str)>;

std::thread_local! {
    static BENCH_HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

/// Mirrors `panic_telemetry::fire()` exactly: borrow the slot, clone the
/// `Rc`, release the borrow, invoke if present.
#[inline(never)]
fn synthetic_fire(msg: &str) {
    let hook = BENCH_HOOK.with_borrow(Option::clone);
    if let Some(hook) = hook {
        let _ = catch_unwind(AssertUnwindSafe(|| hook(msg)));
    }
}
// endregion

// region: Group 1: Slot read cost (the overhead at panic boundaries)

mod read_slot {
    use super::*;

    /// No hook installed (the slot is `None`): borrow, find None, release.
    #[divan::bench]
    fn no_hook() {
        BENCH_HOOK.set(None);
        synthetic_fire("bench");
    }

    /// A hook is installed: clone the `Rc` (refcount bump) and invoke it.
    #[divan::bench]
    fn with_hook_noop() {
        BENCH_HOOK.set(Some(Rc::new(|_msg: &str| {})));
        synthetic_fire("bench");
    }

    /// Clone + invocation with a hook that does minimal work (writes to a
    /// black-boxed variable).
    #[divan::bench]
    fn with_hook_minimal_work() {
        BENCH_HOOK.set(Some(Rc::new(|msg: &str| {
            divan::black_box(msg.len());
        })));
        synthetic_fire("bench panic message");
    }

    /// Bare slot borrow (no Option check, no `Rc` clone) for reference.
    #[divan::bench]
    fn bare_slot_borrow() {
        BENCH_HOOK.with_borrow(|hook| {
            divan::black_box(hook);
        });
    }
}
// endregion

// region: Group 2: Set/clear cost (infrequent operation)

mod set_clear {
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
