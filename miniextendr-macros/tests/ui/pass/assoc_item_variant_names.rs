//! Compile-pass regression test for #1730.
//!
//! Generated impls must not spell an associated item as `Self::<Name>`: in an
//! impl for an enum, `Self::Error` is ambiguous with a variant named `Error`
//! (the deny-by-default `ambiguous_associated_items` lint), and in value
//! position a variant silently wins over an associated const, so
//! `Self::CHOICES` would name a variant `CHOICES`. The derives that accept
//! enums and a `#[miniextendr]` impl block have to compile on enums whose
//! variants take those names. (`try_from_sexp_via_str_parse!` is covered by a
//! unit test in `miniextendr-api/src/from_r.rs`: its `Option<T>` / `Vec<T>`
//! impls only compile inside that crate.)

#![allow(dead_code, non_camel_case_types)]

use miniextendr_api::{
    DataFrameRow, ExternalPtr, MatchArg, PreferExternalPtr, RFactor, miniextendr,
};

// region: MatchArg

#[derive(Copy, Clone, Debug, MatchArg)]
#[match_arg(rename_all = "snake_case")]
pub enum OnFailure {
    Warn,
    Error,
    Value,
    Output,
}

/// Variants named after `MatchArg`'s associated const.
#[derive(Copy, Clone, Debug, MatchArg)]
pub enum ConstNamedChoice {
    CHOICES,
    Other,
}

#[miniextendr]
pub fn on_failure_round_trip(#[miniextendr(match_arg)] x: OnFailure) -> OnFailure {
    x
}

#[miniextendr]
pub fn on_failure_several(#[miniextendr(match_arg, several_ok)] x: Vec<OnFailure>) -> i32 {
    x.len() as i32
}

#[miniextendr]
pub fn const_named_choice(#[miniextendr(match_arg)] x: ConstNamedChoice) -> ConstNamedChoice {
    x
}

// endregion

// region: RFactor

#[derive(Copy, Clone, Debug, RFactor)]
pub enum Outcome {
    Error,
    Value,
    Output,
}

/// Interaction factor whose outer variants are named `Error` and `CHOICES`.
#[derive(Copy, Clone, Debug, RFactor)]
#[r_factor(interaction = ["Error", "Value", "Output"])]
pub enum OutcomeBy {
    Error(Outcome),
    CHOICES(Outcome),
}

#[miniextendr]
pub fn outcome_round_trip(x: Outcome) -> Outcome {
    x
}

#[miniextendr]
pub fn outcome_by_round_trip(x: OutcomeBy) -> OutcomeBy {
    x
}

// endregion

// region: DataFrameRow

/// Unit-only enum: the derive emits `UnitEnumFactor` and `IntoR`.
#[derive(Copy, Clone, Debug, DataFrameRow)]
pub enum UnitStatus {
    Error,
    Value,
    Output,
}

/// Const-generic unit-only enum: the generic `IntoR` emission path.
#[derive(Copy, Clone, Debug, DataFrameRow)]
pub enum GenericStatus<const N: usize> {
    Error,
    Value,
}

/// Payload enum.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum Event {
    Error { code: i32 },
    Value { value: f64 },
    Output { text: String },
}

#[miniextendr]
pub fn unit_status() -> UnitStatus {
    UnitStatus::Error
}

#[miniextendr]
pub fn events() -> EventDataFrame {
    Event::to_dataframe(vec![
        Event::Error { code: 1 },
        Event::Value { value: 2.0 },
        Event::Output {
            text: "three".to_string(),
        },
    ])
}

// endregion

// region: #[miniextendr] impl on an enum

#[derive(Copy, Clone, Debug, ExternalPtr)]
pub enum Mode {
    Error,
    Item,
    Output,
    Value,
}

#[miniextendr]
impl Mode {
    pub fn new(error: bool) -> Self {
        if error { Mode::Error } else { Mode::Value }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Mode::Error)
    }

    pub fn to_output(&self) -> Mode {
        Mode::Output
    }
}

#[miniextendr]
pub fn mode_item() -> Mode {
    Mode::Item
}

/// A `#[miniextendr]` trait with associated types named like `Mode`'s
/// variants. The trait's own `Self::Output` is unambiguous (its `Self` is
/// generic); the impl spells the concrete type, as a hand-written impl for an
/// enum with an `Output` variant has to.
#[miniextendr]
pub trait Describe {
    type Output;
    type Error;

    fn describe(&self) -> Self::Output;
}

#[miniextendr]
impl Describe for Mode {
    type Output = String;
    type Error = String;

    fn describe(&self) -> String {
        format!("{self:?}")
    }
}

#[miniextendr]
impl miniextendr_api::adapter_traits::RIterator for Mode {
    type Item = i32;

    fn next(&self) -> Option<i32> {
        None
    }

    #[miniextendr(skip)]
    fn size_hint(&self) -> (i64, Option<i64>) {
        (0, Some(0))
    }
}

// endregion

// region: PreferExternalPtr on an enum

#[derive(Copy, Clone, Debug, PreferExternalPtr)]
pub enum Boxed {
    Error,
    Value,
}

// A hand-written `TypedExternal` without the `IntoExternalPtr` marker, so the
// derive's `IntoR` is the only one.
impl miniextendr_api::externalptr::TypedExternal for Boxed {
    const TYPE_NAME: &'static str = "Boxed";
    const TYPE_NAME_CSTR: &'static [u8] = b"Boxed\0";
    const TYPE_ID_CSTR: &'static [u8] = b"assoc_item_variant_names::Boxed\0";
}

#[miniextendr]
pub fn boxed_error() -> Boxed {
    Boxed::Error
}

// endregion

fn main() {}
