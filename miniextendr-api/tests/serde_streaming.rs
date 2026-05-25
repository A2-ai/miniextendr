//! Integration tests for `iter_to_dataframe` and `DataFrameBuilder`.
//!
//! Streaming counterpart to `vec_to_dataframe`. Schema is taken from the
//! first row; later rows must match (strict mode) — fields missing from a
//! later row are NA-padded, fields *added* by a later row produce an error.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::IntoR as _;
use miniextendr_api::serde::{DataFrameBuilder, RSerdeError, iter_to_dataframe, vec_to_dataframe};
use miniextendr_api::sys::SexpExt as _;
use serde::Serialize;

// region: round-trip equivalence with vec_to_dataframe

#[derive(Debug, Clone, Serialize)]
struct Plain {
    id: i32,
    val: f64,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct Optional {
    a: Option<i32>,
    b: Option<String>,
}

#[test]
fn iter_to_dataframe_matches_vec_to_dataframe_plain() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Plain> = (0..5)
            .map(|i| Plain {
                id: i,
                val: f64::from(i) * 1.5,
                name: format!("r{i}"),
            })
            .collect();

        let from_vec = vec_to_dataframe(&rows).unwrap().into_sexp();
        let from_iter = iter_to_dataframe(rows.iter().cloned(), None)
            .unwrap()
            .into_sexp();

        assert_eq!(from_vec.xlength(), from_iter.xlength(), "ncol");
        // Each column's length matches (nrow check).
        for i in 0..from_vec.xlength() {
            assert_eq!(
                from_vec.vector_elt(i).xlength(),
                from_iter.vector_elt(i).xlength(),
                "column {i} nrow mismatch"
            );
        }
    });
}

#[test]
fn iter_to_dataframe_matches_vec_to_dataframe_optional() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Optional> = vec![
            Optional {
                a: Some(1),
                b: Some("x".into()),
            },
            Optional {
                a: None,
                b: Some("y".into()),
            },
            Optional {
                a: Some(3),
                b: None,
            },
        ];

        let from_vec = vec_to_dataframe(&rows).unwrap().into_sexp();
        let from_iter = iter_to_dataframe(rows.iter().cloned(), Some(3))
            .unwrap()
            .into_sexp();

        // Same number of columns.
        assert_eq!(from_vec.xlength(), from_iter.xlength());
        // Each column has 3 rows.
        for i in 0..from_iter.xlength() {
            assert_eq!(from_iter.vector_elt(i).xlength(), 3);
        }
    });
}

// endregion

// region: nrow_hint variants + empty iterator

#[test]
fn iter_to_dataframe_nrow_hint_none_grows() {
    r_test_utils::with_r_thread(|| {
        let rows = (0..100i32).map(|i| Plain {
            id: i,
            val: f64::from(i),
            name: format!("r{i}"),
        });
        let df = iter_to_dataframe(rows, None).unwrap().into_sexp();
        assert_eq!(df.xlength(), 3, "expected 3 columns");
        assert_eq!(df.vector_elt(0).xlength(), 100, "expected 100 rows");
    });
}

#[test]
fn iter_to_dataframe_empty_yields_empty_dataframe() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Plain> = vec![];
        let df = iter_to_dataframe(rows, None).unwrap().into_sexp();
        // Matches vec_to_dataframe(&[]) — 0 columns.
        assert_eq!(df.xlength(), 0);
    });
}

// endregion

// region: strict-schema rejection + NA-pad for missing fields

#[test]
fn dataframe_builder_strict_rejects_new_fields() {
    r_test_utils::with_r_thread(|| {
        let mut builder = DataFrameBuilder::<serde_json::Value>::new(None);

        // Use serde_json::Value so we can push heterogeneous shapes via Serialize.
        let row1: serde_json::Value = serde_json::json!({ "x": 1 });
        builder.push(row1).unwrap();

        let row2: serde_json::Value = serde_json::json!({ "x": 2, "y": "extra" });
        let err = builder.push(row2).unwrap_err();
        assert!(
            matches!(err, RSerdeError::Message(ref m) if m.contains("not in initial schema")),
            "expected strict-schema rejection, got: {err:?}"
        );
    });
}

#[test]
fn dataframe_builder_missing_field_na_pads() {
    r_test_utils::with_r_thread(|| {
        // Same struct, but second row's b=None — should NA-pad, not error.
        let mut builder = DataFrameBuilder::<Optional>::new(None);
        builder
            .push(Optional {
                a: Some(1),
                b: Some("x".into()),
            })
            .unwrap();
        builder
            .push(Optional {
                a: Some(2),
                b: None,
            })
            .unwrap();

        let df = builder.finish().unwrap().into_sexp();
        // Both columns present, 2 rows each.
        assert_eq!(df.xlength(), 2);
        assert_eq!(df.vector_elt(0).xlength(), 2);
        assert_eq!(df.vector_elt(1).xlength(), 2);
    });
}

// endregion

// region: DataFrameBuilder direct surface

#[test]
fn dataframe_builder_len_is_empty_finish() {
    r_test_utils::with_r_thread(|| {
        let mut builder = DataFrameBuilder::<Plain>::new(Some(4));
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);

        builder
            .push(Plain {
                id: 1,
                val: 1.5,
                name: "a".into(),
            })
            .unwrap();
        assert_eq!(builder.len(), 1);
        assert!(!builder.is_empty());

        let df = builder.finish().unwrap().into_sexp();
        assert_eq!(df.vector_elt(0).xlength(), 1);
    });
}

// endregion
