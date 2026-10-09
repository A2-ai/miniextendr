//! The argument-count marker: [`NArgs`].
//!
//! R tells some calls apart only by how many arguments were written. A `[`
//! method receives `x[1:3]` and `x[1:3, ]` with the same `i` and a missing `j`;
//! only `nargs()` says the second call had an empty argument (2 against 3), and
//! `[.data.frame` decides between list-style and matrix-style subscripting on
//! it. A `#[miniextendr]` function that needs the count takes a parameter of
//! type `NArgs`:
//!
//! ```ignore
//! use miniextendr_api::{Missing, NArgs, SEXP, miniextendr};
//!
//! /// `[.mx_vec1`: one subscript only.
//! #[miniextendr(s3(generic = "[", class = "mx_vec1"))]
//! pub fn mx_vec1_subset(
//!     x: Vec<f64>,
//!     i: Missing<Vec<i32>>,
//!     _j: Missing<SEXP>,
//!     _drop: Missing<SEXP>,
//!     nargs: NArgs,
//! ) -> Result<Vec<f64>, String> {
//!     if nargs.get() > 2 {
//!         return Err("an mx_vec1 takes one subscript: x[i]".into());
//!     }
//!     // ...
//! #   let _ = (x, i); Ok(Vec::new())
//! }
//! ```
//!
//! The parameter is not an R formal: the generated R wrapper passes `nargs()`
//! at its position in the `.Call()`, so the method above is
//! `` `[.mx_vec1` <- function(x, i, j, drop) `` and the Rust function gets the
//! count. It is the count R's `nargs()` gives in the generated function's own
//! frame, as typed:
//!
//! | Typed | `nargs()` |
//! |---|---|
//! | `x[1:3]` | 2 |
//! | `x[1:3, ]` | 3 |
//! | `x[, 1:3]` | 3 |
//! | `x[1, 2]` | 3 |
//! | `x[1, , drop = FALSE]` | 4 |
//! | `x[]` | 2 |
//! | `x[i] <- v` / `x[i, ] <- v` | 3 / 4 |
//!
//! - `x` counts, and so does every empty argument (`x[1, ]`) and a named
//!   argument such as `drop = FALSE`. A formal with a default that the caller
//!   left out does not.
//! - It is the generated function's own call that is counted. For an S3
//!   method that is the generic's call as dispatched (`[.mx_vec1` above sees
//!   the call to `[`), with the arguments a `...` formal collects. An
//!   inherent R6 or environment-class method counts its own call without the
//!   object (`obj$pick(1)` is 1). A trait method's function takes the object
//!   as its first argument, which counts, and `obj$method(1)` passes it on
//!   (2, as for `Type$Trait$method(obj, 1)`).
//! - The wrapper calls `nargs()` only when the function takes an `NArgs`
//!   parameter; every other wrapper is unchanged.
//!
//! `NArgs` is a parameter type only, matched by its last path segment like the
//! other markers (`NArgs`, `miniextendr_api::NArgs`); `&NArgs` and
//! `Option<NArgs>` are not the marker. It takes no per-parameter option
//! (`default`, `match_arg`, `coerce`, `no_na`, ...), a function takes at most
//! one, and an `extern "C-unwind"` function cannot take it (it has no generated
//! R wrapper to pass the count). Standalone functions, `s3(...)` methods, and
//! the methods of impl blocks and trait impls accept it. It holds a plain
//! number, so it does not keep the function on R's main thread.

use crate::SEXP;
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;

/// The number of arguments in the generated R function's own call, as typed
/// (`nargs()`): `x` included, an empty argument (`x[1, ]`) included, a named
/// `drop = FALSE` included. See the [module docs](self).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NArgs(usize);

impl NArgs {
    /// Wrap a count. Generated code converts the parameter from `nargs()`;
    /// this is for calling such a function from Rust.
    #[inline]
    pub const fn new(count: usize) -> Self {
        NArgs(count)
    }

    /// The count.
    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl From<NArgs> for usize {
    #[inline]
    fn from(nargs: NArgs) -> usize {
        nargs.0
    }
}

impl std::fmt::Display for NArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The integer scalar `nargs()` returns, as a non-negative count.
impl TryFromSexp for NArgs {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        usize::try_from_sexp(sexp).map(NArgs)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { usize::try_from_sexp_unchecked(sexp) }.map(NArgs)
    }
}

/// The count as an R integer, for a `#[miniextendr]` trait's method that takes
/// an `NArgs` parameter (its view passes each argument to the vtable as a
/// SEXP).
impl IntoR for NArgs {
    type Error = <usize as IntoR>::Error;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        self.0.try_into_sexp()
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        unsafe { self.0.try_into_sexp_unchecked() }
    }
}
