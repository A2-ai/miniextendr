//! Tuple conversions: read an R list (VECSXP) positionally into `(A, B, ...)`.
//!
//! Inbound counterpart of the `IntoR` tuple family (`crate::into_r`, tuple to
//! unnamed list region) — same arities (1 through 12), same VECSXP shape.
//!
//! Semantics:
//! - The input must be a list (VECSXP) of exactly N elements; names are
//!   ignored (conversion is positional).
//! - Element `i` converts via `<Ti as TryFromSexp>::try_from_sexp`.
//! - All failing elements are collected into one batched diagnostic
//!   (1-based positions, matching R indexing) instead of bailing on the
//!   first failure.

use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::{SEXP, SEXPTYPE, SexpExt};

fn check_list_shape(
    sexp: SEXP,
    expected_len: usize,
    len: impl FnOnce() -> usize,
) -> Result<(), SexpError> {
    let actual = sexp.type_of();
    if actual != SEXPTYPE::VECSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::VECSXP,
            actual,
        }
        .into());
    }
    // A rejected ALTREP of another vector type must not run its Length callback.
    let len = len();
    if len != expected_len {
        return Err(SexpLengthError {
            expected: expected_len,
            actual: len,
        }
        .into());
    }
    Ok(())
}

/// Tuple rows keep their own R object/class, as `Vec<List>` does. Validators
/// therefore run per row; nullable rows skip the validator for NULL.
macro_rules! impl_tuple_vec_try_from_sexp {
    (($($T:ident),+), $row:ty, $nullable:literal) => {
        impl<$($T: TryFromSexp),+> TryFromSexp for Vec<$row>
        where
            $($T::Error: Into<SexpError>,)+
        {
            type Error = SexpError;
            const NATIVE_BORROW: Option<crate::from_r::NativeBorrow> =
                crate::from_r::NativeBorrow::in_list(<$row>::NATIVE_BORROW);

            fn __mx_has_na(&self) -> bool {
                self.iter().any(<$row>::__mx_has_na)
            }

            fn __mx_prepare<'root>(
                root: crate::Root<'root>,
            ) -> Result<crate::from_r::prepared::Prepared<'root, Self>, Self::Error>
            where Self: 'root,
            {
                crate::from_r::prepared::list(root, <$row>::__mx_prepare)
            }

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::__mx_try_from_sexp_with_check(sexp, &crate::from_r::no_element_check)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                unsafe {
                    Self::__mx_try_from_sexp_unchecked_with_check(sexp, &crate::from_r::no_element_check)
                }
            }

            fn __mx_try_from_sexp_with_check(
                sexp: SEXP,
                check: &crate::from_r::ElementCheck<'_>,
            ) -> Result<Self, SexpError> {
                crate::from_r::map_vecsxp_batched(sexp, |row| {
                    if !$nullable || row.type_of() != SEXPTYPE::NILSXP {
                        check(row)?;
                    }
                    <$row>::try_from_sexp(row)
                })
            }

            unsafe fn __mx_try_from_sexp_unchecked_with_check(
                sexp: SEXP,
                check: &crate::from_r::ElementCheck<'_>,
            ) -> Result<Self, SexpError> {
                crate::from_r::map_vecsxp_batched(sexp, |row| {
                    if !$nullable || row.type_of() != SEXPTYPE::NILSXP {
                        check(row)?;
                    }
                    unsafe { <$row>::try_from_sexp_unchecked(row) }
                })
            }
        }
    };
}

/// Implement `TryFromSexp` for tuples of various sizes (1-12).
/// Reads an unnamed R list (VECSXP) positionally; mirrors `impl_tuple_into_r!`.
macro_rules! impl_tuple_try_from_sexp {
    ($n:literal, $name:literal; $($T:ident $R:ident $idx:tt),+) => {
        impl<$($T: TryFromSexp),+> TryFromSexp for ($($T,)+)
        where
            $(<$T as TryFromSexp>::Error: Into<SexpError>,)+
        {
            type Error = SexpError;
            const NATIVE_BORROW: Option<crate::from_r::NativeBorrow> =
                crate::from_r::NativeBorrow::tuple(&[$($T::NATIVE_BORROW,)+]);

            fn __mx_has_na(&self) -> bool {
                $(self.$idx.__mx_has_na())||+
            }

            fn __mx_prepare<'root>(
                root: crate::Root<'root>,
            ) -> Result<crate::from_r::prepared::Prepared<'root, Self>, Self::Error>
            where Self: 'root,
            {
                let sexp = root.get();
                check_list_shape(sexp, $n, || sexp.len())?;
                let mut errors = crate::from_r::ElementErrors::default();
                let partial = (
                    $(
                        match <$T as TryFromSexp>::__mx_prepare(unsafe {
                            root.protect_child(sexp.vector_elt($idx))
                        }) {
                            Ok(value) => Some(value),
                            Err(error) => {
                                errors.push($idx, error.into());
                                None
                            }
                        },
                    )+
                );
                let partial = errors.into_result(partial)?;
                let partial = ($(partial.$idx.unwrap(),)+);
                let mut borrows = Vec::new();
                $(borrows.extend_from_slice(partial.$idx.borrows());)+
                Ok(unsafe {
                    crate::from_r::prepared::Prepared::deferred(root, borrows, move || {
                        ($(partial.$idx.finish(),)+)
                    })
                })
            }

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                check_list_shape(sexp, $n, || sexp.len())?;

                let mut errors = crate::from_r::ElementErrors::default();
                let partial = (
                    $(
                        match <$T as TryFromSexp>::try_from_sexp(
                            sexp.vector_elt($idx as isize),
                        ) {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push($idx, e.into());
                                None
                            }
                        },
                    )+
                );
                let partial = errors.into_result(partial)?;
                // Every slot is Some: no error was recorded.
                Ok(($(partial.$idx.unwrap(),)+))
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                check_list_shape(sexp, $n, || unsafe { sexp.len_unchecked() })?;

                let mut errors = crate::from_r::ElementErrors::default();
                let partial = (
                    $(
                        match unsafe {
                            <$T as TryFromSexp>::try_from_sexp_unchecked(
                                sexp.vector_elt_unchecked($idx as isize),
                            )
                        } {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push($idx, e.into());
                                None
                            }
                        },
                    )+
                );
                let partial = errors.into_result(partial)?;
                Ok(($(partial.$idx.unwrap(),)+))
            }
        }

        impl<$($T: TryFromSexp),+> TryFromSexp for Option<($($T,)+)>
        where
            $($T::Error: Into<SexpError>,)+
        {
            type Error = SexpError;
            const NATIVE_BORROW: Option<crate::from_r::NativeBorrow> =
                <($($T,)+)>::NATIVE_BORROW;

            fn __mx_has_na(&self) -> bool {
                self.as_ref().is_some_and(<($($T,)+)>::__mx_has_na)
            }

            fn __mx_prepare<'root>(
                root: crate::Root<'root>,
            ) -> Result<crate::from_r::prepared::Prepared<'root, Self>, Self::Error>
            where Self: 'root,
            {
                if root.get().type_of() == SEXPTYPE::NILSXP {
                    Ok(crate::from_r::prepared::Prepared::ready(root, None))
                } else {
                    <($($T,)+)>::__mx_prepare(root)
                        .map(|value| unsafe { value.map(Some) })
                }
            }

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    Ok(None)
                } else {
                    <($($T,)+)>::try_from_sexp(sexp).map(Some)
                }
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    Ok(None)
                } else {
                    unsafe { <($($T,)+)>::try_from_sexp_unchecked(sexp).map(Some) }
                }
            }
        }

        impl_tuple_vec_try_from_sexp!(($($T),+), ($($T,)+), false);
        impl_tuple_vec_try_from_sexp!(($($T),+), Option<($($T,)+)>, true);
    };
}

crate::tuple_macros::for_each_tuple!(impl_tuple_try_from_sexp);
