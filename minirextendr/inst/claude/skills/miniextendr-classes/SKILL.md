---
name: miniextendr-classes
description: Use when exposing a Rust struct as an R class in a miniextendr package — choosing between R6, S3, S4, S7, Env, or vctrs; writing #[miniextendr] impl blocks; constructors, methods, properties/active bindings; trait methods callable from R; or debugging class dispatch ("could not find method", vctrs errors, S7 parent errors).
---

# Exposing Rust structs as R classes

Annotate an `impl` block with `#[miniextendr(...)]` and the framework
generates the complete R class — constructor, methods, documentation — backed
by your Rust struct held as an external pointer. You write Rust; the R side
is generated into `R/<pkg>-wrappers.R` on every build.

## Choosing a class system

| System | Attribute | R usage | Reach for it when |
|---|---|---|---|
| **Env** | `#[miniextendr]` | `Type$new(...)`, `obj$method()` | simplest wrapper, zero R dependencies |
| **R6** | `#[miniextendr(r6)]` | `Type$new(...)`, `obj$method()` | reference semantics, active bindings, private methods, chaining |
| **S3** | `#[miniextendr(s3)]` | `new_type(...)`, `generic(obj)` | idiomatic R generics, broad ecosystem interop |
| **S4** | `#[miniextendr(s4)]` | `Type(...)`, `setMethod` dispatch | formal/multiple dispatch, S4-heavy codebases |
| **S7** | `#[miniextendr(s7)]` | `Type(...)`, `method(obj)` | modern OOP, declared properties, inheritance |
| **vctrs** | `#[miniextendr(vctrs)]` | `new_type(...)` vectors | tidyverse-compatible vector/record types |

Default to **R6** for stateful objects and **S3** for lightweight
generic-style APIs. R6/S7/vctrs add the corresponding R package to your
`DESCRIPTION` — `minirextendr::use_r6()` / `use_s7()` / `use_vctrs()` /
`use_s4()` set that up.

## A complete R6 example

```rust
use miniextendr_api::miniextendr;

pub struct Counter {
    value: i32,
}

#[miniextendr(r6)]
impl Counter {
    /// Create a new counter.
    pub fn new(start: i32) -> Self {
        Counter { value: start }
    }

    /// Increment and return self for chaining.
    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn get(&self) -> i32 {
        self.value
    }
}
```

After `minirextendr::miniextendr_build()`:

```r
c <- Counter$new(10L)
c$increment()$increment()   # void Rust methods return invisible(self) → chainable
c$get()
#> [1] 12
```

## How receivers map

| Rust receiver | Becomes |
|---|---|
| `&self` | instance method (read-only) |
| `&mut self` | instance method (mutating; R6 returns `invisible(self)` for `()` returns) |
| no receiver, returns `Self` | constructor (`new`) or static factory |
| no receiver, other return | static method: `Type$method()` / `Type_method()` |

Multiple `impl` blocks for one type need labels:
`#[miniextendr(r6, label = "extra")]` — otherwise the build errors on
duplicate class definitions.

## What the generated code already handles (don't re-do it)

- **Constructor error propagation**: a `panic!` in `new()` becomes a proper R
  error. The generated constructors validate the low-level result before
  wrapping it — if you hand-write an R constructor around a `.Call`, you lose
  that guard and a panicking constructor silently yields a corrupt object.
  Prefer the generated constructors.
- **S7 parent ordering**: parent classes are emitted before children in the
  wrappers file automatically.
- **vctrs load order**: the generated `@importFrom vctrs ...` lines in
  NAMESPACE force the vctrs DLL to load first. If vctrs dispatch fails,
  check those lines survived your last `document()` run.
- **Method documentation**: `///` doc comments with `@param`/`@return` flow
  into `man/` per method.

## Per-system notes

- **Env**: methods are functions in an environment; `obj$method()` works via
  a generated `$` S3 method that binds `self`. No dependencies; not a formal
  class system (no inheritance).
- **R6 extras**: `#[miniextendr(r6(private))]` for private methods,
  `#[miniextendr(r6(prop = "name"))]` for active bindings (computed
  properties), `r6(finalize)` for a GC destructor, `r6(deep_clone)` for
  custom clone logic.
- **S3**: constructor is `new_<type>()` (lowercase); each instance method
  gets a generic with a guard so it won't clobber a generic already defined
  by another package.
- **S4**: the struct is held in a single `ptr` slot; generics/methods are
  registered via `methods::setGeneric`/`setMethod`. Don't name Rust methods
  `s4_*` — the generator prefixes `s4_` itself (the MXL111 lint catches the
  resulting `s4_s4_*`).
- **S7**: properties come from `#[miniextendr(s7(getter))]` /
  `s7(setter)` / `s7(validator)` methods; multiple dispatch via
  `s7(dispatch = "x,y")`.
- **vctrs**: constructor wraps `vctrs::new_vctr()` / `new_rcrd()` /
  `new_list_of()`. Constructors must return `SEXP` (not `Self`) and the impl
  block cannot have instance-method receivers — the build errors otherwise.

## Traits callable from R

Annotate the trait and each impl:

```rust
#[miniextendr]
pub trait Describe {
    fn describe(&self) -> String;
}

#[miniextendr]
impl Describe for Counter {
    fn describe(&self) -> String {
        format!("Counter at {}", self.value)
    }
}
```

R gets dual calling forms: `Counter$Describe$describe(obj)` and
`obj$Describe$describe()`. This is how you share one method surface across
several Rust types.

## Export control

- Default: class and methods are exported.
- `#[miniextendr(internal)]` — exported but `@keywords internal` (hidden from
  the docs index).
- `#[miniextendr(noexport)]` — not exported at all. Don't combine with
  `internal` (redundant; the MXL203 lint flags it).

## Pitfalls

- **Editing `R/<pkg>-wrappers.R`** — regenerated on every build; all edits
  lost. Change the Rust and rebuild.
- **Forgetting the rebuild**: class shape changes (new method, renamed
  property) need `minirextendr::miniextendr_build()`, which also refreshes
  NAMESPACE and reinstalls if exports changed.
- **Holding references across mutations**: the R object is a pointer to the
  Rust value. `obj2 <- obj` aliases the same Rust state in every class system
  (external-pointer semantics), even in "value-semantics" systems like S4/S7.
  Provide an explicit `clone`/copy method if callers need independent copies.
- **Panics in methods** become R errors — fine and intended. But a panic
  mid-mutation leaves the Rust struct in whatever state it reached; design
  mutating methods to validate before mutating.

Full manual (class systems chapter): https://a2-ai.github.io/miniextendr
