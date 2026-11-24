//! `unwind_protect::call_unwinding` under a frame an unwind must not cross
//! (#1946): the pattern of an R callback that a C or Fortran library calls.

use std::any::Any;
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use miniextendr_api::unwind_protect::call_unwinding;
use miniextendr_api::{SEXP, miniextendr, sys};

/// What the callback reads and leaves for the caller.
struct Step {
    f: SEXP,
    value: SEXP,
    exit: Option<Box<dyn Any + Send>>,
}

/// Stands in for a C or Fortran solver. It is `extern "C"`, so an unwind
/// leaving it aborts the process: a passing test shows the R exit never
/// crossed it.
extern "C" fn foreign_solver(
    callback: extern "C" fn(*mut c_void) -> i32,
    data: *mut c_void,
) -> i32 {
    callback(data)
}

/// Runs `f()`, called from C by `call_unwinding`; it holds nothing to drop.
unsafe extern "C-unwind" fn call_f(data: *mut c_void) -> SEXP {
    let step = unsafe { &*data.cast::<Step>() };
    unsafe {
        let call = sys::Rf_protect(sys::Rf_lang1(step.f));
        let value = sys::Rf_eval(call, sys::R_GlobalEnv);
        sys::Rf_unprotect(1);
        value
    }
}

/// The solver's callback: an R exit is caught here, before the solver's
/// frame, and left in `Step` for the caller to resume.
extern "C" fn callback(data: *mut c_void) -> i32 {
    // SAFETY: R's main thread, inside `foreign_frame_call`'s boundary; `data`
    // is the caller's `Step`, which outlives the call.
    let outcome = catch_unwind(AssertUnwindSafe(|| unsafe { call_unwinding(call_f, data) }));
    let step = unsafe { &mut *data.cast::<Step>() };
    match outcome {
        Ok(value) => {
            step.value = value;
            0
        }
        Err(exit) => {
            step.exit = Some(exit);
            1
        }
    }
}

/// Call `f()` through a foreign frame: its value, or the R exit it took
/// (an error, the caller's `tryCatch()` exit, a restart), continued past the
/// frame.
/// @param f A function of no arguments.
#[miniextendr(noexport)]
pub fn foreign_frame_call(f: SEXP) -> SEXP {
    let mut step = Step {
        f,
        value: SEXP::nil(),
        exit: None,
    };
    let status = foreign_solver(callback, (&raw mut step).cast());
    if let Some(exit) = step.exit.take() {
        // Past the solver, in the same call: continue the R exit.
        resume_unwind(exit);
    }
    assert_eq!(status, 0, "the solver stopped without an exit");
    // Unprotected since `f()` returned; nothing has allocated since.
    step.value
}
