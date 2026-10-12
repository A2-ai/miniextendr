//! Conversions whose borrowed result cannot outlive an R root.

use super::{NativeBorrow, SexpError, SexpTypeError, TryFromSexp, charsxp_to_cow, map_strsxp_with};
use crate::{RCow, RNativeType, Root, SEXP, StrVec};
use std::borrow::Cow;

/// Convert an R value while retaining its root's lifetime in the result.
///
/// This conversion carries the root lifetime that raw [`TryFromSexp`]
/// cannot express.
/// Generated wrappers use the rooted `.Call` argument for `StrVec`, `RCow`,
/// and `Cow` inputs (including the character-vector Cow containers).
/// A type alias can use this trait explicitly in the function body.
///
/// A root proves liveness, not exclusive access: this trait does not provide
/// mutable references to R storage. Shared native borrows still participate
/// in a wrapper's alias checks against its other, mutable arguments.
///
/// ```compile_fail
/// use miniextendr_api::{Root, TryFromRoot};
/// use std::borrow::Cow;
/// fn escape<'a>(root: Root<'a>) -> Cow<'static, [i32]> {
///     Cow::try_from_root(root).unwrap()
/// }
/// ```
/// ```compile_fail
/// use miniextendr_api::{Root, TryFromRoot};
/// use std::borrow::Cow;
/// fn escape<'a>(root: Root<'a>) -> Vec<Option<Cow<'static, str>>> {
///     Vec::try_from_root(root).unwrap()
/// }
/// ```
pub trait TryFromRoot<'a>: Sized {
    /// The error reported when the R representation is not accepted.
    type Error;

    /// Native storage borrowed by this conversion, for wrapper alias checks.
    #[doc(hidden)]
    const NATIVE_BORROW: Option<NativeBorrow> = None;

    /// Convert without allowing an R-backed borrow to outlive `root`.
    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error>;
}

impl<'a> TryFromRoot<'a> for StrVec<'a> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        StrVec::try_from_root(root)
    }
}

impl<'a, T: RNativeType> TryFromRoot<'a> for RCow<'a, T> {
    type Error = SexpTypeError;
    const NATIVE_BORROW: Option<NativeBorrow> = RCow::<'a, T>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        RCow::try_from_root(root)
    }
}

impl<'a, T: RNativeType> TryFromRoot<'a> for Cow<'a, [T]> {
    type Error = SexpTypeError;
    const NATIVE_BORROW: Option<NativeBorrow> = <&[T] as TryFromSexp>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        // SAFETY: Root keeps the source live for 'a on R's thread. The
        // native-slice conversion checks the element type before borrowing.
        let data: &'a [T] = unsafe { TryFromSexp::try_from_sexp_unchecked(root.get())? };
        Ok(Cow::Borrowed(data))
    }
}

impl<'a> TryFromRoot<'a> for Cow<'a, str> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        // As with StrVec, the character data stays reachable through root.
        let value: &'a str = unsafe { TryFromSexp::try_from_sexp_unchecked(root.get())? };
        Ok(Cow::Borrowed(value))
    }
}

impl<'a> TryFromRoot<'a> for Vec<Cow<'a, str>> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        map_strsxp_with(root.get(), |value, _| {
            if value == SEXP::na_string() || value == SEXP::blank_string() {
                Ok(Cow::Borrowed(""))
            } else {
                // SAFETY: the parent STRSXP is rooted for 'a.
                Ok(unsafe { charsxp_to_cow(value) })
            }
        })
    }
}

impl<'a> TryFromRoot<'a> for Vec<Option<Cow<'a, str>>> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        map_strsxp_with(root.get(), |value, _| {
            if value == SEXP::na_string() {
                Ok(None)
            } else {
                // SAFETY: the parent STRSXP is rooted for 'a.
                Ok(Some(unsafe { charsxp_to_cow(value) }))
            }
        })
    }
}
