use miniextendr_api::ffi::{SEXP, SexpExt};
use miniextendr_api::miniextendr;
use miniextendr_api::thread::{StackCheckGuard, spawn_with_r};

/// Non-API Thread Tests
///
/// Test spawn_with_r with lean stack (8 MiB) enabled by StackCheckGuard.
#[miniextendr]
/// @name rpkg_nonapi
/// @keywords internal
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
        let sexp = unsafe { crate::raw_ffi::Rf_ScalarInteger(999) };
        sexp.as_integer().unwrap()
    })
    .expect("failed to spawn");

    let result = handle.join().expect("thread panicked");
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// Test StackCheckGuard with Rust's default 2 MiB stack.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_test_stack_check_guard_lean() -> SEXP {
    let handle = std::thread::spawn(|| {
        let _guard = StackCheckGuard::disable();
        let sexp = unsafe { crate::raw_ffi::Rf_ScalarInteger(777) };
        sexp.as_integer().unwrap()
    });

    let result = handle.join().expect("thread panicked");
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}
