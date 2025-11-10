//! Safe API for R's `R_UnwindProtect`
//!
//!
//!  
//!
use std::cell::LazyCell;

use crate::ffi;

thread_local! {
    static R_CONTINUATION_TOKEN: LazyCell<crate::ffi::SEXP> = LazyCell::new(|| unsafe {
        let token = crate::ffi::R_MakeUnwindCont();
        ffi::R_PreserveObject(token);
        token
    });
}

struct ClosureContext<FunClosure, CleanClosure> {
    fun: Option<FunClosure>,
    clean: Option<CleanClosure>,
}

unsafe extern "C" fn fun_tramp<F, C>(data: *mut std::ffi::c_void) -> ffi::SEXP
where
    F: FnOnce() -> ffi::SEXP,
{
    let ctx = unsafe { data.cast::<ClosureContext<F, C>>().as_mut().unwrap() };
    let f = ctx.fun.take().unwrap();
    f()
}

unsafe extern "C" fn clean_tramp<F, C>(data: *mut std::ffi::c_void, jump: ffi::Rboolean)
where
    C: FnOnce(bool),
{
    let mut closure_ctx = unsafe { Box::from_raw(data.cast::<ClosureContext<F, C>>()) };
    if let Some(clean_closure) = closure_ctx.as_mut().clean.take() {
        drop(closure_ctx);
        clean_closure(jump != ffi::Rboolean::FALSE);
    }
}

/// Wrap a Rust closure with `R_UnwindProtect`.
/// `clean` sees `true` if a non-local jump happened, `false` on normal return.
///
///
pub unsafe fn with_unwind_protect<FunClosure, CleanClosure>(
    fun: FunClosure,
    clean: CleanClosure,
) -> ffi::SEXP
where
    FunClosure: FnOnce() -> ffi::SEXP,
    CleanClosure: FnOnce(bool),
{
    let data = Box::into_raw(Box::new(ClosureContext {
        fun: Some(fun),
        clean: Some(clean),
    }));

    unsafe {
        ffi::R_UnwindProtect(
            Some(fun_tramp::<FunClosure, CleanClosure>),
            data.cast(),
            Some(clean_tramp::<FunClosure, CleanClosure>),
            data.cast(),
            R_CONTINUATION_TOKEN.with(|x| **x),
        )
    }
}
