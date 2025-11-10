//!
//!
//!
//!

// Export a rust function to R
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

pub mod altrep;
pub mod ffi;
pub mod into_r;
pub mod unwind;
pub mod unwind_protect;

pub mod error {
    // use crate::ffi::Rprintf;
    // use std::{
    //     cell::RefCell,
    //     ffi::{CStr, CString},
    // };

    // work-in-progress: Use common buffer for the *const char APIs..
    // thread_local! {
    //     /// Buffer using in `rprintln`/`rprint`/`rerror`
    //     pub static R_MESSAGE_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(256));
    // }
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

/// This is used to ensure the macros of `miniextendr-macros` treat this crate as a "user crate"
/// atleast in the `macro_coverage`
#[doc(hidden)]
extern crate self as miniextendr_api;

#[doc(hidden)]
pub mod macro_coverage;
