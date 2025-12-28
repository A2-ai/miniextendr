//! Tests for RThreadBuilder and thread safety.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::thread::RThreadBuilder;
use miniextendr_api::{miniextendr, miniextendr_module};

/// Test RThreadBuilder: spawn with large stack (16 MiB) and call _unchecked R APIs.
/// Works WITHOUT nonapi feature by using large stacks to satisfy R's stack checking.
///
/// # Safety
/// Caller must ensure this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// @title Thread Builder Tests
/// @name rpkg_thread_builder
/// @keywords internal
/// @description Thread builder and lean-stack tests
/// @examples
/// \dontrun{
/// unsafe_C_test_r_thread_builder()
/// unsafe_C_test_r_thread_builder_spawn_join()
/// unsafe_C_test_spawn_with_r_lean_stack()
/// unsafe_C_test_stack_check_guard_lean()
/// }
/// @aliases unsafe_C_test_r_thread_builder unsafe_C_test_r_thread_builder_spawn_join
///   unsafe_C_test_spawn_with_r_lean_stack unsafe_C_test_stack_check_guard_lean
pub unsafe extern "C-unwind" fn C_test_r_thread_builder() -> SEXP {
    let handle = RThreadBuilder::new()
        .stack_size(16 * 1024 * 1024) // 16 MiB
        .name("test-r-worker".to_string())
        .spawn(|| {
            // Use _unchecked APIs (no stack check)
            let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(123) };
            unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
        })
        .expect("failed to spawn thread");

    let result = handle.join().expect("thread panicked");
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Test RThreadBuilder::spawn_join convenience method.
/// Works WITHOUT nonapi by using large stacks.
///
/// # Safety
/// Caller must ensure this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_test_r_thread_builder_spawn_join() -> SEXP {
    let result = RThreadBuilder::new()
        .spawn_join(|| {
            let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(456) };
            unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
        })
        .expect("thread failed");

    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

miniextendr_module! {
    mod thread_tests;

    // Thread safety tests - RThreadBuilder (always available, large stacks)
    extern "C-unwind" fn C_test_r_thread_builder;
    extern "C-unwind" fn C_test_r_thread_builder_spawn_join;
}
