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
    static R_NilValue: SEXP;

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
}

#[derive(Debug)]
struct PanicRError;

thread_local! {
    static R_CONTINUATION_TOKEN: std::cell::RefCell<SEXP> =
        std::cell::RefCell::new(std::ptr::null_mut());
}

/* -------- trampoline: catch Rust panic here and convert to Rf_error -------- */

unsafe extern "C" fn trampoline_mut<F>(p: *mut std::ffi::c_void) -> SEXP
where
    F: FnMut() -> SEXP,
{
    let f: &mut F = unsafe { p.cast::<F>().as_mut() }.unwrap();

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
    match res {
        Ok(v) => v,
        Err(e) => {
            // Map Rust panic -> R error. This longjmps; cleanfun will run.
            if let Some(msg) = e.downcast_ref::<&'static str>() {
                let c = std::ffi::CString::new(*msg).unwrap();
                unsafe { Rf_error(c.as_ptr()) }
            } else if let Some(msg) = e.downcast_ref::<String>() {
                let c = std::ffi::CString::new(msg.clone()).unwrap();
                unsafe { Rf_error(c.as_ptr()) }
            } else {
                let c = std::ffi::CString::new("rust panic").unwrap();
                unsafe { Rf_error(c.as_ptr()) }
            }
        }
    }
}

/* ---------------- cleanfun: drop box; clear token on normal return ---------------- */

unsafe extern "C" fn cleanfun_drop_box_and_mark<F>(p: *mut std::ffi::c_void, jump: Rboolean)
where
    F: FnMut() -> SEXP,
{
    if !p.is_null() {
        drop(unsafe { Box::<F>::from_raw(p.cast()) }); // drops all captures
    }
    if jump == Rboolean::FALSE {
        R_CONTINUATION_TOKEN.with(|tok| {
            tok.replace(std::ptr::null_mut());
        });
    }
}

fn into_c_callback_mut<F>(
    f: F,
) -> (
    Option<unsafe extern "C" fn(*mut std::ffi::c_void) -> SEXP>,
    *mut std::ffi::c_void,
)
where
    F: FnMut() -> SEXP,
{
    (
        Some(trampoline_mut::<F>),
        Box::into_raw(Box::new(f)).cast()
    )
}

/* -------- boundary: convert true R longjmp -> local Rust panic -> ContinueUnwind -------- */

fn run_with_r_unwind_mut<F>(f: F) -> SEXP
where
    F: FnMut() -> SEXP,
{
    R_CONTINUATION_TOKEN.with(|cont| unsafe { cont.replace(R_MakeUnwindCont()) });

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let (fun, fun_data) = into_c_callback_mut::<F>(f);
        let cont = R_CONTINUATION_TOKEN.with(|x| *x.borrow());

        let result = R_UnwindProtect(
            fun,
            fun_data,
            Some(cleanfun_drop_box_and_mark::<F>),
            fun_data,
            cont,
        );

        // Token still set => R longjmp occurred inside `fun`
        if !R_CONTINUATION_TOKEN.with(|tok| *tok.borrow()).is_null() {
            std::panic::panic_any(PanicRError);
        }

        result
    }));

    match result {
        Ok(v) => v,
        Err(p) => {
            if p.is::<PanicRError>() {
                let cont = R_CONTINUATION_TOKEN.with(|tok| tok.replace(std::ptr::null_mut()));
                unsafe { R_ContinueUnwind(cont) }; // no return
            }
            // TODO: use a thread_local buffer for all these Rf_error calls!
            
            // Any other panic here is unexpected; map to R error to be safe.
            if let Some(msg) = p.downcast_ref::<&'static str>() {
                let c = std::ffi::CString::new(*msg).unwrap();
                unsafe { Rf_error(c.as_ptr()) }
            } else if let Some(msg) = p.downcast_ref::<String>() {
                let c = std::ffi::CString::new(msg.clone()).unwrap();
                unsafe { Rf_error(c.as_ptr()) }
            } else {
                let c = std::ffi::CString::new("rust panic outside fun").unwrap();
                unsafe { Rf_error(c.as_ptr()) }
            }
        }
    }
}

/* ---------------- example entry and callee ---------------- */

#[derive(Debug)]
struct A;
impl Drop for A {
    fn drop(&mut self) {
        unsafe { Rprintf(c"A was dropped\n".as_ptr()) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn C_add(left: SEXP, right: SEXP) -> SEXP {
    let a = A; // captured by move into the boxed closure
    run_with_r_unwind_mut(move || {
        let _keep = &a; // keep `a` in the env; do not move it
        let _ = (left, right);
        let _ = add(1, 1); // may panic or call Rf_error
        unsafe { R_NilValue }
    })
}

#[allow(unreachable_code)]
pub fn add(_left: u64, _right: u64) -> u64 {
    // test Rust panic:
    panic!("boom");
    // test R error:
    // unsafe { Rf_error(c"arg1".as_ptr()) };
    0
}
