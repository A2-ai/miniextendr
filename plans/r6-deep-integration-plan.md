# Plan: Deeper R6 Integration (Rust-Inferred, Ergonomic)

Goal: Generate full‑fidelity R6 classes from Rust types/methods with minimal annotations and no manual R glue, covering all documented R6 features: public/private members, active bindings, inheritance, portability, locking, cloning, and finalizers.

## 1) R6 features to cover (from background/R6-main)

- Public + private members
- Active bindings (computed properties)
- Inheritance (`inherit`)
- Portable vs non‑portable (`portable`)
- Class locking (`lock_class`) and object locking (`lock_objects`)
- Class attribute toggle (`class`)
- Cloneability (`cloneable`) + deep cloning (`deep_clone` hook)
- Finalizers (`finalize` method; private preferred)
- `$set()` post‑hoc member addition (optional, ergonomic)

## 2) Rust surface syntax (inferred, minimal annotations)

### 2.1 Impl‑level R6 options

```rust
#[miniextendr(r6(
  props = "annotated" | "pub" | "all", // default: "annotated"
  inherit = ParentType,                   // optional
  portable, non_portable,                 // flags (default portable)
  lock_objects, lock_class,               // flags
  class_attr, no_class_attr,              // flags
  cloneable, non_cloneable                // flags
))]
impl Type { ... }
```

### 2.2 Field‑level membership

```rust
#[r6(public)]
#[r6(private)]
#[r6(skip)]
#[r6(name = "x")]
```

Policies (same as S7):

- `annotated` default: only `#[r6(public)]` / `#[r6(private)]` fields are members.
- `pub`: all `pub` fields are `public` (unless `#[r6(private)]`/`#[r6(skip)]`).
- `all`: all fields are public by default (unless `#[r6(private)]`/`#[r6(skip)]`).

### 2.3 Methods → R6 members

```rust
#[r6(public)]
fn greet(&self) -> String { ... }

#[r6(private)]
fn compute(&self) -> i32 { ... }

#[r6(active)]
fn len(&self) -> i32 { ... }        // active binding getter

#[r6(active, prop = "len")]
fn set_len(&mut self, value: i32) { ... }   // active binding setter
```

Rules:

- Active binding can be getter‑only or getter+setter.
- Setter without getter is an error.

### 2.4 Lifecycle hooks

```rust
#[r6(initialize)]
fn initialize(&mut self, ...) { ... }

#[r6(finalize)]
fn finalize(&mut self) { ... }

#[r6(deep_clone)]
fn deep_clone(&self, name: &str, value: SEXP) -> SEXP { ... }
```

## 3) Codegen strategy

- Generate an R6Class with:
  - `public` list mapped from `#[r6(public)]` methods/fields
  - `private` list mapped from `#[r6(private)]`
  - `active` list from `#[r6(active)]`
  - `initialize` method (if present) calling Rust wrapper
  - `finalize` method (private by default)
- Store Rust state in `private$.ptr` (ExternalPtr) to preserve reference semantics.
- Generate `.Call` wrappers for each method/binding and route through `.ptr`.

### 3.1 Example: Rust input → generated R6

Rust:

```rust
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(r6)]
pub struct Counter {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    value: i32,
}

#[miniextendr(r6(props = "annotated", cloneable, lock_objects))]
impl Counter {
    #[r6(public)]
    pub fn new(initial: i32) -> ExternalPtr<Self> { ... }

    #[r6(public)]
    pub fn inc(&mut self) -> i32 { ... }

    #[r6(active)]
    pub fn count(&self) -> i32 { ... }

    #[r6(active, prop = "count")]
    pub fn set_count(&mut self, value: i32) { ... }

    #[r6(finalize)]
    fn finalize(&mut self) { ... }
}
```

Generated R (sketch):

```r
Counter <- R6::R6Class("Counter",
  public = list(
    initialize = function(initial, .ptr = NULL) {
      if (!is.null(.ptr)) private$.ptr <- .ptr
      else private$.ptr <- .Call(C_Counter__new, .call = match.call(), initial)
    },
    inc = function() .Call(C_Counter__inc, .call = match.call(), private$.ptr)
  ),
  private = list(
    .ptr = NULL,
    finalize = function() .Call(C_Counter__finalize, .call = match.call(), private$.ptr)
  ),
  active = list(
    count = function(value) {
      if (missing(value)) .Call(C_Counter__count, .call = match.call(), private$.ptr)
      else { .Call(C_Counter__set_count, .call = match.call(), private$.ptr, value); invisible(self) }
    }
  ),
  lock_objects = TRUE,
  cloneable = TRUE
)
```

## 4) Inheritance + portability

- `inherit = ParentType` maps to R6 `inherit = ParentType`.
- `portable` (default) uses standard `self$` / `private$` access.
- `non_portable` allows shorter access but warns about cross‑package inheritance.

## 5) Clone semantics

- `cloneable` default TRUE; can be disabled.
- If `#[r6(deep_clone)]` present, wire into R6 deep clone mechanism.

## 6) Finalizers

- `#[r6(finalize)]` generates a private `finalize` member; avoid public finalizers.
- Ensure `reg.finalizer(..., onexit = TRUE)` is used by R6.

## 7) Tests (rpkg/tests/testthat)

- Public/private access + method dispatch
- Active bindings: getter only and getter+setter
- Inheritance + super calls
- Portable vs non‑portable access behavior
- lock_class / lock_objects behavior
- cloneable + deep clone hook
- finalize invocation (use weakref + gc pressure; skip on CRAN if flaky)

## 8) Docs/examples

- Add Rust examples in `rpkg/src/rust/r6_tests.rs`
- Update R docs to show inferred R6 classes
