use super::*;
#[cfg(feature = "nonapi")]
use std::sync::Mutex;

// region: R's main thread

#[test]
fn with_r_thread_panics_before_init() {
    // If another test already called miniextendr_runtime_init, we can't test
    // the pre-init path. Verify at least that it panics from the wrong thread.
    if R_MAIN_THREAD_ID.get().is_some() {
        let handle = std::thread::spawn(|| std::panic::catch_unwind(|| with_r_thread(|| 42)));
        let result = handle.join().expect("thread panicked outside catch_unwind");
        assert!(
            result.is_err(),
            "with_r_thread should panic from non-main thread"
        );
        return;
    }
    let result = std::panic::catch_unwind(|| {
        with_r_thread(|| 42);
    });
    assert!(result.is_err());
    let payload = result.unwrap_err();
    let msg = crate::unwind_protect::panic_payload_to_string(payload.as_ref());
    assert!(
        msg.contains("miniextendr_runtime_init"),
        "expected init error message, got: {msg}"
    );
}

#[test]
fn with_r_thread_runs_inline_on_the_main_thread() {
    miniextendr_runtime_init();
    // If another parallel test already set R_MAIN_THREAD_ID to a different
    // thread (OnceLock), we aren't "main" and with_r_thread rightfully
    // panics. Skip in that case.
    if !is_r_main_thread() {
        return;
    }
    assert_eq!(with_r_thread(|| 42), 42);
}

#[test]
fn with_r_thread_panics_on_another_thread() {
    miniextendr_runtime_init();

    let handle = std::thread::spawn(|| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| with_r_thread(|| 42)))
    });

    let payload = handle
        .join()
        .expect("thread panicked outside catch_unwind")
        .expect_err("with_r_thread should panic when called from a non-main thread");
    let msg = crate::unwind_protect::panic_payload_to_string(payload.as_ref());
    assert!(
        msg.contains("other than R's main thread"),
        "unexpected message: {msg}"
    );
}
// endregion

// region: C-stack bounds (nonapi)

// These tests mutate process-global state (`R_CStackLimit`, `STACK_GUARD_COUNT`,
// `ORIGINAL_STACK_LIMIT`), so they must not run concurrently. Serialize them.
#[cfg(feature = "nonapi")]
static STACK_TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(feature = "nonapi")]
#[test]
fn test_guard_saves_and_restores() {
    let _serial = STACK_TEST_LOCK.lock().unwrap();
    let original = get_r_cstack_limit();

    {
        let _guard = StackCheckGuard::disable();
        // The original limit is saved in ORIGINAL_STACK_LIMIT
        assert_eq!(ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst), original);
        assert!(is_stack_checking_disabled());
    }

    // After guard drops, should be restored
    assert_eq!(get_r_cstack_limit(), original);
}

/// Regression test for the "permanent disable silently undone by a guard drop" bug.
///
/// Ordering (a): a `StackCheckGuard` is active (so `ORIGINAL_STACK_LIMIT` holds the
/// real pre-disable limit) when `disable_stack_checking_permanently()` is called.
/// Before the fix, the guard's `Drop` restored that saved limit, re-enabling stack
/// checking and undoing the "permanent" disable. After the fix,
/// `disable_stack_checking_permanently()` also pins `ORIGINAL_STACK_LIMIT` to
/// `usize::MAX`, so the drop's restore is a no-op and the limit stays at `usize::MAX`.
#[cfg(feature = "nonapi")]
#[test]
fn test_permanent_disable_survives_guard_drop() {
    let _serial = STACK_TEST_LOCK.lock().unwrap();
    let original = get_r_cstack_limit();

    {
        // Guard active: ORIGINAL_STACK_LIMIT now holds the real pre-disable limit.
        let _guard = StackCheckGuard::disable();
        assert_eq!(ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst), original);

        // Permanent disable while the guard is still alive.
        disable_stack_checking_permanently();
        // The saved limit is pinned to MAX so a later restore can't re-enable checking.
        assert_eq!(ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst), usize::MAX);
    }
    // Guard dropped: limit must remain disabled, not bounce back to `original`.
    assert!(is_stack_checking_disabled());
    assert_eq!(get_r_cstack_limit(), usize::MAX);

    // Restore the real limit so we don't leak the permanent disable into other tests.
    unsafe {
        set_r_cstack_limit(original);
    }
    ORIGINAL_STACK_LIMIT.store(0, Ordering::SeqCst);
}
// endregion
