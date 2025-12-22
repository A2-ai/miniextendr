//! Feature-gated tests for nonapi thread utilities.

#![cfg(feature = "nonapi")]

use miniextendr_api::thread::{
    get_stack_config, is_stack_checking_disabled, with_stack_checking_disabled, StackCheckGuard,
};
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize_r() {
    INIT.call_once(|| unsafe {
        let engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        // Initialize in same order as rpkg/src/entrypoint.c.in
        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::worker::miniextendr_worker_init();
        assert!(
            miniextendr_engine::r_initialized_sentinel(),
            "Rf_initialize_R did not set C stack sentinels"
        );
        std::mem::forget(engine);
    });
}

#[test]
fn nonapi_thread_suite() {
    initialize_r();

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
}
