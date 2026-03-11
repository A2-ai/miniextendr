use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;
use miniextendr_api::thread::{StackCheckGuard, spawn_with_r};

/// Test spawn_with_r with lean stack (8 MiB) enabled by StackCheckGuard.
#[miniextendr]
/// @title Non-API Thread Tests
/// @name rpkg_nonapi
/// @keywords internal
/// @description Non-API thread tests (requires nonapi feature).
/// @examples
/// \dontrun{
/// unsafe_C_test_spawn_with_r_lean_stack()
/// unsafe_C_test_stack_check_guard_lean()
/// }
/// @aliases unsafe_C_test_spawn_with_r_lean_stack unsafe_C_test_stack_check_guard_lean
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_test_spawn_with_r_lean_stack() -> SEXP {
    let handle = spawn_with_r(|| {
        let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(999) };
        unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
    })
    .expect("failed to spawn");

    let result = handle.join().expect("thread panicked");
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Test StackCheckGuard with Rust's default 2 MiB stack.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_test_stack_check_guard_lean() -> SEXP {
    let handle = std::thread::spawn(|| {
        let _guard = StackCheckGuard::disable();
        let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(777) };
        unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
    });

    let result = handle.join().expect("thread panicked");
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}
