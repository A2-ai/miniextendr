# Plan: Lifecycle Support (R package lifecycle integration)

Goal: Add first‑class lifecycle support across all miniextendr wrapper generators, including automatic handling of Rust `#[deprecated]` attributes, lifecycle badges in roxygen output, runtime deprecation/signalling calls in generated R wrappers, and project‑level dependency management (Imports: lifecycle).

## 1) Lifecycle features to cover (from background/lifecycle)

Primary stages + signals:
- **Stages:** experimental, stable, superseded, deprecated (plus legacy: soft-deprecated, defunct, questioning, maturing, retired).
- **Signals:**
  - `lifecycle::deprecate_soft()`
  - `lifecycle::deprecate_warn()`
  - `lifecycle::deprecate_stop()` (defunct)
  - `lifecycle::signal_stage()` for experimental/superseded
- **Docs:** `lifecycle::badge()` for stage badges in roxygen.
- **Arguments:** `lifecycle::deprecated()` sentinel + `lifecycle::is_present()` checks for argument deprecation.
- **Testing + diagnostics:** `expect_deprecated()`, `expect_defunct()`, `last_lifecycle_warnings()`.

## 2) User‑facing Rust syntax (new attributes + existing Rust `#[deprecated]`)

### 2.1 Function‑level lifecycle attributes
Add explicit lifecycle metadata via the existing macro attribute namespace:

```rust
#[miniextendr(lifecycle(stage = "deprecated", when = "0.4.0", with = "new_fn()"))]
pub fn old_fn(x: i32) -> i32 { ... }

#[miniextendr(lifecycle(stage = "experimental"))]
pub fn new_fn(x: i32) -> i32 { ... }
```

### 2.2 Rust `#[deprecated]` extraction (required)
Automatically glean Rust deprecation attributes on `#[miniextendr]` functions:

```rust
#[deprecated(note = "Use new_fn()")]
#[miniextendr]
pub fn old_fn(x: i32) -> i32 { ... }
```

Parsing rules:
- Accept `#[deprecated]`, `#[deprecated(note = "...")]`, `#[deprecated(since = "...")]`, and `#[deprecated = "..."]`.
- Map to lifecycle stage `deprecated` unless overridden by explicit `#[miniextendr(lifecycle(...))]`.

### 2.3 Method‑level lifecycle attributes (impl blocks)
Same attribute interface on methods for all class systems:

```rust
#[miniextendr(r6)]
impl Counter {
  #[miniextendr(lifecycle(stage = "deprecated", when = "1.0.0", with = "total()"))]
  pub fn sum(&self) -> i32 { ... }
}
```

### 2.4 Argument‑level deprecation
Support parameter deprecation for standalone functions and impl methods.

Standalone function syntax (parameter‑level attribute):
```rust
#[miniextendr]
pub fn foo(
  #[miniextendr(deprecated(when = "1.0.0", with = "foo(new_arg)"))]
  old_arg: i32,
  new_arg: i32,
) -> i32 { ... }
```

Impl method syntax (method‑level block):
```rust
#[miniextendr(r6, deprecated_args(old_arg = "1.0.0"))]
fn foo(&self, old_arg: i32, new_arg: i32) -> i32 { ... }
```

(Exact surface syntax can be refined; the key is parity between function and method support.)

## 3) Internal metadata model

Create a shared lifecycle metadata struct used by all wrapper generators:

```rust
struct LifecycleSpec {
  stage: Stage,                // experimental|stable|superseded|deprecated|soft|defunct|...
  when: Option<String>,         // version string
  what: Option<String>,         // "foo()" / "foo(arg)"
  with: Option<String>,         // replacement
  details: Option<String>,      // freeform message
  id: Option<String>,           // lifecycle deprecation id
  for_args: HashMap<String, ArgLifecycleSpec>, // argument deprecations
}
```

Rules:
- Explicit `#[miniextendr(lifecycle(...))]` overrides `#[deprecated]` inference.
- `#[deprecated]` note maps to `details` by default.
- `when` precedence: explicit lifecycle `when` > `#[deprecated(since = ...)]` > package version fallback.
- `what` auto‑inferred from wrapper name if not provided.

## 4) R wrapper generation changes (all miniextendr entry points)

### 4.1 Free functions (`miniextendr-macros/src/lib.rs`)
- Insert a lifecycle prelude at the top of the wrapper body before the `.Call()`.
- If stage is deprecated/soft/defunct: emit `lifecycle::deprecate_*()`.
- If stage is experimental/superseded: emit `lifecycle::signal_stage()`.
- For argument deprecations: add `if (lifecycle::is_present(arg)) ...` blocks.

### 4.2 Class systems (`miniextendr-macros/src/miniextendr_impl.rs`)
- Apply the same lifecycle prelude for:
  - Env methods (`Type$method()`)
  - R6 methods (`Class$method()` / `Class$new()`)
  - S3 methods (`generic.class()`)
  - S4 methods (generic function name in `what`)
  - S7 methods
  - vctrs wrappers
- If an entire class is marked lifecycle‑deprecated, inject the signal into the constructor and/or `new_*` function.

### 4.3 Trait impl wrappers (`miniextendr-macros/src/miniextendr_impl_trait.rs`)
- Add lifecycle prelude in each generated wrapper function.
- Use wrapper names in `what` (e.g., `Type$Trait$method()`).

### 4.4 ExternalPtr sidecar wrappers (`miniextendr-macros/src/externalptr_derive.rs`)
- Allow optional lifecycle annotations on `#[r_data]` fields to deprecate accessors.
- Emit lifecycle prelude in generated getter/setter wrappers when marked.

### 4.5 Helper for constructing `what`
Standardize `what` formatting:
- Free functions: `fn_name()`.
- S3 methods: `generic.class()`.
- Env + R6 methods: `Class$method()` or `Class$new()`.
- S4/S7: `generic()` (or explicit override if provided).
- Argument deprecations: `fn_name(arg)` or `Class$method(arg)`.

## 5) Roxygen integration (badges + tags)

- Auto‑inject `lifecycle::badge(stage)` into `@description` when stage is set, unless the user already provided a badge.
- For argument deprecations, append badge to the relevant `@param` line.
- Add `@keywords internal` automatically for deprecated/defunct stages (unless user opts out).
- Preserve existing roxygen tags and respect `@noRd`/`@keywords internal`.

Implementation touchpoints:
- `miniextendr-macros/src/roxygen.rs` (tag detection + helpers)
- `miniextendr-macros/src/r_wrapper_builder.rs` (custom tag injection)
- `miniextendr-macros/src/r_class_formatter.rs` (method doc builder hook)

## 6) Dependency management (Imports: lifecycle)

Required behavior: enabling lifecycle support must add `lifecycle` to the R package Imports.

Planned changes:
- Add `use_lifecycle()` helper in `minirextendr/R/use-feature.R`:
  - Calls `add_import("lifecycle")`.
  - Optionally calls `usethis::use_lifecycle()` to copy badge images.
- Update templates + examples:
  - `rpkg/DESCRIPTION`: include `lifecycle` in Imports.
  - `minirextendr/inst/templates/...` (rpkg + monorepo) to add lifecycle when lifecycle features are enabled.
- Add guardrails:
  - If lifecycle codegen is used but DESCRIPTION lacks lifecycle, emit a warning during `miniextendr_check()` (minirextendr status checks).

## 7) Tests

### 7.1 Macro unit tests
- `miniextendr-macros/src/tests.rs`:
  - Verify `#[deprecated]` produces lifecycle prelude in wrapper string.
  - Verify explicit lifecycle overrides `#[deprecated]`.
  - Verify `@description` includes badge and avoids duplicates.

### 7.2 R package integration tests
- Add testthat cases in `rpkg/tests/testthat/`:
  - `expect_deprecated()` for deprecated wrappers.
  - `expect_defunct()` for defunct wrappers.
  - Argument deprecation via `lifecycle::is_present()`.
  - `last_lifecycle_warnings()` captures calls.

## 8) Documentation updates

- `miniextendr-api/README.md`: add lifecycle usage examples.
- `docs/` (or new doc): describe lifecycle attributes and how they map to lifecycle functions.
- `minirextendr` docs: document `use_lifecycle()` and dependency implications.

## 9) Rollout sequence (phased)

1) **Core parsing + wrapper prelude**
   - Parse `#[deprecated]` in `miniextendr_fn` + method parsing.
   - Implement lifecycle prelude injection for free functions and class methods.

2) **Roxygen + Imports**
   - Add badge/tag injection.
   - Introduce `use_lifecycle()` and update DESCRIPTION templates/examples.

3) **Argument deprecation + extended generators**
   - Add argument‑level deprecation support.
   - Extend to trait wrappers + externalptr sidecars + vctrs.

4) **Tests + docs**
   - Add macro tests and R integration tests.
   - Update docs and README with examples and caveats.

## 10) Open questions / decisions to settle early

- **`when` default:** use `#[deprecated(since = ...)]`, package version, or require explicit `when`?
- **Mapping for `#[deprecated = "..."]`:** treat as `details` vs attempt to parse `with`.
- **Defunct stage UX:** default to `deprecate_stop()` plus `badge("defunct")` or `badge("deprecated")`?
- **Auto‑`@keywords internal`:** should this be default for deprecated/superseded or opt‑in?

