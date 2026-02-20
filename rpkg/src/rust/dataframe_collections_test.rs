//! Test DataFrameRow with various collection types

#![allow(dead_code)]

use miniextendr_api::{DataFrameRow, IntoList};
use std::collections::{BTreeSet, HashSet};

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
        ::miniextendr_api::List::from_raw_pairs(vec![("data", self.data.into_vec().into_sexp())])
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
        ::miniextendr_api::List::from_raw_pairs(vec![("coords", self.coords.to_vec().into_sexp())])
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
        ::miniextendr_api::List::from_raw_pairs(vec![("tags", tags_vec.into_sexp())])
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
        ::miniextendr_api::List::from_raw_pairs(vec![("categories", cats_vec.into_sexp())])
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
        ::miniextendr_api::List::from_raw_pairs(vec![
            ("vec_field", self.vec_field.into_sexp()),
            ("array_field", self.array_field.to_vec().into_sexp()),
            ("boxed_field", self.boxed_field.into_vec().into_sexp()),
        ])
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
        ::miniextendr_api::List::from_raw_pairs(vec![
            ("name", self.name.into_sexp()),
            ("value", self.value.into_sexp()),
        ])
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
        ::miniextendr_api::List::from_raw_pairs(vec![
            ("label", self.name.into_sexp()),
            ("value", self.value.into_sexp()),
        ])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_dataframe() {
        let rows = vec![
            WithVec {
                ids: vec![1, 2, 3],
                names: vec!["a".into(), "b".into()],
            },
            WithVec {
                ids: vec![4, 5],
                names: vec!["c".into()],
            },
        ];

        let df = WithVec::to_dataframe(rows.clone());
        assert_eq!(df.ids.len(), 2);
        assert_eq!(df.names.len(), 2);

        // Test round-trip
        let back: Vec<WithVec> = WithVec::from_dataframe(df);
        assert_eq!(back.len(), 2);
    }

    #[test]
    fn test_boxed_slice_dataframe() {
        let rows = vec![
            WithBoxedSlice {
                data: vec![1.0, 2.0, 3.0].into_boxed_slice(),
            },
            WithBoxedSlice {
                data: vec![4.0, 5.0].into_boxed_slice(),
            },
        ];

        let df = WithBoxedSlice::to_dataframe(rows);
        assert_eq!(df.data.len(), 2);
    }

    #[test]
    fn test_array_expansion() {
        // [f64; 3] now auto-expands to coords_1, coords_2, coords_3
        let rows = vec![
            WithArray {
                coords: [1.0, 2.0, 3.0],
            },
            WithArray {
                coords: [4.0, 5.0, 6.0],
            },
        ];

        let df = WithArray::to_dataframe(rows);
        assert_eq!(df.coords_1.len(), 2);
        assert_eq!(df.coords_2.len(), 2);
        assert_eq!(df.coords_3.len(), 2);
        assert_eq!(df.coords_1[0], 1.0);
        assert_eq!(df.coords_2[0], 2.0);
        assert_eq!(df.coords_3[0], 3.0);
        assert_eq!(df.coords_1[1], 4.0);
        assert_eq!(df.coords_2[1], 5.0);
        assert_eq!(df.coords_3[1], 6.0);
    }

    #[test]
    fn test_array_as_list() {
        // as_list suppresses expansion
        let rows = vec![
            WithArrayAsList {
                coords: [1.0, 2.0, 3.0],
            },
            WithArrayAsList {
                coords: [4.0, 5.0, 6.0],
            },
        ];

        let df = WithArrayAsList::to_dataframe(rows);
        // Single column, not expanded
        assert_eq!(df.coords.len(), 2);
        assert_eq!(df.coords[0], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_hashset_dataframe() {
        let mut tags1 = HashSet::new();
        tags1.insert("rust".to_string());
        tags1.insert("r".to_string());

        let mut tags2 = HashSet::new();
        tags2.insert("python".to_string());

        let rows = vec![WithHashSet { tags: tags1 }, WithHashSet { tags: tags2 }];

        let df = WithHashSet::to_dataframe(rows);
        assert_eq!(df.tags.len(), 2);
    }

    #[test]
    fn test_btreeset_dataframe() {
        let mut cats1 = BTreeSet::new();
        cats1.insert(1);
        cats1.insert(2);

        let mut cats2 = BTreeSet::new();
        cats2.insert(3);

        let rows = vec![
            WithBTreeSet { categories: cats1 },
            WithBTreeSet { categories: cats2 },
        ];

        let df = WithBTreeSet::to_dataframe(rows);
        assert_eq!(df.categories.len(), 2);
    }

    #[test]
    fn test_mixed_collections() {
        // array_field: [f64; 2] now expands to array_field_1, array_field_2
        let rows = vec![
            MixedCollections {
                vec_field: vec![1, 2],
                array_field: [1.0, 2.0],
                boxed_field: vec!["a".into(), "b".into()].into_boxed_slice(),
            },
            MixedCollections {
                vec_field: vec![3, 4, 5],
                array_field: [3.0, 4.0],
                boxed_field: vec!["c".into()].into_boxed_slice(),
            },
        ];

        let df = MixedCollections::to_dataframe(rows);
        assert_eq!(df.vec_field.len(), 2);
        assert_eq!(df.array_field_1.len(), 2);
        assert_eq!(df.array_field_2.len(), 2);
        assert_eq!(df.boxed_field.len(), 2);
        assert_eq!(df.array_field_1[0], 1.0);
        assert_eq!(df.array_field_2[0], 2.0);
        // No from_dataframe for structs with expansion
    }

    #[test]
    fn test_skip_field() {
        let rows = vec![
            WithSkip {
                name: "a".into(),
                internal_id: 42,
                value: 1.0,
            },
            WithSkip {
                name: "b".into(),
                internal_id: 99,
                value: 2.0,
            },
        ];

        let df = WithSkip::to_dataframe(rows);
        // Only name and value columns (internal_id skipped)
        assert_eq!(df.name.len(), 2);
        assert_eq!(df.value.len(), 2);
        assert_eq!(df.name[0], "a");
        assert_eq!(df.value[1], 2.0);
    }

    #[test]
    fn test_rename_field() {
        let rows = vec![WithRename {
            name: "hello".into(),
            value: 1.0,
        }];

        let df = WithRename::to_dataframe(rows);
        // Column is named "label" in the companion struct
        assert_eq!(df.label.len(), 1);
        assert_eq!(df.label[0], "hello");
    }

    #[test]
    fn test_pinned_vec_expansion() {
        let rows = vec![
            WithPinnedVec {
                name: "a".into(),
                scores: vec![1.0, 2.0, 3.0],
            },
            WithPinnedVec {
                name: "b".into(),
                scores: vec![4.0], // shorter: trailing None
            },
            WithPinnedVec {
                name: "c".into(),
                scores: vec![], // empty: all None
            },
        ];

        let df = WithPinnedVec::to_dataframe(rows);
        assert_eq!(df.name.len(), 3);
        assert_eq!(df.scores_1.len(), 3);
        assert_eq!(df.scores_2.len(), 3);
        assert_eq!(df.scores_3.len(), 3);

        // First row: all present
        assert_eq!(df.scores_1[0], Some(1.0));
        assert_eq!(df.scores_2[0], Some(2.0));
        assert_eq!(df.scores_3[0], Some(3.0));

        // Second row: only first present
        assert_eq!(df.scores_1[1], Some(4.0));
        assert_eq!(df.scores_2[1], None);
        assert_eq!(df.scores_3[1], None);

        // Third row: all None
        assert_eq!(df.scores_1[2], None);
        assert_eq!(df.scores_2[2], None);
        assert_eq!(df.scores_3[2], None);
    }

    #[test]
    fn test_renamed_array_expansion() {
        let rows = vec![
            WithRenamedArray { coords: [1.0, 2.0] },
            WithRenamedArray { coords: [3.0, 4.0] },
        ];

        let df = WithRenamedArray::to_dataframe(rows);
        // rename = "pos" → columns pos_1, pos_2
        assert_eq!(df.pos_1.len(), 2);
        assert_eq!(df.pos_2.len(), 2);
        assert_eq!(df.pos_1[0], 1.0);
        assert_eq!(df.pos_2[1], 4.0);
    }

    #[test]
    fn test_enum_array_expansion() {
        let rows = vec![
            EventWithCoords::Click {
                id: 1,
                coords: [10.0, 20.0],
            },
            EventWithCoords::Hover { id: 2 },
            EventWithCoords::Click {
                id: 3,
                coords: [30.0, 40.0],
            },
        ];

        let df = EventWithCoords::to_dataframe(rows);
        assert_eq!(df._tag.len(), 3);
        assert_eq!(df._tag[0], "Click");
        assert_eq!(df._tag[1], "Hover");

        // id is shared — always present
        assert_eq!(df.id, vec![Some(1), Some(2), Some(3)]);

        // coords expand to coords_1, coords_2 with Option
        assert_eq!(df.coords_1, vec![Some(10.0), None, Some(30.0)]);
        assert_eq!(df.coords_2, vec![Some(20.0), None, Some(40.0)]);
    }

    #[test]
    fn test_enum_pinned_vec_expansion() {
        let rows = vec![
            EventWithScores::Result {
                label: "a".into(),
                scores: vec![1.0, 2.0],
            },
            EventWithScores::Empty { label: "b".into() },
            EventWithScores::Result {
                label: "c".into(),
                scores: vec![3.0],
            },
        ];

        let df = EventWithScores::to_dataframe(rows);
        assert_eq!(
            df.label,
            vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ]
        );

        // scores_1, scores_2 — wrapped in Option<Option<f64>> → Option<f64> since enum already wraps
        // Actually: enum columns are Vec<Option<T>>. For ExpandedVec with elem_ty=f64,
        // the value pushed is `binding.get(i).cloned()` which is Option<f64>.
        // So the column is Vec<Option<Option<f64>>>? No — let me check.
        // In register_column, the col_ty is elem_ty (f64), and columns use Vec<Option<col_ty>>.
        // But the push is `binding.get(i).cloned()` which returns Option<f64>.
        // That's being pushed as `col.push(binding.get(i).cloned())` — that would be
        // pushing Option<f64> into Vec<Option<f64>>. That works!
        // For absent variant (Empty), it pushes None (which is Option<f64>::None).

        assert_eq!(df.scores_1, vec![Some(1.0), None, Some(3.0)]);
        assert_eq!(df.scores_2, vec![Some(2.0), None, None]);
    }

    #[test]
    fn test_enum_skip() {
        let rows = vec![
            EventWithSkip::A {
                value: 1.0,
                internal: 42,
            },
            EventWithSkip::B { value: 2.0 },
        ];

        let df = EventWithSkip::to_dataframe(rows);
        assert_eq!(df.value, vec![Some(1.0), Some(2.0)]);
        // No `internal` column
    }
}
