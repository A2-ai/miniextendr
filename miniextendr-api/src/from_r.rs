//! Conversions from R SEXP to Rust slices.

use crate::ffi::{Rboolean, Rf_xlength, SEXP, SEXPTYPE, TYPEOF, DATAPTR_RO};

#[derive(Debug, Clone, Copy)]
pub struct SexpTypeError {
    pub expected: SEXPTYPE,
    pub actual: SEXPTYPE,
}

pub trait SexpExt {
    fn as_i32_slice(self) -> Result<&'static [i32], SexpTypeError>;
    fn as_f64_slice(self) -> Result<&'static [f64], SexpTypeError>;
    fn as_u8_slice(self) -> Result<&'static [u8], SexpTypeError>;
    fn as_logical_slice(self) -> Result<&'static [Rboolean], SexpTypeError>;
}

impl SexpExt for SEXP {
    fn as_i32_slice(self) -> Result<&'static [i32], SexpTypeError> {
        let actual = unsafe { TYPEOF(self) };
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::INTSXP, actual });
        }
        let len = unsafe { Rf_xlength(self) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(self).cast(), len) })
    }

    fn as_f64_slice(self) -> Result<&'static [f64], SexpTypeError> {
        let actual = unsafe { TYPEOF(self) };
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::REALSXP, actual });
        }
        let len = unsafe { Rf_xlength(self) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(self).cast(), len) })
    }

    fn as_u8_slice(self) -> Result<&'static [u8], SexpTypeError> {
        let actual = unsafe { TYPEOF(self) };
        if actual != SEXPTYPE::RAWSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::RAWSXP, actual });
        }
        let len = unsafe { Rf_xlength(self) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(self).cast(), len) })
    }

    fn as_logical_slice(self) -> Result<&'static [Rboolean], SexpTypeError> {
        let actual = unsafe { TYPEOF(self) };
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::LGLSXP, actual });
        }
        let len = unsafe { Rf_xlength(self) } as usize;
        if len == 0 { return Ok(&[]); }
        Ok(unsafe { std::slice::from_raw_parts(DATAPTR_RO(self).cast(), len) })
    }
}
