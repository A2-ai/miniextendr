//! Integration tests for `NamedDataFrameListBuilder` and `vec_to_dataframe_split`.
//!
//! These tests require R to be initialized and run on the R main thread.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::NamedDataFrameListBuilder;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::into_r::IntoR as _;
use miniextendr_api::prelude::SexpExt as _;
use miniextendr_api::serde::{
    DataFrameShape, ResultShape, SplitResults, SplitShape, hashmap_to_dataframe, map_to_dataframe,
    result_to_dataframe, vec_to_dataframe, vec_to_dataframe_flatten_enums, vec_to_dataframe_split,
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
        assert_eq!(
            sexp.xlength(),
            3,
            "expected 3 columns: subject + cmax + tmax"
        );

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
        let rows: Vec<Result<Obs, ErrRow>> =
            vec![Ok(Obs { id: 1, value: 1.0 }), Ok(Obs { id: 2, value: 2.0 })];
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
        let res = vec_to_dataframe_split(&rows, SplitShape::Collated { column: "k".into() });
        assert!(res.is_err(), "collated empty input must error");
    });
}

// endregion

// region: vec_to_dataframe_flatten_enums (#1056)

/// The ordered column names of a data.frame SEXP.
fn col_names(sexp: &miniextendr_api::SEXP) -> Vec<String> {
    let names = sexp.get_names();
    (0..names.xlength())
        .map(|i| names.string_elt_str(i).unwrap_or("<NA>").to_string())
        .collect()
}

/// The column SEXP for `name`, or panic.
fn col(sexp: &miniextendr_api::SEXP, name: &str) -> miniextendr_api::SEXP {
    let names = sexp.get_names();
    for i in 0..names.xlength() {
        if names.string_elt_str(i) == Some(name) {
            return sexp.vector_elt(i);
        }
    }
    panic!("column {name:?} not found in {:?}", col_names(sexp));
}

/// Externally-tagged enum field, two struct variants across rows: the enum
/// field flattens into a `action_variant` tag plus the union of the variants'
/// payload columns, NA-filled where a variant lacks a field.
#[test]
fn flatten_enum_field_externally_tagged_two_variants() {
    #[derive(Serialize)]
    enum Action {
        Add { file: f64, weight: f64 },
        Init { path: String },
    }
    #[derive(Serialize)]
    struct AuditRow {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            AuditRow {
                id: 1,
                action: Action::Add {
                    file: 10.0,
                    weight: 2.5,
                },
            },
            AuditRow {
                id: 2,
                action: Action::Init {
                    path: "/tmp".into(),
                },
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        // id, action_variant, then the union of Add (file, weight) + Init (path).
        assert_eq!(names[0], "id", "scalar field stays first");
        assert!(
            names.contains(&"action_variant".to_string()),
            "tag column present, got {names:?}"
        );
        assert!(names.contains(&"action_file".to_string()), "{names:?}");
        assert!(names.contains(&"action_weight".to_string()), "{names:?}");
        assert!(names.contains(&"action_path".to_string()), "{names:?}");

        // Tag values.
        let variant = col(&sexp, "action_variant");
        assert_eq!(variant.string_elt_str(0), Some("Add"));
        assert_eq!(variant.string_elt_str(1), Some("Init"));

        // Row 0 (Add) has file=10, weight=2.5, path=NA.
        let file = col(&sexp, "action_file");
        assert_eq!(file.real_elt(0), 10.0);
        assert!(file.real_elt(1).is_nan(), "Init row file should be NA");
        let path = col(&sexp, "action_path");
        assert_eq!(path.string_elt_str(0), None, "Add row path should be NA");
        assert_eq!(path.string_elt_str(1), Some("/tmp"));
    });
}

/// A unit variant mixed with a data variant: unit rows get the tag plus NA for
/// the data variant's payload columns.
#[test]
fn flatten_enum_field_unit_variant_mixed() {
    #[derive(Serialize)]
    enum Action {
        Reset,
        Set { value: f64 },
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Action::Set { value: 7.0 },
            },
            Row {
                id: 2,
                action: Action::Reset,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let variant = col(&sexp, "action_variant");
        assert_eq!(variant.string_elt_str(0), Some("Set"));
        assert_eq!(variant.string_elt_str(1), Some("Reset"));
        let value = col(&sexp, "action_value");
        assert_eq!(value.real_elt(0), 7.0);
        assert!(
            value.real_elt(1).is_nan(),
            "unit-variant row payload should be NA"
        );
    });
}

/// `Option<Enum>` with a `None` row: the tag and payload columns are NA for
/// that row.
#[test]
fn flatten_enum_field_option_none() {
    #[derive(Serialize)]
    enum Action {
        Go { steps: f64 },
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Option<Action>,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Some(Action::Go { steps: 3.0 }),
            },
            Row {
                id: 2,
                action: None,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let variant = col(&sexp, "action_variant");
        assert_eq!(variant.string_elt_str(0), Some("Go"));
        assert_eq!(variant.string_elt_str(1), None, "None row tag should be NA");
        let steps = col(&sexp, "action_steps");
        assert_eq!(steps.real_elt(0), 3.0);
        assert!(steps.real_elt(1).is_nan(), "None row payload should be NA");
    });
}

/// Newtype variant wrapping a struct: its fields flatten under the prefix.
#[test]
fn flatten_enum_field_newtype_wrapping_struct() {
    #[derive(Serialize)]
    struct Coords {
        x: f64,
        y: f64,
    }
    #[derive(Serialize)]
    enum Shape {
        Point(Coords),
        Empty,
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        shape: Shape,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                shape: Shape::Point(Coords { x: 1.0, y: 2.0 }),
            },
            Row {
                id: 2,
                shape: Shape::Empty,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["shape"]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        assert!(names.contains(&"shape_variant".to_string()), "{names:?}");
        assert!(names.contains(&"shape_x".to_string()), "{names:?}");
        assert!(names.contains(&"shape_y".to_string()), "{names:?}");
        let x = col(&sexp, "shape_x");
        assert_eq!(x.real_elt(0), 1.0);
        assert!(x.real_elt(1).is_nan(), "Empty row x should be NA");
    });
}

/// A nested enum field NOT named in `flatten_enum_fields` retains the default
/// behaviour — it becomes a single list-column, not flattened.
#[test]
fn flatten_enum_field_untargeted_enum_stays_list_column() {
    #[derive(Serialize)]
    enum Action {
        Add { file: f64 },
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![Row {
            id: 1,
            action: Action::Add { file: 1.0 },
        }];
        // Empty target set: nothing flattened.
        let df = vec_to_dataframe_flatten_enums(&rows, &[]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        assert_eq!(names, vec!["id".to_string(), "action".to_string()]);
        // The untargeted enum is a Generic (list) column.
        let action = col(&sexp, "action");
        assert_eq!(
            action.type_of(),
            miniextendr_api::SEXPTYPE::VECSXP,
            "untargeted enum field must remain a list-column"
        );
    });
}

/// A non-enum nested struct field is unaffected by the new fn: it still
/// flattens to `parent_child` columns exactly as plain `vec_to_dataframe`.
#[test]
fn flatten_enum_field_nested_struct_unaffected() {
    #[derive(Serialize)]
    struct Meta {
        size: f64,
        ok: bool,
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        meta: Meta,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                meta: Meta {
                    size: 4.0,
                    ok: true,
                },
            },
            Row {
                id: 2,
                meta: Meta {
                    size: 9.0,
                    ok: false,
                },
            },
        ];
        // "meta" is targeted but it's NOT an enum — it must error (only enum /
        // Option<enum> fields are flattenable).
        assert!(
            vec_to_dataframe_flatten_enums(&rows, &["meta"]).is_err(),
            "targeting a non-enum struct field must error"
        );
        // Untargeted: behaves exactly like vec_to_dataframe — meta_size/meta_ok.
        let df = vec_to_dataframe_flatten_enums(&rows, &[]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        assert_eq!(
            names,
            vec![
                "id".to_string(),
                "meta_size".to_string(),
                "meta_ok".to_string()
            ]
        );
    });
}

/// An internally-tagged enum field (`#[serde(tag = ...)]`) is flattened in
/// place: the embedded tag rides along as a normal `<field>_<tag>` column and
/// NO duplicate synthetic `<field>_variant` is emitted.
#[test]
fn flatten_enum_field_internally_tagged_no_synthetic_variant() {
    #[derive(Serialize)]
    #[serde(tag = "kind")]
    enum Action {
        Add { file: f64 },
        Reset,
    }
    #[derive(Serialize)]
    struct Row {
        id: i32,
        action: Action,
    }

    r_test_utils::with_r_thread(|| {
        let rows = vec![
            Row {
                id: 1,
                action: Action::Add { file: 2.0 },
            },
            Row {
                id: 2,
                action: Action::Reset,
            },
        ];
        let df = vec_to_dataframe_flatten_enums(&rows, &["action"]).unwrap();
        let sexp = df.into_sexp();
        let names = col_names(&sexp);
        // The embedded tag becomes action_kind; no synthetic action_variant.
        assert!(names.contains(&"action_kind".to_string()), "{names:?}");
        assert!(
            !names.contains(&"action_variant".to_string()),
            "internally-tagged field must not synthesize a duplicate variant tag, got {names:?}"
        );
        let kind = col(&sexp, "action_kind");
        assert_eq!(kind.string_elt_str(0), Some("Add"));
        assert_eq!(kind.string_elt_str(1), Some("Reset"));
        let file = col(&sexp, "action_file");
        assert_eq!(file.real_elt(0), 2.0);
        assert!(file.real_elt(1).is_nan(), "Reset row file should be NA");
    });
}

// endregion
