//! #2020: generated method calls must stay unambiguous under
//! `use miniextendr_api::*`.
//!
//! The glob brings in the crate's adapter traits, whose blanket impls cover
//! every `Clone`, `Debug`, `Default`, `Hash`, `Ord` or `PartialOrd` type and
//! every exact-size collection: `RClone::clone`, `RDebug::debug_str`,
//! `RDefault::default`, `RHash::hash`, `ROrd::cmp`, `RPartialOrd::partial_cmp`,
//! and `RToVec::{len, is_empty, to_vec}`. A generated `x.clone()` then names two
//! applicable methods (E0034), and on an array or a `Box<[T]>`, `x.len()` finds
//! `RToVec::len` (an `i64`) and `AltrepLen::len` before the slice's. This module
//! glob-imports the crate, then invokes every derive, attribute and
//! function-like macro (`common/every_macro.rs`) plus trait and inherent
//! methods named after the glob's methods. rpkg's `glob_import_tests.rs`
//! covers the paths that need features this crate doesn't have.
#![allow(dead_code)]

use miniextendr_api::*;

include!("common/every_macro.rs");

// region: methods named after the glob's blanket-trait methods

// A trait whose methods share their names with the glob's blanket traits. Each
// class system's trait-impl wrapper calls them on a type the blanket impls
// also cover, so a method-call-syntax call in the wrapper is ambiguous.
#[miniextendr]
pub trait Collide {
    fn hash(&self) -> i32;
    fn debug_str(&self) -> ::std::string::String;
    fn cmp(&self, other: i32) -> i32;
    fn default() -> i32;
}

#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::default::Default,
    ::core::hash::Hash,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::cmp::PartialOrd,
    ::core::cmp::Ord,
    ExternalPtr,
)]
pub struct TallyEnv {
    #[r_data]
    pub count: i32,
    #[r_data]
    pub label: ::std::string::String,
}

#[miniextendr(env)]
impl TallyEnv {
    pub fn new(count: i32) -> Self {
        TallyEnv { count, ..<Self as ::core::default::Default>::default() }
    }
    pub fn len(&self) -> i32 {
        self.count
    }
    pub fn to_vec(&self) -> ::std::vec::Vec<i32> {
        let mut out = ::std::vec::Vec::new();
        out.push(self.count);
        out
    }
}

#[miniextendr(env)]
impl Collide for TallyEnv {
    fn hash(&self) -> i32 {
        self.count
    }
    fn debug_str(&self) -> ::std::string::String {
        <i32 as ::std::string::ToString>::to_string(&self.count)
    }
    fn cmp(&self, other: i32) -> i32 {
        self.count - other
    }
    fn default() -> i32 {
        0
    }
}

#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::default::Default,
    ::core::hash::Hash,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::cmp::PartialOrd,
    ::core::cmp::Ord,
    ExternalPtr,
)]
pub struct TallyS3 {
    pub count: i32,
}

#[miniextendr(s3)]
impl TallyS3 {
    pub fn new(count: i32) -> Self {
        TallyS3 { count }
    }
    pub fn len(&self) -> i32 {
        self.count
    }
}

#[miniextendr(s3)]
impl Collide for TallyS3 {
    fn hash(&self) -> i32 {
        self.count
    }
    fn debug_str(&self) -> ::std::string::String {
        <i32 as ::std::string::ToString>::to_string(&self.count)
    }
    fn cmp(&self, other: i32) -> i32 {
        self.count - other
    }
    fn default() -> i32 {
        0
    }
}

#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::default::Default,
    ::core::hash::Hash,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::cmp::PartialOrd,
    ::core::cmp::Ord,
    ExternalPtr,
)]
pub struct TallyR6 {
    #[r_data]
    pub count: i32,
}

#[miniextendr(r6(r_data_accessors))]
impl TallyR6 {
    pub fn new(count: i32) -> Self {
        TallyR6 { count }
    }
    pub fn len(&self) -> i32 {
        self.count
    }
}

#[miniextendr(r6)]
impl Collide for TallyR6 {
    fn hash(&self) -> i32 {
        self.count
    }
    fn debug_str(&self) -> ::std::string::String {
        <i32 as ::std::string::ToString>::to_string(&self.count)
    }
    fn cmp(&self, other: i32) -> i32 {
        self.count - other
    }
    fn default() -> i32 {
        0
    }
}

#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::default::Default,
    ::core::hash::Hash,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::cmp::PartialOrd,
    ::core::cmp::Ord,
    ExternalPtr,
)]
pub struct TallyS4 {
    pub count: i32,
}

#[miniextendr(s4)]
impl TallyS4 {
    pub fn new(count: i32) -> Self {
        TallyS4 { count }
    }
    pub fn len(&self) -> i32 {
        self.count
    }
}

#[miniextendr(s4)]
impl Collide for TallyS4 {
    fn hash(&self) -> i32 {
        self.count
    }
    fn debug_str(&self) -> ::std::string::String {
        <i32 as ::std::string::ToString>::to_string(&self.count)
    }
    fn cmp(&self, other: i32) -> i32 {
        self.count - other
    }
    fn default() -> i32 {
        0
    }
}

#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::default::Default,
    ::core::hash::Hash,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::cmp::PartialOrd,
    ::core::cmp::Ord,
    ExternalPtr,
)]
pub struct TallyS7 {
    #[r_data]
    pub count: i32,
}

#[miniextendr(s7(r_data_accessors))]
impl TallyS7 {
    pub fn new(count: i32) -> Self {
        TallyS7 { count }
    }
    pub fn len(&self) -> i32 {
        self.count
    }
}

#[miniextendr(s7)]
impl Collide for TallyS7 {
    fn hash(&self) -> i32 {
        self.count
    }
    fn debug_str(&self) -> ::std::string::String {
        <i32 as ::std::string::ToString>::to_string(&self.count)
    }
    fn cmp(&self, other: i32) -> i32 {
        self.count - other
    }
    fn default() -> i32 {
        0
    }
}

// Sidecar fields of every scalar kind: their generated setters call
// `SexpExt` methods and their getters clone the field.
#[derive(::core::clone::Clone, ExternalPtr)]
pub struct Slots {
    #[r_data]
    pub count: i32,
    #[r_data]
    pub ratio: f64,
    #[r_data]
    pub flag: bool,
    #[r_data]
    pub byte: u8,
    #[r_data]
    pub label: ::std::string::String,
}

#[miniextendr(env)]
impl Slots {
    pub fn new(count: i32) -> Self {
        Slots { count, ratio: 0.0, flag: false, byte: 0, label: ::std::string::String::new() }
    }
}

// endregion

// region: arrays and boxed slices, whose `len` is `AltrepLen`'s and `RToVec`'s
// before it is the slice's

// No `tag`: a tagged struct has no reader.
#[derive(::core::clone::Clone, DataFrameRow)]
pub struct Sample {
    pub id: i32,
    pub coords: [f64; 3],
    #[dataframe(expand)]
    pub corners: [i32; 2],
    pub weights: ::std::vec::Vec<f64>,
    pub inner: Inner,
}

#[derive(::core::clone::Clone, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum Reading {
    Point { id: i32, coords: [f64; 2], inner: Inner },
    Batch {
        #[dataframe(width = 2)]
        values: ::std::vec::Vec<f64>,
        // Bare `String`: the reader takes only a bare scalar name as a key.
        counts: ::std::collections::HashMap<String, i32>,
        sorted: ::std::collections::BTreeMap<String, i32>,
    },
    Empty,
}

#[miniextendr]
pub fn samples(frame: SEXP) -> SampleDataFrame {
    let rows = Sample::try_from_dataframe(frame).unwrap_or_default();
    Sample::to_dataframe(rows)
}

#[miniextendr]
pub fn readings(frame: SEXP) -> i32 {
    match Reading::try_from_dataframe(frame) {
        ::core::result::Result::Ok(rows) => i32::try_from(rows.len()).unwrap_or(i32::MAX),
        ::core::result::Result::Err(_) => -1,
    }
}

#[miniextendr]
pub fn array_args(fixed: [i32; 3], floats: [f64; 2]) -> [f64; 2] {
    let _ = fixed;
    floats
}

// `AltrepLen` and `RToVec` both cover `Box<[T]>` too.
#[miniextendr]
pub fn boxed(xs: Box<[i32]>, ys: Box<[f64]>) -> Box<[f64]> {
    let _ = xs;
    ys
}

// endregion

miniextendr_api::miniextendr_init!(glob_import);

fn main() {}
