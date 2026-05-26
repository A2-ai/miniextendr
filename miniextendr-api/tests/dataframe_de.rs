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

// region: test — one-level nested struct round-trip

#[test]
fn round_trip_nested_struct_one_level() {
    r_test_utils::with_r_thread(|| {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Address {
            city: String,
            zip: String,
        }
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Person {
            name: String,
            address: Address,
        }

        let original = vec![
            Person {
                name: "alice".into(),
                address: Address {
                    city: "Lyon".into(),
                    zip: "69000".into(),
                },
            },
            Person {
                name: "bob".into(),
                address: Address {
                    city: "Paris".into(),
                    zip: "75001".into(),
                },
            },
        ];
        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<Person> = dataframe_to_vec(sexp).expect("dataframe_to_vec");
        assert_eq!(back, original);
    });
}

// endregion

// region: test — two-level nested struct round-trip

#[test]
fn round_trip_nested_struct_two_levels() {
    r_test_utils::with_r_thread(|| {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Address {
            city: String,
            zip: String,
        }
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Customer {
            name: String,
            address: Address,
        }
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Order {
            id: i32,
            customer: Customer,
        }

        let original = vec![
            Order {
                id: 1,
                customer: Customer {
                    name: "alice".into(),
                    address: Address {
                        city: "Lyon".into(),
                        zip: "69000".into(),
                    },
                },
            },
            Order {
                id: 2,
                customer: Customer {
                    name: "bob".into(),
                    address: Address {
                        city: "Paris".into(),
                        zip: "75001".into(),
                    },
                },
            },
        ];
        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<Order> = dataframe_to_vec(sexp).expect("dataframe_to_vec");
        assert_eq!(back, original);
    });
}

// endregion

// region: test — nested with Option fields

#[test]
fn round_trip_nested_struct_with_option() {
    r_test_utils::with_r_thread(|| {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Address {
            city: String,
            zip: Option<String>,
        }
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Person {
            name: String,
            address: Address,
        }

        let original = vec![
            Person {
                name: "alice".into(),
                address: Address {
                    city: "Lyon".into(),
                    zip: Some("69000".into()),
                },
            },
            Person {
                name: "bob".into(),
                address: Address {
                    city: "Paris".into(),
                    zip: None,
                },
            },
        ];
        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<Person> = dataframe_to_vec(sexp).expect("dataframe_to_vec");
        assert_eq!(back, original);
    });
}

// endregion

// region: test — missing nested column surfaces the inner-struct field

#[test]
fn nested_missing_column_errors_with_inner_field_name() {
    // serde's standard missing-field path reports the inner-struct field
    // (`zip`), not the prefixed column name (`address_zip`). This is a
    // consequence of letting `T::deserialize` drive the walk: when the
    // inner `RowMapAccess` runs out of keys, the inner visitor decides
    // which of its expected fields is "missing", not the outer
    // deserialiser. Reporting `address_zip` instead would require either
    // schema introspection (target type's expected fields) or stripping
    // the missing-field signal out of serde, both of which trade
    // simplicity for marginal UX gain. Documented limitation.
    r_test_utils::with_r_thread(|| {
        // Source has `address.city` only (no `address.zip`).
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct PartialAddress {
            city: String,
        }
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct PartialPerson {
            name: String,
            address: PartialAddress,
        }
        let partial = vec![PartialPerson {
            name: "alice".into(),
            address: PartialAddress {
                city: "Lyon".into(),
            },
        }];
        let sexp = vec_to_dataframe(&partial)
            .expect("vec_to_dataframe")
            .into_sexp();

        // Target struct requires `address_zip` too.
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Address {
            city: String,
            zip: String,
        }
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Person {
            name: String,
            address: Address,
        }

        let result: Result<Vec<Person>, RSerdeError> = dataframe_to_vec(sexp);
        assert!(result.is_err(), "expected missing-field error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("zip"),
            "error should mention the missing field 'zip', got: {}",
            msg
        );
    });
}

// endregion

// region: test — flat field name with underscore disambiguation

#[test]
fn flat_field_with_underscore_works_when_no_sibling_columns() {
    // Documented limitation: `last_name` looks like a nested path `last.name`.
    // The visitor still gets `last → Some(MaybeNested with bare_col = None)`,
    // which then recurses to a nested map containing only `name`. Serde sees
    // a struct with field `last` containing a sub-struct with `name`. So if
    // the target type is shaped that way, it works; otherwise it doesn't.
    //
    // This test pins the *working* shape so future refactors don't break it:
    // a struct deliberately modelled as nested still round-trips.
    r_test_utils::with_r_thread(|| {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Last {
            name: String,
        }
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct PersonNested {
            id: i32,
            last: Last,
        }

        let original = vec![
            PersonNested {
                id: 1,
                last: Last {
                    name: "smith".into(),
                },
            },
            PersonNested {
                id: 2,
                last: Last {
                    name: "jones".into(),
                },
            },
        ];
        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();
        let back: Vec<PersonNested> = dataframe_to_vec(sexp).expect("dataframe_to_vec");
        assert_eq!(back, original);
    });
}

#[test]
fn flat_field_with_underscore_fails_when_visitor_expects_flat_name() {
    // Documented limitation, locked in via test: if the producer side
    // generated a column literally named `last_name` for a flat string field,
    // the deserialiser greedy-splits to `last` and the visitor — expecting a
    // flat `last_name` field — sees the wrong key. serde reports a
    // missing-field error for `last_name`.
    r_test_utils::with_r_thread(|| {
        // Build a data.frame whose first column is literally `last_name` and
        // whose value is a plain string. We do this by serializing a struct
        // whose field is `last_name`. Since columnar serialization for a
        // primitive field uses the field name verbatim, the column will be
        // named `last_name`.
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct FlatProducer {
            id: i32,
            last_name: String,
        }
        let original = vec![FlatProducer {
            id: 1,
            last_name: "smith".into(),
        }];
        let sexp = vec_to_dataframe(&original)
            .expect("vec_to_dataframe")
            .into_sexp();

        // Identical struct on the consumer side — but the deserialiser
        // will interpret `last_name` as nested `last.name`, so the visitor
        // sees a key `last` and not `last_name`. serde reports "missing
        // field `last_name`".
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        #[allow(dead_code)]
        struct FlatConsumer {
            id: i32,
            last_name: String,
        }
        let result: Result<Vec<FlatConsumer>, RSerdeError> = dataframe_to_vec(sexp);
        assert!(
            result.is_err(),
            "underscore-in-flat-name limitation should surface as an error"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("last_name") || msg.contains("last") || msg.contains("unknown"),
            "error should mention the underscore-bearing field, got: {}",
            msg
        );
    });
}

// endregion

// region: test 11 — non-data.frame input is an error

#[test]
fn non_dataframe_input_is_error() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::sys::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};
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

// region: factor columns (issue #689)

mod factor_tests {
    use super::*;
    use miniextendr_api::factor::{build_factor, build_levels_sexp};
    use miniextendr_api::sys::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE, SexpExt};

    /// Build a one-column data.frame whose sole column is an R factor.
    ///
    /// Caller passes 1-based factor codes (with `NA_INTEGER` for NA cells)
    /// and the levels label set. The returned SEXP is protected on the R
    /// protect stack — caller must unprotect (`Rf_unprotect(1)`) when done.
    ///
    /// # Safety
    ///
    /// Caller must be on the R main thread, and must balance the protect
    /// with a matching `Rf_unprotect(1)`.
    unsafe fn make_factor_dataframe(col_name: &str, codes: &[i32], levels: &[&str]) -> SEXP {
        unsafe {
            let nrow = codes.len();

            // Build the list (data.frame backbone) and protect it first so
            // all child allocations stay GC-rooted via `list`'s VECSXP /
            // attributes.
            let list = Rf_allocVector(SEXPTYPE::VECSXP, 1);
            Rf_protect(list);

            // Levels STRSXP + factor INTSXP. Protect transients across each
            // `set_*` so a GC between alloc and store can't collect them.
            let levels_sexp = build_levels_sexp(levels);
            Rf_protect(levels_sexp);
            let col = build_factor(codes, levels_sexp);
            Rf_protect(col);
            list.set_vector_elt(0, col);
            Rf_unprotect(2); // levels_sexp + col are held by list now

            // names attribute
            let names_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
            Rf_protect(names_sexp);
            names_sexp.set_string_elt(0, SEXP::charsxp(col_name));
            list.set_names(names_sexp);
            Rf_unprotect(1);

            // class = "data.frame"
            let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
            Rf_protect(class_sexp);
            class_sexp.set_string_elt(0, SEXP::charsxp("data.frame"));
            list.set_class(class_sexp);
            Rf_unprotect(1);

            // compact row.names: c(NA_integer_, -nrow)
            let row_names = Rf_allocVector(SEXPTYPE::INTSXP, 2);
            Rf_protect(row_names);
            let rn = row_names.as_mut_slice::<i32>();
            rn[0] = i32::MIN; // NA_integer_
            rn[1] = -(nrow as i32);
            list.set_row_names(row_names);
            Rf_unprotect(1);

            // Leave `list` protected; caller's `Rf_unprotect(1)` releases it.
            list
        }
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct StringFactorRow {
        status: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct OptionalStringFactorRow {
        status: Option<String>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct IntCodeFactorRow {
        status: i32,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct OptionalIntCodeFactorRow {
        status: Option<i32>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct CharFactorRow {
        flag: char,
    }

    #[test]
    fn factor_to_string_returns_label() {
        r_test_utils::with_r_thread(|| unsafe {
            let df =
                make_factor_dataframe("status", &[1, 2, 3, 1], &["active", "pending", "archived"]);

            let rows: Vec<StringFactorRow> = dataframe_to_vec(df).expect("dataframe_to_vec");
            Rf_unprotect(1);

            assert_eq!(
                rows,
                vec![
                    StringFactorRow {
                        status: "active".into()
                    },
                    StringFactorRow {
                        status: "pending".into()
                    },
                    StringFactorRow {
                        status: "archived".into()
                    },
                    StringFactorRow {
                        status: "active".into()
                    },
                ]
            );
        });
    }

    #[test]
    fn factor_to_option_string_handles_na() {
        r_test_utils::with_r_thread(|| unsafe {
            let na = miniextendr_api::altrep_traits::NA_INTEGER;
            let df = make_factor_dataframe("status", &[1, na, 2], &["active", "pending"]);

            let rows: Vec<OptionalStringFactorRow> =
                dataframe_to_vec(df).expect("dataframe_to_vec");
            Rf_unprotect(1);

            assert_eq!(
                rows,
                vec![
                    OptionalStringFactorRow {
                        status: Some("active".into())
                    },
                    OptionalStringFactorRow { status: None },
                    OptionalStringFactorRow {
                        status: Some("pending".into())
                    },
                ]
            );
        });
    }

    #[test]
    fn factor_to_i32_returns_code() {
        r_test_utils::with_r_thread(|| unsafe {
            let df =
                make_factor_dataframe("status", &[1, 2, 3, 1], &["active", "pending", "archived"]);

            let rows: Vec<IntCodeFactorRow> = dataframe_to_vec(df).expect("dataframe_to_vec");
            Rf_unprotect(1);

            assert_eq!(
                rows,
                vec![
                    IntCodeFactorRow { status: 1 },
                    IntCodeFactorRow { status: 2 },
                    IntCodeFactorRow { status: 3 },
                    IntCodeFactorRow { status: 1 },
                ]
            );
        });
    }

    #[test]
    fn factor_to_option_i32_handles_na() {
        r_test_utils::with_r_thread(|| unsafe {
            let na = miniextendr_api::altrep_traits::NA_INTEGER;
            let df = make_factor_dataframe("status", &[1, na, 2], &["active", "pending"]);

            let rows: Vec<OptionalIntCodeFactorRow> =
                dataframe_to_vec(df).expect("dataframe_to_vec");
            Rf_unprotect(1);

            assert_eq!(
                rows,
                vec![
                    OptionalIntCodeFactorRow { status: Some(1) },
                    OptionalIntCodeFactorRow { status: None },
                    OptionalIntCodeFactorRow { status: Some(2) },
                ]
            );
        });
    }

    #[test]
    fn empty_factor_yields_empty_vec() {
        r_test_utils::with_r_thread(|| unsafe {
            let df = make_factor_dataframe("status", &[], &["active", "pending"]);

            let rows: Vec<StringFactorRow> = dataframe_to_vec(df).expect("dataframe_to_vec");
            Rf_unprotect(1);

            assert!(rows.is_empty());
        });
    }

    #[test]
    fn out_of_range_code_errors() {
        r_test_utils::with_r_thread(|| unsafe {
            // Code 5 is invalid; only 2 levels exist.
            let df = make_factor_dataframe("status", &[1, 5], &["active", "pending"]);

            let result: Result<Vec<StringFactorRow>, RSerdeError> = dataframe_to_vec(df);
            Rf_unprotect(1);

            let err = result.expect_err("expected out-of-range error");
            let msg = err.to_string();
            assert!(
                msg.contains("status") && msg.contains('5'),
                "expected error mentioning column and code 5, got: {}",
                msg
            );
        });
    }

    #[test]
    fn factor_to_char_single_char_level() {
        r_test_utils::with_r_thread(|| unsafe {
            let df = make_factor_dataframe("flag", &[1, 2], &["A", "B"]);

            let rows: Vec<CharFactorRow> = dataframe_to_vec(df).expect("dataframe_to_vec");
            Rf_unprotect(1);

            assert_eq!(
                rows,
                vec![CharFactorRow { flag: 'A' }, CharFactorRow { flag: 'B' }]
            );
        });
    }

    #[test]
    fn factor_to_char_multi_char_level_errors() {
        r_test_utils::with_r_thread(|| unsafe {
            let df = make_factor_dataframe("flag", &[1], &["active"]);

            let result: Result<Vec<CharFactorRow>, RSerdeError> = dataframe_to_vec(df);
            Rf_unprotect(1);

            let err = result.expect_err("expected single-character error");
            let msg = err.to_string();
            assert!(
                msg.contains("single character") || msg.contains("length"),
                "expected single-char error, got: {}",
                msg
            );
        });
    }

    /// `deserialize_any` consumers see the label, not the code. Exercised
    /// here via a `serde::de::Visitor` that records the type and value it
    /// receives — `deserialize_any` is what `untagged` enums, `serde_json::Value`,
    /// and other self-describing consumers route through.
    #[test]
    fn factor_via_deserialize_any_sees_label() {
        use serde::de::{Deserialize, Deserializer, Visitor};
        use std::fmt;

        /// A row with a single field that deserialises via `deserialize_any`.
        #[derive(Debug, PartialEq)]
        struct AnyRow {
            status: AnyValue,
        }

        #[derive(Debug, PartialEq)]
        enum AnyValue {
            Str(String),
            Int(i64),
            None,
        }

        struct AnyValueVisitor;
        impl<'de> Visitor<'de> for AnyValueVisitor {
            type Value = AnyValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "anything")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
                Ok(AnyValue::Str(v.to_owned()))
            }
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E> {
                Ok(AnyValue::Str(v.to_owned()))
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
                Ok(AnyValue::Str(v))
            }
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> {
                Ok(AnyValue::Int(v as i64))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(AnyValue::Int(v))
            }
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(AnyValue::None)
            }
        }
        impl<'de> Deserialize<'de> for AnyValue {
            fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                de.deserialize_any(AnyValueVisitor)
            }
        }
        impl<'de> Deserialize<'de> for AnyRow {
            fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                // Hand-rolled MapAccess consumer to keep the test focused.
                struct RowVisitor;
                impl<'de> Visitor<'de> for RowVisitor {
                    type Value = AnyRow;
                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "struct AnyRow")
                    }
                    fn visit_map<A: serde::de::MapAccess<'de>>(
                        self,
                        mut map: A,
                    ) -> Result<Self::Value, A::Error> {
                        let mut status: Option<AnyValue> = None;
                        while let Some(key) = map.next_key::<String>()? {
                            if key == "status" {
                                status = Some(map.next_value::<AnyValue>()?);
                            } else {
                                let _: serde::de::IgnoredAny = map.next_value()?;
                            }
                        }
                        Ok(AnyRow {
                            status: status
                                .ok_or_else(|| serde::de::Error::missing_field("status"))?,
                        })
                    }
                }
                de.deserialize_struct("AnyRow", &["status"], RowVisitor)
            }
        }

        r_test_utils::with_r_thread(|| unsafe {
            let df = make_factor_dataframe("status", &[1, 2], &["active", "pending"]);

            let rows: Vec<AnyRow> = dataframe_to_vec(df).expect("dataframe_to_vec");
            Rf_unprotect(1);

            assert_eq!(
                rows,
                vec![
                    AnyRow {
                        status: AnyValue::Str("active".into())
                    },
                    AnyRow {
                        status: AnyValue::Str("pending".into())
                    },
                ]
            );
        });
    }
}

// endregion
