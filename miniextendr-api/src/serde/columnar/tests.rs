use super::*;
use serde::Serialize;

// region: TypeSpec → ColumnType collapse

/// Each `TypeSpec` variant maps to the expected `ColumnType`. `Optional`
/// unwraps to its inner column type — the NA-tolerance hint never
/// changes the underlying R atomic type.
#[test]
fn typespec_collapses_to_column_type() {
    assert_eq!(TypeSpec::Logical.into_column_type(), ColumnType::Logical);
    assert_eq!(TypeSpec::Integer.into_column_type(), ColumnType::Integer);
    assert_eq!(TypeSpec::Real.into_column_type(), ColumnType::Real);
    assert_eq!(
        TypeSpec::Character.into_column_type(),
        ColumnType::Character
    );
    assert_eq!(TypeSpec::Generic.into_column_type(), ColumnType::Generic);
    assert_eq!(
        TypeSpec::Optional(Box::new(TypeSpec::Integer)).into_column_type(),
        ColumnType::Integer
    );
    // Nested Optional collapses transitively.
    assert_eq!(
        TypeSpec::Optional(Box::new(TypeSpec::Optional(Box::new(TypeSpec::Real))))
            .into_column_type(),
        ColumnType::Real
    );
}

// endregion

// region: SerdeRowBuilder::with_schema (#693)

#[derive(Serialize)]
struct WideRow {
    a: i32,
    b: Option<String>,
}

/// `with_schema` populates the schema before any push, allocates the
/// expected number of column buffers, and seeds `filled` to the matching
/// length. First push therefore skips discovery — the schema is fixed.
#[test]
fn with_schema_allocates_columns_up_front() {
    let b = SerdeRowBuilder::<WideRow>::with_schema(
        [
            ("a", TypeSpec::Integer),
            ("b", TypeSpec::Optional(Box::new(TypeSpec::Character))),
        ],
        Some(4),
    );
    let schema = b.schema.as_ref().expect("schema set by with_schema");
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields[0].name, "a");
    assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
    assert_eq!(schema.fields[1].name, "b");
    // Optional(Character) collapses to Character — NA-tolerance is implicit.
    assert_eq!(schema.fields[1].col_type, ColumnType::Character);
    assert_eq!(b.columns.len(), 2);
    assert_eq!(b.filled.len(), 2);
    assert!(!b.grow);
}

/// `with_schema` + `Optional(Integer)` + first row's value `None` lands
/// in the integer buffer as `NA_INTEGER`, not as a logical-NA column
/// (which is what default discovery would produce when the first row's
/// `Option<i32>` is `None`).
///
/// Inspects the buffer state directly; doesn't call `finish` (avoids R
/// allocation in unit tests).
#[test]
fn with_schema_optional_first_none_keeps_declared_type() {
    let mut b = SerdeRowBuilder::<WideRow>::with_schema(
        [
            ("a", TypeSpec::Integer),
            ("b", TypeSpec::Optional(Box::new(TypeSpec::Character))),
        ],
        None,
    );
    b.push(WideRow { a: 1, b: None })
        .expect("first push should succeed");
    assert_eq!(b.nrow, 1);
    // Column "a" is integer with value 1.
    match &b.columns[0] {
        ColumnBuffer::Integer(v) => assert_eq!(v, &vec![1]),
        _ => panic!("expected Integer buffer"),
    }
    // Column "b" is character with NA (None) — *not* a logical column.
    match &b.columns[1] {
        ColumnBuffer::Character(v) => assert_eq!(v, &vec![None]),
        _ => panic!("expected Character buffer for Optional(Character)"),
    }
}

/// Default mode: first push discovers schema; a later push with a new
/// field is rejected by the strict filler.
#[test]
fn default_mode_rejects_new_field_on_later_push() {
    #[derive(Serialize)]
    struct R1 {
        a: i32,
    }
    #[derive(Serialize)]
    struct R2 {
        a: i32,
        b: i32,
    }
    // Push R1 then R2 — but builder is parameterised over a single T.
    // Use a wrapper enum to allow heterogeneous serialization.
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Either {
        One(R1),
        Two(R2),
    }
    let mut b = SerdeRowBuilder::<Either>::new(None);
    b.push(Either::One(R1 { a: 1 })).expect("first push");
    let err = b
        .push(Either::Two(R2 { a: 2, b: 99 }))
        .expect_err("strict mode should reject field 'b' not in schema");
    match err {
        RSerdeError::Message(m) => {
            assert!(m.contains("row introduced field"), "unexpected error: {m}");
        }
        other => panic!("expected Message variant, got {other:?}"),
    }
}

/// `with_schema` mode: an unknown field on push is rejected.
#[test]
fn with_schema_rejects_unknown_field() {
    #[derive(Serialize)]
    struct R {
        x: i32,
        extra: i32,
    }
    let mut b = SerdeRowBuilder::<R>::with_schema([("x", TypeSpec::Integer)], None);
    let err = b
        .push(R { x: 1, extra: 9 })
        .expect_err("'extra' not declared");
    match err {
        RSerdeError::Message(m) => assert!(m.contains("extra"), "unexpected error: {m}"),
        other => panic!("expected Message variant, got {other:?}"),
    }
}

// endregion

// region: SerdeRowBuilder::grow_schema (#692)

/// `grow_schema()` lets a later row introduce a new field. The new
/// column is back-filled with `self.nrow` NA values so its length
/// matches the existing rows.
#[test]
fn grow_schema_back_fills_na_on_new_field() {
    use std::collections::BTreeMap;
    let mut b = SerdeRowBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
    let r1: BTreeMap<String, i32> = [("a".to_string(), 1)].into_iter().collect();
    let r2: BTreeMap<String, i32> = [("a".to_string(), 2), ("b".to_string(), 3)]
        .into_iter()
        .collect();
    b.push(r1).expect("first push");
    assert_eq!(b.nrow, 1);
    assert_eq!(b.columns.len(), 1, "only 'a' so far");
    b.push(r2).expect("growth push");
    assert_eq!(b.nrow, 2);
    assert_eq!(b.columns.len(), 2, "'b' added");
    match &b.columns[0] {
        ColumnBuffer::Integer(v) => assert_eq!(v, &vec![1, 2]),
        _ => panic!("expected Integer for 'a'"),
    }
    // Column 'b' was allocated on row 1; row 0 back-filled to NA_INTEGER.
    match &b.columns[1] {
        ColumnBuffer::Integer(v) => assert_eq!(v, &vec![i32::MIN, 3]),
        _ => panic!("expected Integer for 'b'"),
    }
}

/// `grow_schema()` composed with `with_schema(...)`: pre-declared
/// columns stay typed; new columns from later rows are appended.
#[test]
fn grow_schema_combined_with_with_schema() {
    use std::collections::BTreeMap;
    let mut b =
        SerdeRowBuilder::<BTreeMap<String, i32>>::with_schema([("a", TypeSpec::Integer)], None)
            .grow_schema();
    let r1: BTreeMap<String, i32> = [("a".to_string(), 10)].into_iter().collect();
    let r2: BTreeMap<String, i32> = [("a".to_string(), 20), ("c".to_string(), 99)]
        .into_iter()
        .collect();
    b.push(r1).expect("push 1");
    b.push(r2).expect("push 2");
    assert_eq!(b.nrow, 2);
    // Original declared column 'a' preserved.
    let schema = b.schema.as_ref().unwrap();
    assert_eq!(schema.fields[0].name, "a");
    assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
    // New column 'c' added after first push that introduced it.
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields[1].name, "c");
    assert_eq!(schema.fields[1].col_type, ColumnType::Integer);
    match &b.columns[1] {
        ColumnBuffer::Integer(v) => assert_eq!(v, &vec![i32::MIN, 99]),
        _ => panic!("expected Integer for 'c'"),
    }
}

/// `grow_schema()` plus a row that introduces no new fields is a no-op
/// on schema state — the discovery pass runs but nothing is added.
#[test]
fn grow_schema_noop_when_no_new_fields() {
    use std::collections::BTreeMap;
    let mut b = SerdeRowBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
    let r: BTreeMap<String, i32> = [("k".to_string(), 5)].into_iter().collect();
    b.push(r.clone()).unwrap();
    let pre_len = b.schema.as_ref().unwrap().fields.len();
    b.push(r).unwrap();
    let post_len = b.schema.as_ref().unwrap().fields.len();
    assert_eq!(pre_len, post_len);
    assert_eq!(b.nrow, 2);
}

// endregion

// region: nested union discovery (#1370)

#[derive(Serialize)]
struct NestedDeep {
    level: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<bool>,
}

#[derive(Serialize)]
struct NestedMeta {
    hash: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deep: Option<NestedDeep>,
}

/// Rows whose nested struct gains fields after the first row: `note` from
/// row 3, `deep` from row 4, and `deep.tag` (two levels down) from row 9.
#[derive(Serialize)]
struct NestedRow {
    id: i32,
    meta: NestedMeta,
    size: f64,
}

fn nested_rows(n: i32) -> Vec<NestedRow> {
    (0..n)
        .map(|i| NestedRow {
            id: i,
            meta: NestedMeta {
                hash: i * 2,
                note: (i % 7 == 3).then(|| format!("n{i}")),
                deep: (i % 5 == 4).then(|| NestedDeep {
                    level: f64::from(i),
                    tag: (i % 10 == 9).then_some(i % 20 == 9),
                }),
            },
            size: 0.5,
        })
        .collect()
}

/// Nested fields first seen in later rows join their parent's block in
/// first-appearance order, ahead of the later top-level `size`, and are NA
/// on the rows that lack them.
#[test]
fn union_nests_late_fields_in_parent_block() {
    let rows = nested_rows(10);
    let (names, cols) = seq_union_columns(&rows);
    assert_eq!(
        names,
        [
            "id",
            "meta_hash",
            "meta_note",
            "meta_deep_level",
            "meta_deep_tag",
            "size"
        ]
    );
    match &cols[2] {
        ColumnBuffer::Character(v) => {
            assert_eq!(v[3].as_deref(), Some("n3"));
            assert!(v[0].is_none() && v[9].is_none(), "rows without `note`");
        }
        _ => panic!("expected character `meta_note` column"),
    }
    match &cols[3] {
        ColumnBuffer::Real(v) => {
            assert!(v[0].is_nan(), "row 0 has no `deep`");
            assert_eq!(v[4], 4.0);
            assert_eq!(v[9], 9.0);
        }
        _ => panic!("expected real `meta_deep_level` column"),
    }
    match &cols[4] {
        ColumnBuffer::Logical(v) => {
            assert_eq!(v[4], NA_LOGICAL, "deep without `tag`");
            assert_eq!(v[9], 1);
        }
        _ => panic!("expected logical `meta_deep_tag` column"),
    }
}

/// `grow_schema` gives the columns of the union path: a nested field a
/// later push introduces moves into its parent's block, the earlier
/// columns keep their values, and the new one is NA on the earlier rows.
#[test]
fn grow_schema_absorbs_nested_fields_like_union() {
    let rows = nested_rows(10);
    let (union_names, union_cols) = seq_union_columns(&rows);
    let mut b = SerdeRowBuilder::<NestedRow>::new(None).grow_schema();
    for row in nested_rows(10) {
        b.push(row).expect("push");
    }
    let schema = b.schema.as_ref().expect("schema");
    let names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
    assert_eq!(names, union_names);
    assert_eq!(b.columns.len(), union_cols.len());
    for (i, (grown, union)) in b.columns.iter().zip(&union_cols).enumerate() {
        match (grown, union) {
            (ColumnBuffer::Integer(x), ColumnBuffer::Integer(y))
            | (ColumnBuffer::Logical(x), ColumnBuffer::Logical(y)) => {
                assert_eq!(x, y, "column {i}")
            }
            (ColumnBuffer::Real(x), ColumnBuffer::Real(y)) => assert!(
                x.iter().zip(y).all(|(a, b)| a.to_bits() == b.to_bits()),
                "column {i}"
            ),
            (ColumnBuffer::Character(x), ColumnBuffer::Character(y)) => {
                assert_eq!(x, y, "column {i}")
            }
            _ => panic!("column {i}: variant mismatch"),
        }
    }
}

/// The default builder keeps the first row's schema: a nested field a
/// later push introduces is an error, not a dropped value.
#[test]
fn default_mode_rejects_new_nested_field_on_later_push() {
    let mut rows = nested_rows(4).into_iter();
    let mut b = SerdeRowBuilder::<NestedRow>::new(None);
    for row in rows.by_ref().take(3) {
        b.push(row).expect("rows 0..3 have the first row's fields");
    }
    let err = b
        .push(rows.next().expect("row 3"))
        .expect_err("row 3's `meta.note` is not in the first row's schema");
    match err {
        RSerdeError::Message(m) => {
            assert!(m.contains("\"note\""), "unexpected error: {m}")
        }
        other => panic!("expected Message variant, got {other:?}"),
    }
}

// endregion

// region: par_iter_to_dataframe (#690)

/// Build the merged column buffers the *sequential* way, for equivalence
/// comparison. Mirrors `iter_to_dataframe` minus the final R assembly:
/// drives a `SerdeRowBuilder` and hands back its raw `ColumnBuffer`s.
#[cfg(feature = "rayon")]
fn seq_columns<T: Serialize>(rows: Vec<T>) -> (Vec<String>, Vec<ColumnBuffer>) {
    let mut b = SerdeRowBuilder::<T>::new(None);
    for r in rows {
        b.push(r).expect("sequential push");
    }
    let schema = b.schema.expect("schema discovered");
    let names = schema.fields.iter().map(|f| f.name.clone()).collect();
    (names, b.columns)
}

/// Two `ColumnBuffer` slices are element-for-element equal. (Generic
/// columns never appear on the parallel path, so they're unreachable here.)
#[cfg(feature = "rayon")]
fn columns_eq(a: &[ColumnBuffer], b: &[ColumnBuffer]) {
    assert_eq!(a.len(), b.len(), "column count mismatch");
    for (i, (ca, cb)) in a.iter().zip(b).enumerate() {
        match (ca, cb) {
            (ColumnBuffer::Logical(x), ColumnBuffer::Logical(y)) => {
                assert_eq!(x, y, "logical column {i}")
            }
            (ColumnBuffer::Integer(x), ColumnBuffer::Integer(y)) => {
                assert_eq!(x, y, "integer column {i}")
            }
            (ColumnBuffer::Real(x), ColumnBuffer::Real(y)) => {
                // Bitwise: NA_REAL is a NaN, so `==` would reject equal
                // columns containing NA cells.
                assert_eq!(x.len(), y.len(), "real column {i} length");
                assert!(
                    x.iter().zip(y).all(|(a, b)| a.to_bits() == b.to_bits()),
                    "real column {i}"
                );
            }
            (ColumnBuffer::Character(x), ColumnBuffer::Character(y)) => {
                assert_eq!(x, y, "character column {i}")
            }
            _ => panic!("column {i}: variant mismatch or unexpected Generic"),
        }
    }
}

/// The parallel build is equivalent to the sequential build, column for
/// column and row for row — including row order. Uses a large input so the
/// fan-out splits into multiple chunks across the rayon pool.
#[cfg(feature = "rayon")]
#[test]
fn par_build_equivalent_to_sequential() {
    #[derive(Serialize, Clone)]
    struct Row {
        id: i32,
        ratio: f64,
        name: String,
        flag: bool,
        maybe: Option<i32>,
    }

    let rows: Vec<Row> = (0..5_000)
        .map(|i| Row {
            id: i,
            ratio: f64::from(i) * 0.5,
            name: format!("item_{i}"),
            flag: i % 2 == 0,
            // Row 0 must be `Some` so first-row schema discovery types this
            // as an Integer column (an all-None first row would degrade to
            // a Generic/list column, which the parallel path rejects).
            // Later `None`s land as NA_INTEGER in the integer buffer.
            maybe: if i != 0 && i % 3 == 0 {
                None
            } else {
                Some(i * 10)
            },
        })
        .collect();

    let (seq_names, seq_cols) = seq_columns(rows.clone());
    let (schema, par_cols, nrow) = par_build_columns(&rows, None)
        .expect("par build ok")
        .expect("non-empty");

    assert_eq!(nrow, 5_000);
    let par_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
    assert_eq!(par_names, seq_names, "column names + order must match");
    columns_eq(&par_cols, &seq_cols);

    // Spot-check row order survived the chunk merge: id column is 0..5000
    // in order.
    match &par_cols[0] {
        ColumnBuffer::Integer(v) => {
            assert_eq!(v.len(), 5_000);
            assert!(
                v.iter().enumerate().all(|(i, &x)| x == i as i32),
                "id column not in row order after merge"
            );
        }
        _ => panic!("expected integer id column"),
    }
}

/// Row order is preserved even when the chunk boundary falls mid-stream:
/// the character column reads back in exact iteration order.
#[cfg(feature = "rayon")]
#[test]
fn par_build_preserves_row_order() {
    #[derive(Serialize)]
    struct Row {
        label: String,
    }
    let rows: Vec<Row> = (0..1_000)
        .map(|i| Row {
            label: format!("r{i:04}"),
        })
        .collect();
    let (_schema, cols, nrow) = par_build_columns(&rows, Some(1_000))
        .expect("par build ok")
        .expect("non-empty");
    assert_eq!(nrow, 1_000);
    match &cols[0] {
        ColumnBuffer::Character(v) => {
            for (i, cell) in v.iter().enumerate() {
                assert_eq!(cell.as_deref(), Some(format!("r{i:04}").as_str()));
            }
        }
        _ => panic!("expected character column"),
    }
}

/// Empty input yields `Ok(None)` (the public fn turns this into a 0-row
/// data.frame without touching R).
#[cfg(feature = "rayon")]
#[test]
fn par_build_empty_is_none() {
    #[derive(Serialize)]
    struct Row {
        x: i32,
    }
    let rows: Vec<Row> = Vec::new();
    let out = par_build_columns(&rows, None).expect("par build ok");
    assert!(out.is_none(), "empty input should produce None");
}

/// A schema with a generic (list) column is rejected — the parallel path
/// can't allocate per-cell SEXPs off the R main thread.
#[cfg(feature = "rayon")]
#[test]
fn par_build_rejects_generic_column() {
    // A nested Vec<i32> field has no atomic-column mapping → Generic column.
    #[derive(Serialize)]
    struct Row {
        id: i32,
        tags: Vec<i32>,
    }
    let rows = vec![
        Row {
            id: 1,
            tags: vec![1, 2],
        },
        Row {
            id: 2,
            tags: vec![3],
        },
    ];
    let err = par_build_columns(&rows, None)
        .err()
        .expect("generic column must be rejected");
    match err {
        RSerdeError::Message(m) => {
            assert!(m.contains("generic"), "unexpected error: {m}")
        }
        other => panic!("expected Message variant, got {other:?}"),
    }
}

/// A row introducing a field absent from the first row's schema is rejected
/// (strict homogeneous-schema contract, same as sequential).
#[cfg(feature = "rayon")]
#[test]
fn par_build_rejects_new_field() {
    #[derive(Serialize)]
    struct R1 {
        a: i32,
    }
    #[derive(Serialize)]
    struct R2 {
        a: i32,
        b: i32,
    }
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Either {
        One(R1),
        Two(R2),
    }
    let rows = vec![Either::One(R1 { a: 1 }), Either::Two(R2 { a: 2, b: 9 })];
    let err = par_build_columns(&rows, None)
        .err()
        .expect("new field must be rejected");
    match err {
        RSerdeError::Message(m) => {
            assert!(m.contains("row introduced field"), "unexpected error: {m}")
        }
        other => panic!("expected Message variant, got {other:?}"),
    }
}

// endregion

// region: par_iter_to_dataframe_growing (#936)

/// Build the merged column buffers the *sequential union* way, for
/// equivalence comparison. Mirrors `vec_to_dataframe`'s Phases 1–3 minus
/// the final R assembly. Only valid for Generic-free schemas (no SEXP is
/// ever pushed, so the ProtectScope stays empty).
fn seq_union_columns<T: Serialize>(rows: &[T]) -> (Vec<String>, Vec<ColumnBuffer>) {
    let mut acc = SchemaAccumulator::new(SchemaMode::Union);
    for row in rows {
        let _ = acc.feed(row);
    }
    let schema = acc.finalize().expect("union schema");
    let ncol = schema.fields.len();
    let mut columns: Vec<ColumnBuffer> = schema
        .fields
        .iter()
        .map(|f| ColumnBuffer::new(f.col_type, rows.len()))
        .collect();
    // SAFETY (test): no Generic column → the scope is never written to.
    let scope = unsafe { crate::ProtectScope::new() };
    let mut filled = vec![false; ncol];
    for row in rows {
        let filler = ColumnFiller {
            columns: &mut columns,
            field_map: &schema.field_map,
            filled: &mut filled,
            col_start: 0,
            col_count: ncol,
            is_top_level: true,
            pending_key: None,
            scope: &scope,
            strict: false,
        };
        row.serialize(filler).expect("sequential fill");
    }
    let names = schema.fields.iter().map(|f| f.name.clone()).collect();
    (names, columns)
}

/// Heterogeneous rows for the growing tests: `score` appears only on
/// `New` rows, `legacy` only on `Old` rows.
#[cfg(feature = "rayon")]
#[derive(Serialize)]
#[serde(untagged)]
enum MixedRow {
    Old { id: i32, legacy: String },
    New { id: i32, score: f64 },
}

/// The growing build matches `vec_to_dataframe`'s union semantics column
/// for column and row for row, with fields introduced mid-stream (across
/// chunk boundaries) NA-padded on the rows that lack them.
#[cfg(feature = "rayon")]
#[test]
fn par_growing_equivalent_to_sequential_union() {
    // 5_000 rows so the fan-out splits into multiple chunks; the variant
    // flips at i % 3 so every chunk sees both shapes, and `legacy` /
    // `score` each first appear early.
    let rows: Vec<MixedRow> = (0..5_000)
        .map(|i| {
            if i % 3 == 0 {
                MixedRow::Old {
                    id: i,
                    legacy: format!("v{i}"),
                }
            } else {
                MixedRow::New {
                    id: i,
                    score: f64::from(i) * 0.25,
                }
            }
        })
        .collect();

    let (seq_names, seq_cols) = seq_union_columns(&rows);
    let (schema, par_cols, nrow) = par_build_columns_growing(&rows, None)
        .expect("growing build ok")
        .expect("non-empty");

    assert_eq!(nrow, 5_000);
    let par_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
    assert_eq!(par_names, seq_names, "column names + order must match");
    columns_eq(&par_cols, &seq_cols);
}

/// A field that first appears in the *last* chunk back-fills NA across
/// every earlier chunk, and key order follows first appearance.
#[cfg(feature = "rayon")]
#[test]
fn par_growing_backfills_field_from_late_chunk() {
    let rows: Vec<MixedRow> = (0..4_000)
        .map(|i| {
            if i < 3_999 {
                MixedRow::New {
                    id: i,
                    score: f64::from(i),
                }
            } else {
                // Only the final row carries `legacy`.
                MixedRow::Old {
                    id: i,
                    legacy: "tail".into(),
                }
            }
        })
        .collect();

    let (schema, cols, nrow) = par_build_columns_growing(&rows, None)
        .expect("growing build ok")
        .expect("non-empty");

    assert_eq!(nrow, 4_000);
    let names: Vec<&str> = schema.fields.iter().map(|f| f.name.as_str()).collect();
    // `legacy` was discovered last → appended after first-seen fields.
    assert_eq!(names, ["id", "score", "legacy"]);
    match &cols[2] {
        ColumnBuffer::Character(v) => {
            assert_eq!(v.len(), 4_000);
            assert!(v[..3_999].iter().all(Option::is_none), "expected NA prefix");
            assert_eq!(v[3_999].as_deref(), Some("tail"));
        }
        _ => panic!("expected character `legacy` column"),
    }
    // `score` is NA on the one Old row.
    match &cols[1] {
        ColumnBuffer::Real(v) => assert!(v[3_999].is_nan(), "score must be NA on Old row"),
        _ => panic!("expected real `score` column"),
    }
}

/// Global candidate resolution: a field that is `None` for every row a
/// chunk sees (which a chunk-local schema would type as Generic) still
/// resolves to its typed column from another chunk's probe — same as the
/// sequential union path, and the case a per-chunk-schema merge would get
/// wrong.
#[cfg(feature = "rayon")]
#[test]
fn par_growing_resolves_none_window_against_late_typed_probe() {
    #[derive(Serialize)]
    struct Row {
        id: i32,
        maybe: Option<i32>,
    }
    // `maybe` is None for the first 3_999 rows (many whole chunks) and
    // typed only by the final row.
    let rows: Vec<Row> = (0..4_000)
        .map(|i| Row {
            id: i,
            maybe: (i == 3_999).then_some(7),
        })
        .collect();

    let (schema, cols, _) = par_build_columns_growing(&rows, None)
        .expect("growing build ok")
        .expect("non-empty");

    assert_eq!(schema.fields[1].col_type, ColumnType::Integer);
    match &cols[1] {
        ColumnBuffer::Integer(v) => {
            assert!(
                v[..3_999].iter().all(|&x| x == i32::MIN),
                "expected NA prefix"
            );
            assert_eq!(v[3_999], 7);
        }
        _ => panic!("expected integer `maybe` column"),
    }
}

/// A union schema containing a Generic column (here: a field that is None
/// in every row) is rejected with a pointer back to the sequential path.
#[cfg(feature = "rayon")]
#[test]
fn par_growing_rejects_generic_column() {
    #[derive(Serialize)]
    struct Row {
        id: i32,
        maybe: Option<i32>,
    }
    let rows: Vec<Row> = (0..100).map(|i| Row { id: i, maybe: None }).collect();
    let err = par_build_columns_growing(&rows, None)
        .err()
        .expect("all-None column must be rejected");
    match err {
        RSerdeError::Message(m) => {
            assert!(m.contains("generic"), "unexpected error: {m}")
        }
        other => panic!("expected Message variant, got {other:?}"),
    }
}

/// Cross-chunk type clash follows the sequential union behaviour:
/// first-seen type wins; mismatched later values land as NA.
#[cfg(feature = "rayon")]
#[test]
fn par_growing_type_clash_first_seen_wins() {
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Row {
        I { x: i32 },
        S { x: String },
    }
    // Integers fill the first chunks; strings only appear near the end.
    let rows: Vec<Row> = (0..4_000)
        .map(|i| {
            if i < 3_998 {
                Row::I { x: i }
            } else {
                Row::S { x: format!("s{i}") }
            }
        })
        .collect();

    let (seq_names, seq_cols) = seq_union_columns(&rows);
    let (schema, par_cols, _) = par_build_columns_growing(&rows, None)
        .expect("growing build ok")
        .expect("non-empty");

    assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
    let par_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
    assert_eq!(par_names, seq_names);
    columns_eq(&par_cols, &seq_cols);
    match &par_cols[0] {
        ColumnBuffer::Integer(v) => {
            assert_eq!(v[0], 0);
            assert_eq!(v[3_999], i32::MIN, "string cell must coerce to NA");
        }
        _ => panic!("expected integer column"),
    }
}

/// Empty input yields `Ok(None)`, same as the homogeneous core.
#[cfg(feature = "rayon")]
#[test]
fn par_growing_empty_is_none() {
    #[derive(Serialize)]
    struct Row {
        x: i32,
    }
    let rows: Vec<Row> = Vec::new();
    let out = par_build_columns_growing(&rows, None).expect("growing build ok");
    assert!(out.is_none(), "empty input should produce None");
}

/// Merging per-chunk accumulators gives the same union as feeding all rows
/// into one accumulator (key order and resolved types), flat and nested.
#[cfg(feature = "rayon")]
#[test]
fn schema_accumulator_merge_matches_single_feed() {
    let rows: Vec<MixedRow> = (0..60)
        .map(|i| {
            if i % 2 == 0 {
                MixedRow::Old {
                    id: i,
                    legacy: "x".into(),
                }
            } else {
                MixedRow::New { id: i, score: 1.0 }
            }
        })
        .collect();
    assert_merge_matches_single_feed(&rows);
    assert_merge_matches_single_feed(&nested_rows(60));
}

#[cfg(feature = "rayon")]
fn assert_merge_matches_single_feed<T: Serialize>(rows: &[T]) {
    let mut single = SchemaAccumulator::new(SchemaMode::Union);
    for row in rows {
        let _ = single.feed(row);
    }
    let single_schema = single.finalize().expect("single schema");

    let mut merged = SchemaAccumulator::new(SchemaMode::Union);
    for chunk in rows.chunks(7) {
        let mut acc = SchemaAccumulator::new(SchemaMode::Union);
        for row in chunk {
            let _ = acc.feed(row);
        }
        merged.merge(acc);
    }
    let merged_schema = merged.finalize().expect("merged schema");

    let single_fields: Vec<(String, ColumnType)> = single_schema
        .fields
        .iter()
        .map(|f| (f.name.clone(), f.col_type))
        .collect();
    let merged_fields: Vec<(String, ColumnType)> = merged_schema
        .fields
        .iter()
        .map(|f| (f.name.clone(), f.col_type))
        .collect();
    assert_eq!(merged_fields, single_fields);
}

/// The nested growing build matches the sequential union too: fields a
/// nested struct gains mid-stream (two levels deep, across chunks) land in
/// the same columns in the same order.
#[cfg(feature = "rayon")]
#[test]
fn par_growing_nested_equivalent_to_sequential_union() {
    let rows = nested_rows(5_000);
    let (seq_names, seq_cols) = seq_union_columns(&rows);
    let (schema, par_cols, nrow) = par_build_columns_growing(&rows, None)
        .expect("growing build ok")
        .expect("non-empty");

    assert_eq!(nrow, 5_000);
    let par_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
    assert_eq!(par_names, seq_names, "column names + order must match");
    columns_eq(&par_cols, &seq_cols);
}

// endregion
