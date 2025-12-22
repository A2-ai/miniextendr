//! Tests for R interrupt checking.

use miniextendr_api::ffi::{R_NilValue, SEXP};
use miniextendr_api::unwind_protect::with_r_unwind_protect;
use miniextendr_api::{miniextendr, miniextendr_module};

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_check_interupt_after() -> SEXP {
    use miniextendr_api::ffi::R_CheckUserInterrupt;

    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        R_CheckUserInterrupt();
        R_NilValue
    }
}

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_check_interupt_unwind() -> SEXP {
    use miniextendr_api::ffi::R_CheckUserInterrupt;

    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        with_r_unwind_protect(
            || {
                R_CheckUserInterrupt();
                R_NilValue
            },
            None,
        );
        R_NilValue
    }
}

miniextendr_module! {
    mod interrupt_tests;

    extern fn C_check_interupt_after;
    extern fn C_check_interupt_unwind;
}
