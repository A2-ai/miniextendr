//! Conversions from R SEXP to Rust types.

use crate::ffi::{Rboolean, Rf_xlength, SEXP, SEXPTYPE, TYPEOF, DATAPTR_RO};

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

// Scalar implementations

impl TryFromSexp for i32 {
    type Error = SexpError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::INTSXP, actual }.into());
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len != 1 {
            return Err(SexpLengthError { expected: 1, actual: len }.into());
        }
        Ok(unsafe { *DATAPTR_RO(sexp).cast::<i32>() })
    }
}

impl TryFromSexp for f64 {
    type Error = SexpError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::REALSXP, actual }.into());
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len != 1 {
            return Err(SexpLengthError { expected: 1, actual: len }.into());
        }
        Ok(unsafe { *DATAPTR_RO(sexp).cast::<f64>() })
    }
}

impl TryFromSexp for u8 {
    type Error = SexpError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::RAWSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::RAWSXP, actual }.into());
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len != 1 {
            return Err(SexpLengthError { expected: 1, actual: len }.into());
        }
        Ok(unsafe { *DATAPTR_RO(sexp).cast::<u8>() })
    }
}

impl TryFromSexp for Rboolean {
    type Error = SexpError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::LGLSXP, actual }.into());
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len != 1 {
            return Err(SexpLengthError { expected: 1, actual: len }.into());
        }
        Ok(unsafe { *DATAPTR_RO(sexp).cast::<Rboolean>() })
    }
}

// Slice implementations

impl TryFromSexp for &'static [i32] {
    type Error = SexpTypeError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::INTSXP, actual });
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(sexp).cast(), len) })
    }
}

impl TryFromSexp for &'static [f64] {
    type Error = SexpTypeError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::REALSXP, actual });
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(sexp).cast(), len) })
    }
}

impl TryFromSexp for &'static [u8] {
    type Error = SexpTypeError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::RAWSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::RAWSXP, actual });
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(sexp).cast(), len) })
    }
}

impl TryFromSexp for &'static [Rboolean] {
    type Error = SexpTypeError;
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { TYPEOF(sexp) };
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::LGLSXP, actual });
        }
        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(sexp).cast(), len) })
    }
}
