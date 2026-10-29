use super::*;
use std::sync::Mutex;

// These tests mutate process-global state (`R_CStackLimit`, `STACK_GUARD_COUNT`,
// `ORIGINAL_STACK_LIMIT`), so they must not run concurrently. Serialize them.
static STACK_TEST_LOCK: Mutex<()> = Mutex::new(());

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
