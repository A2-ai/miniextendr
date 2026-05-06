+++
title = "Issue #363 — Unify make_rust_error_value into make_rust_condition_value"
+++

# Unify `make_rust_error_value` into `make_rust_condition_value`

Closes #363. Removes the legacy 3-element error-value transport and
consolidates every panic-to-R-error path on the 4-element condition transport
introduced by PR #344.

## Why

`miniextendr-api/src/error_value.rs` currently exposes two builders:

- `make_rust_error_value` (3-element list: `error`, `kind`, `call`).
- `make_rust_condition_value` (4-element list: `error`, `kind`, `class`,
  `call`).

The R-side reader looks up slots by **name**, not position. Looking up `$class`
on the 3-element form returns `NULL` (the name is absent); looking up `$class`
on the 4-element form returns the user class or `NULL`. `$call` resolves on
both. Wire formats are therefore observationally identical for every R-side
caller when `make_rust_condition_value(msg, kind, None, call)` is substituted
for `make_rust_error_value(msg, kind, call)`.

This consolidation cuts ~70 lines of duplicated PROTECT bookkeeping, removes
one of the two functions developers have to keep in sync, and concentrates
PROTECT-discipline review (a known segfault hotspot, see `af6b4875`) in one
place.

## Wire-format proof

R-side switch reader (`miniextendr_raise_condition` in the rpkg R wrappers)
reads `$error`, `$kind`, `$class`, `$call` by name. None of those positions
are accessed numerically. The 3-element list omits `class`; `[[ "class" ]]`
returns `NULL`. The 4-element list with `class = NULL` carries an explicit
`R_NilValue` slot at name `class`. Both materialize as `is.null(.val$class)`
true on the R side. The `kind` slot is the discriminant; both transports
agree on its values.

The class attribute (`"rust_condition_value"`) and the `__rust_condition__`
flag attribute are identical in both builders.

## Work plan (flat priority order)

### 1. Extract `to_cstring_lossy` helper in `error_value.rs`

The CString fallback pattern (`CString::new(s).unwrap_or_else(|_|
CString::new(fallback).unwrap())`) appears four times in
`make_rust_condition_value` and four times in `make_rust_error_value`. Extract:

```rust
fn to_cstring_lossy(s: &str, fallback: &str) -> std::ffi::CString {
    std::ffi::CString::new(s)
        .unwrap_or_else(|_| std::ffi::CString::new(fallback).unwrap())
}
```

Used internally by `make_rust_condition_value`. Keeps the unsafe block
focused on the PROTECT story instead of CString plumbing.

### 2. Delete `make_rust_error_value`

After all call sites move to `make_rust_condition_value(msg, kind, None,
call)`, the function and its module-level doc-comment "legacy" subsection
are removed. The doc references in `cached_class.rs` and `condition.rs`
that mention `make_rust_error_value` are updated to reference only
`make_rust_condition_value`.

The `error_names_sexp()` cached symbol (the 3-element names vector
`["error", "kind", "call"]`) becomes unused. Remove it from
`cached_class.rs` together with its initializer and the function definition.

### 3. Codemod the 30 call sites

Files and counts on `origin/main`:

| File | Sites |
|---|---|
| `miniextendr-macros/src/c_wrapper_builder.rs` | 17 |
| `miniextendr-macros/src/rust_conversion_builder.rs` | 7 |
| `miniextendr-macros/src/return_type_analysis.rs` | 4 |
| `miniextendr-api/src/unwind_protect.rs` | 2 |

Mechanical substitution at each site:

```rust
// Before (3 args)
::miniextendr_api::error_value::make_rust_error_value(msg, kind, call)

// After (4 args, class slot = None)
::miniextendr_api::error_value::make_rust_condition_value(msg, kind, None, call)
```

The proc-macro emit sites pass `quote!` token streams; the textual `None`
becomes `::core::option::Option::None` for hygiene safety in case the user
crate has a shadowed `None`. Verify by inspecting the emitted source via the
existing macro snapshot tests (`miniextendr-macros/tests/`).

### 4. PROTECT discipline

The 4-element builder already follows the protect-each-transient-through-to-store
pattern that PR #344's `af6b4875` fix established. No PROTECT changes are
required by this consolidation: every site that previously protected its
list, message, kind, and true-marker still does so. The only new code path
is the codemod-produced `None` for the `class` slot, which takes the
`SEXP::nil()` branch (no allocation, no extra protect needed).

Spot-check after consolidation:
- `make_rust_condition_value` still has `prot` rising to 4 on `Some(class)`
  paths and 3 on `None` paths, matching the original `make_rust_error_value`
  count.
- `Rf_unprotect(prot)` matches `prot` exactly at function exit on every
  branch.

### 5. Update doc comments in adjacent modules

- `miniextendr-api/src/cached_class.rs:condition_names_sexp` doc references
  `make_rust_condition_value` (already correct); remove the
  `error_names_sexp` doc-and-defn pair.
- `miniextendr-api/src/condition.rs` `RCondition` enum doc currently lists
  both builders; collapse to just `make_rust_condition_value`.
- `miniextendr-api/src/error_value.rs` module-level docstring loses the
  "Error value structure (legacy)" subsection.

## Test plan

- `just check` and `just clippy` (default + all-features per CLAUDE.md
  reproduce rules) green on the rebased branch.
- `just rcmdinstall && just devtools-test` green; the regenerated
  `R/miniextendr-wrappers.R` will not change because the R-side reader is
  unchanged.
- `just devtools-document` produces no diff in `NAMESPACE`, `man/`, or the
  wrappers R file.
- The condition matrix tests added in PR #385 (`test_condition_matrix.R`,
  `tests/cross-package/*` trait-ABI condition tests) all pass without
  modification. These exercise both legacy (`panic`, `result_err`,
  `none_err`) and macro-driven (`error!`, `warning!`, `message!`) paths;
  unification cannot hide regressions on either lane if both lanes still
  pass.
- **R-devel CI required**: PR #344 fixed a `recursive gc invocation`
  segfault on this exact hot path that R-release/oldrel did not catch.
  Tag the PR with the R-devel matrix label so the workflow runs before
  merge. R-release green is NOT proof of safety here.
- Snapshot review of one `c_wrapper_builder` macro expansion to confirm
  the `None` substitution emits hygenically (`::core::option::Option::None`
  rather than bare `None`).

## Risks and mitigations

- **R-devel GC regression**. Mitigated by R-devel CI gate above and by the
  fact that no PROTECT count changes; only the call-site argument list does.
- **Hidden caller of `make_rust_error_value` outside the workspace**.
  `git grep` on `origin/main` confirms 30 sites all inside this repo. The
  function is `pub` but the crate is not published; downstream R packages
  that vendor `miniextendr-api` will pick up the change on their next
  vendor refresh, which already happens at every framework upgrade.
- **`error_names_sexp` removal cascades**. `git grep error_names_sexp`
  before deleting; only the one defn site and the `make_rust_error_value`
  internal use.

## Out of scope

- Replacing `kind` with a typed enum on the Rust side (would touch the
  proc-macro emit shape; separate ticket).
- Reworking the R-side switch in `miniextendr_raise_condition` (no change
  needed; it already handles both shapes).
