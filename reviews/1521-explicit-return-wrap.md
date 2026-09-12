# Explicit return class wrapping (#1521)

The initial mutable container walk held a borrow across its exit, so inspecting
or replacing the leaf failed Rust borrow checking. The resolver now validates
through shared borrows, then replaces the validated payload slot in its clone.
Both marker and attribute spellings still produce the same wrapping plan.

The first full-feature R install failed in the new vctrs fixture: `Vctrs` derive calls `VctrsClass` methods, so that trait must be in scope, as in the existing derive examples. Imported `VctrsClass`; default `just check` does not enable this optional fixture.

CI-equivalent clippy caught four style diagnostics in the new parser (while-let, collapsible-if, and two redundant reborrows). Rewrote those forms without changing the parse plan.

The first runtime sweep caught an incorrect test assumption: inherent Option<Vec<T>> methods raise on None, while free functions use the complete Option IntoR mapping (NULL for an absent vector). Kept the existing conversion paths; corrected the method assertion/docs and added both nullable free-function spellings to pin the distinction.

Compiling those free-function cases then exposed the actual constraint: `Option<Vec<T>>: IntoR` only covers native elements (and selected explicit types), not arbitrary registered classes. Explicit free-function class wrapping now selects the same Option-unwrapping boundary as methods, raising on None. This supports both syntaxes without adding broad conversion impls; ordinary unmarked free functions retain their existing mapping. The tests and docs now state this explicit-return rule.
