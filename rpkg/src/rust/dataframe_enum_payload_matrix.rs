//! Cardinality × payload-shape matrix for enum-derived data frames.
//!
//! For each currently-supported payload shape, exercise four cardinality cells
//! against both `to_dataframe` (align/NA-fill) and `into_dataframe_split`
//! (per-variant partition, from the `IntoDataFrameSplit` trait):
//!   - 1v1r: one variant, one row  → split returns a 1-row data.frame in that
//!     variant's slot, and a 0-row data.frame in every other slot
//!   - 1vNr: one variant, many rows
//!   - Nv1r: many variants, one row each
//!   - NvNr: many variants, many rows each
//!
//! A single-variant enum is also exposed for 1v1r / 1vNr to verify the
//! bare-data.frame return path of `into_dataframe_split`.
//!
//! `HashMap`/`BTreeMap` enum payloads are exercised in the map-fields section (issue #457).
//! Nested-enum payloads and struct-in-variant payloads are tracked by GH issues #458 / #459.
//! `&str` and `&[T]` enum payloads are exercised in the borrowed-string section below.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use miniextendr_api::{BuiltDataFrame, DataFrameShape, IntoDataFrame, IntoDataFrameSplit};
use miniextendr_api::{DataFrameRow, IntoList, miniextendr};

// region: 0a. Vec<i32> opaque (no expand/width → list-column)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum VecOpaqueEvent {
    Items { label: String, items: Vec<i32> },
    NoItems { label: String },
}

fn vec_opaque_payload(label: &str, items: Vec<i32>) -> VecOpaqueEvent {
    VecOpaqueEvent::Items {
        label: label.into(),
        items,
    }
}

#[miniextendr]
pub fn vec_opaque_split_1v1r() -> DataFrameShape {
    vec![vec_opaque_payload("a", vec![1, 2, 3])].into_dataframe_split()
}

#[miniextendr]
pub fn vec_opaque_split_1vnr() -> DataFrameShape {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        vec_opaque_payload("b", vec![4, 5]),
        vec_opaque_payload("c", vec![]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_opaque_split_nv1r() -> DataFrameShape {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_opaque_align_nvnr() -> BuiltDataFrame {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
        vec_opaque_payload("c", vec![4, 5]),
        VecOpaqueEvent::NoItems { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn vec_opaque_split_nvnr() -> DataFrameShape {
    vec![
        vec_opaque_payload("a", vec![1, 2, 3]),
        VecOpaqueEvent::NoItems { label: "b".into() },
        vec_opaque_payload("c", vec![4, 5]),
        VecOpaqueEvent::NoItems { label: "d".into() },
    ]
    .into_dataframe_split()
}

// endregion

// region: 0b. HashSet<String> opaque (list-column, unordered elements)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum HashSetEvent {
    Tagged { id: i32, tags: HashSet<String> },
    Untagged { id: i32 },
}

fn hashset_payload(id: i32, tags: &[&str]) -> HashSetEvent {
    HashSetEvent::Tagged {
        id,
        tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}

#[miniextendr]
pub fn hashset_split_1v1r() -> DataFrameShape {
    vec![hashset_payload(1, &["a", "b"])].into_dataframe_split()
}

#[miniextendr]
pub fn hashset_split_1vnr() -> DataFrameShape {
    vec![
        hashset_payload(1, &["a", "b"]),
        hashset_payload(2, &["c"]),
        hashset_payload(3, &[]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn hashset_split_nv1r() -> DataFrameShape {
    vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn hashset_align_nvnr() -> BuiltDataFrame {
    vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
        hashset_payload(3, &["c"]),
        HashSetEvent::Untagged { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn hashset_split_nvnr() -> DataFrameShape {
    vec![
        hashset_payload(1, &["a", "b"]),
        HashSetEvent::Untagged { id: 2 },
        hashset_payload(3, &["c"]),
        HashSetEvent::Untagged { id: 4 },
    ]
    .into_dataframe_split()
}

// endregion

// region: 0c. BTreeSet<i32> opaque (list-column, sorted elements)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BTreeSetEvent {
    Cats { label: String, cats: BTreeSet<i32> },
    NoCats { label: String },
}

fn btreeset_payload(label: &str, cats: &[i32]) -> BTreeSetEvent {
    BTreeSetEvent::Cats {
        label: label.into(),
        cats: cats.iter().copied().collect(),
    }
}

#[miniextendr]
pub fn btreeset_split_1v1r() -> DataFrameShape {
    vec![btreeset_payload("a", &[3, 1, 2])].into_dataframe_split()
}

#[miniextendr]
pub fn btreeset_split_1vnr() -> DataFrameShape {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        btreeset_payload("b", &[5, 4]),
        btreeset_payload("c", &[]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn btreeset_split_nv1r() -> DataFrameShape {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn btreeset_align_nvnr() -> BuiltDataFrame {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
        btreeset_payload("c", &[5, 4]),
        BTreeSetEvent::NoCats { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn btreeset_split_nvnr() -> DataFrameShape {
    vec![
        btreeset_payload("a", &[3, 1, 2]),
        BTreeSetEvent::NoCats { label: "b".into() },
        btreeset_payload("c", &[5, 4]),
        BTreeSetEvent::NoCats { label: "d".into() },
    ]
    .into_dataframe_split()
}

// endregion

// region: 1. Vec<T> width = N (pinned expansion)
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
pub fn vec_width_split_1v1r() -> DataFrameShape {
    vec![vec_width_payload("a", vec![1.0, 2.0, 3.0])].into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_split_1vnr() -> DataFrameShape {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        vec_width_payload("b", vec![4.0]),
        vec_width_payload("c", vec![]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_split_nv1r() -> DataFrameShape {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_split_nvnr() -> DataFrameShape {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
        vec_width_payload("c", vec![4.0]),
        VecWidthEvent::NoScore { label: "d".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_width_align_nvnr() -> BuiltDataFrame {
    vec![
        vec_width_payload("a", vec![1.0, 2.0, 3.0]),
        VecWidthEvent::NoScore { label: "b".into() },
        vec_width_payload("c", vec![4.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 3. Vec<T> expand (auto-expand, runtime column count)
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
pub fn vec_expand_split_1v1r() -> DataFrameShape {
    vec![vec_expand_payload("a", vec![1.0, 2.0])].into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_split_1vnr() -> DataFrameShape {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        vec_expand_payload("b", vec![3.0]),
        vec_expand_payload("c", vec![]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_split_nv1r() -> DataFrameShape {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_split_nvnr() -> DataFrameShape {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
        vec_expand_payload("c", vec![3.0]),
        VecExpandEvent::NoVals { label: "d".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn vec_expand_align_nvnr() -> BuiltDataFrame {
    vec![
        vec_expand_payload("a", vec![1.0, 2.0]),
        VecExpandEvent::NoVals { label: "b".into() },
        vec_expand_payload("c", vec![3.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 4. [T; N] (auto-expand fixed array)
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
pub fn array_split_1v1r() -> DataFrameShape {
    vec![array_payload(1, [10.0, 20.0])].into_dataframe_split()
}

#[miniextendr]
pub fn array_split_1vnr() -> DataFrameShape {
    vec![
        array_payload(1, [10.0, 20.0]),
        array_payload(2, [30.0, 40.0]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn array_split_nv1r() -> DataFrameShape {
    vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn array_split_nvnr() -> DataFrameShape {
    vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
        array_payload(3, [30.0, 40.0]),
        ArrayEvent::NoCoords { id: 4 },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn array_align_nvnr() -> BuiltDataFrame {
    vec![
        array_payload(1, [10.0, 20.0]),
        ArrayEvent::NoCoords { id: 2 },
        array_payload(3, [30.0, 40.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 5. Box<[T]> with expand (auto-expand)
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
pub fn boxed_slice_split_1v1r() -> DataFrameShape {
    vec![boxed_slice_payload("a", &[1.0, 2.0, 3.0])].into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_split_1vnr() -> DataFrameShape {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        boxed_slice_payload("b", &[4.0]),
        boxed_slice_payload("c", &[]),
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_split_nv1r() -> DataFrameShape {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_split_nvnr() -> DataFrameShape {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
        boxed_slice_payload("c", &[4.0]),
        BoxedSliceEvent::NoBuffer { name: "d".into() },
    ]
    .into_dataframe_split()
}

#[miniextendr]
pub fn boxed_slice_align_nvnr() -> BuiltDataFrame {
    vec![
        boxed_slice_payload("a", &[1.0, 2.0, 3.0]),
        BoxedSliceEvent::NoBuffer { name: "b".into() },
        boxed_slice_payload("c", &[4.0]),
    ]
    .into_dataframe()
    .unwrap()
}

// endregion

// region: 5. Single-variant enum: bare-data.frame return from split
#[derive(Clone, Debug, DataFrameRow)]
pub enum SingletonRow {
    Only { id: i32, label: String },
}

#[miniextendr]
pub fn singleton_split_1v1r() -> DataFrameShape {
    vec![SingletonRow::Only {
        id: 1,
        label: "alpha".into(),
    }]
    .into_dataframe_split()
}

#[miniextendr]
pub fn singleton_split_1vnr() -> DataFrameShape {
    vec![
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
    ]
    .into_dataframe_split()
}

// endregion

// region: 6. &str field (borrowed text → STRSXP with NA_character_)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BorrowedStrEvent<'a> {
    Named { id: i32, name: &'a str },
    Bare { id: i32 },
}

#[miniextendr]
pub fn borrowed_str_split_1v1r() -> DataFrameShape {
    let data: Vec<BorrowedStrEvent<'static>> = vec![BorrowedStrEvent::Named {
        id: 1,
        name: "alice",
    }];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_str_split_1vnr() -> DataFrameShape {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Named { id: 2, name: "bob" },
        BorrowedStrEvent::Named {
            id: 3,
            name: "carol",
        },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_str_split_nv1r() -> DataFrameShape {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Bare { id: 2 },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_str_align_nvnr() -> BuiltDataFrame {
    vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Bare { id: 2 },
        BorrowedStrEvent::Named {
            id: 3,
            name: "carol",
        },
        BorrowedStrEvent::Bare { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn borrowed_str_split_nvnr() -> DataFrameShape {
    let data: Vec<BorrowedStrEvent<'static>> = vec![
        BorrowedStrEvent::Named {
            id: 1,
            name: "alice",
        },
        BorrowedStrEvent::Bare { id: 2 },
        BorrowedStrEvent::Named {
            id: 3,
            name: "carol",
        },
        BorrowedStrEvent::Bare { id: 4 },
    ];
    data.into_dataframe_split()
}

// endregion

// region: 7. &[T] field opaque (borrowed slice → list-column with NULL)
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BorrowedSliceEvent<'a> {
    Buffer { label: String, data: &'a [f64] },
    NoBuffer { label: String },
}

#[miniextendr]
pub fn borrowed_slice_split_1v1r() -> DataFrameShape {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![BorrowedSliceEvent::Buffer {
        label: "a".into(),
        data: &[1.0, 2.0, 3.0],
    }];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_slice_split_1vnr() -> DataFrameShape {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::Buffer {
            label: "b".into(),
            data: &[4.0],
        },
        BorrowedSliceEvent::Buffer {
            label: "c".into(),
            data: &[],
        },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_slice_split_nv1r() -> DataFrameShape {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
    ];
    data.into_dataframe_split()
}

#[miniextendr]
pub fn borrowed_slice_align_nvnr() -> BuiltDataFrame {
    vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
        BorrowedSliceEvent::Buffer {
            label: "c".into(),
            data: &[4.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "d".into() },
    ]
    .into_dataframe()
    .unwrap()
}

#[miniextendr]
pub fn borrowed_slice_split_nvnr() -> DataFrameShape {
    let data: Vec<BorrowedSliceEvent<'static>> = vec![
        BorrowedSliceEvent::Buffer {
            label: "a".into(),
            data: &[1.0, 2.0, 3.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "b".into() },
        BorrowedSliceEvent::Buffer {
            label: "c".into(),
            data: &[4.0],
        },
        BorrowedSliceEvent::NoBuffer { label: "d".into() },
    ];
    data.into_dataframe_split()
}

// endregion

// region: 8. Map fields (HashMap<K,V> / BTreeMap<K,V>)
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
pub fn hashmap_split_1v1r() -> DataFrameShape {
    vec![HashMapEvent::Tally {
        label: "a".into(),
        tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe_split()
}

/// 1vNr: one variant (Tally), multiple rows.
#[miniextendr]
pub fn hashmap_split_1vnr() -> DataFrameShape {
    vec![
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
    ]
    .into_dataframe_split()
}

/// Nv1r: both variants, one row each.
#[miniextendr]
pub fn hashmap_split_nv1r() -> DataFrameShape {
    vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe_split()
}

/// 1v1r align: one variant (Tally), one row.
#[miniextendr]
pub fn hashmap_align_1v1r() -> BuiltDataFrame {
    vec![HashMapEvent::Tally {
        label: "a".into(),
        tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align: one variant (Tally), multiple rows.
#[miniextendr]
pub fn hashmap_align_1vnr() -> BuiltDataFrame {
    vec![
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
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align: both variants, one row each.
#[miniextendr]
pub fn hashmap_align_nv1r() -> BuiltDataFrame {
    vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align: both variants, multiple rows each.
#[miniextendr]
pub fn hashmap_align_nvnr() -> BuiltDataFrame {
    vec![
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
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr split: both variants, multiple rows each.
#[miniextendr]
pub fn hashmap_split_nvnr() -> DataFrameShape {
    vec![
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
    ]
    .into_dataframe_split()
}

// endregion: HashMap fixtures

// region: BTreeMap fixtures – all 4 cardinality cells

/// 1v1r: one variant (Tally), one row.
#[miniextendr]
pub fn btreemap_split_1v1r() -> DataFrameShape {
    vec![BTreeMapEvent::Tally {
        label: "a".into(),
        tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe_split()
}

/// 1vNr: one variant (Tally), multiple rows.
#[miniextendr]
pub fn btreemap_split_1vnr() -> DataFrameShape {
    vec![
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
    ]
    .into_dataframe_split()
}

/// Nv1r: both variants, one row each.
#[miniextendr]
pub fn btreemap_split_nv1r() -> DataFrameShape {
    vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe_split()
}

/// 1v1r align: one variant (Tally), one row.
#[miniextendr]
pub fn btreemap_align_1v1r() -> BuiltDataFrame {
    vec![BTreeMapEvent::Tally {
        label: "a".into(),
        tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align: one variant (Tally), multiple rows.
#[miniextendr]
pub fn btreemap_align_1vnr() -> BuiltDataFrame {
    vec![
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
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align: both variants, one row each.
#[miniextendr]
pub fn btreemap_align_nv1r() -> BuiltDataFrame {
    vec![
        BTreeMapEvent::Tally {
            label: "a".into(),
            tally: BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]),
        },
        BTreeMapEvent::Empty { label: "b".into() },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align: both variants, multiple rows each.
#[miniextendr]
pub fn btreemap_align_nvnr() -> BuiltDataFrame {
    vec![
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
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr split: both variants, multiple rows each.
#[miniextendr]
pub fn btreemap_split_nvnr() -> DataFrameShape {
    vec![
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
    ]
    .into_dataframe_split()
}

// endregion: BTreeMap fixtures

// endregion: Map fields

#[cfg(test)]
mod tests;

// region: 9. Struct fields (DataFrameRow flatten / as_list opt-out)
// Struct-typed variant fields flatten into prefixed columns by default.
// The inner struct must #[derive(DataFrameRow)].
// Per-field #[dataframe(as_list)] keeps the struct as an opaque list-column
// (inner must then implement IntoList, e.g. via #[derive(IntoList)]).

/// A simple 2-column inner struct for testing struct-field flattening.
#[derive(Clone, Debug, DataFrameRow, IntoList)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Flatten path: `origin: Point` expands to `origin_x` + `origin_y` columns.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum StructFlattenEvent {
    Located { id: i32, origin: Point },
    Other { id: i32 },
}

/// as_list opt-out: `origin` becomes an opaque list-column.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum StructListEvent {
    Located {
        id: i32,
        #[dataframe(as_list)]
        origin: Point,
    },
    Other {
        id: i32,
    },
}

// region: Flatten fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split: single Located row.
#[miniextendr]
pub fn struct_flatten_split_1v1r() -> DataFrameShape {
    vec![StructFlattenEvent::Located {
        id: 1,
        origin: Point { x: 1.0, y: 2.0 },
    }]
    .into_dataframe_split()
}

/// 1vNr split: multiple Located rows, all same variant.
#[miniextendr]
pub fn struct_flatten_split_1vnr() -> DataFrameShape {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Located {
            id: 3,
            origin: Point { x: 5.0, y: 6.0 },
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split: one Located and one Other row.
#[miniextendr]
pub fn struct_flatten_split_nv1r() -> DataFrameShape {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Other { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split: multiple rows across both variants.
#[miniextendr]
pub fn struct_flatten_split_nvnr() -> DataFrameShape {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Other { id: 3 },
        StructFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe_split()
}

/// NvNr align: aligned data frame with NA-fill for absent variant.
#[miniextendr]
pub fn struct_flatten_align_nvnr() -> BuiltDataFrame {
    vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Other { id: 3 },
        StructFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: Flatten fixtures

// region: as_list fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split (as_list): single Located row, origin as list-column.
#[miniextendr]
pub fn struct_list_split_1v1r() -> DataFrameShape {
    vec![StructListEvent::Located {
        id: 1,
        origin: Point { x: 1.0, y: 2.0 },
    }]
    .into_dataframe_split()
}

/// 1vNr split (as_list): multiple Located rows.
#[miniextendr]
pub fn struct_list_split_1vnr() -> DataFrameShape {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructListEvent::Located {
            id: 3,
            origin: Point { x: 5.0, y: 6.0 },
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (as_list): one Located and one Other row.
#[miniextendr]
pub fn struct_list_split_nv1r() -> DataFrameShape {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Other { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (as_list): multiple rows across both variants.
#[miniextendr]
pub fn struct_list_split_nvnr() -> DataFrameShape {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructListEvent::Other { id: 3 },
        StructListEvent::Other { id: 4 },
    ]
    .into_dataframe_split()
}

/// NvNr align (as_list): aligned data frame, origin as list-column.
#[miniextendr]
pub fn struct_list_align_nvnr() -> BuiltDataFrame {
    vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructListEvent::Other { id: 3 },
        StructListEvent::Other { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: as_list fixtures

// endregion: 9. Struct fields

// region: 10. Nested enum fields (as_factor / flatten / as_list)
// Inner enum `Direction` is a unit-only enum that derives `DataFrameRow`.
// The derive auto-emits `IntoR`, `IntoR for Vec<Option<Self>>`, and `IntoList`
// so it can be used with `as_factor` (factor column) or `as_list` (list column)
// in an outer enum's variant fields.
//
// Inner enum `Status` has payload variants (not unit-only) and thus implements
// `DataFrameRow` for flattening only — its variants get prefixed columns.
//
// Three outer enums exercise the three field-treatment modes:
//   - `NestedFlattenEvent`: `dir: Direction` flattens via DataFrameRow (produces
//     `dir_variant` column from the `tag = "variant"` attribute on Direction)
//   - `NestedFactorEvent`: `#[dataframe(as_factor)] dir: Direction` → factor column
//   - `NestedListEvent`: `#[dataframe(as_list)] dir: Direction` → list column
//
// Note: the inner-payload-field-named-`variant` pattern (i.e., a `Status` payload
// field named `"variant"`) is now a **compile error** enforced via
// `assert_no_payload_field_collision` at the outer `#[derive(DataFrameRow)]` site.
// See issue #486 and PR #542.

/// Unit-only inner enum — derives `DataFrameRow`, which auto-emits `IntoR` and
/// `IntoR for Vec<Option<Self>>` as factor SEXPs, and `IntoList`.
#[derive(Clone, Copy, Debug, DataFrameRow)]
#[dataframe(tag = "variant")]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Payload inner enum — derives `DataFrameRow` for flattening only.
/// `Status` has a payload variant so it is NOT unit-only; no factor impls are emitted.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "variant")]
pub enum Status {
    Ok,
    Err { code: i32 },
}

/// Outer enum: `status: Status` field flattens via DataFrameRow.
/// `Status` is a payload-bearing inner enum (has `Ok` and `Err { code }`).
/// After prefixing: `status_variant` (discriminant), `status_code` (NA for Ok rows).
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedFlattenEvent {
    Tracked { id: i32, status: Status },
    Other { id: i32 },
}

/// Outer enum: `#[dataframe(as_factor)] dir: Direction` → single factor column.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedFactorEvent {
    Move {
        id: i32,
        #[dataframe(as_factor)]
        dir: Direction,
    },
    Stop {
        id: i32,
    },
}

/// Outer enum: `#[dataframe(as_list)] dir: Direction` → opaque list column.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedListEvent {
    Move {
        id: i32,
        #[dataframe(as_list)]
        dir: Direction,
    },
    Stop {
        id: i32,
    },
}

// region: Flatten fixtures — all 4 cardinality cells × 2 modes (split + align)
//
// Uses the payload-bearing `Status` inner enum so the flatten path exercises:
//   - `status_variant` discriminant column
//   - `status_code` payload column (NA for `Status::Ok` rows, Some(i32) for `Status::Err`)

/// 1v1r split (flatten): single Tracked/Ok row.
#[miniextendr]
pub fn nested_flatten_split_1v1r() -> DataFrameShape {
    vec![NestedFlattenEvent::Tracked {
        id: 1,
        status: Status::Ok,
    }]
    .into_dataframe_split()
}

/// 1vNr split (flatten): multiple Tracked rows (mix of Ok and Err).
#[miniextendr]
pub fn nested_flatten_split_1vnr() -> DataFrameShape {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Tracked {
            id: 3,
            status: Status::Err { code: 500 },
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (flatten): one Tracked and one Other row.
#[miniextendr]
pub fn nested_flatten_split_nv1r() -> DataFrameShape {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (flatten): multiple rows across both variants.
#[miniextendr]
pub fn nested_flatten_split_nvnr() -> DataFrameShape {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Other { id: 3 },
        NestedFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe_split()
}

/// 1v1r align (flatten): single Tracked row, aligned data frame.
#[miniextendr]
pub fn nested_flatten_align_1v1r() -> BuiltDataFrame {
    vec![NestedFlattenEvent::Tracked {
        id: 1,
        status: Status::Ok,
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align (flatten): multiple Tracked rows, aligned data frame.
#[miniextendr]
pub fn nested_flatten_align_1vnr() -> BuiltDataFrame {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Tracked {
            id: 3,
            status: Status::Err { code: 500 },
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align (flatten): one Tracked, one Other — NA-fill for Other rows.
#[miniextendr]
pub fn nested_flatten_align_nv1r() -> BuiltDataFrame {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align (flatten): multiple rows across both variants — NA-fill for Other rows.
#[miniextendr]
pub fn nested_flatten_align_nvnr() -> BuiltDataFrame {
    vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Tracked {
            id: 2,
            status: Status::Err { code: 404 },
        },
        NestedFlattenEvent::Other { id: 3 },
        NestedFlattenEvent::Other { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: Flatten fixtures

// region: as_factor fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split (as_factor): single Move row, dir as factor column.
#[miniextendr]
pub fn nested_factor_split_1v1r() -> DataFrameShape {
    vec![NestedFactorEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe_split()
}

/// 1vNr split (as_factor): multiple Move rows.
#[miniextendr]
pub fn nested_factor_split_1vnr() -> DataFrameShape {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedFactorEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (as_factor): one Move and one Stop row.
#[miniextendr]
pub fn nested_factor_split_nv1r() -> DataFrameShape {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedFactorEvent::Stop { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (as_factor): multiple rows across both variants.
#[miniextendr]
pub fn nested_factor_split_nvnr() -> DataFrameShape {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedFactorEvent::Stop { id: 3 },
        NestedFactorEvent::Stop { id: 4 },
    ]
    .into_dataframe_split()
}

/// 1v1r align (as_factor): single Move row.
#[miniextendr]
pub fn nested_factor_align_1v1r() -> BuiltDataFrame {
    vec![NestedFactorEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align (as_factor): multiple Move rows.
#[miniextendr]
pub fn nested_factor_align_1vnr() -> BuiltDataFrame {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedFactorEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align (as_factor): one Move and one Stop — NA for Stop's dir.
#[miniextendr]
pub fn nested_factor_align_nv1r() -> BuiltDataFrame {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedFactorEvent::Stop { id: 2 },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align (as_factor): aligned data frame with NA-fill for dir in Stop rows.
#[miniextendr]
pub fn nested_factor_align_nvnr() -> BuiltDataFrame {
    vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedFactorEvent::Stop { id: 3 },
        NestedFactorEvent::Stop { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: as_factor fixtures

// region: as_list fixtures — all 4 cardinality cells × 2 modes

/// 1v1r split (as_list): single Move row, dir as list column.
#[miniextendr]
pub fn nested_list_split_1v1r() -> DataFrameShape {
    vec![NestedListEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe_split()
}

/// 1vNr split (as_list): multiple Move rows.
#[miniextendr]
pub fn nested_list_split_1vnr() -> DataFrameShape {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedListEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe_split()
}

/// Nv1r split (as_list): one Move and one Stop row.
#[miniextendr]
pub fn nested_list_split_nv1r() -> DataFrameShape {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedListEvent::Stop { id: 2 },
    ]
    .into_dataframe_split()
}

/// NvNr split (as_list): multiple rows across both variants.
#[miniextendr]
pub fn nested_list_split_nvnr() -> DataFrameShape {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedListEvent::Stop { id: 3 },
        NestedListEvent::Stop { id: 4 },
    ]
    .into_dataframe_split()
}

/// 1v1r align (as_list): single Move row.
#[miniextendr]
pub fn nested_list_align_1v1r() -> BuiltDataFrame {
    vec![NestedListEvent::Move {
        id: 1,
        dir: Direction::North,
    }]
    .into_dataframe()
    .unwrap()
}

/// 1vNr align (as_list): multiple Move rows.
#[miniextendr]
pub fn nested_list_align_1vnr() -> BuiltDataFrame {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::South,
        },
        NestedListEvent::Move {
            id: 3,
            dir: Direction::East,
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// Nv1r align (as_list): one Move and one Stop — NULL for Stop's dir.
#[miniextendr]
pub fn nested_list_align_nv1r() -> BuiltDataFrame {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::West,
        },
        NestedListEvent::Stop { id: 2 },
    ]
    .into_dataframe()
    .unwrap()
}

/// NvNr align (as_list): aligned data frame with NULL-fill for dir in Stop rows.
#[miniextendr]
pub fn nested_list_align_nvnr() -> BuiltDataFrame {
    vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Move {
            id: 2,
            dir: Direction::East,
        },
        NestedListEvent::Stop { id: 3 },
        NestedListEvent::Stop { id: 4 },
    ]
    .into_dataframe()
    .unwrap()
}

// endregion: as_list fixtures

// endregion: 10. Nested enum fields
