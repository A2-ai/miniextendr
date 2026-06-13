//! `into_sexp()` inside a `vec!`/array literal — use-after-free idiom.
//!
//! - MXL302: Warns on `into_sexp()` / `into_sexp_unchecked()` calls that appear as
//!   elements *inside* a `vec!` or `&[...]` literal.
//!
//!   Each `into_sexp` allocates a fresh SEXP. When several occur in one literal
//!   (`vec![(k, a.into_sexp()), (k, b.into_sexp())]`), nothing roots the earlier
//!   elements until the whole `Vec` reaches `List::from_raw_pairs`, so building a
//!   later element can trigger a GC that collects an earlier, still-unprotected one
//!   — a use-after-free. This recurred enough (#307, the 2026-05-07 gctorture audit)
//!   that the `IntoList` / `DataFrameRow` derives now wrap every element in
//!   `__scope.protect_raw(...)`; this lint stops new hand-written sites from
//!   reintroducing the raw form silently.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (call_name, line) in &data.vec_into_sexp_calls {
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL302,
                    path,
                    *line,
                    format!(
                        "`{}()` is called inside a `vec!`/array literal. Each `into_sexp` \
                         allocates; an earlier element built this way is left unprotected \
                         across the next element's allocation — a use-after-free under GC.",
                        call_name
                    ),
                )
                .with_help(
                    "Protect each element as it is built: open a `ProtectScope` and wrap each \
                     call, e.g. `__scope.protect_raw(x.into_sexp())`, then hand the `Vec` to \
                     `List::from_raw_pairs` / `from_raw_values`. The `IntoList` and \
                     `DataFrameRow` derives already do this — prefer them over a hand-rolled \
                     literal. Suppress an intentional, provably-safe site with \
                     `// mxl::allow(MXL302)`.",
                ),
            );
        }
    }
}
