use crate::ffi::{R_NilValue, SEXP};

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

impl Dots {
    /// Create an empty Dots (equivalent to no `...` arguments).
    ///
    /// This is useful when calling R functions from Rust that expect
    /// dots arguments but you want to pass nothing.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr]
    /// pub fn my_constructor(x: Doubles, dots: ...) -> Robj {
    ///     // ...
    /// }
    ///
    /// // Call from Rust with empty dots:
    /// let result = my_constructor(data, Dots::empty());
    /// ```
    pub fn empty() -> Self {
        // SAFETY: R_NilValue is always valid and represents empty dots
        Dots {
            inner: unsafe { R_NilValue },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_empty_creates_nil() {
        let dots = Dots::empty();
        assert_eq!(dots.inner, unsafe { R_NilValue });
    }
}
