//! Tuple conversions: read an R list (VECSXP) positionally into `(A, B, ...)`.
//!
//! Inbound counterpart of the `IntoR` tuple family (`crate::into_r`, tuple to
//! unnamed list region) — same arities (1 through 8), same VECSXP shape.
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

fn check_list_shape(sexp: SEXP, expected_len: usize, len: usize) -> Result<(), SexpError> {
    let actual = sexp.type_of();
    if actual != SEXPTYPE::VECSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::VECSXP,
            actual,
        }
        .into());
    }
    if len != expected_len {
        return Err(SexpLengthError {
            expected: expected_len,
            actual: len,
        }
        .into());
    }
    Ok(())
}

/// The reason one list element failed with, in R terms: nothing before it
/// says what that element should have been, so it names the expectation
/// (`expected integer, got character`).
fn element_reason<E: Into<SexpError>>(e: E) -> String {
    Into::<SexpError>::into(e).r_reason(false)
}

/// Implement `TryFromSexp` for tuples of various sizes (1-8).
/// Reads an unnamed R list (VECSXP) positionally; mirrors `impl_tuple_into_r!`.
macro_rules! impl_tuple_try_from_sexp {
    (($($T:ident),+), ($($idx:tt),+), $n:expr) => {
        impl<$($T: TryFromSexp),+> TryFromSexp for ($($T,)+)
        where
            $(<$T as TryFromSexp>::Error: Into<SexpError>,)+
        {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                check_list_shape(sexp, $n, sexp.len())?;

                let mut errors = crate::from_r::BatchedErrors::default();
                let partial = (
                    $(
                        match <$T as TryFromSexp>::try_from_sexp(
                            sexp.vector_elt($idx as isize),
                        ) {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push($idx, || element_reason(e));
                                None
                            }
                        },
                    )+
                );
                if !errors.is_empty() {
                    return Err(errors.into_element_error());
                }
                // Every slot is Some: no error was recorded.
                Ok(($(partial.$idx.unwrap(),)+))
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                check_list_shape(sexp, $n, unsafe { sexp.len_unchecked() })?;

                let mut errors = crate::from_r::BatchedErrors::default();
                let partial = (
                    $(
                        match unsafe {
                            <$T as TryFromSexp>::try_from_sexp_unchecked(
                                sexp.vector_elt_unchecked($idx as isize),
                            )
                        } {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push($idx, || element_reason(e));
                                None
                            }
                        },
                    )+
                );
                if !errors.is_empty() {
                    return Err(errors.into_element_error());
                }
                Ok(($(partial.$idx.unwrap(),)+))
            }
        }
    };
}

// Implement for tuples of sizes 1-8, mirroring the IntoR tuple family.
impl_tuple_try_from_sexp!((A), (0), 1);
impl_tuple_try_from_sexp!((A, B), (0, 1), 2);
impl_tuple_try_from_sexp!((A, B, C), (0, 1, 2), 3);
impl_tuple_try_from_sexp!((A, B, C, D), (0, 1, 2, 3), 4);
impl_tuple_try_from_sexp!((A, B, C, D, E), (0, 1, 2, 3, 4), 5);
impl_tuple_try_from_sexp!((A, B, C, D, E, F), (0, 1, 2, 3, 4, 5), 6);
impl_tuple_try_from_sexp!((A, B, C, D, E, F, G), (0, 1, 2, 3, 4, 5, 6), 7);
impl_tuple_try_from_sexp!((A, B, C, D, E, F, G, H), (0, 1, 2, 3, 4, 5, 6, 7), 8);
