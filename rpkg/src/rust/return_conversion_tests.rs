//! Failed return conversions and fallible computed getters (#2272, #2269).
//!
//! - A return value whose `IntoR` fails raises a classed R error,
//!   `kind = "result_err"`, from a free function, a method and a computed
//!   getter: the error's message, classes and fields (`IntoR::Error`
//!   implements `RConditionData`). No panic, so no file location.
//! - `AsSerialize` (and `#[miniextendr(serialize)]`) that serde refuses, a map
//!   whose keys are not strings, raises `miniextendr_serde_error`; so does
//!   `AsJson` that `serde_json` refuses.
//! - Inside a tuple or a `Vec` of tuples (#2282), the failing elements raise
//!   one error: the batched message, the class they all share (else only
//!   `rust_error`), and `e$errors`, each failing element's own condition with
//!   its position.
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

/// A refusal that only has a message: no classes, no fields.
pub struct PlainRefusal(i32);

impl RConditionData for PlainRefusal {
    fn message(&self) -> String {
        format!("{} is not convertible", self.0)
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

/// Return `x` through a conversion that refuses values above 10 with an
/// error that has only a message.
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

/// The refused map inside a tuple: the tuple raises serde's class, and the
/// field's own condition is `e$errors[[1]]$condition`, at index 2.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn return_conversion_serde_in_tuple() -> (
    i32,
    miniextendr_api::serde::AsSerialize<std::collections::BTreeMap<i32, f64>>,
) {
    (1, miniextendr_api::serde::AsSerialize(integer_keyed()))
}

/// Rows of tuples whose second field serde refuses in rows 2 and 3: one
/// error naming both, `e$errors` indexed by row and field.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn return_conversion_serde_in_tuple_rows() -> Vec<(
    i32,
    miniextendr_api::serde::AsSerialize<std::collections::BTreeMap<i32, f64>>,
)> {
    vec![
        (1, miniextendr_api::serde::AsSerialize(Default::default())),
        (2, miniextendr_api::serde::AsSerialize(integer_keyed())),
        (3, miniextendr_api::serde::AsSerialize(integer_keyed())),
    ]
}

/// The capped value of `x` and the refused map: when `x` is above 10 both
/// fields fail with different classes, so the tuple raises only the
/// `rust_error` classes and `e$errors` has both.
/// @param x An integer.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn return_conversion_mixed_in_tuple(
    x: i32,
) -> (
    Capped,
    miniextendr_api::serde::AsSerialize<std::collections::BTreeMap<i32, f64>>,
) {
    (
        Capped(x),
        miniextendr_api::serde::AsSerialize(integer_keyed()),
    )
}

/// The refused map as the `Ok` of an `unwrap_in_r` `Result`: the map's own
/// serde error, not `list(error = ...)`, which is only for an `Err`.
#[cfg(feature = "serde")]
#[miniextendr(unwrap_in_r, noexport)]
pub fn return_conversion_serde_unwrap_in_r()
-> Result<miniextendr_api::serde::AsSerialize<std::collections::BTreeMap<i32, f64>>, String> {
    Ok(miniextendr_api::serde::AsSerialize(integer_keyed()))
}
// endregion

// region: JSON text serde_json refuses

/// A map whose keys are pairs: JSON object keys must be strings, so
/// `serde_json` refuses it.
#[cfg(feature = "serde_json")]
type PairKeyed = std::collections::BTreeMap<(i32, i32), f64>;

/// A [`PairKeyed`] map with one entry.
#[cfg(feature = "serde_json")]
fn pair_keyed() -> PairKeyed {
    PairKeyed::from([((1, 2), 2.5)])
}

/// An `AsJson` return that `serde_json` refuses: `miniextendr_serde_error`.
#[cfg(feature = "serde_json")]
#[miniextendr(noexport)]
pub fn return_conversion_json_pair_keys() -> miniextendr_api::serde::AsJson<PairKeyed> {
    miniextendr_api::serde::AsJson(pair_keyed())
}

/// The refused JSON inside a tuple, as its first field.
#[cfg(feature = "serde_json")]
#[miniextendr(noexport)]
pub fn return_conversion_json_in_tuple() -> (miniextendr_api::serde::AsJson<PairKeyed>, i32) {
    (miniextendr_api::serde::AsJson(pair_keyed()), 2)
}

/// JSON text of three maps, the first and third refused: every refused
/// element is named, with its position.
#[cfg(feature = "serde_json")]
#[miniextendr(noexport)]
pub fn return_conversion_json_vec() -> miniextendr_api::serde::AsJsonVec<PairKeyed> {
    miniextendr_api::serde::AsJsonVec(vec![pair_keyed(), Default::default(), pair_keyed()])
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
