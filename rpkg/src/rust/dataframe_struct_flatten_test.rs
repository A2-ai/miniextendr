//! Test fixtures for `DataFrameRow`: struct-in-struct flattening (issue #485).
//!
//! When a top-level `DataFrameRow` struct has a field whose type also derives
//! `DataFrameRow`, that field should flatten into prefixed columns
//! (`<field>_<inner_col>`), mirroring the existing enum-variant struct-field
//! flattening shipped in #477.
//!
//! These tests are the canonical TDD harness for #485. **Do not weaken
//! assertions to make a partial fix pass.** Talk to the author before
//! changing a column-name expectation or removing a case.
//!
//! Cases covered:
//!   - basic 2-column inner (1 row, N rows)
//!   - multiple struct fields on the same outer
//!   - inner with non-numeric (String) columns
//!   - `#[dataframe(rename = "...")]` controls the prefix
//!   - `#[dataframe(skip)]` drops the struct field entirely
//!   - `#[dataframe(as_list)]` preserves the opaque-list workaround
//!   - mixed scalar + struct + scalar field order is preserved
//!   - tuple-struct outer (`_0_x` / `_1_x` etc.)
//!   - zero-row input → empty DF with full column shape
//!   - nested struct-in-struct (Outer → Inner → SubInner) cascades
//!   - multi-segment qualified path (`geom::QualPoint`) — fix for issue #514
//!
//! All fixtures also exercise `gc_stress_struct_flatten_*` for the no-arg
//! gctorture sweep, per `rpkg/CLAUDE.md`'s SEXP-storage convention.

#![allow(dead_code)]

use miniextendr_api::convert::ToDataFrame;
use miniextendr_api::{DataFrameRow, IntoList, List, miniextendr};

// region: Inner types

/// A 2-column inner struct. Used as the field type in most fixtures below.
///
/// `IntoList` is derived so the `as_list` opt-out branch keeps compiling.
/// `Default` is derived so the `#[dataframe(skip)]` fixture compiles
/// (the macro reconstructs skipped fields via `Default::default()` in
/// `IntoIterator::next()`).
#[derive(Clone, Debug, Default, DataFrameRow, IntoList)]
pub struct FlatPoint {
    pub x: f64,
    pub y: f64,
}

/// An inner struct with mixed scalar types (String + i32).
#[derive(Clone, Debug, DataFrameRow, IntoList)]
pub struct FlatPerson {
    pub name: String,
    pub age: i32,
}

/// Innermost type for the nested-struct test. `IntoList` is needed because
/// `FlatSubInner` itself has only scalar columns — the macro requires the
/// row type to be `IntoList` when no expansion is in play.
#[derive(Clone, Debug, DataFrameRow, IntoList)]
pub struct FlatSubInner {
    pub depth: f64,
}

/// Middle type for the nested-struct test — itself contains a struct field.
/// After the fix, `FlatInner::to_dataframe` should produce columns
/// `a` + `sub_depth` (flattening cascades).
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatInner {
    pub a: f64,
    pub sub: FlatSubInner,
}

/// Sub-module with its own `DataFrameRow` type — used to verify that
/// multi-segment paths (`geom::QualPoint`) are correctly classified as
/// `FieldTypeKind::Struct` rather than falling through to `Scalar` (#514).
pub mod geom {
    use miniextendr_api::{DataFrameRow, IntoList};

    /// A 2-column inner struct referenced via a qualified path (`geom::QualPoint`).
    #[derive(Clone, Debug, DataFrameRow, IntoList)]
    pub struct QualPoint {
        pub qx: f64,
        pub qy: f64,
    }
}

// endregion

// region: Outer types — one fixture per case

/// Basic case: scalar `id` + struct `origin: FlatPoint`.
/// Expected columns: `id`, `origin_x`, `origin_y`.
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatLocated {
    pub id: i32,
    pub origin: FlatPoint,
}

/// Two struct fields on the same outer.
/// Expected columns: `id`, `a_x`, `a_y`, `b_x`, `b_y`.
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatSegment {
    pub id: i32,
    pub a: FlatPoint,
    pub b: FlatPoint,
}

/// Mixed inner types (String + i32 column inside the inner struct).
/// Expected columns: `id`, `owner_name`, `owner_age`.
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatTagged {
    pub id: i32,
    pub owner: FlatPerson,
}

/// `rename` on the struct field controls the prefix.
/// Expected columns: `id`, `loc_x`, `loc_y` (NOT `origin_x`/`origin_y`).
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatRenamed {
    pub id: i32,
    #[dataframe(rename = "loc")]
    pub origin: FlatPoint,
}

/// `skip` drops the struct field entirely.
/// Expected columns: `id` only. `IntoList` is hand-rolled (rather than derived)
/// because the auto-derive can't see `#[dataframe(skip)]` and would try to
/// serialize `origin: FlatPoint` — which doesn't implement `IntoR` directly.
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatSkip {
    pub id: i32,
    #[dataframe(skip)]
    pub origin: FlatPoint,
}

impl ::miniextendr_api::list::IntoList for FlatSkip {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        ::miniextendr_api::List::from_raw_pairs(vec![("id", self.id.into_sexp())])
    }
}

/// `as_list` opt-out preserves the legacy opaque-list-column workaround.
/// Expected columns: `id`, `origin` (the latter is a list-column whose
/// elements are the R reps of `FlatPoint`).
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatAsList {
    pub id: i32,
    #[dataframe(as_list)]
    pub origin: FlatPoint,
}

/// An outer struct with ONLY an `as_list`-on-struct field and no scalars.
/// Expected companion columns: `data` (a list-column).
///
/// This is the regression case for the `par_len_field` guard (#513 review):
/// `flat_cols` is non-empty (the as_list field lives there with
/// `needs_into_list=true`), so the companion struct must NOT have a `_len`
/// field — the `data: Vec<List>` column provides the row-count directly.
/// `from_rows_par` must compile and produce correct row-count without `_len`.
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatOnlyAsList {
    #[dataframe(as_list)]
    pub data: FlatPoint,
}

/// Scalar, struct, scalar — verify column ordering is preserved.
/// Expected columns: `id`, `p_x`, `p_y`, `label`.
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatMixedOrder {
    pub id: i32,
    pub p: FlatPoint,
    pub label: String,
}

/// Tuple-struct outer with two struct fields.
/// Expected columns: `_0_x`, `_0_y`, `_1_x`, `_1_y`.
/// (Tuple-struct field names follow the existing `_0`/`_1` convention used
/// elsewhere in the macro.)
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatPair(pub FlatPoint, pub FlatPoint);

/// Nested struct-in-struct: outer field → inner has its own struct field.
/// Expected columns after recursive flattening: `id`, `inner_a`,
/// `inner_sub_depth`.
#[derive(Clone, Debug, DataFrameRow)]
pub struct FlatNested {
    pub id: i32,
    pub inner: FlatInner,
}

/// Multi-segment-path field: `geom::QualPoint` (fix for issue #514).
/// Expected columns: `id`, `pos_qx`, `pos_qy`.
///
/// Before the fix, `classify_field_type` required `segs.len() == 1`, so
/// `geom::QualPoint` fell through to `Scalar` (opaque list column).
#[derive(Clone, Debug, DataFrameRow)]
pub struct QualLocated {
    pub id: i32,
    #[dataframe(rename = "pos")]
    pub origin: geom::QualPoint,
}

// endregion

// region: #[miniextendr] entrypoints — one per case + multi-row variants

#[miniextendr]
pub fn flat_basic_1row() -> ToDataFrame<FlatLocatedDataFrame> {
    ToDataFrame(FlatLocated::to_dataframe(vec![FlatLocated {
        id: 1,
        origin: FlatPoint { x: 1.0, y: 2.0 },
    }]))
}

#[miniextendr]
pub fn flat_basic_nrow() -> ToDataFrame<FlatLocatedDataFrame> {
    ToDataFrame(FlatLocated::to_dataframe(vec![
        FlatLocated { id: 1, origin: FlatPoint { x: 1.0, y: 2.0 } },
        FlatLocated { id: 2, origin: FlatPoint { x: 3.0, y: 4.0 } },
        FlatLocated { id: 3, origin: FlatPoint { x: 5.0, y: 6.0 } },
    ]))
}

#[miniextendr]
pub fn flat_basic_zero_rows() -> ToDataFrame<FlatLocatedDataFrame> {
    ToDataFrame(FlatLocated::to_dataframe(vec![]))
}

#[miniextendr]
pub fn flat_two_struct_fields() -> ToDataFrame<FlatSegmentDataFrame> {
    ToDataFrame(FlatSegment::to_dataframe(vec![
        FlatSegment {
            id: 10,
            a: FlatPoint { x: 1.0, y: 2.0 },
            b: FlatPoint { x: 3.0, y: 4.0 },
        },
        FlatSegment {
            id: 20,
            a: FlatPoint { x: 5.0, y: 6.0 },
            b: FlatPoint { x: 7.0, y: 8.0 },
        },
    ]))
}

#[miniextendr]
pub fn flat_mixed_inner_types() -> ToDataFrame<FlatTaggedDataFrame> {
    ToDataFrame(FlatTagged::to_dataframe(vec![
        FlatTagged {
            id: 1,
            owner: FlatPerson { name: "Ada".to_string(), age: 30 },
        },
        FlatTagged {
            id: 2,
            owner: FlatPerson { name: "Linus".to_string(), age: 50 },
        },
    ]))
}

#[miniextendr]
pub fn flat_renamed_inner() -> ToDataFrame<FlatRenamedDataFrame> {
    ToDataFrame(FlatRenamed::to_dataframe(vec![FlatRenamed {
        id: 1,
        origin: FlatPoint { x: 1.0, y: 2.0 },
    }]))
}

#[miniextendr]
pub fn flat_skip_inner() -> ToDataFrame<FlatSkipDataFrame> {
    ToDataFrame(FlatSkip::to_dataframe(vec![
        FlatSkip { id: 1, origin: FlatPoint { x: 1.0, y: 2.0 } },
        FlatSkip { id: 2, origin: FlatPoint { x: 3.0, y: 4.0 } },
    ]))
}

#[miniextendr]
pub fn flat_as_list_inner() -> ToDataFrame<FlatAsListDataFrame> {
    ToDataFrame(FlatAsList::to_dataframe(vec![
        FlatAsList { id: 1, origin: FlatPoint { x: 1.0, y: 2.0 } },
        FlatAsList { id: 2, origin: FlatPoint { x: 3.0, y: 4.0 } },
    ]))
}

#[miniextendr]
pub fn flat_mixed_order() -> ToDataFrame<FlatMixedOrderDataFrame> {
    ToDataFrame(FlatMixedOrder::to_dataframe(vec![
        FlatMixedOrder {
            id: 1,
            p: FlatPoint { x: 1.0, y: 2.0 },
            label: "first".to_string(),
        },
        FlatMixedOrder {
            id: 2,
            p: FlatPoint { x: 3.0, y: 4.0 },
            label: "second".to_string(),
        },
    ]))
}

#[miniextendr]
pub fn flat_tuple_struct() -> ToDataFrame<FlatPairDataFrame> {
    ToDataFrame(FlatPair::to_dataframe(vec![
        FlatPair(FlatPoint { x: 1.0, y: 2.0 }, FlatPoint { x: 3.0, y: 4.0 }),
        FlatPair(FlatPoint { x: 5.0, y: 6.0 }, FlatPoint { x: 7.0, y: 8.0 }),
    ]))
}

#[miniextendr]
pub fn flat_nested_struct() -> ToDataFrame<FlatNestedDataFrame> {
    ToDataFrame(FlatNested::to_dataframe(vec![
        FlatNested {
            id: 1,
            inner: FlatInner {
                a: 10.0,
                sub: FlatSubInner { depth: 100.0 },
            },
        },
        FlatNested {
            id: 2,
            inner: FlatInner {
                a: 20.0,
                sub: FlatSubInner { depth: 200.0 },
            },
        },
    ]))
}

/// Multi-segment-path field: `geom::QualPoint` (fix for issue #514).
/// Returns a 1-row data.frame with columns `id`, `pos_qx`, `pos_qy`.
#[miniextendr]
pub fn qual_located_basic() -> ToDataFrame<QualLocatedDataFrame> {
    ToDataFrame(QualLocated::to_dataframe(vec![QualLocated {
        id: 42,
        origin: geom::QualPoint { qx: 1.5, qy: 2.5 },
    }]))
}

// endregion

// region: gctorture fixtures (no-arg, self-contained)

/// Drives flattening with a non-trivial row count under gctorture.
/// Pairs with the existing `rpkg/src/rust/gc_stress_fixtures.rs` sweep.
#[miniextendr]
pub fn gc_stress_struct_flatten() -> List {
    let rows: Vec<FlatLocated> = (0..32)
        .map(|i| FlatLocated {
            id: i,
            origin: FlatPoint {
                x: i as f64,
                y: (i as f64) * 2.0,
            },
        })
        .collect();
    let df = FlatLocated::to_dataframe(rows);
    use miniextendr_api::convert::IntoDataFrame as _;
    df.into_data_frame()
}

/// Drives nested flattening under gctorture.
#[miniextendr]
pub fn gc_stress_struct_flatten_nested() -> List {
    let rows: Vec<FlatNested> = (0..16)
        .map(|i| FlatNested {
            id: i,
            inner: FlatInner {
                a: i as f64,
                sub: FlatSubInner {
                    depth: (i as f64) * 10.0,
                },
            },
        })
        .collect();
    let df = FlatNested::to_dataframe(rows);
    use miniextendr_api::convert::IntoDataFrame as _;
    df.into_data_frame()
}

// endregion

// region: Rust-level compile-time shape assertions
//
// Companion struct holds the inner type itself (`Vec<Inner>`) — the same type
// users already pass into `to_dataframe(vec![...])`. R-side columns flatten
// via `into_data_frame()`; no `*DataFrame` type names leak into user-facing
// construction.
//
// These constructors don't run — they fail to *compile* if the macro emits
// the wrong companion shape, which is exactly the regression we want to
// guard against.

const _: () = {
    fn _shape_basic() {
        let _ = FlatLocatedDataFrame {
            id: vec![1],
            origin: vec![FlatPoint { x: 1.0, y: 2.0 }],
        };
    }

    fn _shape_two_struct_fields() {
        let _ = FlatSegmentDataFrame {
            id: vec![1],
            a: vec![FlatPoint { x: 1.0, y: 2.0 }],
            b: vec![FlatPoint { x: 3.0, y: 4.0 }],
        };
    }

    fn _shape_mixed_inner_types() {
        let _ = FlatTaggedDataFrame {
            id: vec![1],
            owner: vec![FlatPerson { name: "x".to_string(), age: 1 }],
        };
    }

    fn _shape_renamed() {
        // `rename = "loc"` controls both the companion field name and the
        // R-side column prefix.
        let _ = FlatRenamedDataFrame {
            id: vec![1],
            loc: vec![FlatPoint { x: 1.0, y: 2.0 }],
        };
    }

    fn _shape_mixed_order() {
        // Field declaration order in the companion struct mirrors the outer
        // field declaration order; the struct field holds `Vec<Inner>`.
        let _ = FlatMixedOrderDataFrame {
            id: vec![1],
            p: vec![FlatPoint { x: 1.0, y: 2.0 }],
            label: vec!["x".to_string()],
        };
    }

    fn _shape_nested() {
        let _ = FlatNestedDataFrame {
            id: vec![1],
            inner: vec![FlatInner {
                a: 1.0,
                sub: FlatSubInner { depth: 1.0 },
            }],
        };
    }

    /// Companion struct for `QualLocated` holds `Vec<geom::QualPoint>` — NOT
    /// `Vec<List>` (which would indicate the field fell through to `Scalar`).
    /// This assertion fails to compile if issue #514 regresses.
    fn _shape_qual_located() {
        let _ = QualLocatedDataFrame {
            id: vec![42],
            pos: vec![geom::QualPoint { qx: 1.5, qy: 2.5 }],
        };
    }

    // Regression for #513 review: `FlatOnlyAsList` has ONLY an `as_list`-on-struct
    // field and no scalars. The companion must NOT have a `_len` field (the
    // `Vec<List>` column provides the row-count). This verifies that `par_len_field`
    // codegen correctly omits `_len` for this shape and the struct compiles.
    fn _shape_only_as_list() {
        let _ = FlatOnlyAsListDataFrame {
            data: vec![],
        };
    }
};

// endregion

// region: #[miniextendr] par entrypoints — struct-flatten + as_list par path (#513)

/// `from_rows_par` for the basic struct-flatten case (FlatLocated → 3 rows).
/// Expected columns: `id`, `origin_x`, `origin_y`.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn flat_basic_par() -> ToDataFrame<FlatLocatedDataFrame> {
    ToDataFrame(FlatLocatedDataFrame::from_rows_par(vec![
        FlatLocated { id: 1, origin: FlatPoint { x: 1.0, y: 2.0 } },
        FlatLocated { id: 2, origin: FlatPoint { x: 3.0, y: 4.0 } },
        FlatLocated { id: 3, origin: FlatPoint { x: 5.0, y: 6.0 } },
    ]))
}

/// `from_rows_par` for the two-struct-fields case (FlatSegment).
/// Expected columns: `id`, `a_x`, `a_y`, `b_x`, `b_y`.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn flat_two_struct_fields_par() -> ToDataFrame<FlatSegmentDataFrame> {
    ToDataFrame(FlatSegmentDataFrame::from_rows_par(vec![
        FlatSegment {
            id: 10,
            a: FlatPoint { x: 1.0, y: 2.0 },
            b: FlatPoint { x: 3.0, y: 4.0 },
        },
        FlatSegment {
            id: 20,
            a: FlatPoint { x: 5.0, y: 6.0 },
            b: FlatPoint { x: 7.0, y: 8.0 },
        },
    ]))
}

/// `from_rows_par` for the `as_list` opt-out case (FlatAsList).
/// Expected columns: `id`, `origin` (list column).
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn flat_as_list_par() -> ToDataFrame<FlatAsListDataFrame> {
    ToDataFrame(FlatAsListDataFrame::from_rows_par(vec![
        FlatAsList { id: 1, origin: FlatPoint { x: 1.0, y: 2.0 } },
        FlatAsList { id: 2, origin: FlatPoint { x: 3.0, y: 4.0 } },
    ]))
}

/// `from_rows_par` for the nested struct-in-struct case (FlatNested).
/// Expected columns: `id`, `inner_a`, `inner_sub_depth`.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn flat_nested_par() -> ToDataFrame<FlatNestedDataFrame> {
    ToDataFrame(FlatNestedDataFrame::from_rows_par(vec![
        FlatNested {
            id: 1,
            inner: FlatInner { a: 10.0, sub: FlatSubInner { depth: 100.0 } },
        },
        FlatNested {
            id: 2,
            inner: FlatInner { a: 20.0, sub: FlatSubInner { depth: 200.0 } },
        },
    ]))
}

// endregion

// region: Rust-level unit tests — par matches sequential (#513)

#[cfg(test)]
mod par_tests {
    use super::*;

    #[cfg(feature = "rayon")]
    #[test]
    fn test_from_rows_par_struct_flatten_matches_sequential() {
        let make_rows = || -> Vec<FlatLocated> {
            (0..100)
                .map(|i| FlatLocated {
                    id: i,
                    origin: FlatPoint { x: i as f64, y: (i as f64) * 2.0 },
                })
                .collect()
        };

        let df_seq = FlatLocatedDataFrame::from_rows(make_rows());
        let df_par = FlatLocatedDataFrame::from_rows_par(make_rows());

        assert_eq!(df_seq.id, df_par.id);
        assert_eq!(df_seq.origin.len(), df_par.origin.len());
        for (s, p) in df_seq.origin.iter().zip(df_par.origin.iter()) {
            assert_eq!(s.x, p.x);
            assert_eq!(s.y, p.y);
        }
    }

    // Note: `test_from_rows_par_as_list_matches_sequential` would require R FFI
    // because `FlatAsListDataFrame::from_rows_par` calls `IntoList::into_list()`
    // in the sequential pre-pass, which allocates R SEXPs. R-side assertions live
    // in `rpkg/tests/testthat/test-dataframe-struct-flatten.R` (flat_as_list_par).

    // Regression for #513 review — `par_len_field` guard for as_list-only structs.
    //
    // `FlatOnlyAsList` has ONLY a `#[dataframe(as_list)]` struct field and no
    // scalars. The companion `FlatOnlyAsListDataFrame` must NOT have a `_len`
    // field (the `Vec<List>` column provides the row-count directly via
    // `into_data_frame`). Calling `from_rows_par` on this shape requires R FFI,
    // so we verify only that the generated `from_rows_par` type-checks correctly —
    // the function pointer cast confirms the method signature is valid.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_from_rows_par_only_as_list_compiles() {
        // The companion struct must NOT have a `_len` field — verify the field
        // layout is `{ data: Vec<List> }` by constructing it directly.
        let df = FlatOnlyAsListDataFrame { data: vec![] };
        // The `data` vec is empty (0 rows), so len == 0 is trivially correct.
        assert_eq!(df.data.len(), 0);
        // Verify `from_rows_par` is callable as a function (type-check only).
        // Actual invocation requires R FFI; R-level test is in
        // `test-dataframe-struct-flatten.R` (flat_only_aslist_par when available).
        let _ = FlatOnlyAsListDataFrame::from_rows_par as fn(Vec<FlatOnlyAsList>) -> _;
    }
}

// endregion
