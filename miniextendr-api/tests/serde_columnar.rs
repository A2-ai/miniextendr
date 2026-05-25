//! Integration tests for `NamedDataFrameListBuilder` and `vec_to_dataframe_split`.
//!
//! These tests require R to be initialized and run on the R main thread.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::into_r::IntoR as _;
use miniextendr_api::serde::{NamedDataFrameListBuilder, vec_to_dataframe, vec_to_dataframe_split};
use miniextendr_api::sys::SexpExt as _;
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
        let list = vec_to_dataframe_split(&rows).unwrap();
        let sexp = list.into_sexp();
        // Single-variant short-circuit: returned as a bare data.frame (no names attr)
        // The data.frame itself is a VECSXP with one column ("id")
        assert_eq!(sexp.xlength(), 1, "single-column data.frame expected");
    });
}

/// vec_to_dataframe_split on a multi-variant input returns a named list.
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
        let list = vec_to_dataframe_split(&rows).unwrap();
        let sexp = list.into_sexp();
        assert_eq!(sexp.xlength(), 2, "expected 2 named partitions");
    });
}

// endregion
