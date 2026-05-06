//! Tests for `ColumnarDataFrame::from_rows` all-`None` column downgrade.
//!
//! When every row has `None` for an `Option<T>` field, the column lands as a
//! logical NA vector (LGLSXP) rather than `list(NULL, NULL, …)`.  R coerces
//! logical NA to the surrounding type on first use.

use crate::serde::Serialize;
use miniextendr_api::miniextendr;
use miniextendr_api::serde::ColumnarDataFrame;
use std::collections::HashMap;

// region: Test types

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptU64 {
    name: String,
    stored: Option<u64>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptString {
    id: i32,
    label: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptBool {
    id: i32,
    flag: Option<bool>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct InnerStruct {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptUserStruct {
    id: i32,
    point: Option<InnerStruct>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptHashMap {
    id: i32,
    attrs: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptBytes {
    id: i32,
    data: Option<Vec<u8>>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithBytesAndOpt {
    raw: Vec<u8>,
    stored: Option<u64>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct InnerWithOpt {
    // No skip_serializing_if — size is always serialized so it appears in the
    // schema and can demonstrate the all-None downgrade via flatten.
    size: Option<u64>,
    name: String,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithFlattenedOptField {
    id: i32,
    #[serde(flatten)]
    inner: InnerWithOpt,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
#[serde(tag = "kind")]
enum EventWithOptX {
    A { x: Option<u64> },
    B { x: Option<u64> },
}

/// For compound-vs-compound different-shapes test.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptPoint {
    id: i32,
    point: Option<InnerStruct>,
}

/// Enum with two variants that have different nested struct shapes.
/// Used to verify existing-wins semantics for compound-vs-compound.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
#[serde(tag = "kind")]
enum EventDifferentNested {
    A { value: f64 },
    B { value: f64, extra: f64 },
}

// endregion

// region: All-None fixtures — downgrade fires

/// All-None `Option<u64>` column — single row (the dvs2 trigger case).
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_u64_all_none_single() -> ColumnarDataFrame {
    let rows = vec![WithOptU64 {
        name: "a".into(),
        stored: None,
    }];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// All-None `Option<u64>` column — multiple rows.
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_u64_all_none_multi() -> ColumnarDataFrame {
    let rows = vec![
        WithOptU64 { name: "a".into(), stored: None },
        WithOptU64 { name: "b".into(), stored: None },
        WithOptU64 { name: "c".into(), stored: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// All-None `Option<String>` column.
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_string_all_none() -> ColumnarDataFrame {
    let rows = vec![
        WithOptString { id: 1, label: None },
        WithOptString { id: 2, label: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// All-None `Option<bool>` column.
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_bool_all_none() -> ColumnarDataFrame {
    let rows = vec![
        WithOptBool { id: 1, flag: None },
        WithOptBool { id: 2, flag: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// All-None `Option<UserStruct>` — nested struct with all entries `None`.
///
/// When every row is `None`, the probe never sees any struct fields, so struct
/// expansion never fires.  The entire `point` field becomes a single logical NA
/// column under the field name `"point"` — not per-subfield `"point_x"`/`"point_y"`.
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_user_struct_all_none() -> ColumnarDataFrame {
    let rows = vec![
        WithOptUserStruct { id: 1, point: None },
        WithOptUserStruct { id: 2, point: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// All-None `Option<HashMap<…>>` — foreign generic, downgrade still fires.
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_hashmap_all_none() -> ColumnarDataFrame {
    let rows = vec![
        WithOptHashMap { id: 1, attrs: None },
        WithOptHashMap { id: 2, attrs: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// All-None `Option<Vec<u8>>` — downgrade fires (no values, no list semantics to preserve).
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_bytes_all_none() -> ColumnarDataFrame {
    let rows = vec![
        WithOptBytes { id: 1, data: None },
        WithOptBytes { id: 2, data: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion

// region: Mixed Some/None fixtures — downgrade must NOT fire

/// Mixed `Option<u64>`: some rows have values, no downgrade.
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_u64_mixed() -> ColumnarDataFrame {
    let rows = vec![
        WithOptU64 { name: "a".into(), stored: Some(42) },
        WithOptU64 { name: "b".into(), stored: None },
        WithOptU64 { name: "c".into(), stored: Some(99) },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// Mixed `Option<String>`: some rows have values.
///
/// @export
#[miniextendr]
pub fn test_columnar_opt_string_mixed() -> ColumnarDataFrame {
    let rows = vec![
        WithOptString { id: 1, label: Some("hello".into()) },
        WithOptString { id: 2, label: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion

// region: Vec<u8> with values — no downgrade (stays a list column)

/// `Vec<u8>` field with values — stays a list column regardless.
///
/// @export
#[miniextendr]
pub fn test_columnar_bytes_with_values() -> ColumnarDataFrame {
    let rows = vec![
        WithOptBytes { id: 1, data: Some(vec![1u8, 2, 3]) },
        WithOptBytes { id: 2, data: Some(vec![4u8, 5]) },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion

// region: Mixed columns — bytes with values + optional all-None

/// `Vec<u8>` column with values alongside an all-None `Option<u64>` column.
/// The bytes column stays a list; the optional column downgrades to logical NA.
///
/// @export
#[miniextendr]
pub fn test_columnar_bytes_and_opt_none() -> ColumnarDataFrame {
    let rows = vec![
        WithBytesAndOpt { raw: vec![1u8, 2], stored: None },
        WithBytesAndOpt { raw: vec![3u8], stored: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion

// region: Flatten with all-None inner field

/// `#[serde(flatten)]` with all-None inner field: the flattened optional field
/// becomes a logical NA column.
///
/// @export
#[miniextendr]
pub fn test_columnar_flatten_all_none() -> ColumnarDataFrame {
    let rows = vec![
        WithFlattenedOptField {
            id: 1,
            inner: InnerWithOpt { size: None, name: "a".into() },
        },
        WithFlattenedOptField {
            id: 2,
            inner: InnerWithOpt { size: None, name: "b".into() },
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion

// region: Enum union: all-variant-A with x = None, then one variant-B with x = Some

/// Enum: all variant-A rows have `x = None` → logical NA column.
///
/// @export
#[miniextendr]
pub fn test_columnar_enum_all_none() -> ColumnarDataFrame {
    let rows = vec![
        EventWithOptX::A { x: None },
        EventWithOptX::A { x: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// Enum: variant-A rows with `x = None`, then one variant-B row with `x = Some(42)`.
///
/// With the two-phase discovery, the probe scans all rows before resolving the
/// schema.  Variant-B's `x = Some(42u64)` contributes a `Scalar(Real)` candidate
/// which beats the `Scalar(Generic)` from variant-A's `x = None`.  The column
/// ends up as a numeric vector with `NA` in row 1.
///
/// @export
#[miniextendr]
pub fn test_columnar_enum_some_flips_type() -> ColumnarDataFrame {
    let rows = vec![
        EventWithOptX::A { x: None },
        EventWithOptX::B { x: Some(42) },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion

// region: Schema upgrade — first-row-None then Some

/// First-row `x = None`, second row `x = Some(42u64)`.
///
/// Two-phase discovery: the `Scalar(Real)` candidate from row 2 beats
/// `Scalar(Generic)` from row 1.  Result: numeric column with NA at index 1.
///
/// @export
#[miniextendr]
pub fn test_columnar_schema_upgrade_scalar() -> ColumnarDataFrame {
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        x: Option<u64>,
    }
    let rows = vec![Row { x: None }, Row { x: Some(42) }];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// First-row `point = None`, second row `point = Some(Point{x:1.0, y:2.0})`.
///
/// Two-phase discovery: `Compound` candidate from row 2 beats `Scalar(Generic)`
/// from row 1.  Result: columns `point_x` and `point_y`, with NA in row 1.
///
/// @export
#[miniextendr]
pub fn test_columnar_schema_upgrade_nested() -> ColumnarDataFrame {
    let rows = vec![
        WithOptPoint { id: 1, point: None },
        WithOptPoint { id: 2, point: Some(InnerStruct { x: 1.0, y: 2.0 }) },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// Multiple leading None rows before a Some value.
///
/// Rows: None, None, Some(42u64), None.  Two-phase discovery resolves the
/// column to `Scalar(Real)`.  Result: numeric column with NA at positions 1, 2, 4.
///
/// @export
#[miniextendr]
pub fn test_columnar_schema_upgrade_multi_none_first() -> ColumnarDataFrame {
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        x: Option<u64>,
    }
    let rows = vec![
        Row { x: None },
        Row { x: None },
        Row { x: Some(42) },
        Row { x: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// Compound-vs-compound with different struct shapes: existing-wins semantics.
///
/// Rows alternate between variant A (has `value` only) and variant B (has `value`
/// and `extra`).  The `extra` field only appears in B rows; `value` appears in both.
/// Because both rows contribute scalar candidates for `value`, the first non-Generic
/// wins.  The schema discovers both `value` and `extra` fields (from their respective
/// rows) because they are distinct keys — they are *not* the same compound key.
///
/// This fixture tests that the per-key candidate accumulation works correctly for
/// an enum with fields that differ between variants.
///
/// # TODO: union recursion
/// When a single *key* maps to two different Compound shapes (e.g. two variant
/// rows of an internally-tagged enum where the nested struct differs per variant),
/// the first Compound wins silently.  Recursive Compound union is tracked as a
/// separate follow-up issue.
///
/// @export
#[miniextendr]
pub fn test_columnar_compound_different_shapes() -> ColumnarDataFrame {
    let rows = vec![
        EventDifferentNested::A { value: 1.0 },
        EventDifferentNested::B { value: 2.0, extra: 3.0 },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion
