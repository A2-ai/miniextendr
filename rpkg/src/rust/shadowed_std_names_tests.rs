//! #1991: generated wrappers resolve their own standard-library names. This
//! module shadows them, so it compiles only if every emitted reference is
//! absolute. The UI fixture `tests/ui/pass/shadowed_standard_names.rs` covers
//! every macro the UI tests' crate can compile; this module covers the paths
//! that need features that crate lacks: the Vctrs derive (`vctrs`), the
//! parallel `DataFrameRow` methods (`rayon`) and the serde-classed `Result`
//! error and `serde_condition!` (`serde`).
//! Keep the shadow list in step with the UI fixture's.
#![allow(dead_code, unused_macros, non_camel_case_types)]

// region: shadows

macro_rules! vec {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `vec!` reached")
    };
}
macro_rules! format {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `format!` reached")
    };
}
macro_rules! format_args {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `format_args!` reached")
    };
}
macro_rules! panic {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `panic!` reached")
    };
}
macro_rules! assert {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `assert!` reached")
    };
}
macro_rules! assert_eq {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `assert_eq!` reached")
    };
}
macro_rules! assert_ne {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `assert_ne!` reached")
    };
}
macro_rules! debug_assert {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `debug_assert!` reached")
    };
}
macro_rules! debug_assert_eq {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `debug_assert_eq!` reached")
    };
}
macro_rules! debug_assert_ne {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `debug_assert_ne!` reached")
    };
}
macro_rules! unreachable {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `unreachable!` reached")
    };
}
macro_rules! unimplemented {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `unimplemented!` reached")
    };
}
macro_rules! todo {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `todo!` reached")
    };
}
macro_rules! write {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `write!` reached")
    };
}
macro_rules! writeln {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `writeln!` reached")
    };
}
macro_rules! print {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `print!` reached")
    };
}
macro_rules! println {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `println!` reached")
    };
}
macro_rules! eprint {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `eprint!` reached")
    };
}
macro_rules! eprintln {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `eprintln!` reached")
    };
}
macro_rules! dbg {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `dbg!` reached")
    };
}
macro_rules! concat {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `concat!` reached")
    };
}
macro_rules! stringify {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `stringify!` reached")
    };
}
macro_rules! matches {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `matches!` reached")
    };
}
macro_rules! env {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `env!` reached")
    };
}
macro_rules! option_env {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `option_env!` reached")
    };
}
macro_rules! include {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `include!` reached")
    };
}
macro_rules! include_str {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `include_str!` reached")
    };
}
macro_rules! include_bytes {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `include_bytes!` reached")
    };
}
macro_rules! line {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `line!` reached")
    };
}
macro_rules! column {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `column!` reached")
    };
}
macro_rules! file {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `file!` reached")
    };
}
macro_rules! module_path {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `module_path!` reached")
    };
}
macro_rules! compile_error {
    ($($t:tt)*) => {
        ::core::compile_error!("shadowed `compile_error!` reached")
    };
}

struct Some;
struct None;
struct Ok;
struct Err;
struct Option;
struct Result;
struct Box;
struct Vec;
struct String;
fn drop(_: u8) {}

trait Default {}
trait Clone {}
trait Copy {}
trait Iterator {}
trait IntoIterator {}
trait DoubleEndedIterator {}
trait ExactSizeIterator {}
trait Extend {}
trait From {}
trait Into {}
trait TryFrom {}
trait TryInto {}
trait ToString {}
trait ToOwned {}
trait AsRef {}
trait AsMut {}
trait PartialEq {}
trait Eq {}
trait PartialOrd {}
trait Ord {}
trait Drop {}
trait Send {}
trait Sync {}
trait Sized {}
trait Unpin {}
trait Fn {}
trait FnMut {}
trait FnOnce {}

mod std {}
mod core {}
mod alloc {}

// endregion

#[cfg(feature = "serde")]
use miniextendr_api::miniextendr;

/// A vctrs class derived with the standard names shadowed.
#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs)]
#[vctrs(class = "shadowed_percent", base = "double")]
pub struct ShadowedPercent {
    #[vctrs(data)]
    values: ::std::vec::Vec<f64>,
}

/// A vctrs record derived with the standard names shadowed.
#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs, miniextendr_api::PreferVctrs)]
#[vctrs(class = "shadowed_ratio", base = "record")]
pub struct ShadowedRatio {
    #[vctrs(data)]
    num: ::std::vec::Vec<i32>,
    den: ::std::vec::Vec<i32>,
}

// With `rayon`, the derive also emits the parallel conversions.
#[derive(miniextendr_api::DataFrameRow)]
#[dataframe(tag = "_kind")]
pub struct ShadowedRow {
    pub id: i32,
    #[dataframe(width = 2)]
    pub scores: ::std::vec::Vec<f64>,
    #[dataframe(expand)]
    pub extra: ::std::vec::Vec<i32>,
    #[dataframe(as_list)]
    pub listed: ::std::vec::Vec<i32>,
    pub counts: ::std::collections::HashMap<::std::string::String, i32>,
}

// Untagged, so it gets both readers: the qualified scalars read back (#2036).
#[derive(miniextendr_api::DataFrameRow)]
pub struct ShadowedReadable {
    pub id: i32,
    pub label: ::std::string::String,
    pub note: ::core::option::Option<::std::string::String>,
    pub weight: ::core::primitive::f64,
    pub coords: [::core::primitive::f64; 2],
    #[dataframe(width = 2)]
    pub names: ::std::vec::Vec<::std::string::String>,
}

#[derive(miniextendr_api::DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum ShadowedEvent {
    Value {
        value: i32,
        label: ::std::string::String,
    },
    Scores {
        #[dataframe(width = 2)]
        scores: ::std::vec::Vec<f64>,
        #[dataframe(expand)]
        extra: ::std::vec::Vec<i32>,
        counts: ::std::collections::BTreeMap<::std::string::String, i32>,
    },
    Empty,
}

/// Stand-in for a serde-tagged error enum.
#[cfg(feature = "serde")]
#[derive(Debug, crate::serde::Serialize)]
#[serde(crate = "crate::serde", tag = "kind", rename_all = "snake_case")]
pub enum ShadowedSerdeError {
    Refused { code: i32 },
}

#[cfg(feature = "serde")]
impl ::core::fmt::Display for ShadowedSerdeError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            ShadowedSerdeError::Refused { code } => ::core::write!(f, "refused with {code}"),
        }
    }
}

/// The serde-classed error's options as a named shape (#2207).
#[cfg(feature = "serde")]
const SHADOWED_SHAPE: miniextendr_api::condition::SerdeErrorShape =
    miniextendr_api::condition::SerdeErrorShape::new().prefix("shadowed");

/// Refuse `code` with a serde-classed error.
/// @param code An integer.
#[cfg(feature = "serde")]
#[miniextendr(noexport, serde_error(shape = SHADOWED_SHAPE, as_integer("code")))]
pub fn shadowed_std_names_serde(code: i32) -> ::core::result::Result<i32, ShadowedSerdeError> {
    ::core::result::Result::Err(ShadowedSerdeError::Refused { code })
}

/// Refuse `code` with the condition `serde_condition!` builds from the same
/// error (#2217), returned as `Err(RCondition)`.
/// @param code An integer.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn shadowed_std_names_serde_condition(
    code: i32,
) -> ::core::result::Result<i32, miniextendr_api::condition::RCondition> {
    ::core::result::Result::Err(miniextendr_api::serde_condition!(
        &ShadowedSerdeError::Refused { code }
    ))
}
