//! Equivalent standard-library spellings must not change an enum's R columns.
extern crate alloc;
mod r_test_utils;

use miniextendr_api::dataframe::{FromDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, TryFromSexp};

#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "kind")]
pub enum Scalars {
    Qualified {
        id: ::core::primitive::i32,
        label: ::alloc::string::String,
    },
    Bare {
        id: i32,
        label: String,
    },
    Empty,
}

#[derive(DataFrameRow)]
#[dataframe(align, tag = "kind", conflicts = "string")]
pub enum CoercionEnabled {
    Qualified { value: std::primitive::i32 },
    Bare { value: i32 },
}

#[derive(DataFrameRow)]
#[dataframe(align, tag = "kind", conflicts = "string")]
pub enum RealConflict {
    Numeric { value: f64 },
    Text { value: String },
}

#[derive(DataFrameRow)]
#[dataframe(align, tag = "kind")]
pub enum Containers {
    Qualified {
        #[dataframe(as_list)]
        values: ::std::vec::Vec<::core::primitive::i32>,
    },
    Bare {
        #[dataframe(as_list)]
        values: Vec<i32>,
    },
}

#[derive(DataFrameRow)]
#[dataframe(align, tag = "kind")]
pub enum Expanded {
    Qualified {
        #[dataframe(expand)]
        values: ::alloc::vec::Vec<::core::primitive::i32>,
    },
    Bare {
        #[dataframe(expand)]
        values: Vec<i32>,
    },
}

#[test]
fn equivalent_scalar_spellings_keep_native_columns_and_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Scalars::Qualified {
                id: 1,
                label: "first".into(),
            },
            Scalars::Bare {
                id: 2,
                label: "second".into(),
            },
            Scalars::Empty,
        ];
        let frame = rows.clone().into_dataframe().unwrap();
        assert_eq!(
            frame.column::<Vec<Option<i32>>>("id").unwrap(),
            [Some(1), Some(2), None]
        );
        assert_eq!(
            frame.column::<Vec<Option<String>>>("label").unwrap(),
            [Some("first".into()), Some("second".into()), None]
        );
        assert_eq!(Vec::<Scalars>::from_dataframe(&frame).unwrap(), rows);
        #[cfg(feature = "rayon")]
        {
            let frame = rows.clone().into_dataframe_par().unwrap();
            assert_eq!(Vec::<Scalars>::from_dataframe_par(&frame).unwrap(), rows);
        }
    });
}

#[test]
fn string_conflicts_coerce_only_different_types() {
    r_test_utils::with_r_thread(|| {
        let frame = vec![
            CoercionEnabled::Qualified { value: 1 },
            CoercionEnabled::Bare { value: 2 },
        ]
        .into_dataframe()
        .unwrap();
        assert_eq!(frame.column::<Vec<i32>>("value").unwrap(), [1, 2]);
        let frame = vec![
            RealConflict::Numeric { value: 1.5 },
            RealConflict::Text {
                value: "text".into(),
            },
        ]
        .into_dataframe()
        .unwrap();
        assert_eq!(
            frame.column::<Vec<String>>("value").unwrap(),
            ["1.5", "text"]
        );
    });
}

#[test]
fn qualified_containers_merge_static_and_dynamic_columns() {
    r_test_utils::with_r_thread(|| {
        let frame = vec![
            Containers::Qualified { values: vec![1, 2] },
            Containers::Bare { values: vec![3] },
        ]
        .into_dataframe()
        .unwrap();
        let values = Vec::<Vec<i32>>::try_from_sexp(frame.column_raw("values").unwrap()).unwrap();
        assert_eq!(values, [vec![1, 2], vec![3]]);
        let frame = vec![
            Expanded::Qualified { values: vec![1, 2] },
            Expanded::Bare { values: vec![3] },
        ]
        .into_dataframe()
        .unwrap();
        assert_eq!(
            frame.column::<Vec<Option<i32>>>("values_1").unwrap(),
            [Some(1), Some(3)]
        );
        assert_eq!(
            frame.column::<Vec<Option<i32>>>("values_2").unwrap(),
            [Some(2), None]
        );
    });
}
