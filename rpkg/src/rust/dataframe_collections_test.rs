//! Test DataFrameRow with various collection types

#![allow(dead_code)]

use miniextendr_api::{DataFrameRow, IntoList};
use std::collections::{BTreeSet, HashMap, HashSet};

// Test with Vec fields (no expansion — stays opaque)
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct WithVec {
    pub ids: Vec<i32>,
    pub names: Vec<String>,
}

// Test with boxed slices (scalar — no expansion)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBoxedSlice {
    pub data: Box<[f64]>,
}

impl ::miniextendr_api::list::IntoList for WithBoxedSlice {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // Protect each value as it is built so it survives `from_raw_pairs`'s internal
        // allocations (VECSXP + names STRSXP via `charsxp`) — mirrors `#[derive(IntoList)]`.
        // SAFETY: IntoList runs on the R main thread.
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "data",
                __scope.protect_raw(self.data.into_vec().into_sexp()),
            )])
        }
    }
}

// Test with arrays — now auto-expands to suffixed columns
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithArray {
    pub coords: [f64; 3],
}

// Test with arrays + as_list to suppress expansion
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithArrayAsList {
    #[dataframe(as_list)]
    pub coords: [f64; 3],
}

impl ::miniextendr_api::list::IntoList for WithArrayAsList {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "coords",
                __scope.protect_raw(self.coords.to_vec().into_sexp()),
            )])
        }
    }
}

// Test with HashSet
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithHashSet {
    pub tags: HashSet<String>,
}

impl ::miniextendr_api::list::IntoList for WithHashSet {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        let tags_vec: Vec<String> = self.tags.into_iter().collect();
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "tags",
                __scope.protect_raw(tags_vec.into_sexp()),
            )])
        }
    }
}

// Test with BTreeSet
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBTreeSet {
    pub categories: BTreeSet<i32>,
}

impl ::miniextendr_api::list::IntoList for WithBTreeSet {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        let cats_vec: Vec<i32> = self.categories.into_iter().collect();
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![(
                "categories",
                __scope.protect_raw(cats_vec.into_sexp()),
            )])
        }
    }
}

// Test with mixed collection types — array auto-expands, vec stays opaque
#[derive(Clone, Debug, DataFrameRow)]
pub struct MixedCollections {
    pub vec_field: Vec<i32>,
    pub array_field: [f64; 2],
    pub boxed_field: Box<[String]>,
}

impl ::miniextendr_api::list::IntoList for MixedCollections {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![
                ("vec_field", __scope.protect_raw(self.vec_field.into_sexp())),
                (
                    "array_field",
                    __scope.protect_raw(self.array_field.to_vec().into_sexp()),
                ),
                (
                    "boxed_field",
                    __scope.protect_raw(self.boxed_field.into_vec().into_sexp()),
                ),
            ])
        }
    }
}

// Test skip attribute
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithSkip {
    pub name: String,
    #[dataframe(skip)]
    pub internal_id: u64,
    pub value: f64,
}

impl ::miniextendr_api::list::IntoList for WithSkip {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![
                ("name", __scope.protect_raw(self.name.into_sexp())),
                ("value", __scope.protect_raw(self.value.into_sexp())),
            ])
        }
    }
}

// Test rename attribute
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithRename {
    #[dataframe(rename = "label")]
    pub name: String,
    pub value: f64,
}

impl ::miniextendr_api::list::IntoList for WithRename {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        // SAFETY: IntoList runs on the R main thread. Protect-as-built (see WithBoxedSlice).
        unsafe {
            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
            ::miniextendr_api::List::from_raw_pairs(vec![
                ("label", __scope.protect_raw(self.name.into_sexp())),
                ("value", __scope.protect_raw(self.value.into_sexp())),
            ])
        }
    }
}

// Test Vec with width (pinned expansion)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithPinnedVec {
    pub name: String,
    #[dataframe(width = 3)]
    pub scores: Vec<f64>,
}

// Test rename on expanded array
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithRenamedArray {
    #[dataframe(rename = "pos")]
    pub coords: [f64; 2],
}

// Test enum with expanded array field
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum EventWithCoords {
    Click { id: i32, coords: [f64; 2] },
    Hover { id: i32 },
}

// Test enum with pinned Vec expansion
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum EventWithScores {
    Result {
        label: String,
        #[dataframe(width = 2)]
        scores: Vec<f64>,
    },
    Empty {
        label: String,
    },
}

// Test Vec with auto-expand (runtime column count)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithAutoExpand {
    pub name: String,
    #[dataframe(expand)]
    pub values: Vec<f64>,
}

// Test Vec with unnest alias
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithUnnest {
    pub label: String,
    #[dataframe(unnest)]
    pub items: Vec<i32>,
}

// Test enum with auto-expand
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum EventWithAutoExpand {
    Scores {
        label: String,
        #[dataframe(expand)]
        vals: Vec<f64>,
    },
    Empty {
        label: String,
    },
}

// Test Box<[T]> with auto-expand (runtime column count)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBoxedSliceExpand {
    pub name: String,
    #[dataframe(expand)]
    pub scores: Box<[f64]>,
}

// Test Box<[T]> with pinned width
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithBoxedSlicePinned {
    pub name: String,
    #[dataframe(width = 3)]
    pub coords: Box<[f64]>,
}

// Test &[T] with auto-expand (runtime column count)
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithSliceExpand<'a> {
    pub name: &'a str,
    #[dataframe(expand)]
    pub values: &'a [f64],
}

// Test &[T] with pinned width
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithSlicePinned<'a> {
    pub label: &'a str,
    #[dataframe(width = 2)]
    pub coords: &'a [f64],
}

// Test that DataFrameRow-annotated enums with borrowed fields work when using
// a struct with a lifetime (issue #460). For enums, use String/owned types
// instead of &'a str (Vec<Option<&str>> lacks IntoR in enum context).
// The struct path supports borrowed fields directly (see WithSliceExpand below).
// This enum uses only owned types to verify the enum expand path compiles.
#[derive(Clone, Debug, DataFrameRow)]
pub enum WithLifetimeEnumOwned {
    Row { label: String, value: f64 },
}

// Test enum with skip
#[derive(Clone, Debug, DataFrameRow)]
pub enum EventWithSkip {
    A {
        value: f64,
        #[dataframe(skip)]
        internal: i32,
    },
    B {
        value: f64,
    },
}

// Test parallel fill with expansion (struct)
#[derive(Clone, Debug, DataFrameRow)]
pub struct ParallelExpanded {
    pub id: i32,
    pub coords: [f64; 3],
    #[dataframe(width = 2)]
    pub tags: Vec<String>,
    #[dataframe(expand)]
    pub values: Vec<f64>,
}

// Test parallel fill with expansion (enum)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum ParallelExpandedEvent {
    Measurement { sensor: String, readings: [f64; 2] },
    Status { sensor: String, code: i32 },
}

// Test: `Option<HashMap<K, V>>` with `#[dataframe(as_list)]` — escape hatch for #484.
// The field becomes an opaque list-column (`Vec<Option<HashMap<String, i32>>>` in the
// companion struct).  Without `as_list` the macro now emits a compile error.
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithOptMapAsList {
    pub id: i32,
    #[dataframe(as_list)]
    pub counts: Option<HashMap<String, i32>>,
}

impl IntoList for WithOptMapAsList {
    fn into_list(self) -> miniextendr_api::List {
        use miniextendr_api::{IntoR, SEXP};
        // SAFETY: IntoList runs on the R main thread. Protect both values as built — the
        // pre-built `counts_sexp` must survive `self.id.into_sexp()` and `from_raw_pairs`'s
        // internal allocations. Mirrors `#[derive(IntoList)]`.
        unsafe {
            let __scope = miniextendr_api::gc_protect::ProtectScope::new();
            let counts_sexp = __scope.protect_raw(match self.counts {
                Some(m) => m.into_list().into_sexp(),
                None => SEXP::nil(),
            });
            miniextendr_api::List::from_raw_pairs(vec![
                ("id", __scope.protect_raw(self.id.into_sexp())),
                ("counts", counts_sexp),
            ])
        }
    }
}

#[cfg(test)]
mod tests;
