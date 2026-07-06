//! Tests for R interrupt checking.

use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::unwind_protect::with_r_unwind_protect_or_raise;

/// Test R_CheckUserInterrupt after a 2-second sleep.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_check_interupt_after() -> SEXP {
    use miniextendr_api::sys::R_CheckUserInterrupt;

    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        R_CheckUserInterrupt();
        SEXP::nil()
    }
}

/// Test R_CheckUserInterrupt inside with_r_unwind_protect_or_raise after a 2-second sleep.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_check_interupt_unwind() -> SEXP {
    use miniextendr_api::sys::R_CheckUserInterrupt;

    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        with_r_unwind_protect_or_raise(
            || {
                R_CheckUserInterrupt();
                SEXP::nil()
            },
            None,
        );
        SEXP::nil()
    }
}
