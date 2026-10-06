//! [`DataFrameShape`]: the owned, GC-rooted return shape of the data.frame
//! split verbs.
//!
//! Two producers share it:
//!
//! - `rows.into_dataframe_split()` ([`IntoDataFrameSplit`](super::IntoDataFrameSplit),
//!   from `#[derive(DataFrameRow)]` on an enum) — always
//!   [`Bare`](DataFrameShape::Bare) or [`PerVariantList`](DataFrameShape::PerVariantList);
//! - the serde helpers `vec_to_dataframe_split` / `result_to_dataframe`
//!   (`feature = "serde"`) — every variant.
//!
//! The type lives here rather than under `serde` because the derive split is
//! not serde-gated.

use super::{BuiltDataFrame, NamedDataFrameListBuilder};
use crate::SEXP;
use crate::into_r::IntoR;

/// Categorical return shape of the data.frame split verbs:
/// [`IntoDataFrameSplit::into_dataframe_split`](super::IntoDataFrameSplit::into_dataframe_split)
/// and the serde helpers `vec_to_dataframe_split` / `result_to_dataframe`
/// (`feature = "serde"`).
///
/// Carries enough type information that downstream Rust code can `match`
/// on the variant without dispatching on SEXP type. Convert to a SEXP via
/// the [`IntoR`] impl (or just return it from a `#[miniextendr]`
/// fn), which collapses every variant to the equivalent R value (bare
/// data.frame / named list of data.frames / `list(results=, error=)`).
///
/// # GC discipline (rooted by construction, #1265, #1750)
///
/// Every frame inside the shape is an owned, GC-rooted
/// [`BuiltDataFrame`], and the
/// [`SplitResults::None`] sentinel carries its own [`RootedSentinel`]
/// root — so holding a `DataFrameShape` across R allocations is safe by
/// construction; there is no convert-immediately contract. The
/// [`IntoR`] conversion consumes the handles, re-rooting each
/// child in the assembled R value.
pub enum DataFrameShape {
    /// Single data.frame.
    ///
    /// Used by:
    /// - `into_dataframe_split` on a **single-variant** enum, always.
    /// - serde `vec_to_dataframe_split` when only one variant is present (and
    ///   `SplitShape::PerVariantList` or `SplitShape::PerVariantListWithTag`
    ///   is selected — the single-variant short-circuit) or always under
    ///   `SplitShape::Collated`.
    /// - serde `result_to_dataframe` under `ResultShape::Auto` when every row
    ///   is `Ok`, and always under `ResultShape::Collated`.
    Bare(BuiltDataFrame),

    /// `list(results = <df | sentinel>, error = df)`.
    ///
    /// Produced only by serde `result_to_dataframe`: under
    /// `ResultShape::Auto` when at least one `Err` is present, and always
    /// under `ResultShape::Split`. The split verbs never produce it.
    Split {
        /// The Ok partition.
        results: SplitResults,
        /// The error partition (always present, possibly zero-row).
        error: BuiltDataFrame,
    },

    /// `list(VariantA = df, VariantB = df, …)`.
    ///
    /// Produced by:
    /// - `into_dataframe_split` on a **multi-variant** enum, always: one
    ///   entry per declared variant, in declaration order, keyed by the
    ///   snake_case variant name. A variant absent from the input is a 0-row
    ///   frame with that variant's columns.
    /// - serde `vec_to_dataframe_split` under `SplitShape::PerVariantList` /
    ///   `SplitShape::PerVariantListWithTag` when the input contains more
    ///   than one variant. Order matches first-seen order in the input slice.
    PerVariantList(Vec<(String, BuiltDataFrame)>),
}

/// Result partition for [`DataFrameShape::Split`].
///
/// Used to distinguish "no Ok rows at all" (which lets the caller supply
/// a sentinel value such as `NULL`, `NA`, `FALSE`, …) from a real
/// zero-row data.frame.
pub enum SplitResults {
    /// At least one `Ok` row — partition has a concrete data.frame.
    Some(BuiltDataFrame),
    /// No `Ok` rows — sentinel value supplied by the caller via
    /// `empty_ok_sentinel`, GC-rooted by its [`RootedSentinel`] handle.
    ///
    /// The handle is consumed by the [`IntoR`] impl on
    /// [`DataFrameShape`]; until then its own root keeps the sentinel
    /// reachable (#1265) — no immediate-conversion contract for this leg.
    None(RootedSentinel),
}

/// GC-rooted holder for the empty-`Ok` sentinel in [`SplitResults::None`].
///
/// The sentinel is an arbitrary caller-supplied R value (`NULL`, `NA`,
/// `FALSE`, a zero-row data.frame, …), so it cannot ride in a
/// [`BuiltDataFrame`]; this is the same
/// RAII pattern (`R_PreserveObject` on construction, `R_ReleaseObject` on
/// drop, `!Send`) for one bare SEXP. It exists so every leg of
/// [`DataFrameShape`] is rooted by construction (#1265).
pub struct RootedSentinel {
    sexp: SEXP,
    /// `!Send + !Sync`: rooting/unrooting mutates R's global precious
    /// list, which is R-main-thread state.
    _not_send: std::marker::PhantomData<*mut ()>,
}

impl RootedSentinel {
    /// Convert `value` to a SEXP and take ownership of a fresh root.
    ///
    /// Rooting is immediate: no allocation happens between the
    /// `into_sexp` return and `R_PreserveObject`, and `R_PreserveObject`
    /// keeps its argument reachable across its own cons-cell allocation.
    pub fn new(value: impl IntoR) -> Self {
        let sexp = value.into_sexp();
        // SAFETY: `into_sexp` just ran on the R main thread and returned a
        // valid SEXP; rooting is immediate (see above).
        unsafe {
            crate::sys::R_PreserveObject(sexp);
        }
        Self {
            sexp,
            _not_send: std::marker::PhantomData,
        }
    }

    /// The rooted SEXP. Stays reachable for as long as this handle lives.
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }
}

impl Drop for RootedSentinel {
    fn drop(&mut self) {
        // SAFETY: `!Send` guarantees drop runs on the constructing (R main)
        // thread; release the exact root added in `new`. `R_ReleaseObject`
        // does not allocate.
        unsafe { crate::sys::R_ReleaseObject(self.sexp) };
    }
}

impl IntoR for DataFrameShape {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        match self {
            DataFrameShape::Bare(df) => Ok(df.into_sexp()),
            DataFrameShape::Split { results, error } => {
                // Protect both children via the NamedDataFrameListBuilder's
                // scope so neither is reaped between the set_vector_elt /
                // CHARSXP allocations in from_raw_pairs. Each owned handle
                // stays alive until after its child is protected in the
                // builder's scope, then drops alloc-free — the child is
                // rooted at every instant.
                let mut builder = NamedDataFrameListBuilder::with_capacity(2);
                builder = match results {
                    // `push` protects the view; the `df` handle's own root
                    // releases at the end of this arm, after the protect.
                    SplitResults::Some(df) => builder.push("results", *df),
                    SplitResults::None(sentinel) => {
                        // SAFETY: `sentinel` wraps a valid SEXP kept alive by
                        // its own root; `push_raw` protects it in the
                        // builder's scope before the handle drops at the end
                        // of this arm.
                        unsafe { builder.push_raw("results", sentinel.as_sexp()) }
                    }
                };
                // As above: `error`'s root holds until this arm ends, past
                // the protect inside `push`.
                builder = builder.push("error", *error);
                Ok(builder.build().into_sexp())
            }
            DataFrameShape::PerVariantList(pairs) => {
                let mut builder = NamedDataFrameListBuilder::with_capacity(pairs.len());
                for (name, df) in pairs {
                    // `push` protects the view; the `df` handle's own root
                    // releases at the end of the iteration, after the protect.
                    builder = builder.push(name, *df);
                }
                Ok(builder.build().into_sexp())
            }
        }
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
}
