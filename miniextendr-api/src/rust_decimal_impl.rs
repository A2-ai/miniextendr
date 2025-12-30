//! Integration with the `rust_decimal` crate.
//!
//! Provides conversions between R character vectors and `Decimal`.
//!
//! # Features
//!
//! Enable this module with the `rust_decimal` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["rust_decimal"] }
//! ```

pub use rust_decimal::Decimal;

use crate::ffi::{SEXP, SEXPTYPE};
use crate::from_r::{SexpError, SexpNaError, TryFromSexp};
use crate::into_r::IntoR;
use std::str::FromStr;

fn parse_decimal(s: &str) -> Result<Decimal, SexpError> {
    Decimal::from_str(s).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

impl TryFromSexp for Decimal {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        let s = s.ok_or_else(|| SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::STRSXP }))?;
        parse_decimal(&s)
    }
}

impl TryFromSexp for Option<Decimal> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match s {
            Some(s) => parse_decimal(&s).map(Some),
            None => Ok(None),
        }
    }
}

impl TryFromSexp for Vec<Decimal> {
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
                parse_decimal(&s)
            })
            .collect()
    }
}

impl TryFromSexp for Vec<Option<Decimal>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| match opt {
                Some(s) => parse_decimal(&s).map(Some),
                None => Ok(None),
            })
            .collect()
    }
}

impl IntoR for Decimal {
    fn into_sexp(self) -> SEXP {
        self.to_string().into_sexp()
    }
}

impl IntoR for Option<Decimal> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.to_string()).into_sexp()
    }
}

impl IntoR for Vec<Decimal> {
    fn into_sexp(self) -> SEXP {
        self.into_iter().map(|v| v.to_string()).collect::<Vec<_>>().into_sexp()
    }
}

impl IntoR for Vec<Option<Decimal>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.to_string()))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}
