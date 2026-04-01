# impl Trait in #[miniextendr] — Investigation

## What works

**Return position** (`-> impl IntoR`): The `_` fallback in `analyze_return_type`
generates `IntoR::into_sexp(result)`. The trait bound must include `IntoR` because
the C wrapper is a caller that only sees the opaque type.

## What doesn't work

**Argument position** (`x: impl Trait`): Fundamentally blocked by Rust E0283.

The C wrapper is `extern "C-unwind"` (monomorphic). It receives SEXP and must
convert to a concrete type via `TryFromSexp::try_from_sexp(sexp)`. For `impl Trait`,
the concrete type is unknown — the macro omits the type annotation and hopes
Rust infers it from the function call.

Inference fails because Rust doesn't propagate type constraints backward through
`let` bindings. Even when only ONE type satisfies both `TryFromSexp + Trait`,
the compiler produces E0283 "type annotations needed."

## Approaches tried and abandoned

1. **Untyped conversion** (`conversion_stmt_untyped`): E0283 — inference can't
   resolve even with a unique satisfying type.

2. **Trait-to-type mapper** (`resolve_impl_trait_type`): Hardcoded mappings like
   `impl AsRef<str>` → `String`. Wrong — silently picks the wrong type if the
   user's type implements the trait differently.

3. **Blanket `TryFromSexp for T: IntoExternalPtr`**: Added so user-defined types
   get direct `TryFromSexp`. Still E0283 — the blanket impl doesn't help inference.

4. **Inline call without `let`**: `foo(TryFromSexp::convert(sexp).unwrap())` —
   still E0283.

## Root cause

Rust's trait solver (E0790/E0283) requires the concrete type to be determinable
at the `TryFromSexp::try_from_sexp()` call site. The constraint from the downstream
`foo(x)` call doesn't propagate backward to resolve the type of `x`.

This is a Rust language limitation, not a miniextendr limitation.

## Also fixed during investigation

- `json_string_tests` module gated behind `#[cfg(feature = "serde")]`
  (was causing pre-existing rpkg compilation failure).
