//! Conversions whose borrowed result cannot outlive an R root.

use super::{
    NativeBorrow, SexpError, SexpTypeError, TryFromSexp, charsxp_to_cow, charsxp_to_str,
    map_strsxp_with, map_vecsxp_batched,
};
use crate::{RCow, RNativeType, Root, SEXP, SEXPTYPE, SexpExt, StrVec};
use std::borrow::Cow;

/// Convert an R value while retaining its root's lifetime in the result.
///
/// This conversion carries the root lifetime that raw [`TryFromSexp`]
/// cannot express.
/// Generated wrappers use the rooted `.Call` argument for `StrVec`, `RCow`,
/// and `Cow` inputs (including the character-vector Cow containers). Main-thread
/// wrappers and trait shims also use it for explicit shared native references
/// and optional shared references. Worker shared references copy to Rust-owned
/// scalars, vectors or strings before dispatch, then borrow inside the worker.
/// Nested shared-reference `Vec` inputs retain every list element in the root's
/// protection scope, even when an ALTREP accessor does not cache that element.
/// Reference aliases and custom reference converters select this path with
/// `#[miniextendr(convert)]` on the parameter, or `#[miniextendr(convert(param))]`
/// on an inherent or trait impl method. Other aliases can use this trait
/// explicitly in the function body.
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
/// ```compile_fail
/// use miniextendr_api::{Root, TryFromRoot};
/// fn escape<'a>(root: Root<'a>) -> Vec<Vec<&'static [i32]>> {
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

/// An element type of a `Vec` that converts through [`TryFromRoot`]. The
/// element decides what the vector reads: character elements read a `STRSXP`,
/// the others read a list. One `Vec<E>` impl over this trait, rather than one
/// impl per element shape, keeps a nested `Vec<Vec<..>>` from sending type
/// inference into unbounded recursion when the element type is not yet known.
#[doc(hidden)]
pub trait RootedVecElement<'a>: Sized {
    /// Native storage the vector borrows, for wrapper alias checks.
    const NATIVE_BORROW: Option<NativeBorrow>;

    /// Convert every element, retaining each one an ALTREP accessor returns.
    fn vec_from_root(root: Root<'a>) -> Result<Vec<Self>, SexpError>;
}

impl<'a, E: RootedVecElement<'a>> TryFromRoot<'a> for Vec<E> {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = E::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        E::vec_from_root(root)
    }
}

/// `NULL` is `None`; anything else converts as the `Vec`.
impl<'a, E: RootedVecElement<'a>> TryFromRoot<'a> for Option<Vec<E>> {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = E::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        if root.get().type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            E::vec_from_root(root).map(Some)
        }
    }
}

/// Character elements keep the raw conversions' NA handling: `NA` reads as
/// `""` for the plain forms and as `None` for the optional ones.
macro_rules! character_elements {
    ($life:lifetime; $($element:ty => |$value:ident| $borrow:expr, $na:expr, $blank:expr;)*) => {$(
        impl<$life> RootedVecElement<$life> for $element {
            const NATIVE_BORROW: Option<NativeBorrow> = None;

            fn vec_from_root(root: Root<$life>) -> Result<Vec<Self>, SexpError> {
                map_strsxp_with(root.get(), |value, _| {
                    if value == SEXP::na_string() {
                        Ok($na)
                    } else if value == SEXP::blank_string() {
                        Ok($blank)
                    } else {
                        // An ALTSTRING accessor need not retain this CHARSXP.
                        let $value = unsafe { root.protect_child(value) }.get();
                        Ok($borrow)
                    }
                })
            }
        }
    )*};
}

character_elements! {'a;
    &'a str => |value| unsafe { charsxp_to_str(value) }, "", "";
    Option<&'a str> => |value| Some(unsafe { charsxp_to_str(value) }), None, Some("");
    Cow<'a, str> => |value| unsafe { charsxp_to_cow(value) }, Cow::Borrowed(""), Cow::Borrowed("");
    Option<Cow<'a, str>> => |value| Some(unsafe { charsxp_to_cow(value) }), None, Some(Cow::Borrowed(""));
}

/// Each element gets its own root before conversion or the next Elt callback.
/// Merely rooting an ALTREP list does not retain transient Elt results.
fn rooted_list<'a, T: TryFromRoot<'a>>(root: Root<'a>) -> Result<Vec<T>, SexpError>
where
    T::Error: Into<SexpError>,
{
    map_vecsxp_batched(root.get(), |value| {
        T::try_from_root(unsafe { root.protect_child(value) }).map_err(Into::into)
    })
}

/// List elements: native references (for the native types that have a raw
/// reference conversion), and vectors of any element, nested to any depth.
macro_rules! list_elements {
    ($life:lifetime, $param:ident: $bound:path; $($element:ty),* $(,)?) => {$(
        impl<$life, $param: $bound> RootedVecElement<$life> for $element
        where
            $element: TryFromRoot<$life>,
            <$element as TryFromRoot<$life>>::Error: Into<SexpError>,
        {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<$element as TryFromRoot<$life>>::NATIVE_BORROW);

            fn vec_from_root(root: Root<$life>) -> Result<Vec<Self>, SexpError> {
                rooted_list(root)
            }
        }
    )*};
}

list_elements!('a, T: RNativeType; &'a T, Option<&'a T>, &'a [T], Option<&'a [T]>);
list_elements!('a, E: RootedVecElement<'a>; Vec<E>, Option<Vec<E>>);
