# Plan: S7 Computed/Dynamic Properties via Rust Inference

Goal: Support S7 computed properties (and other documented S7 property features) by **inferring everything from Rust syntax + features**, with no manual edits to generated R wrappers.

## 1) Map S7 features to Rust surface syntax

S7 property features to cover (from background/S7-main):

- Type
- Validator
- Default
- Computed (getter)
- Dynamic (getter + setter)
- Required (default = quoted error) / or validator
- Frozen (custom setter)
- Deprecated (getter + setter warnings)
- Union types
- Class validator, constructor, parent/abstract/package

Rust-driven inference targets:

- Property type → derived from Rust field type
- Default → from field initializer or attribute
- Getter/Setter → from explicit Rust methods/traits or annotations
- Validator → from attribute or Rust method returning Result/Option
- Union types → from Rust enums / Option / Result / custom union marker

## 2) Concrete Rust syntax (maximal ergonomics, minimal annotations)

Design goal: **no R snippets in user code**, no `pub` leakage, and property behavior derived from Rust items.

### 2.1 Property fields (struct-level)

For a type with `#[miniextendr(s7)] impl Type { ... }`, properties are inferred from Rust fields with an **impl-level policy**:

```
#[miniextendr(s7(props = "annotated"))] // default
// alternatives: "pub", "all"
```

Policies:
- `annotated` (default): only fields with `#[s7(prop)]` become properties.
- `pub`: all `pub` fields become properties (except `#[s7(skip)]`).
- `all`: all fields become properties (except `#[s7(skip)]`).

This avoids forcing `pub` for property exposure while allowing opt-in ergonomics.

Field attributes:

```
#[s7(prop)]                 // include this field as a property
#[s7(skip)]                 // exclude field from properties
#[s7(name = "len")]         // rename property
#[s7(required)]             // constructor must supply (enforced in Rust)
#[s7(frozen)]               // read-only after init (setter errors once non-empty)
#[s7(deprecated = "msg")]   // getter/setter warn with msg
#[s7(union)]                // use union type inference for enums/Option
```

Defaults + validators from Rust functions (no R strings):

```
#[s7(default, prop = "len")]      // uses fn returning T
#[s7(validate, prop = "len")]     // uses fn(value: T) -> Result<(), String>
```

### 2.1.1 Accessors for field-backed properties (required for ergonomics)
For every inferred property backed by a Rust field, generate getters/setters and wire them into
`new_property(getter=..., setter=...)`. This makes `@` work without manual R code.
This applies to both normal S7 classes and `#[externalptr(s7)]` types.

### 2.2 Computed + dynamic properties (method-level)

Computed properties are defined by methods; macro emits `.Call` wrappers for getters/setters and wires `new_property(getter = ..., setter = ...)`.

Getter method:

```
#[s7(getter)]
fn length(&self) -> f64 { self.end - self.start }
// property name inferred: "length"
```

Setter method:

```
#[s7(setter)]
fn set_length(&mut self, value: f64) { self.end = self.start + value; }
// property name inferred by stripping `set_` prefix unless overridden
```

Override property name:

```
#[s7(getter, prop = "len")]
fn length(&self) -> f64 { ... }

#[s7(setter, prop = "len")]
fn set_length(&mut self, value: f64) { ... }
```

Validator/default providers (method-level):

```
#[s7(default, prop = "len")]
fn default_len() -> f64 { 0.0 }

#[s7(validate, prop = "len")]
fn validate_len(value: f64) -> Result<(), String> {
  if value < 0.0 { Err("must be >= 0".into()) } else { Ok(()) }
}
```

### 2.3 Union types (inferred from Rust)

Rules:

- `Option<T>` → `NULL | class_T` (S7 union).
- `enum` with single-field tuple variants → union of each payload class if `#[s7(union)]` is present.
- `enum` without `#[s7(union)]` or without a clear payload mapping → compile error with a hint.
- Union inference is **opt-in** for enums to avoid surprising behavior.

### 2.4 Class-level options (Rust-sourced)

Implement via methods/attributes rather than R strings:

```
#[miniextendr(s7)]
impl Range {
  #[s7(class_validator)]
  fn validate(&self) -> Result<(), String> { ... }

  #[s7(constructor)]
  fn new_from(x: Vec<f64>) -> ExternalPtr<Self> { ... }
}
```

Optional Rust attributes on the impl block:

```
#[miniextendr(s7(parent = ParentType, abstract))]
```

- `parent` references another Rust type with an S7 class.
- `abstract` is a boolean flag.

Class validator rules:
- If `#[s7(class_validator)]` returns `Result<(), String>`, map Err to S7 validation errors.
- If it returns `Option<String>`, treat `Some(msg)` as invalid.

Constructor rules:
- Must be an inherent `fn` returning `ExternalPtr<Self>` (or `Self` if no externalptr).
- If not present, default constructor uses generated fields/defaults.

### 2.5 Feature gates (Rust features)

Ergonomics-first: `s7` enables the full property surface by default.
Sub-features are only for opt-out builds:
- `s7` (full)
- optional: `s7-minimal` to disable computed/dynamic/validators/unions
Macro emits compile-time errors only when a feature is explicitly disabled.

## 3) Extend macro parsing and codegen

Targets: `miniextendr-macros/src/miniextendr_impl.rs`, `miniextendr-macros/src/externalptr_derive.rs`

- Parse new S7 property metadata from:
  - impl block attributes (`#[miniextendr(s7(...))]`)
  - struct field attributes (`#[s7(...)]`)
  - method attributes for getters/setters
- Build a property graph:
  - field name, inferred S7 class type, and behavior flags
  - map Rust getters/setters to S7 properties
  - generate accessor wrappers for field-backed properties
- Generate `new_class()` with `properties = list(...)`:
  - Always include `.ptr` property
  - Add inferred properties with `new_property(class = ..., default = ..., validator = ..., getter = ..., setter = ...)`
- For `#[externalptr(s7)]` sidecars:
  - auto-wrap field accessors as `new_property(getter/setter = ...)`
  - maintain current standalone accessor functions for direct use

## 4) Rust feature gates → S7 property features

- Default: `s7` enables the full property surface for best ergonomics.
- Optional: `s7-minimal` to remove computed/dynamic/validators/unions.
- Macro errors only when a user opts out explicitly.

## 5) Tests (rpkg/tests/testthat)

Add tests covering **all documented S7 features**:

- Type enforcement + validator errors
- Defaults from Rust initializer or attribute
- Computed property (getter only)
- Dynamic property (getter + setter)
- Required property (constructor errors)
- Frozen property (setter error after init)
- Deprecated property (warning on get/set)
- Union property (enum/Option) enforcement
- Class validator and constructor inference

## 6) Docs + Examples

- Update Rust docs and generated R docs to show inferred S7 properties.
- Add examples in `rpkg/src/rust/s7_tests.rs` demonstrating computed/dynamic properties.
- Update minirextendr docs to explain Rust syntax and feature flags.

## 7) Rollout steps

- Implement macro parsing changes behind feature flags.
- Add tests (guarded by `s7` feature).
- Validate by regenerating wrappers and running rpkg tests.
