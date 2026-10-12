//! Conversions whose borrowed result cannot outlive an R root.

use super::{
    NativeBorrow, SexpError, SexpLengthError, SexpTypeError, TryFromSexp, charsxp_to_cow,
    charsxp_to_str, map_strsxp_with, map_vecsxp_batched, scalar_charsxp,
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

impl<'a, T: RNativeType> TryFromRoot<'a> for &'a [T] {
    type Error = SexpTypeError;
    const NATIVE_BORROW: Option<NativeBorrow> = Some(NativeBorrow::slice(T::SEXP_TYPE, false));

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        // Root retains the native vector for 'a; the raw conversion checks its
        // type and uses DATAPTR_RO without deriving a writable pointer.
        unsafe { TryFromSexp::try_from_sexp_unchecked(root.get()) }
    }
}

impl<'a, T: RNativeType> TryFromRoot<'a> for &'a T {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = Some(NativeBorrow::scalar(T::SEXP_TYPE, false));

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        let sexp = root.get();
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        // Refuse the wrong scalar shape before materializing an ALTREP vector.
        let slice = <&[T]>::try_from_root(root)?;
        Ok(&slice[0])
    }
}

impl<'a, T: RNativeType> TryFromRoot<'a> for Option<&'a [T]> {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <&'a [T] as TryFromRoot<'a>>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        if root.get().type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            <&[T]>::try_from_root(root).map(Some).map_err(Into::into)
        }
    }
}

impl<'a, T: RNativeType> TryFromRoot<'a> for Option<&'a T> {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <&'a T as TryFromRoot<'a>>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        if root.get().type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            <&T>::try_from_root(root).map(Some)
        }
    }
}

impl<'a> TryFromRoot<'a> for &'a str {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        let value = scalar_charsxp(root.get())?;
        if value == SEXP::na_string() || value == SEXP::blank_string() {
            return Ok("");
        }
        // An ALTSTRING accessor need not retain its returned CHARSXP.
        let value = unsafe { root.protect_child(value) };
        Ok(unsafe { charsxp_to_str(value.get()) })
    }
}

impl<'a> TryFromRoot<'a> for Option<&'a str> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        if root.get().type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value = scalar_charsxp(root.get())?;
        if value == SEXP::na_string() {
            return Ok(None);
        }
        let value = unsafe { root.protect_child(value) };
        Ok(Some(unsafe { charsxp_to_str(value.get()) }))
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
                let value = unsafe { root.protect_child(value) };
                Ok(unsafe { charsxp_to_cow(value.get()) })
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
                let value = unsafe { root.protect_child(value) };
                Ok(Some(unsafe { charsxp_to_cow(value.get()) }))
            }
        })
    }
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

macro_rules! native_lists {
    ($life:lifetime, $native:ident; $($element:ty),* $(,)?) => {$(
        impl<$life, $native: RNativeType> TryFromRoot<$life> for Vec<$element> {
            type Error = SexpError;
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<$element as TryFromRoot<$life>>::NATIVE_BORROW);

            fn try_from_root(root: Root<$life>) -> Result<Self, Self::Error> {
                rooted_list(root)
            }
        }
    )*};
}

native_lists!('a, T; &'a T, Option<&'a T>, &'a [T], Option<&'a [T]>);

impl<'a, T> TryFromRoot<'a> for Vec<Vec<T>>
where
    Vec<T>: TryFromRoot<'a>,
    <Vec<T> as TryFromRoot<'a>>::Error: Into<SexpError>,
{
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> =
        NativeBorrow::in_list(<Vec<T> as TryFromRoot<'a>>::NATIVE_BORROW);

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        rooted_list(root)
    }
}

impl<'a, T> TryFromRoot<'a> for Option<Vec<T>>
where
    Vec<T>: TryFromRoot<'a>,
{
    type Error = <Vec<T> as TryFromRoot<'a>>::Error;
    const NATIVE_BORROW: Option<NativeBorrow> = <Vec<T> as TryFromRoot<'a>>::NATIVE_BORROW;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        if root.get().type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            Vec::<T>::try_from_root(root).map(Some)
        }
    }
}

impl<'a, T> TryFromRoot<'a> for Vec<Option<Vec<T>>>
where
    Vec<T>: TryFromRoot<'a>,
    <Vec<T> as TryFromRoot<'a>>::Error: Into<SexpError>,
{
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> =
        NativeBorrow::in_list(<Vec<T> as TryFromRoot<'a>>::NATIVE_BORROW);

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        rooted_list(root)
    }
}

impl<'a> TryFromRoot<'a> for Vec<&'a str> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        map_strsxp_with(root.get(), |value, _| {
            if value == SEXP::na_string() || value == SEXP::blank_string() {
                Ok("")
            } else {
                let value = unsafe { root.protect_child(value) };
                Ok(unsafe { charsxp_to_str(value.get()) })
            }
        })
    }
}

impl<'a> TryFromRoot<'a> for Vec<Option<&'a str>> {
    type Error = SexpError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        map_strsxp_with(root.get(), |value, _| {
            if value == SEXP::na_string() {
                Ok(None)
            } else {
                let value = unsafe { root.protect_child(value) };
                Ok(Some(unsafe { charsxp_to_str(value.get()) }))
            }
        })
    }
}
