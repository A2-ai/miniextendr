//! Cardinality × payload-shape matrix for enum-derived data frames.
//!
//! For each currently-supported payload shape, exercise four cardinality cells
//! against both `to_dataframe` (align/NA-fill) and `to_dataframe_split` (per-variant
//! partition):
//!   - 1v1r: one variant, one row  → split returns a 1-row data.frame in that
//!     variant's slot, and a 0-row data.frame in every other slot
//!   - 1vNr: one variant, many rows
//!   - Nv1r: many variants, one row each
//!   - NvNr: many variants, many rows each
//!
//! A single-variant enum is also exposed for 1v1r / 1vNr to verify the
//! bare-data.frame return path of `to_dataframe_split`.
//!
//! `HashMap`/`BTreeMap` enum payloads are exercised in the map-fields section (issue #457).
//! Nested-enum payloads and struct-in-variant payloads are tracked by GH issues #458 / #459.
//! `&str` and `&[T]` enum payloads are exercised in the borrowed-string section below.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use miniextendr_api::convert::ToDataFrame;
use miniextendr_api::{DataFrameRow, List, miniextendr};

// region: 0a. Vec<i32> opaque (no expand/width → list-column) ─────────────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum VecOpaqueEvent {
    Items {
        label: String,
        items: Vec<i32>,
    },
    NoItems {
        label: String,
    },
}

fn vec_opaque_payload(label: &str, items: Vec<i32>) -> VecOpaqueEvent {
    VecOpaqueEvent::Items {
        label: label.into(),
        items,
    }
}

#[miniextendr]
pub fn vec_opaque_split_1v1r() -> List {
    VecOpaqueEvent::to_dataframe_split(vec![vec_opaque_payload("a", vec![1, 2, 3])])
}

#[miniextendr]
pub fn vec_opaque_split_1vnr() -> List {
    VecOpaqueEvent::to_dataframe_split(vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        vec_opaque_payload("b", vec![4, 5]),
        vec_opaque_payload("c", vec![]),
    ])
}

#[miniextendr]
pub fn vec_opaque_split_nv1r() -> List {
    VecOpaqueEvent::to_dataframe_split(vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
    ])
}

#[miniextendr]
pub fn vec_opaque_align_nvnr() -> ToDataFrame<VecOpaqueEventDataFrame> {
    ToDataFrame(VecOpaqueEvent::to_dataframe(vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
        vec_opaque_payload("c", vec![4, 5]),
        VecOpaqueEvent::NoItems { label: "d".into() },
    ]))
}

#[miniextendr]
pub fn vec_opaque_split_nvnr() -> List {
    VecOpaqueEvent::to_dataframe_split(vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
        vec_opaque_payload("c", vec![4, 5]),
        VecOpaqueEvent::NoItems { label: "d".into() },
    ])
}

// endregion

// region: 0b. HashSet<String> opaque (list-column, unordered elements) ─────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum HashSetEvent {
    Tagged {
        id: i32,
        tags: HashSet<String>,
    },
    Untagged {
        id: i32,
    },
}

fn hashset_payload(id: i32, tags: &[&str]) -> HashSetEvent {
    HashSetEvent::Tagged {
        id,
        tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}

#[miniextendr]
pub fn hashset_split_1v1r() -> List {
    HashSetEvent::to_dataframe_split(vec![hashset_payload(1, &["a", "b"])])
}

#[miniextendr]
pub fn hashset_split_1vnr() -> List {
    HashSetEvent::to_dataframe_split(vec![
        hashset_payload(1, &["a", "b"]),
        hashset_payload(2, &["c"]),
        hashset_payload(3, &[]),
    ])
}

#[miniextendr]
pub fn hashset_split_nv1r() -> List {
    HashSetEvent::to_dataframe_split(vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
    ])
}

#[miniextendr]
pub fn hashset_align_nvnr() -> ToDataFrame<HashSetEventDataFrame> {
    ToDataFrame(HashSetEvent::to_dataframe(vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
        hashset_payload(3, &["c"]),
        HashSetEvent::Untagged { id: 4 },
    ]))
}

#[miniextendr]
pub fn hashset_split_nvnr() -> List {
    HashSetEvent::to_dataframe_split(vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
        hashset_payload(3, &["c"]),
        HashSetEvent::Untagged { id: 4 },
    ])
}

// endregion

// region: 0c. BTreeSet<i32> opaque (list-column, sorted elements) ──────────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BTreeSetEvent {
    Cats {
        label: String,
        cats: BTreeSet<i32>,
    },
    NoCats {
        label: String,
    },
}

fn btreeset_payload(label: &str, cats: &[i32]) -> BTreeSetEvent {
    BTreeSetEvent::Cats {
        label: label.into(),
        cats: cats.iter().copied().collect(),
    }
}

#[miniextendr]
pub fn btreeset_split_1v1r() -> List {
    BTreeSetEvent::to_dataframe_split(vec![btreeset_payload("a", &[3, 1, 2])])
}

#[miniextendr]
pub fn btreeset_split_1vnr() -> List {
    BTreeSetEvent::to_dataframe_split(vec![
        btreeset_payload("a", &[3, 1, 2]),
        btreeset_payload("b", &[5, 4]),
        btreeset_payload("c", &[]),
    ])
}

#[miniextendr]
pub fn btreeset_split_nv1r() -> List {
    BTreeSetEvent::to_dataframe_split(vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
    ])
}

#[miniextendr]
pub fn btreeset_align_nvnr() -> ToDataFrame<BTreeSetEventDataFrame> {
    ToDataFrame(BTreeSetEvent::to_dataframe(vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
        btreeset_payload("c", &[5, 4]),
        BTreeSetEvent::NoCats { label: "d".into() },
    ]))
}

#[miniextendr]
pub fn btreeset_split_nvnr() -> List {
    BTreeSetEvent::to_dataframe_split(vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
        btreeset_payload("c", &[5, 4]),
        BTreeSetEvent::NoCats { label: "d".into() },
    ])
}

// endregion

// region: 1. Vec<T> width = N (pinned expansion) ─────────────────────────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum VecWidthEvent {
    Score {
        label: String,
        #[dataframe(width = 3)]
        scores: Vec<f64>,
    },
    NoScore {
        label: String,
    },
}

fn vec_width_payload(label: &str, scores: Vec<f64>) -> VecWidthEvent {
    VecWidthEvent::Score {
        label: label.into(),
        scores,
    }
}

#[miniextendr]
pub fn vec_width_split_1v1r() -> List {
    VecWidthEvent::to_dataframe_split(vec![vec_width_payload("a", vec![1.0, 2.0, 3.0])])
}

#[miniextendr]
pub fn vec_width_split_1vnr() -> List {
    VecWidthEvent::to_dataframe_split(vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        vec_width_payload("b", vec![4.0]),
        vec_width_payload("c", vec![]),
    ])
}

#[miniextendr]
pub fn vec_width_split_nv1r() -> List {
    VecWidthEvent::to_dataframe_split(vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
    ])
}

#[miniextendr]
pub fn vec_width_split_nvnr() -> List {
    VecWidthEvent::to_dataframe_split(vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
        vec_width_payload("c", vec![4.0]),
        VecWidthEvent::NoScore { label: "d".into() },
    ])
}

#[miniextendr]
pub fn vec_width_align_nvnr() -> ToDataFrame<VecWidthEventDataFrame> {
    ToDataFrame(VecWidthEvent::to_dataframe(vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
        vec_width_payload("c", vec![4.0]),
    ]))
}

// endregion

// region: 3. Vec<T> expand (auto-expand, runtime column count) ───────────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum VecExpandEvent {
    Vals {
        label: String,
        #[dataframe(expand)]
        vals: Vec<f64>,
    },
    NoVals {
        label: String,
    },
}

fn vec_expand_payload(label: &str, vals: Vec<f64>) -> VecExpandEvent {
    VecExpandEvent::Vals {
        label: label.into(),
        vals,
    }
}

#[miniextendr]
pub fn vec_expand_split_1v1r() -> List {
    VecExpandEvent::to_dataframe_split(vec![vec_expand_payload("a", vec![1.0, 2.0])])
}

#[miniextendr]
pub fn vec_expand_split_1vnr() -> List {
    VecExpandEvent::to_dataframe_split(vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        vec_expand_payload("b", vec![3.0]),
        vec_expand_payload("c", vec![]),
    ])
}

#[miniextendr]
pub fn vec_expand_split_nv1r() -> List {
    VecExpandEvent::to_dataframe_split(vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
    ])
}

#[miniextendr]
pub fn vec_expand_split_nvnr() -> List {
    VecExpandEvent::to_dataframe_split(vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
        vec_expand_payload("c", vec![3.0]),
        VecExpandEvent::NoVals { label: "d".into() },
    ])
}

#[miniextendr]
pub fn vec_expand_align_nvnr() -> ToDataFrame<VecExpandEventDataFrame> {
    ToDataFrame(VecExpandEvent::to_dataframe(vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
        vec_expand_payload("c", vec![3.0]),
    ]))
}

// endregion

// region: 4. [T; N] (auto-expand fixed array) ────────────────────────────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum ArrayEvent {
    Coords { id: i32, coords: [f64; 2] },
    NoCoords { id: i32 },
}

fn array_payload(id: i32, coords: [f64; 2]) -> ArrayEvent {
    ArrayEvent::Coords { id, coords }
}

#[miniextendr]
pub fn array_split_1v1r() -> List {
    ArrayEvent::to_dataframe_split(vec![array_payload(1, [10.0, 20.0])])
}

#[miniextendr]
pub fn array_split_1vnr() -> List {
    ArrayEvent::to_dataframe_split(vec![
        array_payload(1, [10.0, 20.0]),
        array_payload(2, [30.0, 40.0]),
    ])
}

#[miniextendr]
pub fn array_split_nv1r() -> List {
    ArrayEvent::to_dataframe_split(vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
    ])
}

#[miniextendr]
pub fn array_split_nvnr() -> List {
    ArrayEvent::to_dataframe_split(vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
        array_payload(3, [30.0, 40.0]),
        ArrayEvent::NoCoords { id: 4 },
    ])
}

#[miniextendr]
pub fn array_align_nvnr() -> ToDataFrame<ArrayEventDataFrame> {
    ToDataFrame(ArrayEvent::to_dataframe(vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
        array_payload(3, [30.0, 40.0]),
    ]))
}

// endregion

// region: 5. Box<[T]> with expand (auto-expand) ──────────────────────────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BoxedSliceEvent {
    Buffer {
        name: String,
        #[dataframe(expand)]
        data: Box<[f64]>,
    },
    NoBuffer {
        name: String,
    },
}

fn boxed_slice_payload(name: &str, data: &[f64]) -> BoxedSliceEvent {
    BoxedSliceEvent::Buffer {
        name: name.into(),
        data: data.to_vec().into_boxed_slice(),
    }
}

#[miniextendr]
pub fn boxed_slice_split_1v1r() -> List {
    BoxedSliceEvent::to_dataframe_split(vec![boxed_slice_payload("a", &[1.0, 2.0, 3.0])])
}

#[miniextendr]
pub fn boxed_slice_split_1vnr() -> List {
    BoxedSliceEvent::to_dataframe_split(vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        boxed_slice_payload("b", &[4.0]),
        boxed_slice_payload("c", &[]),
    ])
}

#[miniextendr]
pub fn boxed_slice_split_nv1r() -> List {
    BoxedSliceEvent::to_dataframe_split(vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
    ])
}

#[miniextendr]
pub fn boxed_slice_split_nvnr() -> List {
    BoxedSliceEvent::to_dataframe_split(vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
        boxed_slice_payload("c", &[4.0]),
        BoxedSliceEvent::NoBuffer { name: "d".into() },
    ])
}

#[miniextendr]
pub fn boxed_slice_align_nvnr() -> ToDataFrame<BoxedSliceEventDataFrame> {
    ToDataFrame(BoxedSliceEvent::to_dataframe(vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
        boxed_slice_payload("c", &[4.0]),
    ]))
}

// endregion

// region: 5. Single-variant enum: bare-data.frame return from split ──────────

#[derive(Clone, Debug, DataFrameRow)]
pub enum SingletonRow {
    Only { id: i32, label: String },
}

#[miniextendr]
pub fn singleton_split_1v1r() -> List {
    SingletonRow::to_dataframe_split(vec![SingletonRow::Only {
        id: 1,
        label: "alpha".into(),
    }])
}

#[miniextendr]
pub fn singleton_split_1vnr() -> List {
    SingletonRow::to_dataframe_split(vec![
        SingletonRow::Only {
            id: 1,
            label: "alpha".into(),
        },
        SingletonRow::Only {
            id: 2,
            label: "beta".into(),
        },
        SingletonRow::Only {
            id: 3,
            label: "gamma".into(),
        },
    ])
}

// endregion

// region: 6. &str field (borrowed text → STRSXP with NA_character_) ──────────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BorrowedStrEvent<'a> {
    Named { id: i32, name: &'a str },
    Bare { id: i32 },
}

#[miniextendr]
pub fn borrowed_str_split_1v1r() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![BorrowedStrEvent::Named { id: 1, name: "alice" }];
    BorrowedStrEvent::to_dataframe_split(data)
}

#[miniextendr]
pub fn borrowed_str_split_1vnr() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named { id: 1, name: "alice" },
        BorrowedStrEvent::Named { id: 2, name: "bob" },
        BorrowedStrEvent::Named { id: 3, name: "carol" },
    ];
    BorrowedStrEvent::to_dataframe_split(data)
}

#[miniextendr]
pub fn borrowed_str_split_nv1r() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named { id: 1, name: "alice" },
        BorrowedStrEvent::Bare { id: 2 },
    ];
    BorrowedStrEvent::to_dataframe_split(data)
}

#[miniextendr]
pub fn borrowed_str_align_nvnr() -> ToDataFrame<BorrowedStrEventDataFrame<'static>> {
    ToDataFrame(BorrowedStrEvent::to_dataframe(vec![
        BorrowedStrEvent::Named { id: 1, name: "alice" },
        BorrowedStrEvent::Bare { id: 2 },
        BorrowedStrEvent::Named { id: 3, name: "carol" },
        BorrowedStrEvent::Bare { id: 4 },
    ]))
}

#[miniextendr]
pub fn borrowed_str_split_nvnr() -> List {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named { id: 1, name: "alice" },
        BorrowedStrEvent::Bare { id: 2 },
        BorrowedStrEvent::Named { id: 3, name: "carol" },
        BorrowedStrEvent::Bare { id: 4 },
    ];
    BorrowedStrEvent::to_dataframe_split(data)
}

// endregion

// region: 7. &[T] field opaque (borrowed slice → list-column with NULL) ──────

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BorrowedSliceEvent<'a> {
    Buffer { label: String, data: &'a [f64] },
    NoBuffer { label: String },
}

#[miniextendr]
pub fn borrowed_slice_split_1v1r() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> =
        vec![BorrowedSliceEvent::Buffer { label: "a".into(), data: &[1.0, 2.0, 3.0] }];
    BorrowedSliceEvent::to_dataframe_split(data)
}

#[miniextendr]
pub fn borrowed_slice_split_1vnr() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer { label: "a".into(), data: &[1.0, 2.0, 3.0] },
        BorrowedSliceEvent::Buffer { label: "b".into(), data: &[4.0] },
        BorrowedSliceEvent::Buffer { label: "c".into(), data: &[] },
    ];
    BorrowedSliceEvent::to_dataframe_split(data)
}

#[miniextendr]
pub fn borrowed_slice_split_nv1r() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer { label: "a".into(), data: &[1.0, 2.0, 3.0] },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
    ];
    BorrowedSliceEvent::to_dataframe_split(data)
}

#[miniextendr]
pub fn borrowed_slice_align_nvnr() -> ToDataFrame<BorrowedSliceEventDataFrame<'static>> {
    ToDataFrame(BorrowedSliceEvent::to_dataframe(vec![
        BorrowedSliceEvent::Buffer { label: "a".into(), data: &[1.0, 2.0, 3.0] },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
        BorrowedSliceEvent::Buffer { label: "c".into(), data: &[4.0] },
        BorrowedSliceEvent::NoBuffer { label: "d".into() },
    ]))
}

#[miniextendr]
pub fn borrowed_slice_split_nvnr() -> List {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer { label: "a".into(), data: &[1.0, 2.0, 3.0] },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
        BorrowedSliceEvent::Buffer { label: "c".into(), data: &[4.0] },
        BorrowedSliceEvent::NoBuffer { label: "d".into() },
    ];
    BorrowedSliceEvent::to_dataframe_split(data)
}

// endregion

// region: 8. Map fields (HashMap<K,V> / BTreeMap<K,V>) ─────────────────────────
//
// HashMap and BTreeMap fields expand to two parallel list-columns:
//   `<field>_keys` and `<field>_values`.
// Absent-variant rows produce NULL in both. An empty map produces integer(0)/character(0).
// Key order: BTreeMap = sorted; HashMap = non-deterministic.
// Use setequal/sort checks in R tests for HashMap, exact checks for BTreeMap.

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum HashMapEvent {
    Tally {
        label: String,
        tally: HashMap<String, i32>,
    },
    Empty {
        label: String,
    },
}

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BTreeMapEvent {
    Tally {
        label: String,
        tally: BTreeMap<String, i32>,
    },
    Empty {
        label: String,
    },
}

// region: HashMap fixtures – all 4 cardinality cells

/// 1v1r: one variant (Tally), one row.
#[miniextendr]
pub fn hashmap_split_1v1r() -> List {
    HashMapEvent::to_dataframe_split(vec![HashMapEvent::Tally {
        label: "a".into(),
        tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }])
}

/// 1vNr: one variant (Tally), multiple rows.
#[miniextendr]
pub fn hashmap_split_1vnr() -> List {
    HashMapEvent::to_dataframe_split(vec![
        HashMapEvent::Tally {
            label: "x".into(),
            tally: HashMap::from([("x".to_string(), 5i32)]),
        },
        HashMapEvent::Tally {
            label: "y".into(),
            tally: HashMap::new(), // empty map
        },
        HashMapEvent::Tally {
            label: "z".into(),
            tally: HashMap::from([("p".to_string(), 10i32), ("q".to_string(), 20i32)]),
        },
    ])
}

/// Nv1r: both variants, one row each.
#[miniextendr]
pub fn hashmap_split_nv1r() -> List {
    HashMapEvent::to_dataframe_split(vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
    ])
}

/// NvNr align: both variants, multiple rows each.
#[miniextendr]
pub fn hashmap_align_nvnr() -> ToDataFrame<HashMapEventDataFrame> {
    ToDataFrame(HashMapEvent::to_dataframe(vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
        HashMapEvent::Tally {
            label: "c".into(),
            tally: HashMap::from([("x".to_string(), 5i32)]),
        },
        HashMapEvent::Empty { label: "d".into() },
    ]))
}

/// NvNr split: both variants, multiple rows each.
#[miniextendr]
pub fn hashmap_split_nvnr() -> List {
    HashMapEvent::to_dataframe_split(vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
        HashMapEvent::Tally {
            label: "c".into(),
            tally: HashMap::from([("x".to_string(), 5i32)]),
        },
        HashMapEvent::Empty { label: "d".into() },
    ])
}

// endregion: HashMap fixtures

// region: BTreeMap fixtures – all 4 cardinality cells

/// 1v1r: one variant (Tally), one row.
#[miniextendr]
pub fn btreemap_split_1v1r() -> List {
    BTreeMapEvent::to_dataframe_split(vec![BTreeMapEvent::Tally {
        label: "a".into(),
        tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }])
}

/// 1vNr: one variant (Tally), multiple rows.
#[miniextendr]
pub fn btreemap_split_1vnr() -> List {
    BTreeMapEvent::to_dataframe_split(vec![
        BTreeMapEvent::Tally {
            label: "x".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        },
        BTreeMapEvent::Tally {
            label: "y".into(),
            tally: BTreeMap::new(), // empty map
        },
        BTreeMapEvent::Tally {
            label: "w".into(),
            tally: BTreeMap::from([("m".to_string(), 7i32)]),
        },
    ])
}

/// Nv1r: both variants, one row each.
#[miniextendr]
pub fn btreemap_split_nv1r() -> List {
    BTreeMapEvent::to_dataframe_split(vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
    ])
}

/// NvNr align: both variants, multiple rows each.
#[miniextendr]
pub fn btreemap_align_nvnr() -> ToDataFrame<BTreeMapEventDataFrame> {
    ToDataFrame(BTreeMapEvent::to_dataframe(vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
        BTreeMapEvent::Tally {
            label: "c".into(),
            tally: BTreeMap::from([("m".to_string(), 7i32)]),
        },
        BTreeMapEvent::Empty { label: "d".into() },
    ]))
}

/// NvNr split: both variants, multiple rows each.
#[miniextendr]
pub fn btreemap_split_nvnr() -> List {
    BTreeMapEvent::to_dataframe_split(vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
        BTreeMapEvent::Tally {
            label: "c".into(),
            tally: BTreeMap::from([("m".to_string(), 7i32)]),
        },
        BTreeMapEvent::Empty { label: "d".into() },
    ])
}

// endregion: BTreeMap fixtures

// endregion: Map fields

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_width_align_pinned_columns() {
        let df = VecWidthEvent::to_dataframe(vec![
            VecWidthEvent::Score {
                label: "a".into(),
                scores: vec![1.0, 2.0, 3.0],
            },
            VecWidthEvent::NoScore { label: "b".into() },
            VecWidthEvent::Score {
                label: "c".into(),
                scores: vec![4.0],
            },
        ]);
        assert_eq!(df.scores_1, vec![Some(1.0), None, Some(4.0)]);
        assert_eq!(df.scores_2, vec![Some(2.0), None, None]);
        assert_eq!(df.scores_3, vec![Some(3.0), None, None]);
    }

    #[test]
    fn vec_expand_align_runtime_columns() {
        let df = VecExpandEvent::to_dataframe(vec![
            VecExpandEvent::Vals {
                label: "a".into(),
                vals: vec![1.0, 2.0],
            },
            VecExpandEvent::NoVals { label: "b".into() },
            VecExpandEvent::Vals {
                label: "c".into(),
                vals: vec![3.0],
            },
        ]);
        // expand stores Vec<Option<Vec<T>>> in companion struct
        assert_eq!(df.vals[0], Some(vec![1.0, 2.0]));
        assert_eq!(df.vals[1], None);
        assert_eq!(df.vals[2], Some(vec![3.0]));
    }

    #[test]
    fn array_align_expanded_columns() {
        let df = ArrayEvent::to_dataframe(vec![
            ArrayEvent::Coords {
                id: 1,
                coords: [10.0, 20.0],
            },
            ArrayEvent::NoCoords { id: 2 },
            ArrayEvent::Coords {
                id: 3,
                coords: [30.0, 40.0],
            },
        ]);
        assert_eq!(df.coords_1, vec![Some(10.0), None, Some(30.0)]);
        assert_eq!(df.coords_2, vec![Some(20.0), None, Some(40.0)]);
    }

    #[test]
    fn boxed_slice_expand_companion_shape() {
        let df = BoxedSliceEvent::to_dataframe(vec![
            BoxedSliceEvent::Buffer {
                name: "a".into(),
                data: vec![1.0, 2.0, 3.0].into_boxed_slice(),
            },
            BoxedSliceEvent::NoBuffer { name: "b".into() },
        ]);
        // expand on Box<[T]> in enum stores Vec<Option<Box<[T]>>> in companion
        assert_eq!(df.data.len(), 2);
        assert_eq!(df.data[0].as_deref(), Some(&[1.0, 2.0, 3.0][..]));
        assert!(df.data[1].is_none());
    }

    #[test]
    fn singleton_split_returns_bare_dataframe_shape() {
        // Single-variant split returns a bare List (data.frame on R side),
        // not a list-of-lists. Companion check: from_rows lays out as expected.
        let df = SingletonRowDataFrame::from_rows(vec![
            SingletonRow::Only {
                id: 1,
                label: "alpha".into(),
            },
            SingletonRow::Only {
                id: 2,
                label: "beta".into(),
            },
        ]);
        assert_eq!(df.id.len(), 2);
        assert_eq!(df.label.len(), 2);
    }

    // region: Map fields Rust unit tests

    #[test]
    fn test_hashmap_event_align_companion_shape() {
        // Verify companion struct has tally_keys and tally_values with correct option shape.
        let df = HashMapEvent::to_dataframe(vec![
            HashMapEvent::Tally {
                label: "a".into(),
                tally: HashMap::from([("x".to_string(), 1i32)]),
            },
            HashMapEvent::Empty { label: "b".into() },
        ]);
        assert_eq!(df.tally_keys.len(), 2);
        assert_eq!(df.tally_values.len(), 2);
        assert!(df.tally_keys[0].is_some());
        assert!(df.tally_values[0].is_some());
        assert!(df.tally_keys[1].is_none());
        assert!(df.tally_values[1].is_none());
        // Pairwise alignment: same length within a row.
        let k = df.tally_keys[0].as_ref().unwrap();
        let v = df.tally_values[0].as_ref().unwrap();
        assert_eq!(k.len(), v.len());
    }

    #[test]
    fn test_hashmap_empty_map_row() {
        // An empty HashMap produces Some(vec![]) (not None) in both columns.
        let df = HashMapEvent::to_dataframe(vec![HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::new(),
        }]);
        assert_eq!(df.tally_keys[0], Some(vec![]));
        assert_eq!(df.tally_values[0], Some(vec![]));
    }

    #[test]
    fn test_btreemap_keys_sorted() {
        // BTreeMap preserves sorted order: keys should be ["a", "z"], values [1, 3].
        let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
        }]);
        assert_eq!(
            df.tally_keys[0].as_deref(),
            Some(vec!["a".to_string(), "z".to_string()].as_slice())
        );
        assert_eq!(df.tally_values[0].as_deref(), Some(vec![1i32, 3i32].as_slice()));
    }

    #[test]
    fn test_btreemap_empty_map_row() {
        let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::new(),
        }]);
        assert_eq!(df.tally_keys[0], Some(vec![]));
        assert_eq!(df.tally_values[0], Some(vec![]));
    }

    #[test]
    fn test_btreemap_absent_variant_is_none() {
        let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Empty { label: "b".into() }]);
        assert!(df.tally_keys[0].is_none());
        assert!(df.tally_values[0].is_none());
    }

    // endregion: Map fields Rust unit tests
}
