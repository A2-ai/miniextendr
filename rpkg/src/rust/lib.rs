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

#[inline(always)]
fn cont_set() {
    CONT.with(|t| unsafe { t.replace(R_MakeUnwindCont()) });
}
#[inline(always)]
fn cont_get() -> SEXP {
    CONT.with(|t| *t.borrow())
}
#[inline(always)]
fn cont_take() -> SEXP {
    CONT.with(|t| t.replace(std::ptr::null_mut()))
}

#[inline]
fn r_error_from_panic(payload: Box<dyn std::any::Any + Send>) -> ! {
    let panic_kind = if let Some(&panic_message) = payload.downcast_ref::<&'static str>() {
        panic_message
    } else if let Some(panic_message) = payload.downcast_ref::<String>() {
        panic_message.as_str()
    } else {
        // TODO: document that this is a totally unusual panic from rust
        // as it is not in panic!("") or panic!("".to_string())
        "rust panic"
    };
    let c = std::ffi::CString::new(panic_kind).unwrap();
    unsafe { Rf_error(c.as_ptr()) }
}

// inner: catch Rust panics before they hit C; map to Rf_error (longjmp)
#[inline]
pub unsafe extern "C" fn tramp_mut<F>(p: *mut std::ffi::c_void) -> SEXP
where
    F: FnMut() -> SEXP,
{
    let f: &mut F = unsafe { p.cast::<F>().as_mut().unwrap() };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || f())) {
        Ok(v) => v,
        Err(e) => r_error_from_panic(e),
    }
}

// cleanfun: always drop boxed closure; clear token only on normal return
pub unsafe extern "C" fn clean_drop_and_mark<F>(p: *mut std::ffi::c_void, jump: Rboolean)
where
    F: FnMut() -> SEXP,
{
    if !p.is_null() {
        drop(unsafe { Box::<F>::from_raw(p.cast()) });
    }
    if jump == Rboolean::FALSE {
        CONT.with(|t| {
            t.replace(std::ptr::null_mut());
        });
    }
}

// outer: perform local Rust unwind if R longjmp happened, then resume R
pub fn with_r_unwind<F>(f: F) -> SEXP
where
    F: FnMut() -> SEXP + 'static,
{
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || unsafe {
        let f = f;
        let data = Box::into_raw(Box::new(f)).cast();
        cont_set();
        let result = R_UnwindProtect(
            Some(tramp_mut::<F>),
            data,
            Some(clean_drop_and_mark::<F>),
            data,
            cont_get(),
        );
        if !cont_get().is_null() {
            std::panic::panic_any(PanicRError); // local unwind for outer Rust frames
        }
        result
    }));

    match result {
        Ok(v) => v,
        Err(payload) => {
            // don't need to downcast, as the panic doesn't hold useful information
            if payload.is::<PanicRError>() {
                unsafe { R_ContinueUnwind(cont_take()) }; // never returns
            }
            r_error_from_panic(payload) // unexpected outer panic -> Rf_error
        }
    }
}

// region: panics, (), and Result
#[miniextendr_api::miniextendr]
fn take_and_return_nothing() -> () {}

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

#[miniextendr_api::miniextendr]
#[unsafe(no_mangle)]
extern "C" fn C_just_panic() -> SEXP {
    panic!("just panic, no capture");
}

/// If you call a miniextendr function that panics, and then `C_panic_catch`,
/// you'll see that the panic hook was not reset.
#[miniextendr_api::miniextendr]
#[unsafe(no_mangle)]
extern "C" fn C_panic_and_catch() -> SEXP {
    let result = std::panic::catch_unwind(|| panic!("just panic, no capture"));
    let _ = dbg!(result);
    unsafe { R_NilValue }
}

// endregion

// region: dots

#[miniextendr_api::miniextendr]
fn greetings_with_named_dots(_dots: ...) {}

#[miniextendr_api::miniextendr]
fn greetings_with_nameless_dots(...) {}

// LIMITATION: Good!
// #[miniextendr_api::miniextendr]
// fn greetings_with_dots_then_arg(dots: ..., exclamations: i32) {}

#[miniextendr_api::miniextendr]
fn greetings_last_as_named_dots(_exclamations: i32, _dots: ...) {}

#[miniextendr_api::miniextendr]
fn greetings_last_as_nameless_dots(_exclamations: i32, ...) {}

// endregion

// region: miniextendr_module!

miniextendr_api::miniextendr_module! {
    mod rpkg;

    fn add;
    fn add2;
    fn add3;
    fn add4;
    fn add_panic;
    fn add_r_error;

    fn add_left_mut;
    fn add_right_mut;
    fn add_left_right_mut;

    // TODO: make r wrapper, no C wrapper!
    extern "C" fn C_just_panic;
    extern "C" fn C_panic_and_catch;

    // TODO: adjust the R wrapper to include list(...) in the arg that is ...
    fn greetings_with_named_dots;
    fn greetings_with_nameless_dots;
    fn greetings_last_as_named_dots;
    fn greetings_last_as_nameless_dots;
}

#[repr(C)]
#[derive(Debug)]
pub struct DllInfo(std::ffi::c_void);

#[allow(non_camel_case_types)]
pub type DL_FUNC = ::std::option::Option<unsafe extern "C" fn(...) -> SEXP>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CallMethodDef {
    pub name: *const ::std::os::raw::c_char,
    pub fun: DL_FUNC,
    pub numArgs: ::std::os::raw::c_int,
}

// necessary for calling R_init_<module name>
unsafe impl Sync for R_CallMethodDef {}

// FIXME: move to an ffi crate or similar..
unsafe extern "C" {
    pub fn R_registerRoutines(
        info: *mut DllInfo,
        // croutines: *const R_CMethodDef,
        croutines: *const std::ffi::c_void,
        callRoutines: *const R_CallMethodDef,
        // fortranRoutines: *const R_FortranMethodDef,
        fortranRoutines: *const std::ffi::c_void,
        // externalRoutines: *const R_ExternalMethodDef,
        externalRoutines: *const std::ffi::c_void,
    ) -> ::std::os::raw::c_int;

    pub fn R_useDynamicSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
    pub fn R_forceSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
}

// endregion
