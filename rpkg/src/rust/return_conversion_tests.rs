//! Failed return conversions and fallible computed getters (#2272, #2269).
//!
//! - A return value whose `IntoR` fails raises a classed R error,
//!   `kind = "result_err"`, from a free function, a method and a computed
//!   getter: the error's classes and fields when it implements
//!   `RConditionData`, else its `Display` text. No panic, so no file location.
//! - `AsSerialize` (and `#[miniextendr(serialize)]`) that serde refuses, a map
//!   whose keys are not strings, raises `miniextendr_serde_error`.
//! - A computed getter returning `Result` is an error boundary: `Ok` reads the
//!   value, `Err` raises its classed error, `Err(())` of `Result<T, ()>` reads
//!   `NULL`.
//! - `RCondition::from_data(&e).raise()` raises a typed error by hand.

use miniextendr_api::condition::{ConditionData, RCondition, RConditionData};
use miniextendr_api::externalptr::{Computed, RSidecar};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;

// region: fallible `IntoR` values

/// The largest value a [`Capped`] converts.
const CAP: i32 = 10;

/// A refused value, with its classes and fields (`RConditionData`).
#[derive(Debug)]
pub struct OverCapError {
    value: i32,
}

impl std::fmt::Display for OverCapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} exceeds the cap {CAP}", self.value)
    }
}

impl RConditionData for OverCapError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn class(&self) -> Vec<String> {
        vec!["over_cap_error".to_string(), "cap_error".to_string()]
    }

    fn data(&self) -> Option<ConditionData> {
        Some(vec![
            ("value".to_string(), self.value.into()),
            ("cap".to_string(), CAP.into()),
        ])
    }
}

/// An integer whose conversion to R refuses anything above [`CAP`] with the
/// classed [`OverCapError`].
pub struct Capped(i32);

impl IntoR for Capped {
    type Error = OverCapError;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        if self.0 > CAP {
            Err(OverCapError { value: self.0 })
        } else {
            Ok(SEXP::scalar_integer(self.0))
        }
    }
}

/// A refusal that only has `Display`.
pub struct PlainRefusal(i32);

impl std::fmt::Display for PlainRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is not convertible", self.0)
    }
}

/// An integer whose conversion refuses anything above [`CAP`] with the
/// unclassed [`PlainRefusal`].
pub struct PlainCapped(i32);

impl IntoR for PlainCapped {
    type Error = PlainRefusal;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        if self.0 > CAP {
            Err(PlainRefusal(self.0))
        } else {
            Ok(SEXP::scalar_integer(self.0))
        }
    }
}

fn check_cap(value: i32) -> Result<i32, OverCapError> {
    if value > CAP {
        Err(OverCapError { value })
    } else {
        Ok(value)
    }
}
// endregion

// region: free functions and a method

/// Return `x` through a conversion that refuses values above 10 with a
/// classed error.
/// @param x An integer.
#[miniextendr(noexport)]
pub fn return_conversion_capped(x: i32) -> Capped {
    Capped(x)
}

/// Return `x` through a conversion that refuses values above 10 with a
/// `Display`-only error.
/// @param x An integer.
#[miniextendr(noexport)]
pub fn return_conversion_plain(x: i32) -> PlainCapped {
    PlainCapped(x)
}

/// The `Ok` value of a `Result` return converts like any return value.
/// @param x An integer.
#[miniextendr(noexport)]
pub fn return_conversion_capped_ok(x: i32) -> Result<Capped, String> {
    Ok(Capped(x))
}

/// Raise the classed error of a value above 10 by hand, with
/// `RCondition::from_data(&e).raise()`.
/// @param x An integer.
#[miniextendr(noexport)]
pub fn return_conversion_raise_by_hand(x: i32) -> i32 {
    check_cap(x).unwrap_or_else(|e| RCondition::from_data(&e).raise())
}

/// An env class whose method returns a [`Capped`].
#[derive(miniextendr_api::ExternalPtr)]
pub struct CapGauge {
    value: i32,
}

#[miniextendr(env, noexport)]
impl CapGauge {
    /// Create a gauge holding `value`.
    /// @param value The value.
    pub fn new(value: i32) -> Self {
        CapGauge { value }
    }

    /// The value, through a conversion that refuses values above 10.
    pub fn capped(&self) -> Capped {
        Capped(self.value)
    }
}
// endregion

// region: serde values serde refuses

/// A map with integer keys: R named lists need string keys, so serde refuses it.
#[cfg(feature = "serde")]
fn integer_keyed() -> std::collections::BTreeMap<i32, f64> {
    std::collections::BTreeMap::from([(1, 2.5), (2, 5.0)])
}

/// An `AsSerialize` return that serde refuses: a map with integer keys.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn return_conversion_serde_integer_keys()
-> miniextendr_api::serde::AsSerialize<std::collections::BTreeMap<i32, f64>> {
    miniextendr_api::serde::AsSerialize(integer_keyed())
}

/// The same map under `#[miniextendr(serialize)]`.
#[cfg(feature = "serde")]
#[miniextendr(serialize, noexport)]
pub fn return_conversion_serialize_integer_keys() -> std::collections::BTreeMap<i32, f64> {
    integer_keyed()
}

/// A map with string keys converts.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn return_conversion_serde_string_keys()
-> miniextendr_api::serde::AsSerialize<std::collections::BTreeMap<String, f64>> {
    miniextendr_api::serde::AsSerialize(std::collections::BTreeMap::from([
        ("a".to_string(), 2.5),
        ("b".to_string(), 5.0),
    ]))
}

/// The refused map inside a tuple: the tuple's batched conversion error
/// (`IntoRError`, `Display` only) carries serde's message.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn return_conversion_serde_in_tuple() -> (
    i32,
    miniextendr_api::serde::AsSerialize<std::collections::BTreeMap<i32, f64>>,
) {
    (1, miniextendr_api::serde::AsSerialize(integer_keyed()))
}
// endregion

// region: computed getters returning `Result` (#2269)

/// An S3 class whose computed fields return `Result`s and a fallible value.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct ComputedResult {
    #[r_data]
    _r: RSidecar,

    /// The value, or the classed `over_cap_error` above 10.
    #[r_data(get = "Self::checked")]
    pub checked: Computed,

    /// Twice the value, or `NULL` above 10 (`Result<i32, ()>`).
    #[r_data(get = "Self::doubled_or_null")]
    pub doubled_or_null: Computed,

    /// The value through a conversion that refuses values above 10.
    #[r_data(get = "Self::capped")]
    pub capped: Computed,

    /// The value, a struct field.
    #[r_data]
    pub value: i32,
}

/// The getters of the computed fields.
impl ComputedResult {
    fn checked(&self) -> Result<i32, OverCapError> {
        check_cap(self.value)
    }

    fn doubled_or_null(&self) -> Result<i32, ()> {
        check_cap(self.value).map(|value| value * 2).map_err(|_| ())
    }

    fn capped(&self) -> Capped {
        Capped(self.value)
    }
}

/// S3 class registration for ComputedResult: the generated `$` / `[[` read
/// the fields.
#[miniextendr(s3(r_data_accessors))]
impl ComputedResult {
    /// Create a ComputedResult holding `value`.
    /// @param value The value.
    pub fn new(value: i32) -> Self {
        ComputedResult {
            _r: RSidecar,
            checked: Computed,
            doubled_or_null: Computed,
            capped: Computed,
            value,
        }
    }
}
// endregion
