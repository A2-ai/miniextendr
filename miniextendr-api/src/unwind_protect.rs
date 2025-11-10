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

#[inline(never)]
pub fn continue_unwind() -> std::convert::Infallible {
    unsafe { ffi::R_ContinueUnwind(R_CONTINUATION_TOKEN.with(|x| **x)) }
}

struct ClosureContext<FunClosure, CleanClosure, LastClosure> {
    fun: Option<FunClosure>,
    clean: Option<CleanClosure>,
    last: Option<LastClosure>,
}

unsafe extern "C" fn fun_tramp<F, C, L>(data: *mut std::ffi::c_void) -> ffi::SEXP
where
    F: FnOnce() -> ffi::SEXP,
{
    let ctx = unsafe { data.cast::<ClosureContext<F, C, L>>().as_mut().unwrap() };
    let f = ctx.fun.take().unwrap();
    f()
}

unsafe extern "C" fn clean_tramp<F, C, L>(data: *mut std::ffi::c_void, jump: ffi::Rboolean)
where
    C: FnOnce(bool),
    L: FnOnce() -> std::convert::Infallible,
{
    let closure_ctx = unsafe { Box::from_raw(data.cast::<ClosureContext<F, C, L>>()) };
    let ClosureContext { fun, clean, last } = *closure_ctx;
    if let Some(fun) = fun {
        drop(fun)
    }
    if let Some(clean) = clean {
        clean(jump != ffi::Rboolean::FALSE)
    }
    if jump != ffi::Rboolean::FALSE {
        match last {
            Some(last) => last(),
            None => continue_unwind(),
        };
    }
}

/// Wrap a Rust closure with `R_UnwindProtect`.
/// `clean` sees `true` if a non-local jump happened, `false` on normal return.
///
///
pub unsafe fn with_unwind_protect<FunClosure, CleanClosure, LastClosure>(
    fun: FunClosure,
    clean: CleanClosure,
    last: LastClosure,
) -> ffi::SEXP
where
    FunClosure: FnOnce() -> ffi::SEXP,
    CleanClosure: FnOnce(bool),
    LastClosure: FnOnce() -> std::convert::Infallible,
{
    let data = Box::into_raw(Box::new(ClosureContext {
        fun: Some(fun),
        clean: Some(clean),
        last: Some(last),
    }));

    unsafe {
        ffi::R_UnwindProtect(
            Some(fun_tramp::<FunClosure, CleanClosure, LastClosure>),
            data.cast(),
            Some(clean_tramp::<FunClosure, CleanClosure, LastClosure>),
            data.cast(),
            R_CONTINUATION_TOKEN.with(|x| **x),
        )
    }
}
