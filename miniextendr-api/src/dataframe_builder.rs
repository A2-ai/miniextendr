//! Closure-per-column [`DataFrame`](crate::dataframe::DataFrame) builder.
//!
//! [`RDataFrameBuilder`] assembles an R `data.frame` from a set of typed columns,
//! each with its own element type and fill closure. It is the heterogeneous-column
//! analogue of `with_r_matrix`: you declare columns, the builder allocates the
//! backing storage with strict PROTECT discipline, fills it, and assembles the
//! `data.frame` on the R thread.
//!
//! # Fill strategy (parallel with `rayon`, serial otherwise)
//!
//! The builder type and its surface are **available regardless of the `rayon`
//! feature** (#1055). Only the fill pass differs:
//!
//! - **With `rayon`** — the whole job is flattened into a single work-list of
//!   `(column_index, row-range)` items and filled in one work-stealing
//!   `par_iter` pass (see [`RDataFrameBuilder`] docs for the scheduling argument).
//! - **Without `rayon`** — each column is filled in one full-range serial pass.
//!
//! Both paths produce an identical `data.frame`. Nothing else in the builder
//! depends on rayon: `with_r_thread`, `Sendable`, `RNativeType`, the PROTECT
//! discipline, and the assembly step are all rayon-independent. The `Send + Sync`
//! bounds on the fill closures are simply unused when rayon is absent.

use crate::worker::{Sendable, with_r_thread};
use crate::{RNativeType, SEXP, SexpExt};

// region: type-erased column fill machinery

/// Type-erased "fill this row-range of this column" closure.
///
/// Called with `(offset, len)` describing a contiguous half-open row range
/// `[offset, offset + len)` of one column. The concrete closure (captured at
/// `.column::<T>()` / `.column_str()` registration time) knows the column's
/// element type and destination buffer, and writes exactly that range — no
/// other thread touches it (see the safety argument on [`RDataFrameBuilder`]).
type RangeFiller = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Send+Sync wrapper for a raw column-data pointer carried across the flatten
/// boundary. The pointer addresses R-owned memory for native columns or a
/// `Vec<Option<String>>` backing buffer for character columns. Disjointness
/// (per column and per row-range within a column) is the caller's invariant —
/// see the safety argument on [`RDataFrameBuilder`].
#[derive(Clone, Copy)]
struct ColPtr(*mut ());

unsafe impl Send for ColPtr {}
unsafe impl Sync for ColPtr {}

impl ColPtr {
    /// Reinterpret the erased base pointer as `*mut T`.
    ///
    /// Taking `&self` (a method call on the whole struct) makes a capturing
    /// closure capture the `Send + Sync` `ColPtr` as a whole rather than its
    /// raw `*mut ()` field (which is neither), keeping the closure `Send + Sync`.
    #[inline]
    fn cast<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

/// How a column's R storage is materialized after the fill.
enum ColumnKind {
    /// Native column: the fill wrote directly into R memory; the SEXP is already
    /// complete. Holds the allocated (and currently protected) SEXP.
    Native(Sendable<SEXP>),
    /// Character column: the fill computed `Option<String>` values into a `Vec`
    /// (no R API on fill threads). The `CHARSXP`s are set serially on the R
    /// thread during assembly. `None` becomes `NA_character_`.
    Str(Vec<Option<String>>),
}

/// One registered column: a serial allocation step plus the range filler that
/// the flattened work-list (or serial loop) dispatches.
struct ColumnReg {
    /// Allocates the column's backing storage (R SEXP for native columns, an
    /// owned `Vec` for character columns) for `nrow` rows. Runs serially on the
    /// R/worker thread. Returns the [`ColumnKind`] (carrying the protected SEXP
    /// or the owned buffer) and the raw data pointer the range filler writes
    /// through during the fill phase.
    #[allow(clippy::type_complexity)]
    alloc: Box<dyn FnOnce(usize) -> (ColumnKind, ColPtr) + Send>,
    /// Builds the type-erased range filler once the data pointer is known.
    make_filler: Box<dyn FnOnce(ColPtr, usize) -> RangeFiller + Send>,
}

// endregion

// region: RDataFrameBuilder

/// Builder for assembling an R `data.frame` from per-column fill closures.
///
/// This is the heterogeneous-column analogue of `with_r_matrix`: instead of one
/// homogeneous matrix, you declare a set of typed columns (each with its own
/// element type and fill closure) and the builder fills them all, then assembles
/// the `data.frame`.
///
/// # Fill strategy (parallel with `rayon`, serial otherwise)
///
/// The builder is available regardless of the `rayon` feature; only the fill
/// pass differs (see the [module docs](crate::dataframe_builder)). The rest of
/// this section describes the parallel pass that `rayon` enables.
///
/// ## Two axes of parallelism, one work-stealing pass
///
/// There are two ways to parallelise a column fill:
///
/// - **Column-granular** — one task per column. Fan-out width equals the column
///   count, so a 3-column × 10M-row frame only ever uses 3 threads.
/// - **Row-slice-granular** — split *one* column into contiguous row ranges.
///   Great for one long column, but on its own it serialises across columns.
///
/// `RDataFrameBuilder` does **not** choose. With `rayon`,
/// [`build`][RDataFrameBuilder::build] flattens the entire job into a single
/// work-list of `(column_index, row-range)` items — each native/character column
/// is split into `chunk_size = max(1, nrow / (current_num_threads() * 4))`-row
/// chunks (with 4× oversubscription) — then runs **one** `par_iter` over that
/// flat list. Rayon's work-stealing balances both axes automatically:
///
/// - **wide** (100 cols × short) → ~100+ items, column-dominated.
/// - **tall** (3 cols × 10M rows) → each column shatters into `~nthreads*4`
///   chunks → hundreds of items, saturated even with 3 columns.
/// - **skewed** (1 huge col + many tiny) → the huge column's chunks get stolen
///   by threads idle after finishing the tiny columns.
///
/// This also avoids the per-column barrier and repeated pool spin-up that the
/// naive "fill each column, each internally parallel" (nested `par_iter`) shape
/// would cause. Without `rayon`, each column is filled in one full-range pass.
///
/// # Phases
///
/// 1. Allocate each column's backing storage **serially on the R/worker thread**
///    (native columns get a protected R vector; character columns get an owned
///    `Vec<Option<String>>`). Strict PROTECT discipline — the dangerous part.
/// 2. Fill all columns (parallel flat pass with `rayon`, serial otherwise). No R
///    API calls happen inside the parallel region.
/// 3. Set character `CHARSXP`s serially on the R thread (CHARSXP allocation is
///    forbidden on rayon threads), then assemble the `VECSXP`, `names`, compact
///    `row.names` (`c(NA_integer_, -nrow)`), and `class = "data.frame"`.
///
/// # Column kinds
///
/// - [`column::<T>`][RDataFrameBuilder::column] — a native-typed column
///   (`f64`/`i32`/`RLogical`/`u8`/`Rcomplex`). The fill closure receives a
///   mutable chunk and its offset. The buffer is R memory, filled directly with
///   zero intermediate allocation.
/// - [`column_str`][RDataFrameBuilder::column_str] — a character (`STRSXP`)
///   column. The per-row `Option<String>` values are computed during the fill
///   pass, but the `CHARSXP`s are set **serially** afterward. `None` becomes
///   `NA_character_`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::dataframe_builder::RDataFrameBuilder;
///
/// let df = RDataFrameBuilder::new(1000)
///     .column::<f64>("x", |chunk, offset| {
///         for (i, slot) in chunk.iter_mut().enumerate() {
///             *slot = ((offset + i) as f64).sqrt();
///         }
///     })
///     .column::<i32>("y", |chunk, offset| {
///         for (i, slot) in chunk.iter_mut().enumerate() {
///             *slot = (offset + i) as i32;
///         }
///     })
///     .column_str("label", |i| Some(format!("row_{i}")))
///     .build();
/// ```
///
/// # Safety argument (disjoint mutation, no aliasing)
///
/// Neither fill path ever produces two items that overlap:
///
/// - Different columns address **different** backing buffers (distinct R vectors
///   / distinct `Vec`s), so cross-column items are trivially disjoint.
/// - Within a column, the row ranges are a partition of `[0, nrow)`. The serial
///   path uses the single full range; the parallel path chunks `nrow` into
///   fixed-size, non-overlapping spans. Each `(offset, len)` item therefore owns
///   a unique slice of that column's buffer.
///
/// Each `RangeFiller` reconstitutes its slice via
/// `slice::from_raw_parts_mut(base.add(offset), len)` and writes only that span.
/// Because the spans are disjoint, no two threads ever form overlapping `&mut`
/// references — there is no aliasing UB even though the work-list shares the raw
/// base pointers (`ColPtr`, `Send + Sync`).
///
/// # Protection
///
/// Every native column SEXP is PROTECTed from allocation through insertion into
/// the `VECSXP`; the `names` / `row.names` / class transients are likewise
/// protected across each subsequent allocation. After
/// [`build`][RDataFrameBuilder::build] returns, the resulting data.frame SEXP is
/// unprotected and becomes the caller's responsibility (return it from a
/// `#[miniextendr]` fn, or PROTECT it).
pub struct RDataFrameBuilder {
    nrow: usize,
    names: Vec<String>,
    columns: Vec<ColumnReg>,
}

impl RDataFrameBuilder {
    /// Start building a data.frame with `nrow` rows.
    pub fn new(nrow: usize) -> Self {
        // Compact row.names uses i32, so nrow must fit in i32; this also implies
        // it fits in R_xlen_t on all supported pointer widths.
        assert!(
            nrow <= i32::MAX as usize,
            "RDataFrameBuilder: nrow {} exceeds i32 maximum (compact row.names)",
            nrow
        );
        Self {
            nrow,
            names: Vec::new(),
            columns: Vec::new(),
        }
    }

    /// Add a native-typed column (`f64`/`i32`/`RLogical`/`u8`/`Rcomplex`).
    ///
    /// The fill closure `f(chunk, offset)` is dispatched over chunks of the
    /// (already-allocated) R column buffer — in parallel with `rayon`, or in one
    /// full-range pass otherwise. Chunk boundaries are deterministic for a given
    /// `nrow` and thread count.
    pub fn column<T>(
        mut self,
        name: impl Into<String>,
        f: impl Fn(&mut [T], usize) + Send + Sync + 'static,
    ) -> Self
    where
        T: RNativeType + Send + Sync,
    {
        self.names.push(name.into());
        self.columns.push(ColumnReg {
            alloc: Box::new(|nrow| {
                // Allocate + protect the R vector serially on the R thread, then
                // hand back its data pointer for the fill. The protection is
                // balanced during assembly in `build`.
                let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
                    let sexp =
                        crate::sys::Rf_allocVector_unchecked(T::SEXP_TYPE, nrow as crate::R_xlen_t);
                    crate::sys::Rf_protect_unchecked(sexp);
                    let ptr = T::dataptr_mut(sexp);
                    (sexp, Sendable(ptr))
                });
                (ColumnKind::Native(Sendable(sexp)), ColPtr(ptr as *mut ()))
            }),
            make_filler: Box::new(move |base: ColPtr, nrow: usize| {
                Box::new(move |offset: usize, len: usize| {
                    debug_assert!(offset + len <= nrow);
                    // Safety: this range `[offset, offset+len)` is a disjoint
                    // partition of this column's buffer (see the safety argument
                    // on `RDataFrameBuilder`); no other thread writes it. `base`
                    // is the `Send + Sync` `ColPtr`; cast inside the closure so
                    // the closure stays `Send + Sync`.
                    let ptr = base.cast::<T>();
                    let slice = unsafe { std::slice::from_raw_parts_mut(ptr.add(offset), len) };
                    f(slice, offset);
                })
            }),
        });
        self
    }

    /// Add a character (`STRSXP`) column.
    ///
    /// The fill closure `f(i)` returns the value for row `i` as `Option<String>`,
    /// where `None` maps to `NA_character_`. Values are computed during the fill
    /// pass (parallel with `rayon`, serial otherwise), then set into the R
    /// `STRSXP` serially on the R thread (CHARSXP allocation cannot happen on
    /// rayon threads).
    pub fn column_str(
        mut self,
        name: impl Into<String>,
        f: impl Fn(usize) -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        self.names.push(name.into());
        self.columns.push(ColumnReg {
            alloc: Box::new(|nrow| {
                // No R allocation here: the fill phase fills an owned Vec.
                let mut buf: Vec<Option<String>> = (0..nrow).map(|_| None).collect();
                let ptr = buf.as_mut_ptr();
                (ColumnKind::Str(buf), ColPtr(ptr as *mut ()))
            }),
            make_filler: Box::new(move |base: ColPtr, nrow: usize| {
                Box::new(move |offset: usize, len: usize| {
                    debug_assert!(offset + len <= nrow);
                    // Safety: disjoint partition of this column's Vec buffer.
                    // Cast `base` (Send + Sync `ColPtr`) inside the closure.
                    let ptr = base.cast::<Option<String>>();
                    let slice = unsafe { std::slice::from_raw_parts_mut(ptr.add(offset), len) };
                    for (i, slot) in slice.iter_mut().enumerate() {
                        *slot = f(offset + i);
                    }
                })
            }),
        });
        self
    }

    /// Allocate, fill, and assemble the data.frame, returning an owned,
    /// GC-rooted [`BuiltDataFrame`](crate::dataframe::BuiltDataFrame) (it
    /// `Deref`s to [`DataFrame`](crate::dataframe::DataFrame)).
    ///
    /// With `rayon`, flattens every column into a single `(column_index,
    /// row-range)` work-list and runs one parallel pass over it (see the
    /// type-level docs for the scheduling argument); without `rayon`, fills each
    /// column serially. Then assembles the `data.frame` on the R thread.
    pub fn build(self) -> crate::dataframe::BuiltDataFrame {
        // SAFETY: `build_sexp` returns a well-formed data.frame VECSXP; root it
        // immediately (no allocation between assembly and adopt).
        unsafe { crate::dataframe::BuiltDataFrame::adopt_sexp(self.build_sexp()) }
    }

    /// Assemble and return the raw `VECSXP` SEXP (internal; prefer [`build`](Self::build)).
    fn build_sexp(self) -> SEXP {
        let RDataFrameBuilder {
            nrow,
            names,
            columns,
        } = self;
        let ncol = columns.len();
        assert_eq!(
            names.len(),
            ncol,
            "RDataFrameBuilder: names/columns length mismatch"
        );
        // Compact row.names `c(NA, -nrow)` are emitted as INTSXP, so `nrow` must
        // fit in `i32`. Validate up front (panic → R error) rather than letting
        // `-(nrow as i32)` below silently wrap for >2^31-row frames.
        assert!(
            nrow <= i32::MAX as usize,
            "RDataFrameBuilder: nrow {nrow} exceeds i32 maximum for compact row.names"
        );

        // Phase 1: allocate every column's backing storage serially. Native
        // columns return a freshly-protected R SEXP and its data pointer;
        // character columns return an owned `Vec<Option<String>>` and its
        // pointer. We re-protect native columns *as they are allocated* (inside
        // `alloc`), so the per-column allocation in the next iteration cannot GC
        // an earlier column. These protections are balanced during assembly.
        let mut kinds: Vec<ColumnKind> = Vec::with_capacity(ncol);
        let mut fillers: Vec<RangeFiller> = Vec::with_capacity(ncol);
        for col in columns {
            let ColumnReg { alloc, make_filler } = col;
            let (kind, ptr) = alloc(nrow);
            kinds.push(kind);
            fillers.push(make_filler(ptr, nrow));
        }

        // Phase 2: fill every column. No R API calls happen here.
        if nrow > 0 && ncol > 0 {
            #[cfg(feature = "rayon")]
            {
                use rayon::prelude::*;
                crate::optionals::parallel::ensure_pool();
                // Flatten to ONE (column, row-range) work-list and run a single
                // parallel pass. Each item is `(column_index, offset, len)`; the
                // column's type-erased range filler writes exactly that disjoint
                // span. Rayon's work-stealing balances the column axis and the
                // row-slice axis together.
                let chunk_size = std::cmp::max(1, nrow / (rayon::current_num_threads() * 4));
                let work: Vec<(usize, usize, usize)> = (0..ncol)
                    .flat_map(|col_idx| {
                        (0..nrow).step_by(chunk_size).map(move |offset| {
                            let len = std::cmp::min(chunk_size, nrow - offset);
                            (col_idx, offset, len)
                        })
                    })
                    .collect();

                work.par_iter().for_each(|&(col_idx, offset, len)| {
                    (fillers[col_idx])(offset, len);
                });
            }
            #[cfg(not(feature = "rayon"))]
            {
                // Serial fill: each column filled in one full-range pass.
                // ponytail: the chunked work-list only buys parallelism, which is
                // exactly what `rayon` adds; the result is identical either way.
                for filler in &fillers {
                    filler(0, nrow);
                }
            }
        }
        // Fillers are no longer needed; drop them before assembly so any captured
        // closures release before we touch R.
        drop(fillers);

        // Phase 3: assemble on the R thread with strict PROTECT discipline. We
        // are inside `with_r_thread`, a known-safe context, so `_unchecked` FFI
        // is correct here (MXL301).
        //
        // PROTECT-stack invariant on entry: phase 1 left one protection per
        // *native* column (character columns hold no SEXP yet). Track that exact
        // count and balance it precisely.
        with_r_thread(move || unsafe {
            use crate::SEXPTYPE::{INTSXP, STRSXP, VECSXP};

            // Materialize character columns into protected STRSXPs now (CHARSXP
            // allocation must be serial on the R thread). Each freshly allocated
            // STRSXP is protected immediately and stays protected until rooted in
            // the parent VECSXP, exactly like the native columns.
            //
            // `native_protected` counts the protections phase 1 left on the
            // stack; we add one per character column we protect here.
            let mut native_protected = 0i32;
            let mut col_sexps: Vec<SEXP> = Vec::with_capacity(ncol);
            for kind in kinds {
                match kind {
                    ColumnKind::Native(Sendable(sexp)) => {
                        native_protected += 1;
                        col_sexps.push(sexp);
                    }
                    ColumnKind::Str(values) => {
                        let sexp =
                            crate::sys::Rf_allocVector_unchecked(STRSXP, nrow as crate::R_xlen_t);
                        crate::sys::Rf_protect_unchecked(sexp);
                        native_protected += 1;
                        for (i, v) in values.iter().enumerate() {
                            match v {
                                Some(s) => {
                                    sexp.set_string_elt_unchecked(i as isize, SEXP::charsxp(s))
                                }
                                None => {
                                    sexp.set_string_elt_unchecked(i as isize, SEXP::na_string())
                                }
                            }
                        }
                        col_sexps.push(sexp);
                    }
                }
            }
            // SAFETY: `native_protected` is a non-negative running count, so the
            // sign cast to `usize` cannot lose data.
            #[allow(clippy::cast_sign_loss)]
            let native_protected_usize = native_protected as usize;
            debug_assert_eq!(native_protected_usize, ncol);

            // Allocate the parent list and protect it.
            let df = crate::sys::Rf_allocVector_unchecked(VECSXP, ncol as crate::R_xlen_t);
            crate::sys::Rf_protect_unchecked(df);

            // Root every column in the parent (SET_VECTOR_ELT does not allocate).
            for (i, col) in col_sexps.into_iter().enumerate() {
                df.set_vector_elt_unchecked(i as isize, col);
            }

            // The columns are now reachable from `df`, so their individual
            // protections are no longer needed. Drop all `ncol + 1` protections
            // (the columns and `df`) and immediately re-protect `df` — no
            // allocation happens between the two calls, so `df` cannot be
            // collected in the gap.
            crate::sys::Rf_unprotect_unchecked(native_protected + 1);
            crate::sys::Rf_protect_unchecked(df);

            // names: STRSXP of column names. Protect across CHARSXP allocations.
            let names_sexp = crate::sys::Rf_allocVector_unchecked(STRSXP, ncol as crate::R_xlen_t);
            crate::sys::Rf_protect_unchecked(names_sexp);
            for (i, name) in names.iter().enumerate() {
                let charsxp = SEXP::charsxp(name);
                names_sexp.set_string_elt_unchecked(i as isize, charsxp);
            }
            df.set_names(names_sexp);
            crate::sys::Rf_unprotect_unchecked(1); // names_sexp now reachable via df

            // Compact row.names: c(NA_integer_, -nrow).
            let row_names = crate::sys::Rf_allocVector_unchecked(INTSXP, 2);
            crate::sys::Rf_protect_unchecked(row_names);
            row_names.set_integer_elt(0, i32::MIN); // NA_integer_
            // SAFETY: `nrow <= i32::MAX` asserted in `build_sexp`, so the
            // narrowing cast cannot truncate the compact row.names count.
            #[allow(clippy::cast_possible_truncation)]
            let neg_nrow = -(nrow as i32);
            row_names.set_integer_elt(1, neg_nrow);
            df.set_row_names(row_names);
            crate::sys::Rf_unprotect_unchecked(1); // row_names now reachable via df

            // class = "data.frame" (cached STRSXP — no fresh allocation).
            df.set_class(crate::cached_class::data_frame_class_sexp());

            // Balance the remaining `df` protection. No allocation follows, so
            // `df` survives until the caller takes ownership.
            crate::sys::Rf_unprotect_unchecked(1);
            df
        })
    }
}

// endregion

#[cfg(test)]
mod tests {
    use super::*;

    /// Serial-path proof (#1055): the builder API compiles and is reachable
    /// without `rayon`, registering native and character columns. This builds
    /// the registration machinery without invoking R (no `build()` call, which
    /// needs the R runtime), so it runs in the plain `cargo test` harness on
    /// both feature settings — the primary guarantee is that this module
    /// compiles and the surface is callable serially.
    #[test]
    fn builder_surface_is_callable_without_rayon() {
        let builder = RDataFrameBuilder::new(8)
            .column::<f64>("x", |chunk, offset| {
                for (i, slot) in chunk.iter_mut().enumerate() {
                    *slot = (offset + i) as f64;
                }
            })
            .column_str("label", |i| Some(format!("row{i}")));
        assert_eq!(builder.nrow, 8);
        assert_eq!(builder.names, vec!["x".to_string(), "label".to_string()]);
        assert_eq!(builder.columns.len(), 2);
    }
}
