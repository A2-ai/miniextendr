/// Export a rust function to R
///
/// ```
/// use miniextendr_api::miniextendr;
///
/// #[miniextendr]
/// fn foo() {}
/// ```
///
/// produces a C wrapper named `C_foo`, and an R wrapper called `foo`.
///
/// In case of function arguments beginning with `_*`, then the R wrapper renames the argument
/// to `unused_*`, as it is not allowed for a variable to begin with `_` in R.
///
/// ## `extern "C"`
///
/// A function with the C ABI may be provided as
///
/// ```
/// use miniextendr_api::miniextendr;
/// use miniextendr_api::ffi::{SEXP, R_NilValue};
///
/// #[miniextendr]
/// #[unsafe(no_mangle)]
/// extern "C" fn C_foo() -> SEXP { unsafe { R_NilValue } }
/// ```
///
/// Here, the provided function definition is the C wrapper, there are no Rust definition, therefore
/// the R wrapper is named `unsafe_*` together with the provided name.
///
///
/// ## Variadic support: [`Dots`] / DotDotDot / `...`
///
/// It is possible to provide `...` as the last argument in an `miniextendr`-`fn`.
/// The corresponding R wrapper will then provide this argument as an evaluated arguments `list(...)`.
///
/// Since Rust does not have variadic support, the provided `fn`'s `...` is overwritten with [`&Dots`].
/// While R can handle unnamed, variadic arguments i.e. `...`, regular Rust `fn` cannot, therefore
/// when `...` is provided, the Rust function has its last argument renamed to `_dots`. Normally,
/// the R wrapper would have its `_*` arguments renamed to `unused_*`, but this is unnecessary in this case.
///
/// It is necessary to add register these functions using [`miniextendr_module`] in order for them to
/// be available in the surrounding R package.
///
/// ## R wrappers
///
// TODO
///
/// [`&Dots`]: dots::Dots
/// [`Dots`]: dots::Dots
pub use miniextendr_macros::miniextendr;
pub use miniextendr_macros::miniextendr_module;

pub mod ffi;

pub mod unwind {
    use crate::ffi::{
        R_ClearExternalPtr, R_ExternalPtrAddr, R_MakeExternalPtr, R_MakeUnwindCont, R_NilValue,
        R_UnwindProtect, Rboolean, Rf_protect, Rf_unprotect, SEXP,
    };
    use std::{cell::Cell, ffi::c_void};

    thread_local! {
            static HAS_JUMPED: Cell<bool> = Cell::new(false);
            static HAS_PANICED: Cell<bool> = Cell::new(false);
    }

    unsafe extern "C" fn fun_trampoline<O>(p: *mut c_void) -> SEXP
    where
        O: FnOnce() -> SEXP,
    {
        let slot = unsafe { p.cast::<Option<O>>().as_mut().unwrap() };
        slot.take().unwrap()()
    }

    unsafe extern "C" fn clean_trampoline<F>(p: *mut c_void, jump: Rboolean)
    where
        F: FnOnce(),
    {
        if let Some(finalizer) = unsafe { p.cast::<Option<F>>().as_mut().unwrap().take() } {
            finalizer();
        }
        if jump != Rboolean::FALSE {
            //     // an R error occurred
            //     panic!()
            HAS_JUMPED.with(|x| x.replace(true));
        }
    }

    pub fn with_r_unwind_protect<O, F>(op: O, fin: F) -> SEXP
    where
        O: FnOnce() -> SEXP,
        F: FnOnce(),
    {
        let mut op_slot: Option<O> = Some(op);
        let mut fin_slot: Option<F> = Some(fin);
        // TODO: save this in a thread_local!
        let continuation = unsafe { R_MakeUnwindCont() };
        HAS_JUMPED.replace(false);
        HAS_PANICED.replace(false);

        unsafe {
            R_UnwindProtect(
                Some(fun_trampoline::<O>),
                (&mut op_slot as *mut Option<O>).cast(),
                Some(clean_trampoline::<F>),
                (&mut fin_slot as *mut Option<F>).cast(),
                continuation,
            )
        }
    }

    // region: passing the payload!

    type Payload = Box<dyn std::any::Any + Send + 'static>;

    pub unsafe fn payload_to_sexp(payload: Payload) -> SEXP {
        // box-in-a-box to make a thin pointer
        let outer: Box<Payload> = Box::new(payload);
        let raw = Box::into_raw(outer).cast::<c_void>();
        // TODO: no tag right now... but maybe?
        // let ext = unsafe { Rf_protect(R_MakeExternalPtr(raw, payload_tag(), R_NilValue)) };
        let ext = unsafe { Rf_protect(R_MakeExternalPtr(raw, R_NilValue, R_NilValue)) };
        unsafe { Rf_unprotect(1) };
        ext
    }

    pub unsafe fn sexp_to_payload(s: SEXP) -> Result<Payload, ()> {
        // no type checking...
        // if R_ExternalPtrTag(s) != payload_tag() {
        //     return Err(());
        // }

        let p = unsafe { R_ExternalPtrAddr(s) };
        if p.is_null() {
            return Err(());
        }
        // reclaim outer box, then move out inner payload
        let outer: Box<Payload> = unsafe { Box::from_raw(p.cast()) };
        unsafe { R_ClearExternalPtr(s) }; // sets `s` to C NULL
        Ok(*outer)
    }
    // endregion
}
pub mod dots {
    use crate::ffi::SEXP;

    /// Rust type representing `...`.
    ///
    /// See [`miniextendr`] macro for more information.
    ///
    /// [`miniextendr`]: crate::miniextendr
    #[derive(Debug)]
    pub struct Dots {
        // Dots is always passed to us, they need no protection.
        pub inner: SEXP,
    }
}
