#![feature(never_type)]

#[non_exhaustive]
#[repr(transparent)]
#[derive(Debug)]
pub struct SEXPREC(std::ffi::c_void);
pub type SEXP = *mut SEXPREC;

#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Rboolean {
    FALSE = 0,
    TRUE = 1,
}

unsafe extern "C" {
    #[allow(dead_code)]
    static R_NilValue: SEXP;

    // R_ext/Error.h
    pub fn Rf_error(arg1: *const ::std::os::raw::c_char, ...) -> !;
    pub fn Rprintf(arg1: *const ::std::os::raw::c_char, ...);

    pub fn R_MakeUnwindCont() -> SEXP;
    pub fn R_ContinueUnwind(cont: SEXP) -> !;
    pub fn R_UnwindProtect(
        fun: ::std::option::Option<unsafe extern "C" fn(*mut ::std::os::raw::c_void) -> SEXP>,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C" fn(*mut ::std::os::raw::c_void, Rboolean),
        >,
        cleanfun_data: *mut ::std::os::raw::c_void,
        cont: SEXP,
    ) -> SEXP;

    // Rinternals.h
    // pub fn Rf_ScalarComplex(arg1: Rcomplex) -> SEXP;
    pub fn Rf_ScalarInteger(arg1: ::std::os::raw::c_int) -> SEXP;
    pub fn Rf_ScalarLogical(arg1: ::std::os::raw::c_int) -> SEXP;
    // pub fn Rf_ScalarRaw(arg1: Rbyte) -> SEXP;
    pub fn Rf_ScalarReal(arg1: f64) -> SEXP;
    pub fn Rf_ScalarString(arg1: SEXP) -> SEXP;

    // Rinternals.h
    pub fn DATAPTR(x: SEXP) -> *mut ::std::os::raw::c_void;
    pub fn DATAPTR_RO(x: SEXP) -> *const ::std::os::raw::c_void;
    pub fn DATAPTR_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_void;
    pub fn LOGICAL_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn INTEGER_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn REAL_OR_NULL(x: SEXP) -> *const f64;
    // pub fn COMPLEX_OR_NULL(x: SEXP) -> *const Rcomplex;
    // pub fn RAW_OR_NULL(x: SEXP) -> *const Rbyte;
    // pub fn INTEGER_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    // pub fn REAL_ELT(x: SEXP, i: R_xlen_t) -> f64;
    // pub fn LOGICAL_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    // pub fn COMPLEX_ELT(x: SEXP, i: R_xlen_t) -> Rcomplex;
    // pub fn RAW_ELT(x: SEXP, i: R_xlen_t) -> Rbyte;
    // pub fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    // pub fn SET_LOGICAL_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    // pub fn SET_INTEGER_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    // pub fn SET_REAL_ELT(x: SEXP, i: R_xlen_t, v: f64);
    // pub fn SET_COMPLEX_ELT(x: SEXP, i: R_xlen_t, v: Rcomplex);
    // pub fn SET_RAW_ELT(x: SEXP, i: R_xlen_t, v: Rbyte);

    pub fn ALTREP_CLASS(x: SEXP) -> SEXP;
    pub fn R_altrep_data1(x: SEXP) -> SEXP;
    pub fn R_altrep_data2(x: SEXP) -> SEXP;
    pub fn R_set_altrep_data1(x: SEXP, v: SEXP);
    pub fn R_set_altrep_data2(x: SEXP, v: SEXP);
    pub fn LOGICAL0(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn INTEGER0(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn REAL0(x: SEXP) -> *mut f64;
    // pub fn COMPLEX0(x: SEXP) -> *mut Rcomplex;
    // pub fn RAW0(x: SEXP) -> *mut Rbyte;
    pub fn ALTREP(x: SEXP) -> ::std::os::raw::c_int;
}

#[derive(Debug)]
struct PanicRError;
thread_local! { static CONT: std::cell::RefCell<SEXP> = std::cell::RefCell::new(std::ptr::null_mut()); }

/// Initialize tracing subscriber based on EXTENDR_TRACE environment variable.
/// Only initializes once, on first call.
fn init_tracing() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        if let Ok(level) = std::env::var("EXTENDR_TRACE") {
            use tracing_subscriber::{fmt, EnvFilter};

            let filter = EnvFilter::try_new(&level)
                .unwrap_or_else(|_| EnvFilter::new("trace"));

            fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_thread_ids(false)
                .with_line_number(true)
                .init();
        }
    });
}

#[inline(always)]
fn cont_set() {
    CONT.with(|t| unsafe {
        let cont = R_MakeUnwindCont();
        tracing::trace!("cont_set: created continuation token: {:p}", cont);
        t.replace(cont)
    });
}
#[inline(always)]
fn cont_get() -> SEXP {
    CONT.with(|t| {
        let cont = *t.borrow();
        tracing::trace!("cont_get: returning continuation token: {:p}", cont);
        cont
    })
}
#[inline(always)]
fn cont_take() -> SEXP {
    CONT.with(|t| {
        let cont = t.replace(std::ptr::null_mut());
        tracing::trace!("cont_take: took continuation token: {:p}, reset to null", cont);
        cont
    })
}

#[inline]
#[tracing::instrument(name = "r_error_from_panic", skip_all)]
fn r_error_from_panic(payload: Box<dyn std::any::Any + Send>) -> ! {
    let panic_kind = if let Some(&panic_message) = payload.downcast_ref::<&'static str>() {
        tracing::error!("panic with &'static str: {}", panic_message);
        panic_message
    } else if let Some(panic_message) = payload.downcast_ref::<String>() {
        tracing::error!("panic with String: {}", panic_message);
        panic_message.as_str()
    } else {
        // TODO: document that this is a totally unusual panic from rust
        // as it is not in panic!("") or panic!("".to_string())
        tracing::error!("unusual rust panic (not &str or String)");
        "rust panic"
    };
    let c = std::ffi::CString::new(panic_kind).unwrap();
    tracing::trace!("calling Rf_error with: {:?}", panic_kind);
    unsafe { Rf_error(c.as_ptr()) }
}

// inner: catch Rust panics before they hit C; map to Rf_error (longjmp)
#[inline]
pub unsafe extern "C" fn tramp_mut<F>(p: *mut std::ffi::c_void) -> SEXP
where
    F: FnMut() -> SEXP,
{
    let _span = tracing::trace_span!("tramp_mut", ptr = ?p).entered();
    tracing::trace!("entering tramp_mut, unwrapping closure from pointer: {:p}", p);

    let f: &mut F = unsafe { p.cast::<F>().as_mut().unwrap() };
    tracing::trace!("closure unwrapped, executing with catch_unwind");

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _inner = tracing::trace_span!("user_closure_execution").entered();
        tracing::trace!("executing user closure inside tramp_mut");
        f()
    })) {
        Ok(v) => {
            tracing::trace!("user closure succeeded, returning SEXP: {:p}", v);
            v
        },
        Err(e) => {
            tracing::warn!("caught panic from user closure, converting to R error");
            r_error_from_panic(e)
        },
    }
}

// cleanfun: always drop boxed closure; clear token only on normal return
pub unsafe extern "C" fn clean_drop_and_mark<F>(p: *mut std::ffi::c_void, jump: Rboolean)
where
    F: FnMut() -> SEXP,
{
    let _span = tracing::trace_span!("clean_drop_and_mark", ptr = ?p, ?jump).entered();
    tracing::trace!("cleanup called with jump={:?}", jump);

    if !p.is_null() {
        tracing::trace!("dropping boxed closure at {:p}", p);
        drop(unsafe { Box::<F>::from_raw(p.cast()) });
    } else {
        tracing::warn!("cleanup called with null pointer");
    }

    if jump == Rboolean::FALSE {
        tracing::trace!("normal return (jump=FALSE), clearing continuation token");
        CONT.with(|t| {
            let old = t.replace(std::ptr::null_mut());
            tracing::trace!("cleared continuation token, was: {:p}", old);
        });
    } else {
        tracing::warn!("longjmp detected (jump=TRUE), keeping continuation token for unwind");
    }
}

// outer: perform local Rust unwind if R longjmp happened, then resume R
#[tracing::instrument(name = "with_r_unwind", skip_all)]
pub fn with_r_unwind<F>(f: F) -> SEXP
where
    F: FnMut() -> SEXP + 'static,
{
    init_tracing();
    tracing::trace!("entering with_r_unwind");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _span = tracing::trace_span!("inner_catch_unwind").entered();
        tracing::trace!("inside inner catch_unwind closure");

        unsafe {
            let f = f;
            let data = Box::into_raw(Box::new(f)).cast();
            tracing::trace!("boxed closure, data pointer: {:p}", data);

            cont_set();

            let result = R_UnwindProtect(
                Some(tramp_mut::<F>),
                data,
                Some(clean_drop_and_mark::<F>),
                data,
                cont_get(),
            );
            tracing::trace!("R_UnwindProtect returned, result: {:p}", result);

            let cont_after = cont_get();
            tracing::trace!("continuation token after R_UnwindProtect: {:p}", cont_after);

            if !cont_after.is_null() {
                tracing::warn!("R longjmp detected, triggering local Rust unwind with PanicRError");
                std::panic::panic_any(PanicRError); // local unwind for outer Rust frames
            }
            tracing::trace!("normal return path from inner closure");
            result
        }
    }));

    match result {
        Ok(v) => {
            tracing::trace!("outer catch_unwind succeeded, returning SEXP: {:p}", v);
            v
        },
        Err(payload) => {
            let _span = tracing::error_span!("error_handling").entered();
            // don't need to downcast, as the panic doesn't hold useful information
            if payload.is::<PanicRError>() {
                tracing::warn!("caught PanicRError, continuing R unwind");
                let cont = cont_take();
                tracing::trace!("took continuation token: {:p}, calling R_ContinueUnwind", cont);
                unsafe { R_ContinueUnwind(cont) }; // never returns
            }
            tracing::error!("unexpected outer panic, converting to R error");
            r_error_from_panic(payload) // unexpected outer panic -> Rf_error
        }
    }
}

// region: panics, (), and Result

#[miniextendr_api::miniextendr]
fn add(left: i32, right: i32) -> i32 {
    left + right
}

#[miniextendr_api::miniextendr]
fn add2(left: i32, right: i32, _dummy: ()) -> i32 {
    left + right
}

#[miniextendr_api::miniextendr]
fn add3(left: i32, right: i32, _dummy: ()) -> Result<i32, ()> {
    left.checked_add(right).ok_or_else(|| ())
}

#[miniextendr_api::miniextendr]
fn add4(left: i32, right: i32) -> Result<i32, &'static str> {
    Ok(left
        .checked_div(right)
        .ok_or_else(|| "don't divide by zero dude")?)
}

#[miniextendr_api::miniextendr]
fn add_panic(_left: i32, _right: i32) -> i32 {
    panic!("we cannot add right now! ");
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[miniextendr_api::miniextendr]
fn add_r_error(_left: i32, _right: i32) -> i32 {
    unsafe { Rf_error(c"r error in `add_r_error`".as_ptr()) };
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

// endregion

// region: `mut` checks

#[miniextendr_api::miniextendr]
fn add_left_mut(mut left: i32, right: i32) -> i32 {
    let left = &mut left;
    *left + right
}

#[miniextendr_api::miniextendr]
fn add_right_mut(left: i32, mut right: i32) -> i32 {
    left + *&mut right
}

#[miniextendr_api::miniextendr]
fn add_left_right_mut(mut left: i32, mut right: i32) -> i32 {
    *&mut left + *&mut right
}

// endregion

// region: panic printing

#[unsafe(no_mangle)]
extern "C" fn C_just_panic() -> SEXP {
    panic!("just panic, no capture");
}

/// If you call a miniextendr function that panics, and then `C_panic_catch`,
/// you'll see that the panic hook was not reset.
#[unsafe(no_mangle)]
extern "C" fn C_panic_and_catch() -> SEXP {
    let result = std::panic::catch_unwind(|| panic!("just panic, no capture"));
    let _ = dbg!(result);
    unsafe { R_NilValue }
}

// endregion

// region: dots

#[miniextendr_api::miniextendr]
fn greetings_with_dots(_dots: ...) {}

#[miniextendr_api::miniextendr]
fn greetings_with_nameless_dots(...) {}

// LIMITATION: Good!
// #[miniextendr_api::miniextendr]
// fn greetings_with_dots_then_arg(dots: ..., exclamations: i32) {}

#[miniextendr_api::miniextendr]
fn greetings_with_dots_then_arg(_exclamations: i32, _dots: ...) {}

// endregion
