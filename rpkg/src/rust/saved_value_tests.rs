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
//! `&mut self` call holds it: a borrow conflict.

use crate::serde::{Deserialize, Deserializer, Serialize};
use miniextendr_api::expression::RCall;
use miniextendr_api::externalptr::{ExternalPtr, RSidecar, Sidecar};
use miniextendr_api::{SEXP, miniextendr};

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
