//! JSON string adapters: `AsJson<T>`, `AsJsonPretty<T>`, `FromJson<T>`, `AsJsonVec<T>`.
//!
//! Unlike [`AsSerialize`](super::traits::AsSerialize) which converts `T: Serialize`
//! to native R lists, these wrappers produce/consume actual JSON text as R character
//! scalars. Useful for API responses, config files, logging, or `jsonlite` interop.
//!
//! Requires the `serde_json` feature.

use crate::SEXP;
use crate::condition::RConditionData;
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::into_r_error::{ElementFailures, IntoRError};
use crate::serde::RSerdeError;

/// `serde_json`'s refusal as the serde error R gets: class
/// `miniextendr_serde_error`, serde_json's message.
fn json_error(error: serde_json::Error) -> RSerdeError {
    RSerdeError::Message(error.to_string())
}

/// The JSON text as an R character scalar; text longer than an R string can
/// hold is refused with the same serde error class.
fn json_text(json: String) -> Result<SEXP, RSerdeError> {
    json.try_into_sexp()
        .map_err(|error| RSerdeError::Message(error.message()))
}

/// [`json_text`] without thread checks.
///
/// # Safety
///
/// Must be called from R's main thread.
unsafe fn json_text_unchecked(json: String) -> Result<SEXP, RSerdeError> {
    unsafe { json.try_into_sexp_unchecked() }.map_err(|error| RSerdeError::Message(error.message()))
}

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

/// A value `serde_json` refuses (a map whose keys are not strings, a
/// `Serialize` impl's own `Error::custom`) is an [`RSerdeError`]: in R, an
/// error of class `miniextendr_serde_error` with serde_json's message.
impl<T: serde::Serialize> IntoR for AsJson<T> {
    type Error = RSerdeError;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        json_text(serde_json::to_string(&self.0).map_err(json_error)?)
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        unsafe { json_text_unchecked(serde_json::to_string(&self.0).map_err(json_error)?) }
    }
}
// endregion

// region: AsJsonPretty — T: Serialize → R character (pretty-printed JSON)

/// Serialize `T` to a pretty-printed JSON string, return as R character scalar.
///
/// Same as [`AsJson`] but with indentation for human readability.
pub struct AsJsonPretty<T>(pub T);

/// Refuses what [`AsJson`] refuses, with the same [`RSerdeError`].
impl<T: serde::Serialize> IntoR for AsJsonPretty<T> {
    type Error = RSerdeError;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        json_text(serde_json::to_string_pretty(&self.0).map_err(json_error)?)
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        unsafe { json_text_unchecked(serde_json::to_string_pretty(&self.0).map_err(json_error)?) }
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

/// Every element `serde_json` refuses is reported, with its position, in one
/// [`IntoRError::Elements`]: in R, an error of class
/// `miniextendr_serde_error` naming each refused element, and `e$errors`.
impl<T: serde::Serialize> IntoR for AsJsonVec<T> {
    type Error = IntoRError;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(json_strings(&self.0)?.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(unsafe { json_strings(&self.0)?.into_sexp_unchecked() })
    }
}

/// Each value as JSON text, or the refusal of every value `serde_json`
/// refuses.
fn json_strings<T: serde::Serialize>(values: &[T]) -> Result<Vec<String>, IntoRError> {
    let mut errors = ElementFailures::default();
    let strings = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            serde_json::to_string(value).unwrap_or_else(|error| {
                errors.push(index, &json_error(error));
                String::new()
            })
        })
        .collect();
    if errors.is_empty() {
        Ok(strings)
    } else {
        Err(errors.into_error())
    }
}
// endregion
