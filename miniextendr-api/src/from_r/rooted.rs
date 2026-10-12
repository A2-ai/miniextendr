//! Conversions whose borrowed result cannot outlive an R root.

use super::{NativeBorrow, SexpError, SexpTypeError, TryFromSexp, charsxp_to_cow, map_strsxp_with};
use crate::{RCow, RNativeType, Root, SEXP, StrVec};
use std::borrow::Cow;

/// Convert an R value while retaining its root's lifetime in the result.
///
/// This conversion carries the root lifetime that raw [`TryFromSexp`]
/// cannot express.
/// Generated wrappers use the rooted `.Call` argument for `StrVec`, `RCow`,
/// and `Cow` inputs (including the character-vector Cow containers). Main-thread
/// wrappers and trait shims also use it for explicit shared native references
/// and optional shared references. Worker `&str` inputs still copy to `String`.
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
/// ```compile_fail
/// use miniextendr_api::{Root, TryFromRoot};
/// fn escape<'a>(root: Root<'a>) -> &'static [i32] {
///     <&[i32]>::try_from_root(root).unwrap()
/// }
/// ```
/// ```compile_fail
/// use miniextendr_api::{Root, TryFromRoot};
/// fn escape<'a>(root: Root<'a>) -> Option<&'static str> {
///     <Option<&str>>::try_from_root(root).unwrap()
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

/// A shared reference converts as its raw `'static` form does, with the same
/// checks, errors and alias metadata, and so does a custom `TryFromSexp` impl
/// for a reference to the user's own type. Only the lifetime changes: the
/// result borrows from the root, not from `'static`.
impl<'a, T: ?Sized + 'static> TryFromRoot<'a> for &'a T
where
    &'static T: TryFromSexp,
{
    type Error = <&'static T as TryFromSexp>::Error;
    const NATIVE_BORROW: Option<NativeBorrow> = <&'static T as TryFromSexp>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        // The root retains the vector, and so the data it holds, for 'a.
        <&'static T as TryFromSexp>::try_from_sexp(root.get())
    }
}

/// `NULL` (and `NA` for strings) is `None`, as in the raw conversion.
impl<'a, T: ?Sized + 'static> TryFromRoot<'a> for Option<&'a T>
where
    Option<&'static T>: TryFromSexp,
{
    type Error = <Option<&'static T> as TryFromSexp>::Error;
    const NATIVE_BORROW: Option<NativeBorrow> = <Option<&'static T> as TryFromSexp>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        <Option<&'static T> as TryFromSexp>::try_from_sexp(root.get())
    }
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
    const NATIVE_BORROW: Option<NativeBorrow> = <&[T] as TryFromRoot<'a>>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        <&[T]>::try_from_root(root).map(Cow::Borrowed)
    }
}

impl<'a> TryFromRoot<'a> for Cow<'a, str> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        <&str>::try_from_root(root).map(Cow::Borrowed)
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
