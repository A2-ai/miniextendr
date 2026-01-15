# Plan: Comprehensive S7 Integration via Rust Inference

Goal: cover the full S7 surface area documented in `background/S7-main` while keeping ergonomics Rust-first (no manual R edits; behavior inferred from Rust syntax + features).

Sources covered: README + vignettes (S7, classes-objects, generics-methods, compatibility, packages) + man pages (`new_class`, `new_property`, `new_generic`, `method`, `method<-`, `method_explain`, `new_union`, `convert`, `super`, `S7_data`, `prop`, `props`, `prop_names`, `validate`, `S7_class`, `S7_inherits`, `as_class`, `new_S3_class`, `class_any`, `class_missing`, `methods_register`, `new_external_generic`, `S4_register`, base/base_s3 classes).

## 0) Coverage matrix (S7 feature -> Rust-driven support)

Classes and objects:
- `new_class(name, parent, package, properties, abstract, constructor, validator)`
- `new_object()`
- `S7_object`, `S7_class()`, `S7_inherits()` / `check_is_S7()`
- `as_class()` standardization

Properties:
- `new_property(class, getter, setter, validator, default, name)`
- Computed (getter only) / dynamic (getter + setter)
- Defaults (including quoted calls)
- Property validators
- Common patterns: required, frozen, deprecated
- `prop()` / `@`, `props()` / `props<-` / `set_props()`, `prop_names()` / `prop_exists()`
- `validate()`, `valid_eventually()`, `valid_implicitly()`

Types and unions:
- Base classes `class_*` + unions `class_numeric`, `class_atomic`, `class_vector`, `class_language`
- Base S3 classes `class_factor`, `class_Date`, `class_POSIXct`, `class_POSIXlt`, `class_POSIXt`, `class_data.frame`, `class_formula`
- `new_S3_class()` for custom S3 classes
- `new_union()` / `|`
- Special dispatch classes `class_any`, `class_missing`

Generics and methods:
- `new_generic()` + `S7_dispatch()`
- `method<-`, `method()`, `method_explain()`
- Generic-method compatibility (dots, required/optional args)
- Custom generic bodies (pre-dispatch checks)
- Multiple dispatch
- `super()`
- `convert()` (double dispatch; no inheritance on `to`)

Compatibility and packaging:
- S3/S4 method registration and inheritance rules
- S4 union conversion
- `new_external_generic()` + `methods_register()`
- `S4_register()` for S4 generics
- `S7_data()` / `S7_data<-` for base-type parents
- Package guidance: export constructors, set `package`, R < 4.3 `@` via `prop()` or `@rawNamespace` import

## 1) Rust surface syntax and inference rules (max ergonomics)

Design principles:
- No R snippets in user code by default.
- Infer as much as possible from Rust types, traits, and attributes.
- Allow explicit opt-in attributes only where inference is ambiguous.

### 1.1 Class definition and inheritance

Impl-level attribute drives class metadata:

```
#[miniextendr(s7)]
#[miniextendr(s7(parent = ParentType, abstract, package = "mypkg"))]
impl MyType { ... }
```

Inference rules:
- `parent` defaults to `S7_object` unless specified.
- `package` inferred from crate if exporting; overrideable for cross-package class names.
- `abstract` blocks constructor generation and R instantiation.
- If `parent` is a base class (e.g., `class_double`) or an S3 class wrapper, generate appropriate `new_object()` + `S7_data()` behavior.

### 1.2 Property inference (fields and methods)

Field-level policy:

```
#[miniextendr(s7(props = "annotated"))] // default
// alternatives: "pub", "all"
```

Field attributes:
```
#[s7(prop)]
#[s7(skip)]
#[s7(name = "len")]
#[s7(required)]
#[s7(frozen)]
#[s7(deprecated = "msg")]
#[s7(union)]
```

Type -> S7 class mapping (no R strings):
- Rust scalars (`i32`, `f64`, `bool`, `u8`) -> `class_integer`, `class_double`, `class_logical`, `class_raw`
- `String`, `&str` -> `class_character`
- `Vec<T>` -> `class_list` unless `T` maps to an atomic type -> `class_vector` + class-specific mapping
- `Option<T>` -> `NULL | class_T`
- `Robj` / `SEXP` -> `class_any` (unless annotated)
- `RDate`, `RFactor`, etc. -> `class_Date`, `class_factor`, etc.
- `#[s7(s3 = "foo")]
  field: Robj` -> `new_S3_class("foo")`

Property behaviors:
- Getter-only methods -> computed property
- Getter + setter -> dynamic property
- Field-backed properties generate getter/setter wrappers automatically

Validators and defaults:
```
#[s7(default, prop = "len")]
fn default_len() -> f64 { 0.0 }

#[s7(validate, prop = "len")]
fn validate_len(value: f64) -> Result<(), String> { ... }
```

Required/frozen/deprecated patterns:
- `#[s7(required)]` -> `default = quote(stop("@name is required"))`
- `#[s7(frozen)]` -> setter errors after non-empty value
- `#[s7(deprecated = "msg")]` -> getter/setter warn

### 1.3 Validation controls

Expose Rust-friendly helpers that map to:
- `validate()` (full validation)
- `valid_eventually()` (batch update without intermediate validation)
- `valid_implicitly()` (unsafe fast path)

Plan: generate R helpers that call `.Call` into Rust for bulk updates, then use S7 `validate()` once.

### 1.4 Constructors and `new_object()`

- Default constructor: derived from properties, excluding dynamic properties.
- Custom constructor via:

```
#[s7(constructor)]
fn new_from(x: Vec<f64>) -> ExternalPtr<Self> { ... }
```

- Generated R constructor always uses `new_object()` with the chosen parent.
- For base-parent classes, set `S7_data()` from Rust-returned base value.

### 1.5 Introspection hooks

Ensure R wrappers import/export S7 helpers so users can:
- `S7_class()`, `S7_inherits()`, `check_is_S7()`
- `prop_names()`, `prop_exists()`, `method()`, `method_explain()`

No extra codegen beyond correct `new_class()` + `new_property()` wiring.

## 2) Generics and methods (full S7 dispatch coverage)

### 2.1 Generic creation from Rust methods

Current behavior: instance methods -> `new_generic()` + `method()`.
Extend with Rust-inferred dispatch rules:
- Default: single dispatch on `x` with `...` included
- `#[s7(no_dots)]` removes `...` (for strict generics like `length()`)
- `#[s7(dispatch = "x,y")]` enables multiple dispatch
- `#[s7(required_args = "y,z")]` enforces required non-dispatch args
- `#[s7(optional_arg(name = "na.rm", default = true))]` adds optional args from Rust literals

### 2.2 Special dispatch classes

- `#[s7(fallback)]` -> register method for `class_any`
- `#[s7(dispatch_missing = "y")]` -> method for `class_missing` in multi-dispatch

### 2.3 External generics + S4 support

- `#[miniextendr(s7(generic = "pkg::name"))]` already maps to `new_external_generic()`
- Auto-insert `methods_register()` guidance in package docs
- If a generic is S4, auto-call `S4_register(Class)` before `method<-`

### 2.4 `super()` and `convert()` ergonomics

- Provide Rust-friendly sugar for `super()` calls in generated R methods (no manual edits)
- Auto-generate `convert()` methods from Rust `From`/`TryFrom` impls:
  - `impl From<Child> for Parent` -> `method(convert, list(Child, Parent))`
  - `impl TryFrom<Parent> for Child` -> upcast (requires property defaults)
- Respect S7 rule: no inheritance on `to` dispatch.

## 3) Codegen and macro changes

Targets: `miniextendr-macros/src/miniextendr_impl.rs`, `miniextendr-macros/src/externalptr_derive.rs`.

- Parse new S7 metadata from:
  - impl block attributes (`#[miniextendr(s7(...))]`)
  - struct field attributes (`#[s7(...)]`)
  - method attributes (`#[s7(getter|setter|default|validate|constructor|no_dots|dispatch|fallback|...)]`)

- Build a unified S7 class model:
  - properties (name, class spec, default, validator, getter, setter)
  - class metadata (parent, abstract, package)
  - generic metadata (dispatch args, dots, required/optional args)
  - conversions (From/TryFrom mapping)

- Generate `new_class()` with:
  - `.ptr` plus inferred properties
  - `properties = list(...)` using `new_property()`
  - `validator`, `abstract`, `parent`, `package`
  - constructor using `new_object()` and `S7_data()` where relevant

- For `#[externalptr(s7)]` sidecars:
  - auto-wire field accessors into `new_property(getter/setter)`
  - keep standalone accessors for direct use

## 4) Concrete examples (Rust -> generated R)

### 4.1 Computed property (getter only)

```rust
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(s7)]
pub struct Range {
    #[r_data] _r: RSidecar,
    #[r_data] start: f64,
    #[r_data] end: f64,
}

#[miniextendr(s7(props = "annotated"))]
impl Range {
    #[s7(prop)]
    pub fn start(&self) -> f64 { self.start }

    #[s7(prop)]
    pub fn end(&self) -> f64 { self.end }

    #[s7(getter)]
    pub fn length(&self) -> f64 { self.end - self.start }
}
```

```r
Range <- S7::new_class("Range",
  properties = list(
    .ptr = S7::class_any,
    start = new_property(getter = function(self) Range_get_start(self@.ptr),
                         setter = function(self, value) { Range_set_start(self@.ptr, value); self }),
    end   = new_property(getter = function(self) Range_get_end(self@.ptr),
                         setter = function(self, value) { Range_set_end(self@.ptr, value); self }),
    length = new_property(getter = function(self) Range_length(self@.ptr))
  )
)
```

### 4.2 Dynamic property (getter + setter)

```rust
#[miniextendr(s7)]
impl Range {
    #[s7(getter, prop = "length")]
    fn length(&self) -> f64 { self.end - self.start }

    #[s7(setter, prop = "length")]
    fn set_length(&mut self, value: f64) { self.end = self.start + value; }
}
```

```r
length = new_property(
  getter = function(self) Range_length(self@.ptr),
  setter = function(self, value) { Range_set_length(self@.ptr, value); self }
)
```

### 4.3 Multiple dispatch generic

```rust
#[miniextendr(s7)]
impl Dog {
    #[s7(dispatch = "x,y")]
    fn speak(&self, lang: Language) -> String { ... }
}
```

```r
speak <- S7::new_generic("speak", c("x", "y"), function(x, y, ...) S7::S7_dispatch())
S7::method(speak, list(Dog, Language)) <- function(x, y, ...) { ... }
```

### 4.4 Convert via Rust From/TryFrom

```rust
impl From<Point2D> for Point3D { ... }
impl TryFrom<Point3D> for Point2D { ... }
```

```r
S7::method(convert, list(Point2D, Point3D)) <- function(from, to, ...) { ... }
S7::method(convert, list(Point3D, Point2D)) <- function(from, to, ...) { ... }
```

## 5) Tests (rpkg/tests/testthat)

Add coverage for all documented S7 features:

Classes/objects:
- `new_class` args: `parent`, `abstract`, `package`, `constructor`, `validator`
- `S7_object`, `S7_class`, `S7_inherits` / `check_is_S7`
- Base classes + base S3 classes mapping

Properties:
- Type enforcement, `prop()` / `@` access
- `props()` / `props<-` / `set_props()` batch updates
- `prop_names()` / `prop_exists()`
- Defaults (literal + quoted)
- Property validators
- Computed + dynamic properties
- Required / frozen / deprecated patterns
- `validate()`, `valid_eventually()`, `valid_implicitly()`

Generics/methods:
- `new_generic` + `method<-` generation
- no-dots generics + required/optional args
- multiple dispatch
- `class_any` fallback and `class_missing` dispatch
- `method()` and `method_explain()` introspection
- `super()` usage in methods

Compatibility and packaging:
- `new_S3_class` in property types + S3 generic method registration
- S4 generic method registration + `S4_register()`
- `new_external_generic()` + `methods_register()` integration
- `convert()` upcast/downcast + custom convert
- `S7_data()` for base-parent classes

## 6) Docs and guidance

- Update miniextendr docs to list Rust attributes and inferred mappings.
- Add a compatibility note for R < 4.3: use `prop()` or `@rawNamespace` import.
- Show `methods_register()` in `.onLoad()` in package templates.
- Document the `From`/`TryFrom` -> `convert()` mapping.

## 7) Rollout steps

- Phase 1: property inference + accessor wiring (field-backed + computed/dynamic)
- Phase 2: validation/defaults/required/frozen/deprecated patterns
- Phase 3: generics upgrades (multi-dispatch, no-dots, optional/required args)
- Phase 4: convert/from + S3/S4 interoperability helpers
- Phase 5: docs/tests + stabilization
