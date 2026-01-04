//! Integration with the `rust_decimal` crate.
//!
//! Provides conversions between R values and `Decimal`.
//!
//! # Conversion Paths
//!
//! `Decimal` can be converted from R in two ways:
//!
//! 1. **Character (lossless)**: Parse from R `character` - preserves full precision
//! 2. **Numeric (fast path)**: Convert from R `numeric` - may lose precision for
//!    values that don't fit exactly in IEEE 754 double precision
//!
//! When converting FROM R, the input type determines the path:
//! - `character` input: Uses lossless string parsing
//! - `numeric` input: Uses fast f64 conversion (precision warning below)
//!
//! When converting TO R, `Decimal` always produces `character` to preserve precision.
//!
//! # Precision Warning
//!
//! R's numeric type is IEEE 754 double precision (f64), which can represent:
//! - Integers exactly up to 2^53 (about 9 quadrillion)
//! - Decimals with ~15-17 significant digits
//!
//! `rust_decimal::Decimal` supports 28-29 significant digits. Values outside f64's
//! exact representation range will lose precision when converted from R numeric:
//!
//! ```r
//! # These will lose precision when passed as numeric:
//! decimal_from_r(12345678901234567890)  # f64 can't represent this exactly
//! decimal_from_r(0.1 + 0.2)             # f64 rounding error: 0.30000000000000004
//!
//! # Use character for full precision:
//! decimal_from_r("12345678901234567890")
//! decimal_from_r("0.3")
//! ```
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

use crate::ffi::{SEXP, SEXPTYPE, TYPEOF};
use crate::from_r::{SexpError, SexpNaError, TryFromSexp};
use crate::into_r::IntoR;
use std::str::FromStr;

fn parse_decimal(s: &str) -> Result<Decimal, SexpError> {
    Decimal::from_str(s).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

fn decimal_from_f64(f: f64) -> Result<Decimal, SexpError> {
    Decimal::try_from(f).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

/// Get the SEXP type (safe wrapper)
fn sexp_type(sexp: SEXP) -> SEXPTYPE {
    unsafe { TYPEOF(sexp) }
}

impl TryFromSexp for Decimal {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                // Numeric fast path (may lose precision for large values)
                let f: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
                let f = f.ok_or_else(|| {
                    SexpError::Na(SexpNaError {
                        sexp_type: SEXPTYPE::REALSXP,
                    })
                })?;
                decimal_from_f64(f)
            }
            SEXPTYPE::INTSXP => {
                // Integer path (lossless for i32 range)
                let i: Option<i32> = TryFromSexp::try_from_sexp(sexp)?;
                let i = i.ok_or_else(|| {
                    SexpError::Na(SexpNaError {
                        sexp_type: SEXPTYPE::INTSXP,
                    })
                })?;
                Ok(Decimal::from(i))
            }
            _ => {
                // Character path (lossless)
                let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
                let s = s.ok_or_else(|| {
                    SexpError::Na(SexpNaError {
                        sexp_type: SEXPTYPE::STRSXP,
                    })
                })?;
                parse_decimal(&s)
            }
        }
    }
}

impl TryFromSexp for Option<Decimal> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                let f: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
                match f {
                    Some(f) => decimal_from_f64(f).map(Some),
                    None => Ok(None),
                }
            }
            SEXPTYPE::INTSXP => {
                let i: Option<i32> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(i.map(Decimal::from))
            }
            _ => {
                let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
                match s {
                    Some(s) => parse_decimal(&s).map(Some),
                    None => Ok(None),
                }
            }
        }
    }
}

impl TryFromSexp for Vec<Decimal> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| {
                        let f = opt.ok_or_else(|| {
                            SexpError::Na(SexpNaError {
                                sexp_type: SEXPTYPE::REALSXP,
                            })
                        })?;
                        decimal_from_f64(f)
                    })
                    .collect()
            }
            SEXPTYPE::INTSXP => {
                let values: Vec<Option<i32>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| {
                        let i = opt.ok_or_else(|| {
                            SexpError::Na(SexpNaError {
                                sexp_type: SEXPTYPE::INTSXP,
                            })
                        })?;
                        Ok(Decimal::from(i))
                    })
                    .collect()
            }
            _ => {
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
    }
}

impl TryFromSexp for Vec<Option<Decimal>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| match opt {
                        Some(f) => decimal_from_f64(f).map(Some),
                        None => Ok(None),
                    })
                    .collect()
            }
            SEXPTYPE::INTSXP => {
                let values: Vec<Option<i32>> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(values.into_iter().map(|opt| opt.map(Decimal::from)).collect())
            }
            _ => {
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
        self.into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .into_sexp()
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
