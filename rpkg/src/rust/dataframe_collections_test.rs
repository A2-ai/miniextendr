//! Test DataFrameRow with various collection types

use miniextendr_api::{DataFrameRow, IntoList};
use std::collections::{BTreeSet, HashSet};

// Test with Vec fields
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct WithVec {
    pub ids: Vec<i32>,
    pub names: Vec<String>,
}

// Test with boxed slices
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

// Test with arrays
#[derive(Clone, Debug, DataFrameRow)]
pub struct WithArray {
    pub coords: [f64; 3],
}

impl ::miniextendr_api::list::IntoList for WithArray {
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

// Test with mixed collection types
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
    fn test_array_dataframe() {
        let rows = vec![
            WithArray {
                coords: [1.0, 2.0, 3.0],
            },
            WithArray {
                coords: [4.0, 5.0, 6.0],
            },
        ];

        let df = WithArray::to_dataframe(rows);
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

        let df = MixedCollections::to_dataframe(rows.clone());
        assert_eq!(df.vec_field.len(), 2);
        assert_eq!(df.array_field.len(), 2);
        assert_eq!(df.boxed_field.len(), 2);

        // Test round-trip
        let back: Vec<MixedCollections> = MixedCollections::from_dataframe(df);
        assert_eq!(back.len(), 2);
    }
}
