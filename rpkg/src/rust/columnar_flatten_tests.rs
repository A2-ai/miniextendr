//! Tests for columnar DataFrame serializer: recursive struct flattening,
//! `#[serde(flatten)]`, `#[serde(skip_serializing_if)]`, rename/drop/select API.

use crate::serde::Serialize;
use miniextendr_api::IntoR;
use miniextendr_api::miniextendr;
use miniextendr_api::serde::ColumnarDataFrame;

// region: Test types

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct Inner {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct Outer {
    label: String,
    point: Inner,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithOptionalStruct {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<Inner>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct Deep {
    a: String,
    mid: Mid,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct Mid {
    b: i32,
    leaf: Leaf,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct Leaf {
    c: f64,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithFlatten {
    id: i32,
    #[serde(flatten)]
    coords: Inner,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithSkip {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    value: f64,
}

/// Untagged enum with different fields per variant (like dvs StatusDetail).
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithUntaggedEnum {
    path: String,
    #[serde(flatten)]
    detail: UntaggedDetail,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
#[serde(untagged)]
enum UntaggedDetail {
    Ok { status: String, size: u64 },
    Err { error: String },
}

/// Internally tagged enum.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
#[serde(tag = "kind")]
enum TaggedEvent {
    Click { x: f64, y: f64 },
    Scroll { delta: f64 },
}

// endregion

/// @export
#[miniextendr]
pub fn test_columnar_nested() -> ColumnarDataFrame {
    let rows = vec![
        Outer {
            label: "a".into(),
            point: Inner { x: 1.0, y: 2.0 },
        },
        Outer {
            label: "b".into(),
            point: Inner { x: 3.0, y: 4.0 },
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_optional_struct() -> ColumnarDataFrame {
    let rows = vec![
        WithOptionalStruct {
            name: "has".into(),
            extra: Some(Inner { x: 1.0, y: 2.0 }),
        },
        WithOptionalStruct {
            name: "none".into(),
            extra: None,
        },
        WithOptionalStruct {
            name: "also".into(),
            extra: Some(Inner { x: 5.0, y: 6.0 }),
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_deep_nesting() -> ColumnarDataFrame {
    let rows = vec![
        Deep {
            a: "x".into(),
            mid: Mid {
                b: 1,
                leaf: Leaf { c: 10.0 },
            },
        },
        Deep {
            a: "y".into(),
            mid: Mid {
                b: 2,
                leaf: Leaf { c: 20.0 },
            },
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_serde_flatten() -> ColumnarDataFrame {
    let rows = vec![
        WithFlatten {
            id: 1,
            coords: Inner { x: 10.0, y: 20.0 },
        },
        WithFlatten {
            id: 2,
            coords: Inner { x: 30.0, y: 40.0 },
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_skip_serializing_if() -> ColumnarDataFrame {
    let rows = vec![
        WithSkip {
            name: "a".into(),
            tag: Some("t1".into()),
            value: 1.0,
        },
        WithSkip {
            name: "b".into(),
            tag: None,
            value: 2.0,
        },
        WithSkip {
            name: "c".into(),
            tag: Some("t3".into()),
            value: 3.0,
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_rename() -> ColumnarDataFrame {
    let rows = vec![
        Outer {
            label: "a".into(),
            point: Inner { x: 1.0, y: 2.0 },
        },
        Outer {
            label: "b".into(),
            point: Inner { x: 3.0, y: 4.0 },
        },
    ];
    ColumnarDataFrame::from_rows(&rows)
        .expect("from_rows")
        .rename("point_x", "px")
        .rename("point_y", "py")
}

/// @export
#[miniextendr]
pub fn test_columnar_rename_noop() -> ColumnarDataFrame {
    let rows = vec![Inner { x: 1.0, y: 2.0 }];
    ColumnarDataFrame::from_rows(&rows)
        .expect("from_rows")
        .rename("nonexistent", "z")
}

/// @export
#[miniextendr]
pub fn test_columnar_empty() -> ColumnarDataFrame {
    ColumnarDataFrame::from_rows::<Inner>(&[]).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_drop() -> ColumnarDataFrame {
    let rows = vec![
        Outer {
            label: "a".into(),
            point: Inner { x: 1.0, y: 2.0 },
        },
        Outer {
            label: "b".into(),
            point: Inner { x: 3.0, y: 4.0 },
        },
    ];
    ColumnarDataFrame::from_rows(&rows)
        .expect("from_rows")
        .drop("point_y")
}

/// @export
#[miniextendr]
pub fn test_columnar_select() -> ColumnarDataFrame {
    let rows = vec![
        Outer {
            label: "a".into(),
            point: Inner { x: 1.0, y: 2.0 },
        },
        Outer {
            label: "b".into(),
            point: Inner { x: 3.0, y: 4.0 },
        },
    ];
    ColumnarDataFrame::from_rows(&rows)
        .expect("from_rows")
        .select(&["point_y", "label"])
}

/// with_column: replace an existing integer column with a character SEXP of
/// matching length.
///
/// @export
#[miniextendr]
pub fn test_columnar_with_column_replace() -> ColumnarDataFrame {
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Row {
        id: i32,
        value: f64,
    }
    let rows = vec![
        Row { id: 1, value: 10.0 },
        Row { id: 2, value: 20.0 },
        Row { id: 3, value: 30.0 },
    ];
    let replacement = vec!["a".to_string(), "b".to_string(), "c".to_string()].into_sexp();
    ColumnarDataFrame::from_rows(&rows)
        .expect("from_rows")
        .with_column("id", replacement)
}

/// with_column: append a new column when the name doesn't exist.
///
/// @export
#[miniextendr]
pub fn test_columnar_with_column_append() -> ColumnarDataFrame {
    let rows = vec![Inner { x: 1.0, y: 2.0 }, Inner { x: 3.0, y: 4.0 }];
    let new_col = vec!["first".to_string(), "second".to_string()].into_sexp();
    ColumnarDataFrame::from_rows(&rows)
        .expect("from_rows")
        .with_column("label", new_col)
}

/// strip_prefix: remove "point_" from column names.
///
/// @export
#[miniextendr]
pub fn test_columnar_strip_prefix() -> ColumnarDataFrame {
    let rows = vec![
        Outer {
            label: "a".into(),
            point: Inner { x: 1.0, y: 2.0 },
        },
        Outer {
            label: "b".into(),
            point: Inner { x: 3.0, y: 4.0 },
        },
    ];
    ColumnarDataFrame::from_rows(&rows)
        .expect("from_rows")
        .strip_prefix("point_")
}

/// Untagged enum: Ok rows have status+size, Err rows have error.
/// Multi-row discovery unions them all.
///
/// @export
#[miniextendr]
pub fn test_columnar_untagged_enum() -> ColumnarDataFrame {
    let rows = vec![
        WithUntaggedEnum {
            path: "a.txt".into(),
            detail: UntaggedDetail::Ok {
                status: "current".into(),
                size: 100,
            },
        },
        WithUntaggedEnum {
            path: "b.txt".into(),
            detail: UntaggedDetail::Err {
                error: "not found".into(),
            },
        },
        WithUntaggedEnum {
            path: "c.txt".into(),
            detail: UntaggedDetail::Ok {
                status: "absent".into(),
                size: 200,
            },
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// Internally tagged enum: "kind" column acts as discriminator.
///
/// @export
#[miniextendr]
pub fn test_columnar_tagged_enum() -> ColumnarDataFrame {
    let rows = vec![
        TaggedEvent::Click { x: 10.0, y: 20.0 },
        TaggedEvent::Scroll { delta: -3.5 },
        TaggedEvent::Click { x: 30.0, y: 40.0 },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// region: All-None type-inference fixtures

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithAllNoneReal {
    x: f64,
    score: Option<f64>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithAllNoneInt {
    x: f64,
    count: Option<i32>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithAllNoneBool {
    x: f64,
    flag: Option<bool>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithAllNoneStr {
    x: f64,
    label: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithLeadingNone {
    x: f64,
    value: Option<f64>,
}

/// `Option<Vec<f64>>` is not in the type-name lookup table, so it stays
/// `Generic` even when all rows are `None` → list column with NULLs.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct WithGenericField {
    x: f64,
    tags: Option<Vec<f64>>,
}

/// @export
#[miniextendr]
pub fn test_columnar_all_none_real() -> ColumnarDataFrame {
    let rows = vec![
        WithAllNoneReal {
            x: 1.0,
            score: None,
        },
        WithAllNoneReal {
            x: 2.0,
            score: None,
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_all_none_int() -> ColumnarDataFrame {
    let rows = vec![
        WithAllNoneInt {
            x: 1.0,
            count: None,
        },
        WithAllNoneInt {
            x: 2.0,
            count: None,
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_all_none_bool() -> ColumnarDataFrame {
    let rows = vec![
        WithAllNoneBool { x: 1.0, flag: None },
        WithAllNoneBool { x: 2.0, flag: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// @export
#[miniextendr]
pub fn test_columnar_all_none_str() -> ColumnarDataFrame {
    let rows = vec![
        WithAllNoneStr {
            x: 1.0,
            label: None,
        },
        WithAllNoneStr {
            x: 2.0,
            label: None,
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// Option<Vec<f64>> stays Generic (not in type-name lookup) → list column with NULLs.
///
/// @export
#[miniextendr]
pub fn test_columnar_generic_fallback() -> ColumnarDataFrame {
    let rows = vec![
        WithGenericField { x: 1.0, tags: None },
        WithGenericField { x: 2.0, tags: None },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

/// Leading Nones followed by Some: type is inferred when the Some row is reached.
///
/// @export
#[miniextendr]
pub fn test_columnar_leading_none() -> ColumnarDataFrame {
    let rows = vec![
        WithLeadingNone {
            x: 1.0,
            value: None,
        },
        WithLeadingNone {
            x: 2.0,
            value: None,
        },
        WithLeadingNone {
            x: 3.0,
            value: Some(42.0),
        },
    ];
    ColumnarDataFrame::from_rows(&rows).expect("from_rows")
}

// endregion
