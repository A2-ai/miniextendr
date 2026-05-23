//! Integration tests for `dataframe_to_vec_borrowed` and `BorrowedRows<'a, T>` (#671b).
//!
//! Sister tests to `tests/dataframe_de.rs`; same per-row deserialisation but
//! the result is wrapped in `Protected<'a, Vec<T>>` (alias `BorrowedRows`).

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::IntoR as _;
use miniextendr_api::serde::{
    BorrowedRows, RSerdeError, dataframe_to_vec_borrowed, vec_to_dataframe,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Row {
    id: i32,
    score: f64,
    label: Option<String>,
}

// region: round-trip via BorrowedRows

#[test]
fn round_trip_via_borrowed_rows() {
    r_test_utils::with_r_thread(|| {
        let original = vec![
            Row {
                id: 1,
                score: 1.5,
                label: Some("a".into()),
            },
            Row {
                id: 2,
                score: 2.5,
                label: None,
            },
            Row {
                id: 3,
                score: 3.5,
                label: Some("c".into()),
            },
        ];
        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();

        let bundle: BorrowedRows<'_, Row> =
            dataframe_to_vec_borrowed(sexp).expect("dataframe_to_vec_borrowed");

        // Access via Deref<Target = Vec<Row>>
        assert_eq!(bundle.len(), original.len());
        for (a, b) in original.iter().zip(bundle.iter()) {
            assert_eq!(a, b);
        }
    });
}

// endregion

// region: empty data.frame

#[test]
fn empty_dataframe_yields_empty_borrowed_rows() {
    r_test_utils::with_r_thread(|| {
        let empty: Vec<Row> = vec![];
        let sexp = vec_to_dataframe(&empty)
            .expect("vec_to_dataframe")
            .into_sexp();
        let bundle: BorrowedRows<'_, Row> =
            dataframe_to_vec_borrowed(sexp).expect("dataframe_to_vec_borrowed");
        assert!(bundle.is_empty());
    });
}

// endregion

// region: NA on non-Option field surfaces error

#[test]
fn na_on_non_option_field_errors_via_borrowed() {
    r_test_utils::with_r_thread(|| {
        #[derive(Debug, Serialize, Deserialize)]
        struct OptIn {
            x: Option<i32>,
            y: Option<String>,
        }
        // Two rows so x stays INTSXP (not degraded to logical NA column).
        let with_na = vec![
            OptIn {
                x: Some(1),
                y: Some("a".into()),
            },
            OptIn {
                x: None,
                y: Some("b".into()),
            },
        ];
        let sexp = vec_to_dataframe(&with_na)
            .expect("vec_to_dataframe")
            .into_sexp();

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct NonOpt {
            x: i32,
            y: Option<String>,
        }

        let result: Result<BorrowedRows<'_, NonOpt>, RSerdeError> = dataframe_to_vec_borrowed(sexp);
        assert!(result.is_err(), "expected error on NA in non-Option field");
        let err = result.unwrap_err();
        assert!(
            matches!(err, RSerdeError::UnexpectedNa) || err.to_string().contains("NA"),
            "expected UnexpectedNa, got: {err}"
        );
    });
}

// endregion

// region: non-data.frame input is rejected

#[test]
fn non_dataframe_input_is_error() {
    r_test_utils::with_r_thread(|| {
        // A plain integer vector — not a data.frame.
        let sexp = vec![1i32, 2, 3].into_sexp();
        let result: Result<BorrowedRows<'_, Row>, RSerdeError> = dataframe_to_vec_borrowed(sexp);
        assert!(result.is_err(), "expected error on non-data.frame input");
    });
}

// endregion
