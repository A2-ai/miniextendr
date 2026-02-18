//! Benchmarks for DataFrameRow derive macro conversions.
//!
//! Measures the cost of Vec<Row> → DataFrame transposition and SEXP
//! conversion for named structs, enums with field-union alignment, and
//! varying row/column counts.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::{DataFrameRow, IntoDataFrame, IntoList, IntoR};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Row type definitions
// =============================================================================

/// Simple 3-column named struct (point data).
#[derive(Clone, IntoList, DataFrameRow)]
pub struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

/// Wide 10-column named struct.
#[derive(Clone, IntoList, DataFrameRow)]
pub struct Wide10 {
    c0: f64,
    c1: f64,
    c2: f64,
    c3: f64,
    c4: f64,
    c5: f64,
    c6: f64,
    c7: f64,
    c8: f64,
    c9: f64,
}

/// Mixed-type 5-column struct (string + numeric + bool).
#[derive(Clone, IntoList, DataFrameRow)]
pub struct MixedRow {
    name: String,
    age: i32,
    score: f64,
    active: bool,
    tag: String,
}

/// 2-variant enum with align (field-union pattern).
#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
enum EventRow {
    Click { id: i64, x: f64, y: f64 },
    View { id: i64, page: String },
}

// =============================================================================
// Fixture builders
// =============================================================================

fn make_point3(n: usize) -> Vec<Point3> {
    (0..n)
        .map(|i| Point3 {
            x: i as f64,
            y: i as f64 * 2.0,
            z: i as f64 * 3.0,
        })
        .collect()
}

fn make_wide10(n: usize) -> Vec<Wide10> {
    (0..n)
        .map(|i| {
            let v = i as f64;
            Wide10 {
                c0: v,
                c1: v,
                c2: v,
                c3: v,
                c4: v,
                c5: v,
                c6: v,
                c7: v,
                c8: v,
                c9: v,
            }
        })
        .collect()
}

fn make_mixed(n: usize) -> Vec<MixedRow> {
    (0..n)
        .map(|i| MixedRow {
            name: format!("user_{i}"),
            age: (20 + i % 50) as i32,
            score: i as f64 * 0.1,
            active: i % 2 == 0,
            tag: if i % 3 == 0 {
                "admin".to_string()
            } else {
                "user".to_string()
            },
        })
        .collect()
}

fn make_events(n: usize) -> Vec<EventRow> {
    (0..n)
        .map(|i| {
            if i % 2 == 0 {
                EventRow::Click {
                    id: i as i64,
                    x: i as f64,
                    y: i as f64 * 2.0,
                }
            } else {
                EventRow::View {
                    id: i as i64,
                    page: format!("/page/{i}"),
                }
            }
        })
        .collect()
}

const ROW_COUNTS: &[usize] = &[100, 10_000, 100_000];

// =============================================================================
// Group 1: Named struct transposition (Vec<Row> → Companion)
// =============================================================================

mod transpose {
    use super::*;

    /// 3-column Point3 transposition (numeric only).
    #[divan::bench(args = ROW_COUNTS)]
    fn point3(n: usize) {
        let rows = make_point3(n);
        let df = Point3::to_dataframe(rows);
        divan::black_box(df);
    }

    /// 10-column Wide10 transposition (numeric only, wider struct).
    #[divan::bench(args = ROW_COUNTS)]
    fn wide10(n: usize) {
        let rows = make_wide10(n);
        let df = Wide10::to_dataframe(rows);
        divan::black_box(df);
    }

    /// 5-column mixed types (String + i32 + f64 + bool + String).
    #[divan::bench(args = ROW_COUNTS)]
    fn mixed_row(n: usize) {
        let rows = make_mixed(n);
        let df = MixedRow::to_dataframe(rows);
        divan::black_box(df);
    }

    /// 2-variant enum with align + tag column.
    #[divan::bench(args = ROW_COUNTS)]
    fn event_enum(n: usize) {
        let rows = make_events(n);
        let df = EventRow::to_dataframe(rows);
        divan::black_box(df);
    }
}

// =============================================================================
// Group 2: Full pipeline (Vec<Row> → DataFrame → SEXP)
// =============================================================================

mod full_pipeline {
    use super::*;

    /// Point3: transpose + into_data_frame (SEXP conversion).
    #[divan::bench(args = ROW_COUNTS)]
    fn point3_to_sexp(n: usize) -> SEXP {
        let rows = make_point3(n);
        let df = Point3::to_dataframe(rows);
        divan::black_box(df.into_data_frame().as_sexp().into_sexp())
    }

    /// MixedRow: transpose + into_data_frame (String allocation overhead).
    #[divan::bench(args = ROW_COUNTS)]
    fn mixed_to_sexp(n: usize) -> SEXP {
        let rows = make_mixed(n);
        let df = MixedRow::to_dataframe(rows);
        divan::black_box(df.into_data_frame().as_sexp().into_sexp())
    }

    /// EventRow enum: transpose + into_data_frame (Option<T> columns).
    #[divan::bench(args = ROW_COUNTS)]
    fn event_to_sexp(n: usize) -> SEXP {
        let rows = make_events(n);
        let df = EventRow::to_dataframe(rows);
        divan::black_box(df.into_data_frame().as_sexp().into_sexp())
    }
}

// =============================================================================
// Group 3: Companion → SEXP only (isolate SEXP conversion cost)
// =============================================================================

mod sexp_conversion {
    use super::*;

    /// Point3DataFrame → SEXP (3 numeric columns).
    #[divan::bench(args = ROW_COUNTS)]
    fn point3_into_df(n: usize) -> SEXP {
        let df = Point3::to_dataframe(make_point3(n));
        divan::black_box(df.into_data_frame().as_sexp().into_sexp())
    }

    /// Wide10DataFrame → SEXP (10 numeric columns).
    #[divan::bench(args = ROW_COUNTS)]
    fn wide10_into_df(n: usize) -> SEXP {
        let df = Wide10::to_dataframe(make_wide10(n));
        divan::black_box(df.into_data_frame().as_sexp().into_sexp())
    }
}

// =============================================================================
// Group 4: Row construction cost (isolate allocation from transposition)
// =============================================================================

mod row_construction {
    use super::*;

    /// Cost of building Vec<Point3> (numeric only, no String allocation).
    #[divan::bench(args = ROW_COUNTS)]
    fn build_point3_rows(n: usize) {
        let rows = make_point3(n);
        divan::black_box(rows);
    }

    /// Cost of building Vec<MixedRow> (includes String allocation).
    #[divan::bench(args = ROW_COUNTS)]
    fn build_mixed_rows(n: usize) {
        let rows = make_mixed(n);
        divan::black_box(rows);
    }

    /// Cost of building Vec<EventRow> (enum + String allocation).
    #[divan::bench(args = ROW_COUNTS)]
    fn build_event_rows(n: usize) {
        let rows = make_events(n);
        divan::black_box(rows);
    }
}
