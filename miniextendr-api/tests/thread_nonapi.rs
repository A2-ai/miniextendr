//! Feature-gated tests for nonapi thread utilities.

#![cfg(feature = "nonapi")]

mod r_test_utils;

use miniextendr_api::thread::{
    StackCheckGuard, get_stack_config, is_stack_checking_disabled, with_stack_checking_disabled,
};

#[test]
fn nonapi_thread_suite() {
    r_test_utils::with_r_thread(|| {
        let (start, limit, dir) = get_stack_config();
        assert!(dir == -1 || dir == 1);
        assert_ne!(start, 0);
        assert_ne!(limit, usize::MAX);

        assert!(!is_stack_checking_disabled());
        {
            let _guard = StackCheckGuard::disable();
            assert!(is_stack_checking_disabled());
            assert!(StackCheckGuard::active_count() >= 1);
        }
        assert!(!is_stack_checking_disabled());

        let value = with_stack_checking_disabled(|| 123);
        assert_eq!(value, 123);
    });
}
