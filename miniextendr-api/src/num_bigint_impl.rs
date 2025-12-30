//! Integration with the `num-bigint` crate.
//!
//! Provides conversions between R character vectors and `BigInt` / `BigUint`.
//!
//! # Features
//!
//! Enable this module with the `num-bigint` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["num-bigint"] }
//! ```

pub use num_bigint::{BigInt, BigUint};

use crate::ffi::{SEXP, SEXPTYPE};
use crate::from_r::{SexpError, SexpNaError, TryFromSexp};
use crate::into_r::IntoR;
use std::str::FromStr;

fn parse_bigint(s: &str) -> Result<BigInt, SexpError> {
    BigInt::from_str(s).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

fn parse_biguint(s: &str) -> Result<BigUint, SexpError> {
    BigUint::from_str(s).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

impl TryFromSexp for BigInt {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        let s = s.ok_or_else(|| SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::STRSXP }))?;
        parse_bigint(&s)
    }
}

impl TryFromSexp for BigUint {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        let s = s.ok_or_else(|| SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::STRSXP }))?;
        parse_biguint(&s)
    }
}

impl TryFromSexp for Option<BigInt> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match s {
            Some(s) => parse_bigint(&s).map(Some),
            None => Ok(None),
        }
    }
}

impl TryFromSexp for Option<BigUint> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match s {
            Some(s) => parse_biguint(&s).map(Some),
            None => Ok(None),
        }
    }
}

impl TryFromSexp for Vec<BigInt> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| {
                let s = opt.ok_or_else(|| {
                    SexpError::Na(SexpNaError {
                        sexp_type: SEXPTYPE::STRSXP,
                    })
                })?;
                parse_bigint(&s)
            })
            .collect()
    }
}

impl TryFromSexp for Vec<BigUint> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| {
                let s = opt.ok_or_else(|| {
                    SexpError::Na(SexpNaError {
                        sexp_type: SEXPTYPE::STRSXP,
                    })
                })?;
                parse_biguint(&s)
            })
            .collect()
    }
}

impl TryFromSexp for Vec<Option<BigInt>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| match opt {
                Some(s) => parse_bigint(&s).map(Some),
                None => Ok(None),
            })
            .collect()
    }
}

impl TryFromSexp for Vec<Option<BigUint>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| match opt {
                Some(s) => parse_biguint(&s).map(Some),
                None => Ok(None),
            })
            .collect()
    }
}

impl IntoR for BigInt {
    fn into_sexp(self) -> SEXP {
        self.to_string().into_sexp()
    }
}

impl IntoR for BigUint {
    fn into_sexp(self) -> SEXP {
        self.to_string().into_sexp()
    }
}

impl IntoR for Option<BigInt> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.to_string()).into_sexp()
    }
}

impl IntoR for Option<BigUint> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.to_string()).into_sexp()
    }
}

impl IntoR for Vec<BigInt> {
    fn into_sexp(self) -> SEXP {
        self.into_iter().map(|v| v.to_string()).collect::<Vec<_>>().into_sexp()
    }
}

impl IntoR for Vec<BigUint> {
    fn into_sexp(self) -> SEXP {
        self.into_iter().map(|v| v.to_string()).collect::<Vec<_>>().into_sexp()
    }
}

impl IntoR for Vec<Option<BigInt>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.to_string()))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<Option<BigUint>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.to_string()))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}
