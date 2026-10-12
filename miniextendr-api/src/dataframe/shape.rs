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

use super::BuiltDataFrame;
use crate::into_r::IntoR;
use crate::{NamedListBuilder, Preserved, SEXP};

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
/// [`SplitResults::None`] sentinel carries its own [`Preserved`]
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
    /// `empty_ok_sentinel`, GC-rooted by its [`Preserved`] handle. The
    /// sentinel is an arbitrary R value (`NULL`, `NA`, `FALSE`, a zero-row
    /// data.frame, …), so it cannot ride in a [`BuiltDataFrame`].
    ///
    /// The handle is consumed by the [`IntoR`] impl on
    /// [`DataFrameShape`]; until then its own root keeps the sentinel
    /// reachable (#1265) — no immediate-conversion contract for this leg.
    None(Preserved),
}

impl IntoR for DataFrameShape {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        match self {
            DataFrameShape::Bare(df) => Ok(df.into_sexp()),
            DataFrameShape::Split { results, error } => {
                let mut builder = NamedListBuilder::with_capacity(2);
                match results {
                    SplitResults::Some(df) => builder.push("results", df),
                    SplitResults::None(sentinel) => builder.push("results", sentinel),
                }
                builder.push("error", error);
                Ok(builder.into_sexp())
            }
            DataFrameShape::PerVariantList(pairs) => {
                let mut builder = NamedListBuilder::with_capacity(pairs.len());
                for (name, df) in pairs {
                    builder.push(name, df);
                }
                Ok(builder.into_sexp())
            }
        }
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
}
