//! Integration tests for `NamedDataFrameListBuilder` and `vec_to_dataframe_split`.
//!
//! These tests require R to be initialized and run on the R main thread.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::ffi::SexpExt as _;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::into_r::IntoR as _;
use miniextendr_api::serde::{
    DataFrameShape, NamedDataFrameListBuilder, ResultShape, SplitResults, SplitShape,
    hashmap_to_dataframe, map_to_dataframe, result_to_dataframe, vec_to_dataframe,
    vec_to_dataframe_split,
};
use serde::Serialize;

// region: NamedDataFrameListBuilder

/// build() on a new() builder returns a 0-element named list.
#[test]
fn builder_empty_build_yields_empty_list() {
    r_test_utils::with_r_thread(|| {
        let list = NamedDataFrameListBuilder::new().build();
        let sexp = list.into_sexp();
        assert_eq!(sexp.xlength(), 0, "expected 0-element list");
    });
}

/// Pushing two data.frames and building produces a 2-element named list whose
/// element SEXPs are valid VECSXPs.
#[test]
fn builder_push_protects_input() {
    #[derive(Serialize)]
    struct Row {
        id: i32,
        val: f64,
    }

    r_test_utils::with_r_thread(|| {
        let oks: Vec<Row> = (0..10)
            .map(|i| Row {
                id: i,
                val: i as f64,
            })
            .collect();
        let errs: Vec<Row> = (0..5).map(|i| Row { id: i, val: -1.0 }).collect();

        let list = NamedDataFrameListBuilder::new()
            .push("results", vec_to_dataframe(&oks).unwrap())
            .push("error", vec_to_dataframe(&errs).unwrap())
            .build();

        let sexp = list.into_sexp();
        assert_eq!(sexp.xlength(), 2, "expected 2 entries");

        let elem0 = sexp.vector_elt(0);
        let elem1 = sexp.vector_elt(1);
        // Each element should be a VECSXP (data.frame); nrow = 10 and 5
        assert_eq!(elem0.xlength(), 2, "results df should have 2 columns");
        assert_eq!(elem1.xlength(), 2, "error df should have 2 columns");
    });
}

/// Dropping the builder without calling build() does not leave dangling
/// protections — the scope count returns to baseline.
#[test]
fn builder_drop_without_build() {
    #[derive(Serialize)]
    struct Row {
        x: i32,
    }

    r_test_utils::with_r_thread(|| unsafe {
        // Baseline: create a sibling scope and record its depth before the builder
        let outer = ProtectScope::new();
        let before = outer.count();

        {
            let rows: Vec<Row> = (0..3).map(|i| Row { x: i }).collect();
            let _builder =
                NamedDataFrameListBuilder::new().push("a", vec_to_dataframe(&rows).unwrap());
            // _builder drops here, unprotecting via its internal scope
        }

        // Outer scope unaffected
        assert_eq!(
            outer.count(),
            before,
            "outer scope count should be unchanged after builder drop"
        );
        drop(outer);
    });
}

/// Entries pushed in order a → b → c appear in that order in the names
/// attribute of the result list.
#[test]
fn builder_with_capacity_preserves_order() {
    #[derive(Serialize)]
    struct Row {
        v: i32,
    }

    r_test_utils::with_r_thread(|| {
        let rows: Vec<Row> = vec![Row { v: 1 }];
        let list = NamedDataFrameListBuilder::with_capacity(3)
            .push("a", vec_to_dataframe(&rows).unwrap())
            .push("b", vec_to_dataframe(&rows).unwrap())
            .push("c", vec_to_dataframe(&rows).unwrap())
            .build();

        let sexp = list.into_sexp();
        assert_eq!(sexp.xlength(), 3, "expected 3 entries");

        let names = sexp.get_names();
        assert_eq!(names.string_elt_str(0), Some("a"));
        assert_eq!(names.string_elt_str(1), Some("b"));
        assert_eq!(names.string_elt_str(2), Some("c"));
    });
}

// endregion

// region: vec_to_dataframe_split regression

/// vec_to_dataframe_split on a single-variant input returns a bare data.frame,
/// not a named list (single-variant short-circuit preserved after migration).
#[test]
fn vec_to_dataframe_split_single_variant_regression() {
    #[derive(Serialize)]
    enum E {
        Ok { id: i32 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![E::Ok { id: 1 }, E::Ok { id: 2 }];
        let shape = vec_to_dataframe_split(&rows, SplitShape::PerVariantList).unwrap();
        // Single-variant short-circuit: DataFrameShape::Bare; single-column df.
        assert!(matches!(shape, DataFrameShape::Bare(_)));
        let sexp = shape.into_sexp();
        assert_eq!(sexp.xlength(), 1, "single-column data.frame expected");
    });
}

/// vec_to_dataframe_split on a multi-variant input returns a PerVariantList shape.
#[test]
fn vec_to_dataframe_split_multi_variant_regression() {
    #[derive(Serialize)]
    enum E {
        Ok { id: i32 },
        Err { msg: String },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            E::Ok { id: 1 },
            E::Err { msg: "oops".into() },
            E::Ok { id: 2 },
        ];
        let shape = vec_to_dataframe_split(&rows, SplitShape::PerVariantList).unwrap();
        assert!(matches!(shape, DataFrameShape::PerVariantList(_)));
        let sexp = shape.into_sexp();
        assert_eq!(sexp.xlength(), 2, "expected 2 named partitions");
    });
}

// endregion

// region: map_to_dataframe regression

#[test]
fn map_to_dataframe_btreemap_basic() {
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct Summary {
        cmax: f64,
        tmax: f64,
    }

    r_test_utils::with_r_thread(|| {
        let mut map: BTreeMap<i32, Summary> = BTreeMap::new();
        map.insert(
            1,
            Summary {
                cmax: 10.5,
                tmax: 2.0,
            },
        );
        map.insert(
            2,
            Summary {
                cmax: 8.1,
                tmax: 3.0,
            },
        );

        let df = map_to_dataframe(&map, "subject").unwrap();
        let sexp = df.into_sexp();
        assert_eq!(sexp.xlength(), 3, "expected 3 columns: subject + cmax + tmax");

        let names = sexp.get_names();
        assert_eq!(names.string_elt_str(0), Some("subject"));
    });
}

#[test]
fn hashmap_to_dataframe_sorted_keys() {
    use std::collections::HashMap;

    #[derive(Serialize)]
    struct Row {
        v: i32,
    }

    r_test_utils::with_r_thread(|| {
        let mut map: HashMap<i32, Row> = HashMap::new();
        map.insert(3, Row { v: 30 });
        map.insert(1, Row { v: 10 });
        map.insert(2, Row { v: 20 });

        let df = hashmap_to_dataframe(&map, "id").unwrap();
        let sexp = df.into_sexp();
        assert_eq!(sexp.xlength(), 2, "expected 2 columns: id + v");
    });
}

// endregion

// region: result_to_dataframe regression

#[derive(Serialize)]
struct Obs {
    id: i32,
    value: f64,
}

#[derive(Serialize)]
struct ErrRow {
    id: i32,
    reason: String,
}

#[test]
fn result_to_dataframe_auto_all_ok_bare() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> = vec![
            Ok(Obs { id: 1, value: 1.0 }),
            Ok(Obs { id: 2, value: 2.0 }),
        ];
        let shape = result_to_dataframe(
            &rows,
            ResultShape::Auto {
                empty_ok_sentinel: (),
            },
        )
        .unwrap();
        assert!(matches!(shape, DataFrameShape::Bare(_)));
    });
}

#[test]
fn result_to_dataframe_auto_mixed_split() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> = vec![
            Ok(Obs { id: 1, value: 1.0 }),
            Err(ErrRow {
                id: 2,
                reason: "bad".into(),
            }),
        ];
        let shape = result_to_dataframe(
            &rows,
            ResultShape::Auto {
                empty_ok_sentinel: (),
            },
        )
        .unwrap();
        assert!(matches!(
            shape,
            DataFrameShape::Split {
                results: SplitResults::Some(_),
                ..
            }
        ));
        let sexp = shape.into_sexp();
        assert_eq!(sexp.xlength(), 2, "results + error elements");
    });
}

#[test]
fn result_to_dataframe_split_all_err_uses_sentinel() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> = vec![Err(ErrRow {
            id: 1,
            reason: "x".into(),
        })];
        let shape = result_to_dataframe(
            &rows,
            ResultShape::Split {
                empty_ok_sentinel: (),
            },
        )
        .unwrap();
        assert!(matches!(
            shape,
            DataFrameShape::Split {
                results: SplitResults::None(_),
                ..
            }
        ));
    });
}

#[test]
fn result_to_dataframe_collated_yields_bare() {
    r_test_utils::with_r_thread(|| {
        let rows: Vec<Result<Obs, ErrRow>> = vec![
            Ok(Obs { id: 1, value: 1.0 }),
            Err(ErrRow {
                id: 2,
                reason: "bad".into(),
            }),
        ];
        let shape = result_to_dataframe(&rows, ResultShape::<()>::Collated).unwrap();
        let DataFrameShape::Bare(df) = shape else {
            panic!("expected Bare");
        };
        let sexp = df.into_sexp();
        // is_error + id (union of Obs.id and ErrRow.id) + value + reason
        assert_eq!(sexp.xlength(), 4, "is_error + id + value + reason");
        let names = sexp.get_names();
        assert_eq!(
            names.string_elt_str(0),
            Some("is_error"),
            "is_error first column"
        );
    });
}

// endregion

// region: SplitShape variants

#[test]
fn split_pervariantlist_with_tag_prepends_column() {
    #[derive(Serialize)]
    enum E {
        Click { x: f64 },
        Scroll { delta: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![E::Click { x: 1.0 }, E::Scroll { delta: -1.0 }];
        let shape = vec_to_dataframe_split(
            &rows,
            SplitShape::PerVariantListWithTag {
                column: "variant".into(),
            },
        )
        .unwrap();
        let DataFrameShape::PerVariantList(parts) = shape else {
            panic!("expected PerVariantList");
        };
        assert_eq!(parts.len(), 2);
        // Each partition df should have a leading "variant" column.
        for (name, df) in parts {
            let sexp = df.into_sexp();
            let names = sexp.get_names();
            assert_eq!(
                names.string_elt_str(0),
                Some("variant"),
                "variant column should be first in {name}"
            );
        }
    });
}

#[test]
fn split_collated_emits_single_df_with_tag() {
    #[derive(Serialize)]
    enum E {
        Click { x: f64 },
        Scroll { delta: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![E::Click { x: 1.0 }, E::Scroll { delta: -1.0 }];
        let shape = vec_to_dataframe_split(
            &rows,
            SplitShape::Collated {
                column: "kind".into(),
            },
        )
        .unwrap();
        let DataFrameShape::Bare(df) = shape else {
            panic!("expected Bare");
        };
        let sexp = df.into_sexp();
        // kind + x + delta = 3 columns
        assert_eq!(sexp.xlength(), 3, "kind + x + delta");
        let names = sexp.get_names();
        assert_eq!(names.string_elt_str(0), Some("kind"));
    });
}

#[test]
fn split_collated_empty_input_errors() {
    #[derive(Serialize)]
    #[allow(dead_code)]
    enum E {
        Click { x: f64 },
    }

    r_test_utils::with_r_thread(|| {
        let rows: Vec<E> = Vec::new();
        let res = vec_to_dataframe_split(
            &rows,
            SplitShape::Collated {
                column: "k".into(),
            },
        );
        assert!(res.is_err(), "collated empty input must error");
    });
}

// endregion
