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
