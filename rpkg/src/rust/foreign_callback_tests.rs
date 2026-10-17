//! A foreign C frame must return before a caught R exit is resumed (#1946).

use miniextendr_api::{OwnedProtect, Quoted, SEXP, expression::call_unwinding, miniextendr};
use std::{
    any::Any,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::{AtomicI32, Ordering},
};

static CALLBACK_DROPS: AtomicI32 = AtomicI32::new(0);
static BODY_DROPS: AtomicI32 = AtomicI32::new(0);
static DRIVER_RETURNS: AtomicI32 = AtomicI32::new(0);
static C_CLEANUPS: AtomicI32 = AtomicI32::new(0);

struct Dropped(&'static AtomicI32);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

struct State {
    expr: SEXP,
    env: SEXP,
    exit: Option<Box<dyn Any + Send>>,
}

// This public R C API supplies a real foreign stack frame for the test.
// The callbacks return normally, so neither a Rust unwind nor an R longjmp
// may cross this plain C ABI. No binding is added to the runtime's public API.
unsafe extern "C" {
    #[link_name = "R_ExecWithCleanup"]
    fn foreign_driver(
        callback: unsafe extern "C" fn(*mut c_void) -> SEXP,
        data: *mut c_void,
        cleanup: unsafe extern "C" fn(*mut c_void),
        cleanup_data: *mut c_void,
    ) -> SEXP;
}

unsafe extern "C-unwind" fn evaluate(data: *mut c_void) -> SEXP {
    let state = unsafe { &*data.cast::<State>() };
    // No destructors in this operation; the callback owns its Drop guard.
    unsafe { miniextendr_api::sys::Rf_eval(state.expr, state.env) }
}

unsafe extern "C" fn callback(data: *mut c_void) -> SEXP {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _drop = Dropped(&CALLBACK_DROPS);
        unsafe { call_unwinding(evaluate, data) }
    }));
    match result {
        Ok(value) => value,
        Err(payload) => {
            unsafe { (*data.cast::<State>()).exit = Some(payload) };
            SEXP::nil()
        }
    }
}

unsafe extern "C" fn cleanup(_: *mut c_void) {
    C_CLEANUPS.fetch_add(1, Ordering::Relaxed);
}

/// The caller roots expr/env and has entered a main-thread framework boundary.
pub(crate) unsafe fn roundtrip(expr: SEXP, env: SEXP) -> SEXP {
    let _drop = Dropped(&BODY_DROPS);
    let mut state = State {
        expr,
        env,
        exit: None,
    };
    let value = unsafe {
        foreign_driver(
            callback,
            (&raw mut state).cast(),
            cleanup,
            std::ptr::null_mut(),
        )
    };
    DRIVER_RETURNS.fetch_add(1, Ordering::Relaxed);
    let value = unsafe { OwnedProtect::new(value) };
    // The exit's continuation must keep its condition/restart value alive
    // while foreign code returns and the Rust adapter prepares to resume.
    unsafe { miniextendr_api::sys::R_gc() };
    if let Some(payload) = state.exit.take() {
        std::panic::resume_unwind(payload);
    }
    value.get()
}

/// Carry an R exit through a foreign callback without unwinding through C.
/// @noRd
#[miniextendr(noexport, no_worker)]
pub fn foreign_callback_roundtrip(expr: Quoted<'_>) -> SEXP {
    unsafe { roundtrip(expr.expr(), expr.env()) }
}

/// Observe Rust destructors and normal return through the foreign C frame.
/// @noRd
#[miniextendr(noexport, no_worker)]
pub fn foreign_callback_counts() -> Vec<i32> {
    [&CALLBACK_DROPS, &BODY_DROPS, &DRIVER_RETURNS, &C_CLEANUPS]
        .map(|count| count.load(Ordering::Relaxed))
        .to_vec()
}
