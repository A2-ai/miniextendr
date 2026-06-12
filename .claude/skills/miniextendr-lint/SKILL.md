---
name: miniextendr-lint
description: Use when a lint rule (MXL###) fires during cargo build or cargo check, when you want to understand what a specific rule checks and why it exists, when adding a new lint rule, or when diagnosing a build.rs static analysis failure. Also use when MINIEXTENDR_LINT=0 is being considered.
---

# miniextendr-lint

miniextendr-lint is a build-time static analysis tool that runs automatically
during `cargo build` and `cargo check`. It walks your crate's source tree,
evaluates `#[cfg(feature = "...")]` gates, and checks for correctness problems
that would otherwise produce confusing runtime errors, silent misbehavior, or
codegen failures. Every MXL rule has a specific safety or correctness reason;
violations are always bugs to fix, never to silence.

## When to use this skill

- "I'm getting an MXL### error — what does it mean?"
- "Why does the build say MXL300 when I call Rf_error?"
- "How do I add a new lint rule?"
- "How does the lint walk feature-gated modules?"
- "Can I turn the lint off temporarily?"
- "What is the relationship between the lint and miniextendr-macros?"

## Key concepts

### How the lint runs

miniextendr-lint is invoked from a downstream crate's `build.rs`, via an
integration hook in `miniextendr-api`. It does not run as a `cargo check`
plugin or proc-macro; it runs as a normal build script. The linting occurs at
`cargo build` / `cargo check` time, before compilation, by parsing source files
with the `syn` crate.

Disable temporarily with `MINIEXTENDR_LINT=0 cargo check`. This is an escape
hatch for exploration; violations must still be fixed before the code is
committed. The policy from `CLAUDE.md` is explicit: "Fix warnings you see: no
known issues."

### Module-tree walking and cfg evaluation

`miniextendr-lint/src/crate_index.rs` resolves `mod foo;` declarations into
file paths, following the same rules as `rustc`: a bare `mod foo;` maps to
`foo.rs` or `foo/mod.rs`. When a module is guarded with `#[cfg(feature = "...")]`,
the walker evaluates whether that feature is active for the current build before
deciding to visit the module. This means linting correctly skips dead code and
only flags live paths. Do not re-implement cfg evaluation in new rules; use the
utilities from `crate_index.rs`.

### Shared parser layer

The lint shares its parser layer with `miniextendr-macros`. Since the retirement
of `miniextendr-macros-core` as a separate crate, the shared parsing lives
inside `miniextendr-macros` and is consumed directly by `miniextendr-lint` as a
workspace dependency. New rules that need to parse `#[miniextendr]` attributes
should use the same parser types rather than duplicating parsing logic.

## MXL rule catalogue

### Trait and impl rules

**MXL008** — Trait-impl class-system compat with inherent impl.
An impl block decorated for trait-ABI class systems must be compatible with the
corresponding inherent impl. Triggers when the class-system configuration
conflicts with the method signatures or receiver types on the inherent impl.
Fix: align the trait-ABI configuration with the inherent impl structure.

**MXL009** — Multiple impl blocks without distinct labels.
When a type has more than one `#[miniextendr]` impl block, each must carry a
`label = "..."` attribute to disambiguate R class names. Without labels the
codegen produces duplicate identifiers.
Fix: add `#[miniextendr(label = "my_label")]` to each impl block.

**MXL010** — Duplicate labels on impl blocks.
Two impl blocks on the same type share the same `label = "..."` value.
Fix: assign unique labels to each block.

### Export and visibility rules

**MXL106** — Non-`pub` function that would receive `@export`.
A function is marked `#[miniextendr]` but is not `pub`. Without `pub`, the
function is not part of the crate's public API; exporting it from R would
create a dangling entry point.
Fix: add `pub`, or add `#[miniextendr(noexport)]` if the function is
intentionally internal.

**MXL203** — Redundant `internal` + `noexport`.
Both `internal` and `noexport` appear on the same item. These are not additive;
`noexport` alone is sufficient to suppress R-side export.
Fix: remove the redundant attribute.

### Parameter rules

**MXL110** — Parameter name is an R reserved word.
An argument name to a `#[miniextendr]` function collides with an R keyword or
built-in (`if`, `else`, `for`, `function`, `NULL`, `NA`, etc.). The generated
R wrapper would be syntactically invalid.
Fix: rename the parameter.

### Codegen-compatibility rules

**MXL111** — `s4_*` method name on an `#[miniextendr(s4)]` impl.
The S4 class codegen auto-prefixes generated method names with `s4_`. Writing
`s4_my_method` in Rust produces `s4_s4_my_method` in R.
Fix: drop the `s4_` prefix from the Rust method name; the codegen adds it.

**MXL112** — Explicit lifetime parameter on a `#[miniextendr]` fn or impl block.
An `extern "C-unwind" #[no_mangle]` function is incompatible with any generic
parameter, including lifetime parameters. The codegen rejects lifetime params
at macro-expansion time; the lint catches them before expansion.
Fix: replace `&str` with `String`, `&[T]` with `Vec<T>`, etc. Lifetime elision
on `&str` / `&[T]` arguments works fine — the lint only fires on explicit
`<'a>` annotations.

**MXL120** — Invalid vctrs constructor or receiver.
A vctrs constructor returns `Self` or a named type (must return `SEXP`), or a
vctrs impl block includes an instance-method receiver (`&self`, `self: &ExternalPtr<Self>`, etc.).
Mirrors the hard error in `miniextendr-macros`.
Fix: change the return type to `SEXP` and ensure no instance methods are on the
vctrs impl.

### FFI safety rules (most-commonly-tripped)

**MXL300** — Direct `Rf_error` or `Rf_errorcall` call.
Calling the R error longjmp functions directly bypasses the
`with_r_unwind_protect` transport that runs Rust destructors before jumping.
The tagged-SEXP payload (Box allocated by `make_rust_condition_value`) is
leaked on the R longjmp path when `Rf_error` is called inside
`with_r_unwind_protect` — approximately 8 bytes per error invocation.
The framework converts `panic!()` to R errors correctly.
Fix: replace `Rf_error(...)` / `Rf_errorcall(...)` with `panic!(...)`.

**MXL301** — `_unchecked` FFI call outside a known-safe context.
The `#[r_ffi_checked]` attribute generates both checked and `*_unchecked`
variants of each R API entry point. The checked variant asserts that the caller
is on the R main thread. The `*_unchecked` variant skips this assertion.
Calling `*_unchecked` outside a context where the main-thread property is
already established is a logic error.
Safe contexts where `*_unchecked` is valid: ALTREP callbacks, inside
`with_r_unwind_protect`, inside `with_r_thread` blocks.
Fix: either move the call into a safe context, or switch to the checked variant.

## How it works

### Adding a new rule

1. Create a new file under `miniextendr-lint/src/rules/` (e.g., `my_rule.rs`).
2. Register the rule in `miniextendr-lint/src/rules.rs`.
3. Add the `MXL###` code to `miniextendr-lint/src/lint_code.rs`.
4. Use helpers from `miniextendr-lint/src/helpers.rs` for common AST predicates.
5. Do not re-implement cfg evaluation — call into `crate_index.rs` utilities.

## Decision trees

### My lint is firing — silence or fix?

Always fix. The project policy (CLAUDE.md) is "no known issues." Every MXL
rule prevents a real safety or correctness defect. `MINIEXTENDR_LINT=0` is an
exploration escape hatch only; committed code must be clean.

### Which context makes `*_unchecked` calls safe?

- Inside an ALTREP callback registered via the ALTREP bridge? Safe.
- Inside `with_r_unwind_protect { ... }`? Safe.
- Inside `with_r_thread { ... }`? Safe.
- Anywhere else, including global init, background threads, or standalone
  functions? Unsafe — use the checked variant.

## Key files

- `miniextendr-lint/src/lib.rs` — entrypoint and module-tree walker
- `miniextendr-lint/src/crate_index.rs` — `mod` resolution + cfg evaluation
- `miniextendr-lint/src/rules.rs` — rule dispatcher
- `miniextendr-lint/src/rules/rf_error.rs` — MXL300
- `miniextendr-lint/src/rules/ffi_unchecked.rs` — MXL301
- `miniextendr-lint/src/rules/lifetime_param.rs` — MXL112
- `miniextendr-lint/src/rules/impl_validation.rs` — MXL008 / MXL009 / MXL010 / MXL111
- `miniextendr-lint/src/rules/fn_visibility.rs` — MXL106
- `miniextendr-lint/src/lint_code.rs` — MXL code registry
- `miniextendr-lint/src/helpers.rs` — shared AST predicates
- `miniextendr-lint/CLAUDE.md` — authoritative rule catalogue

## Common pitfalls

- **MXL300 fires on `_unchecked` FFI, not just user code.** The lint visits all
  reachable modules, including internal FFI shims. If you see MXL300 on a path
  you didn't write, it is still a bug — the call should go through `panic!()`.

- **MXL112 does not fire on lifetime elision.** `fn foo(x: &str)` is fine —
  the macro infers the conversion internally. Only explicit `<'a>` annotations
  trigger MXL112.

- **Cfg-gated modules are only linted when the feature is active.** If a rule
  does not fire during normal `cargo check` but fires in CI with a large feature
  set, the affected code is in a feature-gated module that was not active
  locally. Reproduce with the CI feature list from `.github/workflows/ci.yml`.

- **`miniextendr-macros-core` is retired.** The issue tracker (#168) references
  it; that crate no longer exists. Shared parsing now lives inside
  `miniextendr-macros` and is consumed directly.

## Related skills

- `miniextendr-macros` — the proc-macro side of attribute parsing and codegen
  that the lint validates against.
- `miniextendr-ffi` — `#[r_ffi_checked]`, `_unchecked` variants, and the
  threading model that MXL301 enforces.
- `miniextendr-build` — how `build.rs` integration runs the lint at
  `cargo build` / `cargo check` time.
