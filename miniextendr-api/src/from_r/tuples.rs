//! Tuple conversions: read an R list (VECSXP) positionally into `(A, B, ...)`.
//!
//! Inbound counterpart of the `IntoR` tuple family (`crate::into_r`, tuple to
//! unnamed list region) — same arities (2 through 8), same VECSXP shape.
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

fn batch_tuple_errors(errors: Vec<String>) -> SexpError {
    SexpError::InvalidValue(format!("tuple conversion failed: {}", errors.join("; ")))
}

/// Implement `TryFromSexp` for tuples of various sizes.
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

                let mut errors: Vec<String> = Vec::new();
                let partial = (
                    $(
                        match <$T as TryFromSexp>::try_from_sexp(
                            sexp.vector_elt($idx as isize),
                        ) {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push(format!(
                                    "element {}: {}",
                                    $idx + 1,
                                    Into::<SexpError>::into(e)
                                ));
                                None
                            }
                        },
                    )+
                );
                if !errors.is_empty() {
                    return Err(batch_tuple_errors(errors));
                }
                // Every slot is Some: the errors vec was empty.
                Ok(($(partial.$idx.unwrap(),)+))
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                check_list_shape(sexp, $n, unsafe { sexp.len_unchecked() })?;

                let mut errors: Vec<String> = Vec::new();
                let partial = (
                    $(
                        match unsafe {
                            <$T as TryFromSexp>::try_from_sexp_unchecked(
                                sexp.vector_elt_unchecked($idx as isize),
                            )
                        } {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push(format!(
                                    "element {}: {}",
                                    $idx + 1,
                                    Into::<SexpError>::into(e)
                                ));
                                None
                            }
                        },
                    )+
                );
                if !errors.is_empty() {
                    return Err(batch_tuple_errors(errors));
                }
                Ok(($(partial.$idx.unwrap(),)+))
            }
        }
    };
}

// Implement for tuples of sizes 2-8, mirroring the IntoR tuple family.
impl_tuple_try_from_sexp!((A, B), (0, 1), 2);
impl_tuple_try_from_sexp!((A, B, C), (0, 1, 2), 3);
impl_tuple_try_from_sexp!((A, B, C, D), (0, 1, 2, 3), 4);
impl_tuple_try_from_sexp!((A, B, C, D, E), (0, 1, 2, 3, 4), 5);
impl_tuple_try_from_sexp!((A, B, C, D, E, F), (0, 1, 2, 3, 4, 5), 6);
impl_tuple_try_from_sexp!((A, B, C, D, E, F, G), (0, 1, 2, 3, 4, 5, 6), 7);
impl_tuple_try_from_sexp!((A, B, C, D, E, F, G, H), (0, 1, 2, 3, 4, 5, 6, 7), 8);
