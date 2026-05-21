//! Integration tests for `dataframe_to_vec` and `with_dataframe_rows`.
//!
//! Each test builds an R data.frame via `vec_to_dataframe`, then exercises the
//! inverse path. All tests run on the R thread via `r_test_utils::with_r_thread`.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::IntoR as _;
use miniextendr_api::serde::{
    RSerdeError, dataframe_to_vec, vec_to_dataframe, with_dataframe_rows,
};
use serde::{Deserialize, Serialize};

// region: test types

/// Flat struct with three primitive types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FlatRow {
    a: i32,
    b: f64,
    c: String,
}

/// Flat struct with optional fields for NA testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OptionRow {
    x: Option<i32>,
    y: Option<String>,
}

/// Subset of `FlatRow` columns — extra column `c` is silently ignored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SubsetRow {
    a: i32,
    b: f64,
    // `c` intentionally omitted — should be silently ignored
}

/// Struct used to test callback that returns owned summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ScoreRow {
    score: f64,
}

// endregion

// region: test 1 — round-trip flat struct (owned, plain)

#[test]
fn round_trip_flat_struct_owned() {
    r_test_utils::with_r_thread(|| {
        let original = vec![
            FlatRow {
                a: 1,
                b: 1.5,
                c: "hello".into(),
            },
            FlatRow {
                a: 2,
                b: 2.5,
                c: "world".into(),
            },
            FlatRow {
                a: 3,
                b: 3.5,
                c: "foo".into(),
            },
        ];

        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<FlatRow> = dataframe_to_vec(sexp).expect("dataframe_to_vec");

        assert_eq!(back, original);
    });
}

// endregion

// region: test 2 — round-trip with Option fields (mixed Some/None)

#[test]
fn round_trip_option_fields() {
    r_test_utils::with_r_thread(|| {
        let original = vec![
            OptionRow {
                x: Some(1),
                y: Some("a".into()),
            },
            OptionRow { x: None, y: None },
            OptionRow {
                x: Some(3),
                y: Some("c".into()),
            },
            OptionRow {
                x: None,
                y: Some("d".into()),
            },
        ];

        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<OptionRow> = dataframe_to_vec(sexp).expect("dataframe_to_vec");

        assert_eq!(back, original);
    });
}

// endregion

// region: test 3 — NA on non-Option field returns error

#[test]
fn na_on_non_option_field_errors() {
    r_test_utils::with_r_thread(|| {
        // Build a data.frame with mixed Some/None for column `x`. A single
        // all-None row would degrade x to a logical-NA column (see CLAUDE.md
        // "FFI / codegen gotchas: ColumnarDataFrame::from_rows all-None
        // columns"); we need at least one Some to keep x as INTSXP so the
        // deserialiser sees a real INTSXP NA cell.
        let rows_with_na = vec![
            OptionRow {
                x: Some(1),
                y: Some("a".into()),
            },
            OptionRow {
                x: None,
                y: Some("b".into()),
            },
        ];
        let sexp = vec_to_dataframe(&rows_with_na)
            .expect("vec_to_dataframe")
            .into_sexp();

        // NonOptRow has `x: i32` (non-optional) — reading the NA cell must error
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct NonOptRow {
            x: i32,
            y: Option<String>,
        }

        let result: Result<Vec<NonOptRow>, RSerdeError> = dataframe_to_vec(sexp);
        assert!(
            result.is_err(),
            "expected error on NA in non-optional field"
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            matches!(err, RSerdeError::UnexpectedNa) || msg.contains("NA"),
            "expected UnexpectedNa error, got: {}",
            msg
        );
    });
}

// endregion

// region: test 4 — NA on Option field yields None

#[test]
fn na_on_option_field_yields_none() {
    r_test_utils::with_r_thread(|| {
        let rows_with_na = vec![
            OptionRow {
                x: Some(1),
                y: None,
            },
            OptionRow {
                x: None,
                y: Some("b".into()),
            },
        ];
        let sexp = vec_to_dataframe(&rows_with_na)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<OptionRow> = dataframe_to_vec(sexp).expect("dataframe_to_vec");
        assert_eq!(back, rows_with_na);
    });
}

// endregion

// region: test 5 — type-mismatch error includes column name

#[test]
fn type_mismatch_error_includes_column_name() {
    r_test_utils::with_r_thread(|| {
        // Build a data.frame with column `y` as character.
        let string_rows = vec![OptionRow {
            x: Some(1),
            y: Some("text".into()),
        }];
        let sexp = vec_to_dataframe(&string_rows)
            .expect("vec_to_dataframe")
            .into_sexp();

        #[derive(Debug, Deserialize)]
        struct WrongType {
            #[allow(dead_code)]
            x: Option<i32>,
            #[allow(dead_code)]
            y: i32, // mismatch: column `y` is character, field is i32
        }

        let result: Result<Vec<WrongType>, RSerdeError> = dataframe_to_vec(sexp);
        assert!(result.is_err(), "expected type mismatch error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("\"y\"") || msg.contains("y"),
            "error should mention column name 'y', got: {}",
            msg
        );
    });
}

// endregion

// region: test 6 — missing column is an error

#[test]
fn missing_column_is_error() {
    r_test_utils::with_r_thread(|| {
        // data.frame has `a` and `b` only; FlatRow also needs `c`
        let partial_rows = vec![SubsetRow { a: 1, b: 1.5 }];
        let sexp = vec_to_dataframe(&partial_rows)
            .expect("vec_to_dataframe")
            .into_sexp();

        // Try to deserialize as FlatRow (requires `c`)
        let result: Result<Vec<FlatRow>, RSerdeError> = dataframe_to_vec(sexp);
        // serde reports a missing field error (it won't find `c` in the MapAccess)
        assert!(result.is_err(), "expected missing-column error");
    });
}

// endregion

// region: test 7 — extra column is silently ignored

#[test]
fn extra_column_silently_ignored() {
    r_test_utils::with_r_thread(|| {
        // data.frame has `a`, `b`, `c` — SubsetRow only uses `a` and `b`
        let original = vec![
            FlatRow {
                a: 10,
                b: 2.0,
                c: "extra".into(),
            },
            FlatRow {
                a: 20,
                b: 4.0,
                c: "ignored".into(),
            },
        ];
        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();

        // Deserialize as SubsetRow — column `c` should be silently skipped
        let back: Vec<SubsetRow> = dataframe_to_vec(sexp).expect("dataframe_to_vec");
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].a, 10);
        assert_eq!(back[0].b, 2.0);
        assert_eq!(back[1].a, 20);
        assert_eq!(back[1].b, 4.0);
    });
}

// endregion

// region: test 8 — empty data.frame yields Ok(vec![])

#[test]
fn empty_dataframe_yields_empty_vec() {
    r_test_utils::with_r_thread(|| {
        let empty: Vec<FlatRow> = vec![];
        let sexp = vec_to_dataframe(&empty)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<FlatRow> = dataframe_to_vec(sexp).expect("dataframe_to_vec");
        assert!(back.is_empty());
    });
}

// endregion

// region: test 9 — with_dataframe_rows reads rows via callback

#[test]
fn with_dataframe_rows_reads_via_callback() {
    r_test_utils::with_r_thread(|| {
        // with_dataframe_rows uses T: for<'a> Deserialize<'a> (= DeserializeOwned),
        // so character fields materialise as String. The callback-scoped approach
        // avoids allocating intermediate row-list SEXPs.
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct NameId {
            name: String,
            id: i32,
        }

        let source = vec![
            NameId {
                name: "alice".into(),
                id: 1,
            },
            NameId {
                name: "bob".into(),
                id: 2,
            },
        ];
        let sexp = vec_to_dataframe(&source)
            .expect("vec_to_dataframe")
            .into_sexp();

        // The callback receives &[T]; borrows are scoped to the closure.
        let count = with_dataframe_rows(sexp, |rows: &[NameId]| {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].name, "alice");
            assert_eq!(rows[0].id, 1);
            assert_eq!(rows[1].name, "bob");
            assert_eq!(rows[1].id, 2);
            rows.len()
        })
        .expect("with_dataframe_rows");

        assert_eq!(count, 2);
    });
}

// endregion

// region: test 10 — with_dataframe_rows callback can return owned summary

#[test]
fn with_dataframe_rows_callback_returns_owned() {
    r_test_utils::with_r_thread(|| {
        let source = vec![
            ScoreRow { score: 10.0 },
            ScoreRow { score: 20.0 },
            ScoreRow { score: 30.0 },
        ];
        let sexp = vec_to_dataframe(&source)
            .expect("vec_to_dataframe")
            .into_sexp();

        let total: f64 =
            with_dataframe_rows::<ScoreRow, _, _>(sexp, |rows| rows.iter().map(|r| r.score).sum())
                .expect("with_dataframe_rows");

        assert!((total - 60.0).abs() < 1e-10);
    });
}

// endregion

// region: test 11 — non-data.frame input is an error

#[test]
fn non_dataframe_input_is_error() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};
        // Allocate a plain list (not a data.frame)
        let list_sexp = unsafe {
            let s = Rf_allocVector(SEXPTYPE::VECSXP, 2);
            Rf_protect(s);
            s
        };

        let result: Result<Vec<FlatRow>, RSerdeError> = dataframe_to_vec(list_sexp);
        unsafe { Rf_unprotect(1) };

        assert!(result.is_err(), "expected error on non-data.frame input");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("data.frame") || msg.contains("data frame"),
            "error should mention data.frame, got: {}",
            msg
        );
    });
}

// endregion
