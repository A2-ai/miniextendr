//! Tests for columnar DataFrame serializer: recursive struct flattening,
//! `#[serde(flatten)]`, `#[serde(skip_serializing_if)]`, rename/drop/select API.

use crate::serde::Serialize;
use miniextendr_api::IntoR;
use miniextendr_api::miniextendr;
use miniextendr_api::serde::{
    ColumnarDataFrame, DataFrameShape, SplitShape, vec_to_dataframe_split,
};

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

// region: vec_to_dataframe_split fixtures

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
enum ExtEvent {
    Click { x: f64, y: f64 },
    Scroll { delta: f64 },
}

/// Externally-tagged enum split: per-variant data.frames, no NA columns.
///
/// @export
#[miniextendr]
pub fn test_columnar_ext_tagged_split() -> DataFrameShape {
    let rows = vec![
        ExtEvent::Click { x: 1.0, y: 2.0 },
        ExtEvent::Scroll { delta: -3.0 },
        ExtEvent::Click { x: 5.0, y: 6.0 },
    ];
    vec_to_dataframe_split(&rows, SplitShape::PerVariantList).expect("split")
}

/// Internally-tagged enum split: tag column dropped, per-variant data.frames.
///
/// @export
#[miniextendr]
pub fn test_columnar_int_tagged_split() -> DataFrameShape {
    let rows = vec![
        TaggedEvent::Click { x: 10.0, y: 20.0 },
        TaggedEvent::Scroll { delta: -3.5 },
        TaggedEvent::Click { x: 30.0, y: 40.0 },
    ];
    vec_to_dataframe_split(&rows, SplitShape::PerVariantList).expect("split")
}

/// Single-variant externally-tagged split returns a bare data.frame.
///
/// @export
#[miniextendr]
pub fn test_columnar_single_variant_split() -> DataFrameShape {
    let rows = vec![
        ExtEvent::Click { x: 1.0, y: 2.0 },
        ExtEvent::Click { x: 3.0, y: 4.0 },
    ];
    vec_to_dataframe_split(&rows, SplitShape::PerVariantList).expect("split")
}

/// Empty input split — variant set is unknowable, returns an empty named list.
///
/// @export
#[miniextendr]
pub fn test_columnar_empty_split() -> DataFrameShape {
    let rows: Vec<ExtEvent> = Vec::new();
    vec_to_dataframe_split(&rows, SplitShape::PerVariantList).expect("split")
}

/// Externally-tagged enum split WITH a variant-tag column on each partition.
///
/// @export
#[miniextendr]
pub fn test_columnar_split_with_tag() -> DataFrameShape {
    let rows = vec![
        ExtEvent::Click { x: 1.0, y: 2.0 },
        ExtEvent::Scroll { delta: -3.0 },
        ExtEvent::Click { x: 5.0, y: 6.0 },
    ];
    vec_to_dataframe_split(
        &rows,
        SplitShape::PerVariantListWithTag {
            column: "variant".into(),
        },
    )
    .expect("split")
}

/// Collated split: single data.frame with union schema + variant column.
///
/// @export
#[miniextendr]
pub fn test_columnar_split_collated() -> DataFrameShape {
    let rows = vec![
        ExtEvent::Click { x: 1.0, y: 2.0 },
        ExtEvent::Scroll { delta: -3.0 },
        ExtEvent::Click { x: 5.0, y: 6.0 },
    ];
    vec_to_dataframe_split(
        &rows,
        SplitShape::Collated {
            column: "variant".into(),
        },
    )
    .expect("split")
}

// endregion

// region: map_to_dataframe fixtures

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct CmaxValue {
    cmax: f64,
    tmax: f64,
}

/// map_to_dataframe over BTreeMap<i32, CmaxValue>.
///
/// @export
#[miniextendr]
pub fn test_map_to_dataframe_btreemap() -> ColumnarDataFrame {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<i32, CmaxValue> = BTreeMap::new();
    map.insert(
        1,
        CmaxValue {
            cmax: 10.5,
            tmax: 2.0,
        },
    );
    map.insert(
        2,
        CmaxValue {
            cmax: 8.1,
            tmax: 3.5,
        },
    );
    map.insert(
        3,
        CmaxValue {
            cmax: 12.0,
            tmax: 1.0,
        },
    );
    miniextendr_api::serde::map_to_dataframe(&map, "subject").expect("map_to_dataframe")
}

/// hashmap_to_dataframe over HashMap<i32, CmaxValue>.
///
/// @export
#[miniextendr]
pub fn test_hashmap_to_dataframe() -> ColumnarDataFrame {
    use std::collections::HashMap;
    let mut map: HashMap<i32, CmaxValue> = HashMap::new();
    map.insert(
        3,
        CmaxValue {
            cmax: 12.0,
            tmax: 1.0,
        },
    );
    map.insert(
        1,
        CmaxValue {
            cmax: 10.5,
            tmax: 2.0,
        },
    );
    map.insert(
        2,
        CmaxValue {
            cmax: 8.1,
            tmax: 3.5,
        },
    );
    miniextendr_api::serde::hashmap_to_dataframe(&map, "subject").expect("hashmap_to_dataframe")
}

// endregion

// region: result_to_dataframe fixtures

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct GoodRow {
    id: i32,
    value: f64,
}

#[derive(Serialize)]
#[serde(crate = "crate::serde")]
struct BadRow {
    id: i32,
    reason: String,
}

fn make_mixed_rows() -> Vec<Result<GoodRow, BadRow>> {
    vec![
        Ok(GoodRow { id: 1, value: 1.0 }),
        Err(BadRow {
            id: 2,
            reason: "bad".into(),
        }),
        Ok(GoodRow { id: 3, value: 3.0 }),
    ]
}

/// result_to_dataframe under Auto with all-Ok input → bare data.frame.
///
/// @export
#[miniextendr]
pub fn test_result_to_dataframe_auto_all_ok() -> DataFrameShape {
    use miniextendr_api::serde::ResultShape;
    let rows: Vec<Result<GoodRow, BadRow>> = vec![
        Ok(GoodRow { id: 1, value: 1.0 }),
        Ok(GoodRow { id: 2, value: 2.0 }),
    ];
    miniextendr_api::serde::result_to_dataframe(
        &rows,
        ResultShape::Auto {
            empty_ok_sentinel: (),
        },
    )
    .expect("result_to_dataframe")
}

/// result_to_dataframe under Auto with mixed Ok/Err → list(results=, error=).
///
/// @export
#[miniextendr]
pub fn test_result_to_dataframe_auto_mixed() -> DataFrameShape {
    use miniextendr_api::serde::ResultShape;
    miniextendr_api::serde::result_to_dataframe(
        &make_mixed_rows(),
        ResultShape::Auto {
            empty_ok_sentinel: (),
        },
    )
    .expect("result_to_dataframe")
}

/// result_to_dataframe under Collated → single data.frame with is_error column.
///
/// @export
#[miniextendr]
pub fn test_result_to_dataframe_collated() -> DataFrameShape {
    use miniextendr_api::serde::ResultShape;
    miniextendr_api::serde::result_to_dataframe::<_, _, ()>(
        &make_mixed_rows(),
        ResultShape::Collated,
    )
    .expect("result_to_dataframe")
}

/// result_to_dataframe under Split with all-Err input → sentinel in results slot.
///
/// @export
#[miniextendr]
pub fn test_result_to_dataframe_split_all_err() -> DataFrameShape {
    use miniextendr_api::serde::ResultShape;
    let rows: Vec<Result<GoodRow, BadRow>> = vec![
        Err(BadRow {
            id: 1,
            reason: "x".into(),
        }),
        Err(BadRow {
            id: 2,
            reason: "y".into(),
        }),
    ];
    miniextendr_api::serde::result_to_dataframe(
        &rows,
        ResultShape::Split {
            empty_ok_sentinel: (),
        },
    )
    .expect("result_to_dataframe")
}

// endregion
