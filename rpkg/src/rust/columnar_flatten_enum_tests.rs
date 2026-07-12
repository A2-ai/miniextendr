//! Fixtures for `vec_to_dataframe_flatten_enums` (#1056): flatten a nested
//! *enum* field into a `<field>_variant` tag column plus prefixed
//! `<field>_<subfield>` payload columns.
//!
//! Each fixture is no-arg so the fast `gctorture(TRUE)` sweep over the
//! package's exports exercises the SEXP-storage path (the columnar generic
//! buffers + the prefixed map emission) automatically.

use crate::serde::{Deserialize, Serialize};
use miniextendr_api::dataframe::{BuiltDataFrame, DataFrame};
use miniextendr_api::miniextendr;
use miniextendr_api::serde::{dataframe_to_vec, vec_to_dataframe_flatten_enums};

// region: Test types

/// Externally-tagged data enum: two struct variants with disjoint payloads.
#[derive(Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
enum Action {
    Add { file: f64, weight: f64 },
    Init { path: String },
}

/// Row carrying a scalar plus a nested enum field.
#[derive(Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
struct AuditRow {
    id: i32,
    user: String,
    action: Action,
}

/// Enum mixing a unit variant with a data variant.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
enum Lifecycle {
    Reset,
    Set { value: f64 },
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct LifecycleRow {
    id: i32,
    event: Lifecycle,
}

/// Row whose enum field is optional — `None` rows NA-fill the tag + payload.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct MaybeRow {
    id: i32,
    action: Option<Action>,
}

// endregion

// region: Fixtures

/// The issue's concrete case: an externally-tagged enum field flattened into
/// `action_variant` + the union of `action_file` / `action_weight` /
/// `action_path`. Rows mix the `Add` and `Init` variants so NA-fill applies.
#[miniextendr]
pub fn flatten_enum_field_fixture() -> BuiltDataFrame {
    let rows = vec![
        AuditRow {
            id: 1,
            user: "alice".into(),
            action: Action::Add {
                file: 10.0,
                weight: 2.5,
            },
        },
        AuditRow {
            id: 2,
            user: "bob".into(),
            action: Action::Init {
                path: "/tmp".into(),
            },
        },
        AuditRow {
            id: 3,
            user: "carol".into(),
            action: Action::Add {
                file: 30.0,
                weight: 4.0,
            },
        },
    ];
    vec_to_dataframe_flatten_enums(&rows, &["action"]).expect("flatten enum field")
}

/// Unit variant mixed with a data variant: the unit row's payload column is NA.
#[miniextendr]
pub fn flatten_enum_unit_variant_fixture() -> BuiltDataFrame {
    let rows = vec![
        LifecycleRow {
            id: 1,
            event: Lifecycle::Set { value: 7.0 },
        },
        LifecycleRow {
            id: 2,
            event: Lifecycle::Reset,
        },
    ];
    vec_to_dataframe_flatten_enums(&rows, &["event"]).expect("flatten unit variant")
}

/// `Option<Enum>` with a `None` row: tag + payload columns are NA for it.
#[miniextendr]
pub fn flatten_enum_option_none_fixture() -> BuiltDataFrame {
    let rows = vec![
        MaybeRow {
            id: 1,
            action: Some(Action::Add {
                file: 5.0,
                weight: 1.0,
            }),
        },
        MaybeRow {
            id: 2,
            action: None,
        },
    ];
    vec_to_dataframe_flatten_enums(&rows, &["action"]).expect("flatten option none")
}

/// Reverse of [`flatten_enum_field_fixture`] (#1060): read a flattened-enum
/// data.frame back into `Vec<AuditRow>` via `dataframe_to_vec`, then
/// re-serialize it. A correct round trip returns an identical frame — the
/// R-facing proof that the split/flattened enum reader is symmetric with the
/// writer.
/// @param df A data.frame in `flatten_enum_field_fixture()`'s shape (`id`,
///   `user`, `action_variant`, `action_file`, `action_weight`, `action_path`).
#[miniextendr]
pub fn flatten_enum_field_roundtrip(df: DataFrame) -> BuiltDataFrame {
    let rows: Vec<AuditRow> =
        dataframe_to_vec(df.as_sexp()).expect("read flattened enum data.frame back to Vec<T>");
    vec_to_dataframe_flatten_enums(&rows, &["action"]).expect("re-flatten enum field")
}

// endregion
