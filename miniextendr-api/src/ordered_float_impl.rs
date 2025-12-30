//! Integration with the `ordered-float` crate.
//!
//! Provides conversions for `OrderedFloat<f64>` and `OrderedFloat<f32>`.
//!
//! # Features
//!
//! Enable this module with the `ordered-float` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["ordered-float"] }
//! ```

pub use ordered_float::OrderedFloat;

use crate::ffi::{SEXP, SEXPTYPE};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

fn parse_f64(sexp: SEXP) -> Result<f64, SexpError> {
    let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
    Ok(value)
}

fn parse_f32(sexp: SEXP) -> Result<f32, SexpError> {
    let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
    Ok(value as f32)
}

impl TryFromSexp for OrderedFloat<f64> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        parse_f64(sexp).map(OrderedFloat)
    }
}

impl TryFromSexp for OrderedFloat<f32> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        parse_f32(sexp).map(OrderedFloat)
    }
}

impl TryFromSexp for Option<OrderedFloat<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(value.map(OrderedFloat))
    }
}

impl TryFromSexp for Option<OrderedFloat<f32>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(value.map(|v| OrderedFloat(v as f32)))
    }
}

impl TryFromSexp for Vec<OrderedFloat<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }

        let slice: &[f64] = unsafe { sexp.as_slice::<f64>() };
        Ok(slice.iter().copied().map(OrderedFloat).collect())
    }
}

impl TryFromSexp for Vec<OrderedFloat<f32>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }

        let slice: &[f64] = unsafe { sexp.as_slice::<f64>() };
        Ok(slice.iter().map(|v| OrderedFloat(*v as f32)).collect())
    }
}

impl TryFromSexp for Vec<Option<OrderedFloat<f64>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(values.into_iter().map(|v| v.map(OrderedFloat)).collect())
    }
}

impl TryFromSexp for Vec<Option<OrderedFloat<f32>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(values
            .into_iter()
            .map(|v| v.map(|val| OrderedFloat(val as f32)))
            .collect())
    }
}

impl IntoR for OrderedFloat<f64> {
    fn into_sexp(self) -> SEXP {
        self.0.into_sexp()
    }
}

impl IntoR for OrderedFloat<f32> {
    fn into_sexp(self) -> SEXP {
        (self.0 as f64).into_sexp()
    }
}

impl IntoR for Option<OrderedFloat<f64>> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.0).into_sexp()
    }
}

impl IntoR for Option<OrderedFloat<f32>> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.0 as f64).into_sexp()
    }
}

impl IntoR for Vec<OrderedFloat<f64>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter().map(|v| v.0).collect::<Vec<_>>().into_sexp()
    }
}

impl IntoR for Vec<OrderedFloat<f32>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.0 as f64)
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<Option<OrderedFloat<f64>>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter().map(|v| v.map(|val| val.0)).collect::<Vec<_>>().into_sexp()
    }
}

impl IntoR for Vec<Option<OrderedFloat<f32>>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.0 as f64))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}
