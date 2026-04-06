//! Tests for RThreadBuilder and thread safety.

use miniextendr_api::ffi::{SEXP, SexpExt};
use miniextendr_api::miniextendr;
use miniextendr_api::thread::RThreadBuilder;

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// @name rpkg_thread_builder
/// @noRd
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
            // mxl::allow(MXL301)
            let sexp = unsafe { crate::raw_ffi::Rf_ScalarInteger(123) };
            sexp.as_integer().unwrap()
        })
        .expect("failed to spawn thread");

    let result = handle.join().expect("thread panicked");
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_test_r_thread_builder_spawn_join() -> SEXP {
    let result = RThreadBuilder::new()
        .spawn_join(|| {
            // mxl::allow(MXL301)
            let sexp = unsafe { crate::raw_ffi::Rf_ScalarInteger(456) };
            sexp.as_integer().unwrap()
        })
        .expect("thread failed");

    miniextendr_api::ffi::SEXP::scalar_integer(result)
}
