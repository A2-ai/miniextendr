//! Narrow R-call fences for worker input conversion (#1302).
//!
//! An outer R_UnwindProtect cannot clean up locals in its body: R has already
//! longjmped past them before calling cleanfun. Fence each checked R call below
//! those locals, then resume the R continuation after Rust unwinds the converter.

use std::{
    any::Any,
    cell::{Cell, RefCell},
    ffi::{CStr, CString, c_void},
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    rc::Rc,
};

use crate::{Rboolean, SEXP, sys};

thread_local! {
    static DEPTH: Cell<usize> = const { Cell::new(0) };
    // Tokens are preserved for reuse. A pending R error owns its token until
    // R_ContinueUnwind; successful cleanup calls must use a different token.
    static TOKENS: RefCell<Vec<SEXP>> = const { RefCell::new(Vec::new()) };
}

/// Generated worker wrappers enable checked-call fences before converting inputs.
/// The scope ends before dispatch, leaving worker callback transport unchanged.
#[doc(hidden)]
pub struct InputConversionScope {
    previous: usize,
    _main_thread: PhantomData<Rc<()>>,
}

impl InputConversionScope {
    /// Enter a worker wrapper's input conversion boundary.
    ///
    /// # Safety
    /// Call on R's main thread before constructing conversion resources. The
    /// generated wrapper must catch panics and call [`resume_input_error`] after
    /// dropping converted arguments and completing RNG cleanup.
    pub unsafe fn new() -> Self {
        // Allocate before conversions: allocating lazily in the fence for
        // Rf_protect would collect the fresh, still-unrooted object being protected.
        // One token can carry an error while the other serves cleanup-side calls.
        // Reentrant worker wrappers replenish the pool at their own entry.
        while TOKENS.with(|tokens| tokens.borrow().len()) < 2 {
            let token = unsafe {
                let token = sys::Rf_protect_unchecked(sys::R_MakeUnwindCont_unchecked());
                sys::R_PreserveObject_unchecked(token);
                sys::Rf_unprotect_unchecked(1);
                token
            };
            TOKENS.with(|tokens| tokens.borrow_mut().push(token));
        }
        let previous = DEPTH.with(|depth| depth.replace(depth.get() + 1));
        Self {
            previous,
            _main_thread: PhantomData,
        }
    }
}

impl Drop for InputConversionScope {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(self.previous));
    }
}

struct Suspension(usize);

impl Drop for Suspension {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(self.0));
    }
}

struct Token(SEXP);

impl Drop for Token {
    fn drop(&mut self) {
        TOKENS.with(|tokens| tokens.borrow_mut().push(self.0));
    }
}

struct InputError {
    token: Token,
    // Simple-error exiting handlers may defer reading R's global error buffer
    // until the continuation resumes. Rust destructors can catch another R
    // error and overwrite it in the meantime.
    message: CString,
}
struct LeafError;

/// Call one raw R function, fencing R errors during worker input conversion.
///
/// # Safety
/// Call on R's main thread. `f` must contain only the direct R FFI operation,
/// with no Rust resources whose destructors R's longjmp could skip. Generated
/// checked FFI wrappers supply Copy arguments and return values.
#[doc(hidden)]
#[inline]
pub unsafe fn with_input_conversion_call<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Copy,
    T: Copy,
{
    if DEPTH.with(Cell::get) == 0 {
        return f();
    }
    unsafe { fenced_call(f) }
}

#[inline(never)]
unsafe fn fenced_call<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Copy,
    T: Copy,
{
    struct CallData<F, T> {
        f: F,
        result: Option<T>,
        panic: Option<Box<dyn Any + Send>>,
    }

    unsafe extern "C-unwind" fn trampoline<F, T>(data: *mut c_void) -> SEXP
    where
        F: FnOnce() -> T + Copy,
        T: Copy,
    {
        let data = unsafe { &mut *data.cast::<CallData<F, T>>() };
        // Rust panics must also let R_UnwindProtect close its R context.
        match catch_unwind(AssertUnwindSafe(data.f)) {
            Ok(result) => data.result = Some(result),
            Err(payload) => data.panic = Some(payload),
        }
        SEXP::nil()
    }

    unsafe extern "C-unwind" fn cleanup(_data: *mut c_void, jump: Rboolean) {
        if jump != Rboolean::FALSE {
            // Bypass the panic hook: this is an existing R condition, not a Rust
            // panic, and must not produce panic telemetry or a source location.
            resume_unwind(Box::new(LeafError));
        }
    }

    let token = Token(TOKENS.with(|tokens| {
        tokens
            .borrow_mut()
            .pop()
            .expect("input conversion token pool is prewarmed")
    }));
    let mut data = CallData {
        f,
        result: None,
        panic: None,
    };
    // R can reenter Rust through finalizers, active bindings, or evaluation.
    // Those entry points retain their own error boundaries; they must not inherit
    // the caller's conversion scope or receive its private panic payload.
    let suspension = Suspension(DEPTH.with(|depth| depth.replace(0)));
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        sys::R_UnwindProtect_C_unwind(
            Some(trampoline::<F, T>),
            std::ptr::from_mut(&mut data).cast(),
            Some(cleanup),
            std::ptr::null_mut(),
            token.0,
        )
    }));
    drop(suspension);
    match result {
        Err(payload) if payload.is::<LeafError>() => {
            drop(payload);
            let message = unsafe { CStr::from_ptr(sys::R_curErrorBuf()).to_owned() };
            resume_unwind(Box::new(InputError { token, message }))
        }
        Err(payload) => resume_unwind(payload),
        Ok(_) => {
            if let Some(payload) = data.panic {
                resume_unwind(payload);
            }
            data.result.expect("R call completed without a result")
        }
    }
}

// Legacy guards may be called inside a converter. Once their R context has
// closed, they must forward our continuation instead of tagging it as a panic.
pub(super) fn propagate_input_error(payload: Box<dyn Any + Send>) -> Box<dyn Any + Send> {
    if payload.is::<InputError>() {
        resume_unwind(payload);
    }
    payload
}

/// Resume a conversion's R error, or return an ordinary Rust panic unchanged.
///
/// # Safety
/// Call on R's main thread after the conversion scope, arguments, and RNG cleanup
/// have finished. Resuming an R error longjmps; no Rust cleanup may remain pending.
#[doc(hidden)]
pub unsafe fn resume_input_error(payload: Box<dyn Any + Send>) -> Box<dyn Any + Send> {
    match payload.downcast::<InputError>() {
        Ok(error) => {
            let InputError { token, message } = InputError::from(error);
            let token = std::mem::ManuallyDrop::new(token).0;
            // R_ContinueUnwind can run on.exit code that reenters a worker wrapper
            // before installing the original return value. Keep the token rooted
            // and unavailable to that wrapper; R's jump removes this protection.
            // Successful calls recycle tokens; the next input scope replenishes
            // this retired error token before constructing conversion resources.
            unsafe {
                sys::Rf_protect_unchecked(token);
                sys::R_ReleaseObject_unchecked(token);
                restore_error_message(&message);
                drop(message);
                sys::R_ContinueUnwind_unchecked(token)
            }
        }
        Err(payload) => payload,
    }
}

// Moving fields out of Box in the diverging match arm would leave allocation
// cleanup after R_ContinueUnwind. This ordinary Rust return frees the allocation
// before the caller resumes R, even though the payload fields remain owned.
impl From<Box<InputError>> for InputError {
    fn from(error: Box<InputError>) -> Self {
        *error
    }
}

// There is no public setter for the error buffer. A private exiting handler
// around errorcall restores the original simple-error bytes without notifying
// outer calling/global handlers or invoking options(error). Its handler returns
// normally; the original continuation still carries the original call/condition.
unsafe fn restore_error_message(message: &CStr) {
    if unsafe { CStr::from_ptr(sys::R_curErrorBuf()) } == message {
        return;
    }
    unsafe extern "C-unwind" fn body(data: *mut c_void) -> SEXP {
        // This leaf has no Rust resources for R's longjmp to skip. The owned
        // message remains above R_tryCatchError in resume_input_error's frame.
        unsafe {
            sys::Rf_errorcall_unchecked(
                SEXP::nil(),
                c"%s".as_ptr(),
                data.cast::<std::ffi::c_char>(),
            )
        }
    }
    unsafe extern "C-unwind" fn handler(_condition: SEXP, _data: *mut c_void) -> SEXP {
        SEXP::nil()
    }
    unsafe {
        sys::R_tryCatchError_unchecked(
            Some(body),
            message.as_ptr().cast_mut().cast(),
            Some(handler),
            std::ptr::null_mut(),
        );
    }
}
