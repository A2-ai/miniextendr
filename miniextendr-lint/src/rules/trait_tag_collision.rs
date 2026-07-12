//! MXL303: trait-impl vtable-symbol collision detection.
//!
//! Each `#[miniextendr] impl Trait for Type` emits a vtable static named
//! `__VTABLE_{CRATE}_{TRAIT}_FOR_{TYPE}` (see
//! `miniextendr-macros/src/naming.rs::vtable_static_ident`, called from
//! `miniextendr-macros/src/miniextendr_impl_trait/vtable.rs`), where `{CRATE}`
//! is the consuming crate's uppercased name (#1273 webR cross-package symbol
//! uniqueness), the trait name is `trait_ident.to_uppercase()`, and the type
//! name is the last path segment uppercased (`type_to_uppercase_name`). The
//! static is emitted with `#[unsafe(no_mangle)]`, so two impls whose
//! `(TRAIT, TYPE)` pair collapses to the same uppercased symbol produce
//! **duplicate `no_mangle` symbols** — a hard linker failure whose message is
//! divorced from the source.
//!
//! The lint compares symbols *within one crate*, where the crate prefix is a
//! constant — so the comparison here uses the crate-invariant
//! `__VTABLE_{TRAIT}_FOR_{TYPE}` suffix rather than reconstructing the prefix
//! (the lint runs from the user crate's `build.rs`, which has no
//! `CARGO_CRATE_NAME` for the crate being linted anyway). Verdicts are
//! identical either way.
//!
//! Rust's coherence rules already forbid the *same* trait implemented twice for
//! the *same* type, so the only way two distinct impls collide is via the
//! macro's **case-folding**: e.g. `impl Counter for Foo` and `impl counter for
//! Foo`, or `impl Trait for Foo` and `impl Trait for foo`, all collapse to the
//! same `__VTABLE_…` symbol. This rule catches that before the linker does.
//!
//! Scope: crate-wide. The two colliding impls may live in different files, so
//! the check aggregates across the whole [`CrateIndex`] rather than per-file.
//!
//! Escape hatch: `// mxl::allow(MXL303)` on (or directly above) either impl
//! line suppresses the diagnostic for that impl. `MINIEXTENDR_LINT=0` disables
//! all rules.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

/// A single occurrence of a trait impl, used for collision reporting.
struct Occurrence {
    path: PathBuf,
    line: usize,
    /// Original (case-preserving) `impl Trait for Type` rendering.
    rendered: String,
    suppressed: bool,
}

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    // Group every attributed trait impl by the case-folded vtable symbol the
    // macro would emit. A bucket with >1 distinct rendering is a collision.
    let mut buckets: HashMap<String, Vec<Occurrence>> = HashMap::new();

    for (path, data) in &index.file_data {
        for ati in &data.attributed_trait_impls {
            let symbol = vtable_symbol(&ati.trait_name, &ati.type_name);
            buckets.entry(symbol).or_default().push(Occurrence {
                path: path.clone(),
                line: ati.line,
                rendered: format!("impl {} for {}", ati.trait_name, ati.type_name),
                suppressed: ati.suppressed_mxl303,
            });
        }
    }

    for (symbol, mut occurrences) in buckets {
        // Distinct source renderings only — Rust coherence forbids a verbatim
        // duplicate, so >1 here always means a case-fold collapse.
        let distinct: std::collections::BTreeSet<&str> =
            occurrences.iter().map(|o| o.rendered.as_str()).collect();
        if distinct.len() < 2 {
            continue;
        }

        // Deterministic ordering for stable diagnostics.
        occurrences.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));

        let all_renderings = occurrences
            .iter()
            .map(|o| format!("`{}` ({}:{})", o.rendered, o.path.display(), o.line))
            .collect::<Vec<_>>()
            .join(", ");

        for occ in &occurrences {
            if occ.suppressed {
                continue;
            }
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL303,
                    &occ.path,
                    occ.line,
                    format!(
                        "trait impl `{}` collides on the generated vtable symbol \
                         `{}` with another impl. The macro upper-cases the trait \
                         and type names when naming the `#[no_mangle]` vtable \
                         static, so these distinct impls emit duplicate symbols \
                         and the build fails at link time. Colliding impls: {}.",
                        occ.rendered, symbol, all_renderings,
                    ),
                )
                .with_help(
                    "rename one of the colliding trait or type identifiers so the \
                     upper-cased names differ, or suppress with \
                     `// mxl::allow(MXL303)` if the collision is intentional",
                ),
            );
        }
    }
}

/// Reconstruct the crate-invariant suffix of the vtable static symbol the
/// macro emits for `impl Trait for Type`.
///
/// Mirrors `miniextendr-macros/src/naming.rs::vtable_static_ident` minus the
/// crate prefix: the emitted symbol is
/// `__VTABLE_{CRATE}_{trait.to_uppercase()}_FOR_{type.to_uppercase()}` (#1273),
/// but within one crate the `{CRATE}` prefix is constant, so comparing the
/// unprefixed suffix yields identical collision verdicts without needing
/// `CARGO_CRATE_NAME` (which the linting `build.rs` doesn't have for the
/// crate being linted). The lint only sees the bare type ident (no generic
/// args), so it folds case the same way the macro's `type_to_uppercase_name`
/// does for plain (non-generic) types.
fn vtable_symbol(trait_name: &str, type_name: &str) -> String {
    format!(
        "__VTABLE_{}_FOR_{}",
        trait_name.to_uppercase(),
        type_name.to_uppercase()
    )
}

#[cfg(test)]
mod tests {
    use super::vtable_symbol;

    #[test]
    fn case_fold_collapses_to_same_symbol() {
        // Distinct traits differing only in case → same symbol.
        assert_eq!(
            vtable_symbol("Counter", "Foo"),
            vtable_symbol("counter", "Foo")
        );
        // Distinct types differing only in case → same symbol.
        assert_eq!(
            vtable_symbol("Counter", "Foo"),
            vtable_symbol("Counter", "foo")
        );
    }

    #[test]
    fn distinct_names_distinct_symbols() {
        assert_ne!(
            vtable_symbol("Counter", "Foo"),
            vtable_symbol("Resettable", "Foo")
        );
        assert_ne!(
            vtable_symbol("Counter", "Foo"),
            vtable_symbol("Counter", "Bar")
        );
    }
}
