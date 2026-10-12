//! #2020: generated method calls, and the importing crate's own std method
//! calls, must stay unambiguous under `use miniextendr_api::*` or a named
//! import of the adapter traits.
//!
//! The glob brings in the crate's adapter traits, whose blanket impls cover
//! every `Clone`, `Debug`, `Default`, `Hash`, `Ord`, `PartialOrd` or `FromStr`
//! type and every exact-size collection. When their methods had std's names
//! (`RClone::clone`, `RDefault::default`, `RHash::hash`, `ROrd::cmp`,
//! `RPartialOrd::partial_cmp`, `RFromStr::from_str`,
//! `RToVec::{len, is_empty, to_vec}`), any `x.clone()` named two applicable
//! methods (E0034), and on an array or a `Box<[T]>`, `x.len()` found
//! `RToVec::len` (an `i64`) before the slice's. Those methods are now
//! `r_`-prefixed in Rust (`r_clone`, ...) and keep the std names in R. On an
//! array or a `Box<[T]>` of an R-native element type, `AltrepLen::len` (a
//! `usize`) still resolves before the slice's.
//!
//! This module glob-imports the crate, then invokes every derive, attribute
//! and function-like macro (`common/every_macro.rs`), trait and inherent
//! methods named after the glob's methods, the adapter traits' empty impls,
//! and user code calling the std methods. The `named` module imports the
//! adapter traits by name instead. rpkg's `glob_import_tests.rs` covers the
//! paths that need features this crate doesn't have.
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

// region: the adapter traits' own impls

// Empty impls expand from the trait's metadata, which carries the R names
// (`clone`, `default`, `hash`, `cmp`, `partial_cmp`, `from_str`) of the
// `r_`-prefixed Rust methods. The impls name the trait by its module path,
// where its vtable items live; the glob brings in only the traits.
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
pub struct Adapted {
    pub count: i32,
}

impl ::core::str::FromStr for Adapted {
    type Err = ::core::num::ParseIntError;

    fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
        <i32 as ::core::str::FromStr>::from_str(s).map(|count| Adapted { count })
    }
}

#[miniextendr(env)]
impl Adapted {
    pub fn new(count: i32) -> Self {
        Adapted { count }
    }
}

#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RClone for Adapted {}
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RDefault for Adapted {}
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RHash for Adapted {}
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::ROrd for Adapted {}
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RPartialOrd for Adapted {}
#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RFromStr for Adapted {}

// The generic traits take an impl with a body, which repeats the R names.
#[derive(ExternalPtr)]
pub struct Bag(::std::cell::RefCell<::std::vec::Vec<i32>>);

#[miniextendr(env)]
impl Bag {
    pub fn new() -> Self {
        Bag(::std::cell::RefCell::new(::std::vec::Vec::new()))
    }
}

#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RToVec<i32> for Bag {
    #[miniextendr(r_name = "to_vec")]
    fn r_to_vec(&self) -> ::std::vec::Vec<i32> {
        self.0.borrow().r_to_vec()
    }
    #[miniextendr(r_name = "len")]
    fn r_len(&self) -> i64 {
        self.0.borrow().r_len()
    }
    #[miniextendr(r_name = "is_empty")]
    fn r_is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

#[miniextendr(env)]
impl miniextendr_api::adapter_traits::RExtend<i32> for Bag {
    fn extend_from_vec(&self, items: ::std::vec::Vec<i32>) {
        self.0.borrow_mut().extend(items);
    }
    #[miniextendr(r_name = "len")]
    fn r_len(&self) -> i64 {
        self.0.borrow().r_len()
    }
    #[miniextendr(r_name = "is_empty")]
    fn r_is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

// endregion

// region: user code calling std methods next to the glob

// Each call names its std method only, and the annotations pin std's result
// types. `Word` has no `AltrepLen` impl, so its arrays and boxed slices reach
// the slice's inherent `len` / `is_empty` / `to_vec`.

#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::default::Default,
    ::core::hash::Hash,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::core::cmp::PartialOrd,
    ::core::cmp::Ord,
)]
pub struct Version(u32, u32);

impl ::core::str::FromStr for Version {
    type Err = ::std::string::String;

    fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
        let (major, minor) = s.split_once('.').ok_or("expected major.minor")?;
        let major = major.parse().map_err(|_| "bad major")?;
        let minor = minor.parse().map_err(|_| "bad minor")?;
        ::core::result::Result::Ok(Version(major, minor))
    }
}

#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::cmp::PartialEq)]
pub struct Word(&'static str);

/// Calls every std method an adapter trait used to shadow. Expanded once at
/// the top level (the glob) and once in `named`.
macro_rules! std_method_calls {
    () => {
        pub fn std_method_calls() {
            use ::core::cmp::Ordering;
            use ::core::hash::{Hash, Hasher};
            use ::core::str::FromStr;
            use ::std::hash::DefaultHasher;
            use ::std::string::String;
            use ::std::vec::Vec;

            let v = Version(1, 2);
            let w: Version = v.clone();
            let d: Version = Version::default();
            let s: String = String::default();
            let ord: Ordering = v.cmp(&w);
            let partial: ::core::option::Option<Ordering> = 1.5f64.partial_cmp(&2.0);
            let mut hasher = DefaultHasher::new();
            v.hash(&mut hasher);
            let _: u64 = hasher.finish();
            let parsed: ::core::result::Result<Version, String> = Version::from_str("3.4");
            let n: ::core::result::Result<i32, ::core::num::ParseIntError> = i32::from_str("7");
            ::core::assert_eq!((d, s, ord), (Version(0, 0), String::new(), Ordering::Equal));
            ::core::assert_eq!(partial, ::core::option::Option::Some(Ordering::Less));
            ::core::assert_eq!(parsed, ::core::result::Result::Ok(Version(3, 4)));
            ::core::assert_eq!(n, ::core::result::Result::Ok(7));

            // `AltrepLen` covers `i32`: its `len` comes first, also a `usize`.
            let ints = [1i32, 2, 3];
            let ints_len: usize = ints.len();
            let ints_empty: bool = ints.is_empty();
            let ints_vec: Vec<i32> = ints.to_vec();
            let boxed: Box<[i32]> = ::std::vec![4, 5].into_boxed_slice();
            let boxed_len: usize = boxed.len();
            let boxed_vec: Vec<i32> = boxed.to_vec();
            ::core::assert_eq!((ints_len, ints_empty, boxed_len), (3, false, 2));
            ::core::assert_eq!((ints_vec, boxed_vec), (::std::vec![1, 2, 3], ::std::vec![4, 5]));

            let words = [Word("a"), Word("b")];
            let words_len: usize = words.len();
            let words_empty: bool = words.is_empty();
            let words_vec: Vec<Word> = words.to_vec();
            let boxed_words: Box<[Word]> = ::std::vec![Word("c")].into_boxed_slice();
            let boxed_words_len: usize = boxed_words.len();
            let boxed_words_vec: Vec<Word> = boxed_words.to_vec();
            ::core::assert_eq!((words_len, words_empty, words_vec.len()), (2, false, 2));
            ::core::assert_eq!(boxed_words_len, 1);
            ::core::assert_eq!(boxed_words_vec, ::std::vec![Word("c")]);
        }
    };
}

std_method_calls!();

/// User code that imports the adapter traits by name. No call resolves to
/// them, which is the point, so the imports are unused.
mod named {
    use super::{Version, Word};
    #[allow(unused_imports)]
    use miniextendr_api::{RClone, RDefault, RExtend, RFromStr, RHash, ROrd, RPartialOrd, RToVec};

    std_method_calls!();
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

fn main() {
    std_method_calls();
    named::std_method_calls();
}
