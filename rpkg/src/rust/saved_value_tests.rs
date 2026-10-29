//! A pointer to a serde type comes back after `readRDS()` (#2186).
//!
//! A type that derives `Serialize` and `Deserialize` keeps its value in the
//! save: R calls the carrier the pointer holds during `serialize()`, and
//! again during `unserialize()`, which decodes the value; the first access
//! attaches it. One type per class system (`SavedR6`, `SavedS7`, `SavedS4`,
//! `SavedEnv`, `SavedS3`), next to `PlainR6`, which is not serde and is
//! refused as before. `SavedPositive` refuses to decode a negative value;
//! `SavedRecord` reads a save that lacks a field it added with
//! `#[serde(default)]`, and refuses one that lacks a field without a default.
//! `SavedEnv::bump_and_call` saves the object from a callback while a
//! `&mut self` call holds it: a borrow conflict. `saved_rewrite_type` makes
//! a live object's save read as made by a type at another path, as one
//! saved before the type moved to another module would. `SavedShapes` holds
//! values the general serde-to-R mapping can't read back, which the save's
//! exact encoding keeps (#2256), and `SavedFloats` a long `Vec<f64>`, which
//! the save keeps as one double vector.

use std::collections::HashMap;

use crate::serde::{Deserialize, Deserializer, Serialize};
use miniextendr_api::expression::RCall;
use miniextendr_api::externalptr::{ExternalPtr, RSidecar, Sidecar};
use miniextendr_api::{SEXP, SEXPTYPE, SexpExt, miniextendr};

// region: One type per class system

/// An R6 class with a struct field and a `Sidecar` field: the save keeps the
/// sidecar value itself, so serde skips it.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
#[externalptr(r6)]
pub struct SavedR6 {
    #[serde(skip)]
    #[r_data]
    _r: RSidecar,
    /// The count, read through the struct-field binding.
    #[r_data]
    pub n: i32,
    /// Labels a save keeps in the pointer's protection list.
    #[serde(skip)]
    #[r_data]
    pub tags: Sidecar<Vec<String>>,
}

/// @title A serde R6 class that comes back after `readRDS()`
/// @description `SavedR6` keeps a count in Rust and tags in a sidecar field.
/// @field n The count (active binding over the struct field).
/// @field tags Labels kept in the pointer's protection list (active binding).
#[miniextendr(r6(r_data_accessors))]
impl SavedR6 {
    /// @param n The count.
    pub fn new(n: i32) -> Self {
        SavedR6 {
            _r: RSidecar,
            n,
            tags: Sidecar::new(Vec::new()),
        }
    }

    /// The count.
    pub fn value(&self) -> i32 {
        self.n
    }

    /// Adds one to the count.
    pub fn bump(&mut self) {
        self.n += 1;
    }
}

/// An S7 class.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedS7 {
    n: i32,
}

/// @title A serde S7 class that comes back after `readRDS()`
/// @description `SavedS7` keeps a count in Rust.
#[miniextendr(s7)]
impl SavedS7 {
    /// @param n The count.
    pub fn new(n: i32) -> Self {
        SavedS7 { n }
    }

    /// The count.
    pub fn saved_s7_value(&self) -> i32 {
        self.n
    }
}

/// An S4 class.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedS4 {
    n: i32,
}

/// @title A serde S4 class that comes back after `readRDS()`
/// @description `SavedS4` keeps a count in Rust.
#[miniextendr(s4)]
impl SavedS4 {
    /// @param n The count.
    pub fn new(n: i32) -> Self {
        SavedS4 { n }
    }

    /// The count.
    pub fn saved_s4_value(&self) -> i32 {
        self.n
    }
}

/// An environment class: the object is the pointer itself.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedEnv {
    n: i32,
    label: String,
}

/// @title A serde environment class that comes back after `readRDS()`
/// @description `SavedEnv` keeps a count and a label in Rust.
#[miniextendr(env)]
impl SavedEnv {
    /// @param n The count.
    /// @param label The label.
    pub fn new(n: i32, label: String) -> Self {
        SavedEnv { n, label }
    }

    /// The count.
    pub fn value(&self) -> i32 {
        self.n
    }

    /// The label.
    pub fn label(&self) -> String {
        self.label.clone()
    }

    /// Adds one to the count.
    pub fn bump(&mut self) {
        self.n += 1;
    }

    /// Adds one to the count, then calls `f()` while this call still holds
    /// the object mutably: a callback that saves the object reads it
    /// through a borrow, which conflicts.
    /// @param f A function of no arguments.
    pub fn bump_and_call(&mut self, f: SEXP) {
        self.n += 1;
        // SAFETY: on R's main thread, inside the method's boundary; an R
        // error unwinds through this frame and ends the borrow.
        unsafe {
            RCall::from_sexp(f).eval_with_handlers(miniextendr_api::sys::R_BaseEnv);
        }
    }
}

/// The count of a `SavedEnv` argument, borrowed for the call.
/// @param x A `SavedEnv`.
#[miniextendr]
pub fn saved_env_value_of(x: &SavedEnv) -> i32 {
    x.n
}

/// The count of a `SavedEnv` handle argument.
/// @param x A `SavedEnv`.
#[miniextendr]
pub fn saved_env_handle_value(x: ExternalPtr<SavedEnv>) -> i32 {
    x.borrow().n
}

/// An S3 class.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedS3 {
    n: i32,
}

/// @title A serde S3 class that comes back after `readRDS()`
/// @description `SavedS3` keeps a count in Rust.
#[miniextendr(s3)]
impl SavedS3 {
    /// @param n The count.
    pub fn new(n: i32) -> Self {
        SavedS3 { n }
    }

    /// The count.
    pub fn saved_s3_value(&self) -> i32 {
        self.n
    }
}

/// An R6 class that is not serde: refused after a reload, as before.
#[derive(miniextendr_api::ExternalPtr)]
pub struct PlainR6 {
    n: i32,
}

/// @title An R6 class that is not serde
/// @description `PlainR6` is refused after `readRDS()`.
#[miniextendr(r6)]
impl PlainR6 {
    /// @param n The count.
    pub fn new(n: i32) -> Self {
        PlainR6 { n }
    }

    /// The count.
    pub fn value(&self) -> i32 {
        self.n
    }
}

// endregion

// region: Decoding failures and other versions

/// Saves any count, but decodes only a count that is not negative.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedPositive {
    #[serde(deserialize_with = "non_negative")]
    n: i32,
}

fn non_negative<'de, D: Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
    let n = <i32 as Deserialize>::deserialize(d)?;
    if n < 0 {
        return Err(crate::serde::de::Error::custom(format!(
            "the count {n} is negative"
        )));
    }
    Ok(n)
}

/// @title A serde class whose decoding refuses a negative count
/// @description `SavedPositive` saves any count.
#[miniextendr(env)]
impl SavedPositive {
    /// @param n The count.
    pub fn new(n: i32) -> Self {
        SavedPositive { n }
    }

    /// The count.
    pub fn value(&self) -> i32 {
        self.n
    }
}

/// A record whose save leaves out an empty `note` and a zero `weight`, as a
/// version without those fields would have saved it. `note` decodes with a
/// default when the save lacks it; `weight` has none, so such a save can't
/// be read.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedRecord {
    n: i32,
    #[serde(default = "default_note", skip_serializing_if = "String::is_empty")]
    note: String,
    #[serde(skip_serializing_if = "is_zero")]
    weight: i32,
}

fn default_note() -> String {
    "added later".to_owned()
}

fn is_zero(weight: &i32) -> bool {
    *weight == 0
}

/// @title A serde record with fields an older save lacks
/// @description `SavedRecord` leaves an empty note and a zero weight out of
///   its save.
#[miniextendr(env)]
impl SavedRecord {
    /// @param n The count.
    /// @param note The note; an empty one is not saved.
    /// @param weight The weight; zero is not saved.
    pub fn new(n: i32, note: String, weight: i32) -> Self {
        SavedRecord { n, note, weight }
    }

    /// The count, note and weight, as a string.
    pub fn summary(&self) -> String {
        format!("{}/{}/{}", self.n, self.note, self.weight)
    }
}

// endregion

// region: A type at another path

/// Rewrites the type ID an external pointer keeps (`crate@version::path`),
/// keeping the version, so its next save reads as made by `crate_name`'s
/// type at `path` (`module::Type`): a type since moved to another module,
/// renamed, or of another crate. A live pointer keeps working.
/// @param x An external pointer.
/// @param crate_name The crate to write, e.g. `"miniextendr"`.
/// @param path The module path and type name to write.
#[miniextendr(noexport)]
pub fn saved_rewrite_type(x: SEXP, crate_name: String, path: String) {
    assert_eq!(
        x.type_of(),
        SEXPTYPE::EXTPTRSXP,
        "`x` is an external pointer"
    );
    // SAFETY: on R's main thread (a `#[miniextendr]` fn); `x` is an EXTPTRSXP.
    unsafe { miniextendr_api::externalptr::rewrite_stored_type_for_tests(x, &crate_name, &path) };
}

// endregion

// region: Values the general serde-to-R mapping can't read back (#2256)

/// An untagged enum whose common value is a one-element `Vec`.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "crate::serde", untagged)]
pub enum ShapeValues {
    Many(Vec<String>),
    Flag(bool),
}

/// A column and the values selected in it.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "crate::serde")]
pub struct ShapeSelector {
    name: String,
    values: Vec<String>,
}

/// An untagged enum holding a `Vec` of structs, and itself.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "crate::serde", untagged)]
pub enum ShapeScope {
    Selectors(Vec<ShapeSelector>),
    Any { any: Vec<ShapeScope> },
}

/// An internally tagged enum with a `Vec` field.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "crate::serde", tag = "kind", rename_all = "snake_case")]
pub enum ShapeNotice {
    Dropped { profiles: Vec<String> },
    Counted { n: usize },
}

/// A struct whose second part is flattened into it.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "crate::serde")]
pub struct ShapeFlat {
    label: String,
    #[serde(flatten)]
    part: ShapePart,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "crate::serde")]
pub struct ShapePart {
    values: ShapeValues,
    ids: Vec<u64>,
}

/// `NA_real_`: R's NA payload, 1954, in a NaN.
const NA_REAL_BITS: u64 = 0x7ff0_0000_0000_07a2;

/// Values the general serde-to-R mapping (`serde::to_r` / `from_r`) can't
/// read back: one-element `Vec`s in untagged and internally tagged enums and
/// a flattened struct, R's `NA` in an `f64`, `i32::MIN`, integers above
/// 2^53, maps with keys that are not strings, `Some(None)` and a string
/// holding a NUL byte. The save keeps them all.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedShapes {
    values: ShapeValues,
    scope: ShapeScope,
    nested_scope: ShapeScope,
    notices: Vec<ShapeNotice>,
    flat: ShapeFlat,
    na: f64,
    nan_payload: f64,
    neg_zero: f64,
    in_option: Option<f64>,
    reals: Vec<f64>,
    nested: Option<Option<f64>>,
    min: i32,
    above_i32: u32,
    big: u64,
    wide: i128,
    profiles: HashMap<Vec<(String, String)>, usize>,
    by_id: HashMap<u32, f64>,
    nul: String,
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|&s| s.to_owned()).collect()
}

fn bits(x: Option<f64>) -> Option<u64> {
    x.map(f64::to_bits)
}

/// @title A serde class holding values only an exact save keeps
/// @description `SavedShapes` holds one-element vectors in untagged and
///   internally tagged enums, R's `NA` in a double, wide integers, maps with
///   keys that are not strings and nested options.
#[miniextendr(env)]
impl SavedShapes {
    /// The value with every shape.
    pub fn new() -> Self {
        let selector = ShapeSelector {
            name: "ID".to_owned(),
            values: strings(&["1"]),
        };
        SavedShapes {
            values: ShapeValues::Many(strings(&["1"])),
            scope: ShapeScope::Selectors(vec![ShapeSelector {
                name: "ID".to_owned(),
                values: strings(&["1"]),
            }]),
            nested_scope: ShapeScope::Any {
                any: vec![ShapeScope::Selectors(vec![selector])],
            },
            notices: vec![
                ShapeNotice::Dropped {
                    profiles: strings(&["ID=1"]),
                },
                ShapeNotice::Counted { n: 3 },
            ],
            flat: ShapeFlat {
                label: "flat".to_owned(),
                part: ShapePart {
                    values: ShapeValues::Many(strings(&["2"])),
                    ids: vec![9_007_199_254_740_993],
                },
            },
            na: f64::from_bits(NA_REAL_BITS),
            nan_payload: f64::from_bits(0x7ff8_0000_0000_0123),
            neg_zero: -0.0,
            in_option: Some(f64::from_bits(NA_REAL_BITS)),
            reals: vec![f64::from_bits(NA_REAL_BITS), -0.0, f64::INFINITY],
            nested: Some(None),
            min: i32::MIN,
            above_i32: u32::MAX,
            big: 9_007_199_254_740_993,
            wide: i128::MIN,
            profiles: [(vec![("ID".to_owned(), "1".to_owned())], 0)].into(),
            by_id: [(1, 0.5), (u32::MAX, -0.0)].into(),
            nul: "a\0b".to_owned(),
        }
    }

    /// The names of the fields whose value differs from a new object's,
    /// doubles compared by their bits: `character(0)` when every field
    /// matches.
    pub fn mismatches(&self) -> Vec<String> {
        let new = Self::new();
        let mut out = Vec::new();
        let mut check = |name: &str, same: bool| {
            if !same {
                out.push(name.to_owned());
            }
        };
        check("values", self.values == new.values);
        check("scope", self.scope == new.scope);
        check("nested_scope", self.nested_scope == new.nested_scope);
        check("notices", self.notices == new.notices);
        check("flat", self.flat == new.flat);
        check("na", self.na.to_bits() == new.na.to_bits());
        check(
            "nan_payload",
            self.nan_payload.to_bits() == new.nan_payload.to_bits(),
        );
        check(
            "neg_zero",
            self.neg_zero.to_bits() == new.neg_zero.to_bits(),
        );
        check("in_option", bits(self.in_option) == bits(new.in_option));
        check(
            "reals",
            self.reals
                .iter()
                .map(|x| x.to_bits())
                .eq(new.reals.iter().map(|x| x.to_bits())),
        );
        check("nested", self.nested.map(bits) == new.nested.map(bits));
        check("min", self.min == new.min);
        check("above_i32", self.above_i32 == new.above_i32);
        check("big", self.big == new.big);
        check("wide", self.wide == new.wide);
        check("profiles", self.profiles == new.profiles);
        check(
            "by_id",
            self.by_id.len() == new.by_id.len()
                && self
                    .by_id
                    .iter()
                    .all(|(k, v)| new.by_id.get(k).map(|x| x.to_bits()) == Some(v.to_bits())),
        );
        check("nul", self.nul == new.nul);
        out
    }
}

/// A long `Vec<f64>`: the save keeps it as one double vector.
#[derive(miniextendr_api::ExternalPtr, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct SavedFloats {
    values: Vec<f64>,
}

/// @title A serde class holding a long vector of doubles
/// @description `SavedFloats` holds `0, 1, ..., n - 1`.
#[miniextendr(env)]
impl SavedFloats {
    /// @param n The number of values.
    pub fn new(n: i32) -> Self {
        SavedFloats {
            values: (0..n).map(f64::from).collect(),
        }
    }

    /// Whether the values are `0, 1, ..., n - 1` for `n` values.
    /// @param n The number of values expected.
    pub fn holds(&self, n: i32) -> bool {
        self.values.iter().copied().eq((0..n).map(f64::from))
    }
}

// endregion
