//! Conversions from R SEXP to Rust types.

use crate::ffi::{RNativeType, SEXP, SEXPTYPE, SexpExt};

#[derive(Debug, Clone, Copy)]
pub struct SexpTypeError {
    pub expected: SEXPTYPE,
    pub actual: SEXPTYPE,
}

#[derive(Debug, Clone, Copy)]
pub struct SexpLengthError {
    pub expected: usize,
    pub actual: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum SexpError {
    Type(SexpTypeError),
    Length(SexpLengthError),
}

impl From<SexpTypeError> for SexpError {
    fn from(e: SexpTypeError) -> Self {
        SexpError::Type(e)
    }
}

impl From<SexpLengthError> for SexpError {
    fn from(e: SexpLengthError) -> Self {
        SexpError::Length(e)
    }
}

/// TryFrom-style trait for converting SEXP to Rust types.
pub trait TryFromSexp: Sized {
    type Error;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error>;
}

// Blanket implementation for scalar R native types
impl<T: RNativeType> TryFromSexp for T {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = sexp.xlength() as usize;
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        sexp.as_slice::<T>().first().cloned().ok_or_else(|| {
            SexpLengthError {
                expected: 1,
                actual: 0,
            }
            .into()
        })
    }
}

// Blanket implementation for slices of R native types
impl<T: RNativeType> TryFromSexp for &'static [T] {
    type Error = SexpTypeError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        Ok(sexp.as_slice::<T>())
    }
}
