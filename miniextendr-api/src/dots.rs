//! R's `...` (variadic arguments) support.
//!
//! Provides [`Dots`](crate::dots::Dots), the Rust representation of R's `...` parameter. The generated
//! R wrapper captures `...` as `list(...)` and passes it to Rust.
//!
//! # Usage
//!
//! Use `...` as the parameter type — the macro handles the rest:
//!
//! ```ignore
//! #[miniextendr]
//! pub fn greet(prefix: &str, dots: ...) {
//!     let list = dots.as_list();
//!     // Access by name: list.get_named::<String>("key")
//!     // Access by position: list.get_index::<i32>(0)
//! }
//! ```
//!
//! Use `name: ...` syntax for a custom parameter name, or combine with
//! [`typed_list!`](crate::typed_list!) for structure validation:
//!
//! ```ignore
//! #[miniextendr(dots = typed_list!(x: i32, y: f64))]
//! pub fn validated(args: ...) {
//!     // dots_typed.x and dots_typed.y are available
//! }
//! ```

use crate::from_r::{SexpTypeError, TryFromSexp};
use crate::list::List;
use crate::typed_list::{TypedList, TypedListError, TypedListSpec, validate_list};
use crate::{SEXP, SexpExt};

/// Rust type representing R's `...` (variadic arguments).
///
/// The generated R wrapper captures `...` as `list(...)` and passes it to Rust,
/// so `Dots` holds a list SEXP. Use [`as_list`](Dots::as_list) or
/// [`try_list`](Dots::try_list) to access elements by name or position.
///
/// Declare it as a `&Dots` parameter at any position, one per function:
/// `fn foo(x: i32, rest: &Dots, flag: bool)` becomes the R formals
/// `x, ..., flag`, and parameters after it are matched by name only in R.
/// Rust `...` (or `name: ...`) is the same as a trailing `&Dots` parameter.
#[derive(Debug)]
pub struct Dots {
    // Dots is always passed to us, they need no protection.
    // The R wrapper passes list(...), so this is typically a VECSXP.
    /// Raw list backing this `...` capture.
    ///
    /// This is usually a `VECSXP` built from `list(...)` by generated wrappers.
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
        Dots { inner: SEXP::nil() }
    }

    /// Convert to a [`List`] without additional validation.
    ///
    /// This is a zero-cost conversion since the R wrapper already passes
    /// `list(...)` to us. Use this when you trust the input or want
    /// maximum performance.
    ///
    /// # Safety Note
    ///
    /// This is safe because the miniextendr macro always wraps `...` in
    /// `list(...)` on the R side. However, if you're receiving a SEXP
    /// from another source, use [`try_list`](Dots::try_list) instead.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr]
    /// pub fn process_dots(dots: ...) -> i32 {
    ///     let list = dots.as_list();
    ///     list.len() as i32
    /// }
    /// ```
    #[inline]
    pub fn as_list(&self) -> List {
        // SAFETY: The R wrapper always passes list(...), which is a VECSXP.
        // If this assumption is violated, we're in undefined behavior territory
        // anyway, so wrapping in List is the safest reasonable choice.
        unsafe { List::from_raw(self.inner) }
    }

    /// Convert to a [`List`], checking that the underlying SEXP is a list.
    ///
    /// Accepts a list (`VECSXP`) only, as `TryFromSexp for List` does; a
    /// pairlist is refused (#1866). Use this when `inner` may not come from a
    /// generated wrapper's `list(...)`.
    ///
    /// Names are not checked: R lets the caller repeat a name in `...`
    /// (`f(a = 1, a = 2)`). To refuse that, check
    /// [`List::first_duplicate_name`]; a [`typed`](Dots::typed) spec refuses
    /// repeated names itself.
    ///
    /// # Errors
    ///
    /// Returns [`SexpTypeError`] if the SEXP is not a list.
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr]
    /// pub fn count_named_dots(dots: ...) -> Result<i32, String> {
    ///     let list = dots.try_list().map_err(|e| e.to_string())?;
    ///     if let Some(name) = list.first_duplicate_name() {
    ///         return Err(format!("argument {name:?} was given more than once"));
    ///     }
    ///     Ok(list.len() as i32)
    /// }
    /// ```
    #[inline]
    pub fn try_list(&self) -> Result<List, SexpTypeError> {
        List::try_from_sexp(self.inner)
    }

    /// Get the number of elements in the dots list.
    ///
    /// This is equivalent to `dots.as_list().len()` but avoids
    /// creating an intermediate List wrapper.
    #[inline]
    pub fn len(&self) -> isize {
        self.inner.xlength()
    }

    /// Returns true if no arguments were passed to `...`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Validate the dots against a typed list specification.
    ///
    /// This provides structured validation with clear error messages for
    /// functions that expect specific named arguments via `...`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use miniextendr_api::typed_list::{TypedListSpec, TypedEntry, TypeSpec};
    ///
    /// #[miniextendr]
    /// pub fn process_args(dots: ...) -> Result<i32, String> {
    ///     let spec = TypedListSpec::new(vec![
    ///         TypedEntry::required("alpha", TypeSpec::Numeric(Some(4))),
    ///         TypedEntry::optional("beta", TypeSpec::List(None)),
    ///     ]);
    ///
    ///     let validated = dots.typed(spec).map_err(|e| e.to_string())?;
    ///     let alpha: Vec<f64> = validated.get("alpha").map_err(|e| e.to_string())?;
    ///     Ok(alpha.len() as i32)
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`TypedListError`] if:
    /// - The dots are not a valid list
    /// - A required field is missing
    /// - A field has the wrong type or length
    /// - Extra fields exist when `allow_extra = false`
    #[inline]
    pub fn typed(&self, spec: TypedListSpec) -> Result<TypedList, TypedListError> {
        let list = self.try_list().map_err(TypedListError::NotList)?;
        validate_list(list, &spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_empty_creates_nil() {
        let dots = Dots::empty();
        assert_eq!(dots.inner, SEXP::nil());
    }
}
