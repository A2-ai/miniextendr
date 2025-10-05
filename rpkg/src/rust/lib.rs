use miniextendr_api::miniextendr;

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
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f())) {
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
    cont_set();

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let data = Box::into_raw(Box::new(f)) as *mut std::ffi::c_void;
        let ret = R_UnwindProtect(
            Some(tramp_mut::<F>),
            data,
            Some(clean_drop_and_mark::<F>),
            data,
            cont_get(),
        );
        if !cont_get().is_null() {
            std::panic::panic_any(PanicRError); // local unwind for outer Rust frames
        }
        ret
    }));

    match res {
        Ok(v) => v,
        Err(p) => {
            // don't need to downcast, as the panic doesn't hold useful information
            if p.is::<PanicRError>() {
                unsafe { R_ContinueUnwind(cont_take()) }; // never returns
            }
            r_error_from_panic(p) // unexpected outer panic -> Rf_error
        }
    }
}

#[derive(Debug)]
struct A;
impl Drop for A {
    fn drop(&mut self) {
        unsafe { Rprintf(c"A was dropped\n".as_ptr()) };
    }
}

fn add(left: i32, right: i32) -> i32 {
    left + right
}

#[miniextendr_api::miniextendr]
fn add2(left: i32, right: i32, _dummy: ()) -> i32 {
    left + right
}

#[unsafe(no_mangle)]
unsafe extern "C" fn C_add(left: SEXP, right: SEXP) -> SEXP {
    let a = A;
    with_r_unwind(move || unsafe {
        #[allow(unused_imports)]
        use std::borrow::{Borrow, BorrowMut};

        #[allow(unused_variables)]
        let a = a.borrow();
        let left = *DATAPTR_RO(left).cast();
        let left = *DATAPTR_RO(left).cast();
        let right = *DATAPTR_RO(right).cast();

        let result = add(left, right);
        Rf_ScalarInteger(result)
        // unsafe { R_NilValue }
    })
}

fn add_panic(left: i32, right: i32) -> i32 {
    left + right
}

#[unsafe(no_mangle)]
unsafe extern "C" fn C_add_panic(left: SEXP, right: SEXP) -> SEXP {
    let a = A;
    with_r_unwind(move || unsafe {
        #[allow(unused_imports)]
        use std::borrow::{Borrow, BorrowMut};
        #[allow(unused_variables)]
        let a = a.borrow();
        #[allow(unused_variables)]
        let left = *DATAPTR_RO(left).cast();
        #[allow(unused_variables)]
        let right = *DATAPTR_RO(right).cast();

        #[allow(unreachable_code, unused_variables)]
        let result = add_panic(left, right);
        Rf_ScalarInteger(result)
        // unsafe { R_NilValue }
    })
}

fn add_r_error(_left: i32, _right: i32) -> i32 {
    unsafe { Rf_error(c"r error in `add_r_error".as_ptr()) };
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn C_add_r_error(left: SEXP, right: SEXP) -> SEXP {
    let a = A;
    with_r_unwind(move || unsafe {
        #[allow(unused_imports)]
        use std::borrow::{Borrow, BorrowMut};

        #[allow(unused_variables)]
        let a = a.borrow();
        let left = *DATAPTR_RO(left).cast();
        let right = *DATAPTR_RO(right).cast();

        let result = add_r_error(left, right);
        Rf_ScalarInteger(result)
        // unsafe { R_NilValue }
    })
}
