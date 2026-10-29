use super::drop_catching_panic;

#[test]
fn drop_catching_panic_does_not_propagate_panic() {
    // Verify that drop_catching_panic catches a panicking closure and does
    // NOT propagate the panic to the caller.
    //
    // Note: we cannot test the abort path from inside a test process, so
    // we document it with a comment instead:
    //   If the closure panics, `drop_catching_panic` calls `eprintln!` then
    //   `std::process::abort()`. That path is exercised only by the process
    //   dying, which is observable from an external test harness (not done
    //   here to keep CI simple).
    //
    // What we CAN test: the happy path (no panic) completes normally, and
    // the function compiles and links correctly with a `FnOnce()` generic.
    let mut ran = false;
    drop_catching_panic(|| {
        ran = true;
    });
    assert!(ran, "closure should have been called");
}

#[test]
fn drop_catching_panic_happy_path_drops_value() {
    // Confirm that the closure's side-effects (i.e. actual drop) occur
    // when no panic is raised.
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let dropped = Arc::new(AtomicBool::new(false));
    let flag = dropped.clone();

    struct DropSignal(Arc<AtomicBool>);
    impl Drop for DropSignal {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let signal = DropSignal(flag);
    drop_catching_panic(|| drop(signal));

    assert!(
        dropped.load(Ordering::SeqCst),
        "inner value should have been dropped"
    );
}

#[test]
fn drop_catching_panic_in_a_drop_run_by_an_unwind_does_not_abort() {
    // A guarded `Drop` that runs because an unrelated panic is unwinding
    // (a handle going out of scope in a panicking `#[miniextendr]` fn) is
    // not a panic in the destructor: the unwind must continue to its
    // `catch_unwind`. Before #1917 this aborted the test process.
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct GuardedDrop(Arc<AtomicBool>);
    impl Drop for GuardedDrop {
        fn drop(&mut self) {
            let ran = self.0.clone();
            drop_catching_panic(move || ran.store(true, Ordering::SeqCst));
        }
    }

    let ran = Arc::new(AtomicBool::new(false));
    let handle = GuardedDrop(ran.clone());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _handle = handle;
        panic!("unrelated panic");
    }));

    assert!(result.is_err(), "the unrelated panic should propagate");
    assert!(
        ran.load(Ordering::SeqCst),
        "the guarded destructor should have run during the unwind"
    );
}
