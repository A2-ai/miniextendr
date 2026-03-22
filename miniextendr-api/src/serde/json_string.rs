//! JSON string adapters: `AsJson<T>`, `AsJsonPretty<T>`, `FromJson<T>`, `AsJsonVec<T>`.
//!
//! Unlike [`AsSerialize`](super::traits::AsSerialize) which converts `T: Serialize`
//! to native R lists, these wrappers produce/consume actual JSON text as R character
//! scalars. Useful for API responses, config files, logging, or `jsonlite` interop.
//!
//! Requires the `serde_json` feature.

use crate::ffi::SEXP;
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::into_r_error::IntoRError;

// region: AsJson — T: Serialize → R character (compact JSON)

/// Serialize `T` to a compact JSON string, return as R character scalar.
///
/// # Example
///
/// ```ignore
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Response { status: i32, body: String }
///
/// #[miniextendr]
/// fn api_response() -> AsJson<Response> {
///     AsJson(Response { status: 200, body: "ok".into() })
/// }
/// // R gets: '{"status":200,"body":"ok"}'
/// ```
pub struct AsJson<T>(pub T);

impl<T: serde::Serialize> IntoR for AsJson<T> {
    type Error = IntoRError;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let json =
            serde_json::to_string(&self.0).map_err(|e| IntoRError::Inner(e.to_string()))?;
        Ok(json.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let json =
            serde_json::to_string(&self.0).map_err(|e| IntoRError::Inner(e.to_string()))?;
        Ok(unsafe { json.into_sexp_unchecked() })
    }
}
// endregion

// region: AsJsonPretty — T: Serialize → R character (pretty-printed JSON)

/// Serialize `T` to a pretty-printed JSON string, return as R character scalar.
///
/// Same as [`AsJson`] but with indentation for human readability.
pub struct AsJsonPretty<T>(pub T);

impl<T: serde::Serialize> IntoR for AsJsonPretty<T> {
    type Error = IntoRError;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let json = serde_json::to_string_pretty(&self.0)
            .map_err(|e| IntoRError::Inner(e.to_string()))?;
        Ok(json.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let json = serde_json::to_string_pretty(&self.0)
            .map_err(|e| IntoRError::Inner(e.to_string()))?;
        Ok(unsafe { json.into_sexp_unchecked() })
    }
}
// endregion

// region: FromJson — R character scalar → T: Deserialize

/// Parse an R character scalar as JSON into `T: Deserialize`.
///
/// # Example
///
/// ```ignore
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config { max_threads: i32 }
///
/// #[miniextendr]
/// fn parse_config(json: FromJson<Config>) -> i32 {
///     json.0.max_threads
/// }
/// // R: parse_config('{"max_threads": 4}')
/// ```
pub struct FromJson<T>(pub T);

impl<T: for<'de> serde::Deserialize<'de>> TryFromSexp for FromJson<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: String = TryFromSexp::try_from_sexp(sexp)?;
        let value: T = serde_json::from_str(&s)
            .map_err(|e| SexpError::InvalidValue(format!("JSON parse error: {e}")))?;
        Ok(FromJson(value))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: String = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let value: T = serde_json::from_str(&s)
            .map_err(|e| SexpError::InvalidValue(format!("JSON parse error: {e}")))?;
        Ok(FromJson(value))
    }
}
// endregion

// region: AsJsonVec — Vec<T: Serialize> → R character vector of JSON strings

/// Serialize each element of a `Vec<T>` to a JSON string, return as R character vector.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn serialize_points(points: Vec<Point>) -> AsJsonVec<Point> {
///     AsJsonVec(points)
/// }
/// // R gets: c('{"x":1,"y":2}', '{"x":3,"y":4}')
/// ```
pub struct AsJsonVec<T>(pub Vec<T>);

impl<T: serde::Serialize> IntoR for AsJsonVec<T> {
    type Error = IntoRError;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let strings: Vec<String> = self
            .0
            .iter()
            .map(|v| serde_json::to_string(v))
            .collect::<Result<_, _>>()
            .map_err(|e| IntoRError::Inner(e.to_string()))?;
        Ok(strings.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let strings: Vec<String> = self
            .0
            .iter()
            .map(|v| serde_json::to_string(v))
            .collect::<Result<_, _>>()
            .map_err(|e| IntoRError::Inner(e.to_string()))?;
        Ok(unsafe { strings.into_sexp_unchecked() })
    }
}
// endregion
