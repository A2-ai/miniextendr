use miniextendr_api::SEXP;
use miniextendr_api::{BuiltDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, IntoList, miniextendr};

#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct ParPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

/// Create a large parallel points data frame.
///
/// @param n Number of rows to create.
/// @export
#[miniextendr]
pub fn create_large_par_points(n: i32) -> BuiltDataFrame {
    let rows: Vec<ParPoint> = (0..n)
        .map(|i| ParPoint {
            x: i as f64,
            y: (i * 2) as f64,
            label: format!("pt_{i}"),
        })
        .collect();
    rows.into_dataframe().unwrap()
}

/// Read a `ParPoint` data.frame in parallel and return the row count.
///
/// Exercises the parallel from-R reader `ParPoint::try_from_dataframe_par`:
/// columns are pulled out of R on the R thread, rows are assembled off-thread
/// via rayon, then we recompute a checksum to prove the rows materialised
/// correctly.
///
/// @param df A data.frame with numeric `x`, `y` and character `label` columns.
/// @export
#[miniextendr]
pub fn par_read_points_checksum(df: SEXP) -> f64 {
    let rows: Vec<ParPoint> = ParPoint::try_from_dataframe_par(df).unwrap();
    rows.iter().map(|r| r.x + r.y + r.label.len() as f64).sum()
}

/// Sequential counterpart to the parallel reader, for cross-checking.
///
/// @param df A data.frame with numeric `x`, `y` and character `label` columns.
/// @export
#[miniextendr]
pub fn seq_read_points_checksum(df: SEXP) -> f64 {
    let rows: Vec<ParPoint> = ParPoint::try_from_dataframe(df).unwrap();
    rows.iter().map(|r| r.x + r.y + r.label.len() as f64).sum()
}

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_kind")]
pub enum ParEvent {
    A { id: i32, value: f64 },
    B { id: i32, name: String },
}

/// Create a large parallel events data frame.
///
/// @param n Number of rows to create.
/// @export
#[miniextendr]
pub fn create_large_par_events(n: i32) -> BuiltDataFrame {
    let rows: Vec<ParEvent> = (0..n)
        .map(|i| {
            if i % 2 == 0 {
                ParEvent::A {
                    id: i,
                    value: i as f64 * 0.5,
                }
            } else {
                ParEvent::B {
                    id: i,
                    name: format!("evt_{i}"),
                }
            }
        })
        .collect();
    rows.into_dataframe().unwrap()
}

/// GC-stress fixture for the parallel from-R reader (`try_from_dataframe_par`).
///
/// No-arg so the fast `gctorture(TRUE)` sweep over `rpkg`'s exports exercises it.
/// Synthesises a `ParPoint` data.frame SEXP internally, then drives the parallel
/// reader. The SEXP-touching part is the per-column extraction (each
/// `DataFrame::column` call allocates an owned `Vec` and may trigger GC); the
/// parallel region itself makes no R API calls. Driving this under gctorture
/// proves the extraction step's PROTECT discipline holds before rayon takes over.
///
#[miniextendr(noexport)]
pub fn gc_stress_dataframe_par_reader() {
    use miniextendr_api::into_r::IntoR as _;

    let rows: Vec<ParPoint> = (0..128)
        .map(|i| ParPoint {
            x: i as f64,
            y: (i * 3) as f64,
            label: format!("pt_{i}"),
        })
        .collect();
    // Build a real data.frame SEXP and keep it protected across the reads —
    // the column-extraction step allocates, and gctorture must not collect the
    // backing data.frame out from under us.
    let df_sexp = rows.into_dataframe().unwrap().into_sexp();
    // SAFETY: runs on the R thread; `df_sexp` is a freshly-allocated valid SEXP.
    let df_guard = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(df_sexp) };
    let df_sexp = df_guard.get();
    let par_rows = ParPoint::try_from_dataframe_par(df_sexp).unwrap();
    let seq_rows = ParPoint::try_from_dataframe(df_sexp).unwrap();
    assert_eq!(par_rows.len(), seq_rows.len());
    // Touch the materialised String fields so the optimiser can't elide them.
    let _checksum: usize = par_rows.iter().map(|r| r.label.len()).sum();
}
