//! Rayon bridge benchmarks (feature-gated).

#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::rayon::prelude::*;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::{RDataFrameBuilder, reduce, with_r_vec};
#[cfg(feature = "rayon")]
use miniextendr_bench::raw_ffi;

#[cfg(feature = "rayon")]
fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[cfg(not(feature = "rayon"))]
fn main() {}

#[cfg(feature = "rayon")]
#[divan::bench(args = miniextendr_bench::SIZES)]
fn rayon_with_r_vec(len: usize) {
    let sexp = with_r_vec::<f64, _>(len, |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = (offset + i) as f64;
        }
    });
    divan::black_box(sexp);
}

#[cfg(feature = "rayon")]
#[divan::bench(args = miniextendr_bench::SIZES)]
fn rayon_collect_vec(len: usize) {
    let vec: Vec<i32> = (0..len as i32).into_par_iter().collect();
    divan::black_box(vec.len());
}

#[cfg(feature = "rayon")]
#[divan::bench(args = [0, 2, 4])]
fn rayon_reduce_sum(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
    unsafe {
        let ptr = raw_ffi::REAL(sexp);
        let len = raw_ffi::Rf_xlength(sexp) as usize;
        let slice = std::slice::from_raw_parts(ptr, len);
        let out = reduce::sum(slice);
        divan::black_box(out);
    }
}

// region: data.frame fill — few-long-columns scaling (flatten vs column-granular)
//
// The flattened `RDataFrameBuilder::build()` expresses the whole fill job as one
// `(column, row-range)` work-list. For a FEW columns x MANY rows, the old
// column-granular shape (one parallel task per column) under-saturates: with
// `NCOLS` columns it can only ever keep `NCOLS` threads busy. The flattened path
// shatters each column into `~nthreads*4` chunks, so even 3 columns saturate the
// pool. These two benches let you compare the two strategies head-to-head on the
// few-long-columns case the change targets.

#[cfg(feature = "rayon")]
const DF_ROWS: usize = 4_000_000;
#[cfg(feature = "rayon")]
const DF_NCOLS: usize = 3;

/// Per-element fill kernel shared by both paths so the comparison measures only
/// the scheduling, not differing arithmetic. A handful of transcendental ops
/// make the fill compute-bound (otherwise a 4M-row fill is allocation/memcpy
/// dominated and the scheduling difference hides in noise).
#[cfg(feature = "rayon")]
#[inline(never)]
fn df_cell(row: usize, col: usize) -> f64 {
    let mut v = (row as f64 + col as f64 + 1.0).sqrt();
    for _ in 0..8 {
        v = (v.sin() + 1.5).ln().sqrt();
    }
    v
}

/// FLATTENED path: the production `RDataFrameBuilder` (one flat work-list).
#[cfg(feature = "rayon")]
#[divan::bench(sample_count = 30)]
fn dataframe_few_long_flattened() {
    let mut b = RDataFrameBuilder::new(DF_ROWS);
    for j in 0..DF_NCOLS {
        b = b.column::<f64>(format!("c{j}"), move |chunk, offset| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = df_cell(offset + i, j);
            }
        });
    }
    divan::black_box(b.build());
}

/// COLUMN-GRANULAR baseline: one rayon task per column, each task filling its
/// whole column **serially** (no internal row-slice chunking). This is what a
/// naive column-per-task scheduler does — the fan-out width is only `DF_NCOLS`,
/// so for a few long columns the pool sits mostly idle while a handful of
/// threads each crunch a whole 4M-row column. Columns are allocated and
/// assembled with the same PROTECT discipline as the builder so the comparison
/// only measures the fill scheduling.
#[cfg(feature = "rayon")]
#[divan::bench(sample_count = 30)]
fn dataframe_few_long_column_granular() {
    use miniextendr_api::SEXP;
    use miniextendr_api::SexpExt;
    use miniextendr_api::worker::with_r_thread;

    // Phase 1: allocate every column up-front (serially on the R thread) and
    // protect; collect the data pointers.
    struct ColBuf {
        sexp: SEXP,
        ptr: *mut f64,
    }
    unsafe impl Send for ColBuf {}
    unsafe impl Sync for ColBuf {}

    let cols: Vec<ColBuf> = with_r_thread(|| unsafe {
        use miniextendr_api::RNativeType;
        use miniextendr_api::SEXPTYPE::REALSXP;
        (0..DF_NCOLS)
            .map(|_| {
                let sexp = miniextendr_api::sys::Rf_allocVector_unchecked(
                    REALSXP,
                    DF_ROWS as miniextendr_api::R_xlen_t,
                );
                miniextendr_api::sys::Rf_protect_unchecked(sexp);
                let ptr = <f64 as RNativeType>::dataptr_mut(sexp);
                ColBuf { sexp, ptr }
            })
            .collect()
    });

    // Phase 2: COLUMN-granular parallelism — one task per column, each filling
    // its whole column serially. Only `DF_NCOLS` tasks => under-saturation.
    let col_ptrs: Vec<usize> = cols.iter().map(|c| c.ptr as usize).collect();
    col_ptrs.par_iter().enumerate().for_each(|(j, &ptr_bits)| {
        let ptr = ptr_bits as *mut f64;
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, DF_ROWS) };
        for (i, slot) in slice.iter_mut().enumerate() {
            *slot = df_cell(i, j);
        }
    });

    // Phase 3: assemble (identical to the builder's assembly).
    let df = with_r_thread(move || unsafe {
        use miniextendr_api::SEXPTYPE::VECSXP;
        let df = miniextendr_api::sys::Rf_allocVector_unchecked(
            VECSXP,
            DF_NCOLS as miniextendr_api::R_xlen_t,
        );
        miniextendr_api::sys::Rf_protect_unchecked(df);
        for (i, c) in cols.iter().enumerate() {
            df.set_vector_elt_unchecked(i as isize, c.sexp);
        }
        miniextendr_api::sys::Rf_unprotect_unchecked(DF_NCOLS as i32 + 1);
        df
    });
    divan::black_box(df);
}

// endregion

// region: serde growing-schema build — sequential union vs parallel (#936)
//
// `par_iter_to_dataframe_growing` is the rayon analogue of
// `vec_to_dataframe`'s union path. Both pay two serialization passes per row
// (union discovery + fill); the parallel variant fans both passes out across
// chunks. This pair answers the #936 cost-model question head-to-head on a
// heterogeneous input (two untagged-enum shapes, divergent fields).

#[cfg(all(feature = "rayon", feature = "serde"))]
mod serde_growing {
    use miniextendr_api::serde::{Serialize, par_iter_to_dataframe_growing, vec_to_dataframe};
    use miniextendr_api::worker::with_r_thread;
    use std::sync::LazyLock;

    const GROW_ROWS: usize = 500_000;

    #[derive(Serialize)]
    #[serde(untagged)]
    enum MixedRow {
        Old { id: i32, legacy: String },
        New { id: i32, score: f64 },
    }

    static ROWS: LazyLock<Vec<MixedRow>> = LazyLock::new(|| {
        (0..GROW_ROWS as i32)
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
            .collect()
    });

    #[divan::bench(sample_count = 10)]
    fn dataframe_growing_sequential_union() {
        let df = with_r_thread(|| vec_to_dataframe(ROWS.as_slice()).unwrap());
        divan::black_box(df);
    }

    #[divan::bench(sample_count = 10)]
    fn dataframe_growing_parallel() {
        let df =
            with_r_thread(|| par_iter_to_dataframe_growing(ROWS.iter(), Some(GROW_ROWS)).unwrap());
        divan::black_box(df);
    }
}

// endregion
