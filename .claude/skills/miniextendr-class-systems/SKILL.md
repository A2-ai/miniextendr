---
name: miniextendr-class-systems
description: Use when the user asks how to expose a Rust struct as an R class, which R class system to use (R6 vs S3 vs S4 vs S7 vs Env vs Vctrs), how method dispatch works, how constructors are generated, how trait methods map to R, or when working in miniextendr-macros/src/miniextendr_impl/*.rs or miniextendr-macros/src/r_class_formatter.rs.
---

# miniextendr R Class Systems

miniextendr supports six R class systems for wrapping Rust structs. When a Rust
`impl` block is annotated with `#[miniextendr(r6)]`, `#[miniextendr(s7)]`, etc.,
the macro generates a complete R class definition — constructor, instance methods,
static methods, and roxygen documentation — with proper `.Call()` wrappers. This
skill covers all six systems, their dispatch mechanics, and how to choose between
them.

## When to use this skill

- "How do I expose a Rust struct as an R6 class?"
- "Which class system should I use?"
- "How do instance methods map from Rust to R?"
- "How do trait methods appear in the generated R code?"
- "Why does my S7 class need its parent defined first?"
- "What does `condition_check_lines` do in constructors?"
- "What is RWrapperPriority and why does the ordering matter?"
- "How do I use vctrs with miniextendr?"
- "What is `invisible(self)` in the generated R6 code?"

## Key concepts

### Class system overview

| System | Syntax | R pattern | Constructor | Best for |
|---|---|---|---|---|
| **Env** | `#[miniextendr]` on impl | `obj$method()` via env `$` | `Type$new(...)` | Simple, env-based dispatch |
| **R6** | `#[miniextendr(r6)]` | `R6Class` with `$new()` | `Type$new(...)` | OOP with encapsulation, active bindings |
| **S3** | `#[miniextendr(s3)]` | `generic(obj)` dispatch | `new_type(...)` (lowercase) | Idiomatic R generics, S3 dispatch |
| **S4** | `#[miniextendr(s4)]` | `setClass`/`setMethod` | `Type(...)` | Formal OOP, `methods::` integration |
| **S7** | `#[miniextendr(s7)]` | `new_class`/`new_generic` | inline constructor in class | Modern R OOP, property constraints |
| **Vctrs** | `#[miniextendr(vctrs)]` | `new_vctr`/`new_rcrd`/`new_list_of` | `new_type(...)` | vctrs-compatible vectors and records |

### Method dispatch model

Every class system maps Rust method receivers to R dispatch differently.

**Receiver kinds** (from `miniextendr-macros/src/miniextendr_impl.rs`):

| Rust receiver | Category | Generated as |
|---|---|---|
| `&self` | Instance (immutable) | Instance method on object |
| `&mut self` | Instance (mutable) | Instance method, chainable (`invisible(self)` in R6) |
| `self: &ExternalPtr<Self>` | Instance (full ExternalPtr access) | Same |
| (none) / returns `Self` | Static or constructor | `Type$method()` or `new_type()` |

Static methods for Env/S4/S7/S3 are named `<Class>_<method>(...)` or `<Class>$<method>(...)`.

### In-place builder idiom (`&mut self -> &mut Self`)

A method signature of `&mut self -> &mut Self` (not just `&mut self -> ()`)
is a deliberately supported, cross-class-system idiom for builder-pattern
types — mutate a field, return the same receiver, so calls compose into a
chain. It's implemented and tested for **all six class systems**, not just
R6:

- **R6 / Env** (reference-semantic): the chain reads through the *same*
  underlying object — `builder$set_name("x")$set_loud(TRUE)$build()` /
  `b$add(1L)$add(2L)$total()`. Codegen must return the receiver directly,
  **not** re-wrap it (e.g. `R6PipeBuilder$new(.ptr = .val)` would mint a
  fresh R6 environment around the same pointer and silently break object
  identity for anything holding the original reference — see issue #769,
  guarded by `rpkg/src/rust/pipe_builder_tests.rs`'s R6 fixture).
- **S3 / S4 / S7** (free-function dispatch): the mutator methods become free
  functions taking the object first (`builder_set_name(x, "World")`,
  `s4_add(x, 1L)`, `s7_add(x, 1L)`) whose body returns the receiver — which
  means they compose directly under R's native pipe: `b <- new_builder() |>
  builder_set_name("World") |> builder_set_loud(TRUE)`. S4 method names are
  auto-prefixed (`add` → `s4_add`; naming the Rust method `s4_*` yourself
  trips MXL111). S7 names are **not** auto-prefixed, so fixtures write
  `s7_add`/`s7_total` explicitly to keep the generated generics in a clean
  namespace.
- A terminal step normally takes `&self` (not `self`) and returns a
  different type — e.g. `pub fn build(&self) -> String` — converted to R
  via the ordinary `IntoR` path, ending the chain.

This is distinct from the plain `&mut self -> ()` case in the table above:
that one exists for regular mutators that don't need chaining and merely
gets `invisible(self)` in R6 so a bare `obj$mutate()` doesn't print `NULL`;
`&mut self -> &mut Self` is the idiom to reach for when you're deliberately
designing a chainable/builder API, and it's the only form validated to
preserve object identity across R6 chains and to compose under `|>` for the
free-function class systems. See `rpkg/src/rust/pipe_builder_tests.rs` for
one worked fixture per class system, and
`rpkg/tests/testthat/test-pipe-builder.R` for the R-side usage each produces.

### Constructor generation and error checking

For every class system, the constructor calls `.Call(C_Type__new, ...)` and
**must** validate the returned value. All six class generators wire through
`condition_check_lines()` from `miniextendr-macros/src/method_return_builder.rs`:

```r
.val <- .Call(C_Type__new, ...)
if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
  return(.miniextendr_raise_condition(.val, sys.call()))
```

Skipping this check means a panic from the Rust constructor silently corrupts the
object rather than raising an R error. This applies to **all five class generators
with constructors** (Env, R6, S3, S4, S7 — and Vctrs via S3). Read
`method_return_builder.rs` `condition_check_lines` for the canonical guard line.

### RWrapperPriority — output ordering

`RWrapperPriority` in `miniextendr-api/src/registry.rs` controls the order of
entries in `R/miniextendr-wrappers.R`. R evaluates the file top-to-bottom, so
dependencies must come first:

| Priority | Variant | Contents |
|---|---|---|
| 0 (first) | `Sidecar` | `#[r_data]` getter/setter accessors — must precede class definitions |
| 1 | `Class` | Class definitions (all six impl-block systems) |
| 2 | `Function` | Standalone `#[miniextendr]` functions |
| 3 | `TraitImpl` | Trait impl wrappers (`impl Trait for Type`) |
| 4 (last) | `Vctrs` | Vctrs S3 method wrappers from `#[derive(Vctrs)]` |

Within the `Class` priority group, S7 class definitions are topologically sorted
by `sort_s7_classes` (in `miniextendr-api/src/registry.rs` around L584) to ensure
parents are defined before children. S7's `parent = X` argument requires `X` to
already exist in the R session.

### Export control

`ClassDocBuilder::with_export_control(internal, noexport)` handles export logic
uniformly across all six generators. Pass `parsed_impl.noexport || parsed_impl.internal`:

- Default: `@export` emitted; class and methods appear in NAMESPACE.
- `#[miniextendr(internal)]`: adds `@keywords internal`; still exported but hidden
  from user-facing docs.
- `#[miniextendr(noexport)]`: omits `@export`; class not exported from NAMESPACE.

For S3, `internal` suppresses the generic export but keeps `S3method(generic, Class)`
registration so dispatch still works for instances of the class (see S3 generator
in `s3_class.rs`). The `noexport` flag suppresses `S3method` registration entirely.

## How it works

### Env class

File: `miniextendr-macros/src/miniextendr_impl/env_class.rs`

The Env system creates a plain R environment as the class namespace:

```r
Type <- new.env(parent = emptyenv())
Type$new <- function(...) { ... }
Type$method_name <- function(x = self, ...) { ... }
```

Instance methods use a default argument `x = self` pattern. The generated
`$.Type` S3 method re-parents the method environment so that `self` is in scope
when the user writes `obj$method()`. Specifically: `environment(obj) <- environment()`
on the method binds `self` in the dispatch frame before forwarding. `[[.Type` is
an alias to `$.Type`.

Active-binding wrappers for trait namespace dispatch use `local({ m <- method; bound[[name]] <<- function(...) m(self, ...) })` to avoid shared-variable capture in a loop.

### R6 class

File: `miniextendr-macros/src/miniextendr_impl/r6_class.rs`

R6 generates an `R6::R6Class(...)` definition with:

- `initialize`: calls `C_Type__new`, error-checks, stores result in `private$.ptr`.
- Public methods: one R function per `&self`/`&mut self` method, calling
  `.Call(C_Type__method, private$.ptr, ...)`.
- Private methods: methods marked `#[miniextendr(r6(private))]`.
- Active bindings: getter/setter properties via `#[miniextendr(r6(prop = "name"))]`.
- Finalizer: optional destructor called on GC via `#[miniextendr(r6(finalize))]`.
- Deep clone: optional custom clone logic via `#[miniextendr(r6(deep_clone))]`.
- Static methods: emitted as `ClassName$method_name <- function(...)` outside the
  class definition.

For void instance methods (`-> ()` return type), the generated R method body ends
with `invisible(self)` to support method chaining. See `method_return_builder.rs`
`build_r6_body` around L260.

The `DotCallBuilder` uses `.null_call_attribution()` for the R6 finalizer and
deep_clone methods — `match.call()` in those contexts captures an internal
dispatch frame rather than the user call.

Static methods that return `Self` add a `.ptr` parameter to `initialize`, allowing
factory methods: the constructor accepts either user arguments or a pre-built
`.ptr` from a static factory.

### S3 class

File: `miniextendr-macros/src/miniextendr_impl/s3_class.rs`

S3 generates:

- Constructor: `new_<class>(...)` (lowercase class name). Returns `structure(.val, class = "<Class>")`.
- S3 generics: `if (!exists("generic", mode = "function")) { generic <- function(x, ...) UseMethod("generic") }` for each instance method.
- S3 methods: `generic.<Class> <- function(x, ...) { ... }` dispatching via `.Call`.
- Static methods: `<Class>_<method>(...)`.

The conditional generic guard uses `if (!exists(...))` to avoid clobbering
existing generics from other packages. **Do not write `#' @export` on the
conditional block** — roxygen2 cannot introspect it and will drift the `@export`
onto the next function. Always use `#' @export generic_name` (an explicit
target). The macro generates this correctly; this only matters when adding manual
roxygen to adjacent code.

S3 also generates an Env-style class environment (`Type <- new.env(parent = emptyenv())`)
for `Type$new()` syntax and trait namespace compatibility.

### S4 class

File: `miniextendr-macros/src/miniextendr_impl/s4_class.rs`

S4 generates:

- Class definition: `methods::setClass("<class>", slots = c(ptr = "externalptr"))`.
  The Rust struct is held in a single `ptr` slot.
- Constructor: `<Class>(...)` that calls `C_Type__new`, error-checks, and wraps
  with `methods::new("<class>", ptr = .val)`.
- Generics: `methods::setGeneric(...)` for each instance method (idempotent,
  always emitted).
- Methods: `methods::setMethod("<generic>", "<class>", function(x, ...) ...)` with
  `x@ptr` to extract the ExternalPtr from the S4 slot.
- Static methods: `<Class>_<method>(...)`.

S4 helpers (`slot()`, `slot<-()`) live in the `methods` package. Access via
`getNamespace("methods")` — not `R_BaseEnv` — to ensure the methods DLL is loaded.

The S4 generator imports `@importFrom methods setClass setGeneric setMethod new`.
`methods::new("<class>", ...)` is the correct constructor call (not bare `new(...)`).

### S7 class

File: `miniextendr-macros/src/miniextendr_impl/s7_class.rs`

S7 generates:

- Class definition: `S7::new_class("<class>", parent = ..., properties = list(...), constructor = function(...) ...)`.
- Properties: `S7::new_property(...)` for each `#[miniextendr(s7(getter))]` /
  `#[miniextendr(s7(setter))]` / `#[miniextendr(s7(validator))]` annotated method.
  The `.ptr` property holds the ExternalPtr.
- Instance methods: `S7::new_generic(...)` + `S7::method(generic, class, function(x, ...) ...)`.
  The ExternalPtr is extracted from the S7 object via `x@.ptr`.
- Fallback methods: `S7::method(generic, S7::class_any)` for methods that accept
  any S7 object. Since non-S7 objects cannot use `@`, the generated code guards:
  `if (inherits(x, "S7_object")) x@.ptr else stop(...)`.
- External generics: `S7::new_external_generic("pkg", "name")` for overriding
  generics from other packages.
- Multiple dispatch: via `#[miniextendr(s7(dispatch = "x,y"))]`.
- Static methods: `<Class>_<method>(...)`.
- Convert methods: `S7::method(convert, list(From, To))` for `convert_from`/`convert_to`.

**Topological ordering**: S7's `parent = X` requires `X` to already be defined.
`collect_r_wrappers()` topologically sorts all S7 class fragments so parents
appear before children in `R/miniextendr-wrappers.R`. If you are working with
manually written S7 wrappers alongside generated ones, ensure the same ordering.

The `DotCallBuilder` uses `.null_call_attribution()` for S7 property
getter/setter/validator methods.

### Vctrs class

File: `miniextendr-macros/src/miniextendr_impl/vctrs_class.rs`

Vctrs generates an S3-based class compatible with the vctrs type system:

- Constructor: `new_<class>(...)` wrapping with `vctrs::new_vctr()`,
  `vctrs::new_rcrd()`, or `vctrs::new_list_of()` depending on the `VctrsKind`.
- Protocol boilerplate: `vec_ptype_abbr.<class>`, `vec_ptype2.<class>.<class>`,
  `vec_cast.<class>.<class>`.
- S3 generics and methods for instance methods.

The vctrs generator emits `@importFrom vctrs new_vctr new_rcrd new_list_of
vec_ptype2 vec_cast vec_ptype_abbr` at the class level. This forces the vctrs
DLL to load before the importing package, which is required because
`R_GetCCallable("vctrs", ...)` throws an R error (longjmp, not NULL) if vctrs
is not loaded. Always verify this `importFrom` line is present in `NAMESPACE`
when debugging vctrs dispatch failures.

For `#[derive(Vctrs)]` (struct-level derive, not impl block), the output appears
at `RWrapperPriority::Vctrs` (last).

### Trait methods across class systems

When `impl Trait for Type` is annotated with `#[miniextendr]`, it goes through
`miniextendr-macros/src/miniextendr_impl_trait.rs`. The trait's C-callable shims
are already declared by `#[miniextendr]` on the trait definition itself (in
`miniextendr_trait.rs`). The impl block generates vtable registration in
`MX_TRAIT_DISPATCH` and emits R-side trait wrappers at
`RWrapperPriority::TraitImpl`. The wrapper shape is class-system-specific.

True dual calling — a trait method callable *both* as `Type$Trait$method(obj)`
and as `obj$Trait$method()` — is available only for the Env class system. For Env:

- `Type$Trait$method(obj)` — the class environment holds a nested environment for
  the trait, with methods that accept the object as their first argument.
- `obj$Trait$method()` — via `$.Type` dispatch, the trait namespace environment is
  found and `self` is bound. Instance methods are stamped with a
  `.__mx_instance__` attribute so the dispatcher knows to prepend `self`.

This mirrors the pattern from `miniextendr-macros/src/miniextendr_impl/env_class.rs`,
where `local({ m <- method; bound[[name]] <<- function(...) m(self, ...) })` creates
bound closures that avoid shared-variable capture in loops.

The other five systems surface trait methods through their own dispatch
conventions, not dual calling. Each system's R-visible name is owned by
`trait_namespace_target` in
`miniextendr-macros/src/miniextendr_impl_trait/method_context.rs` — one policy the
five generators previously hand-rolled independently, unified by #1141 / #1219:

- **R6**: `Type$Trait$method(obj)` — a class-scoped trait namespace on the
  generator object (the R6 generator is itself an environment, so
  `Type$Trait <- new.env()` attaches cleanly). Both instance and static methods
  live here. This is single-form, *not* dual: R6 classes are defined
  monolithically in one `R6Class()` call, so a trait method cannot be injected as
  `obj$method()`. Before #1219, R6 trait methods were unqualified standalone
  `r6_trait_<Trait>_<method>(obj)` functions; that name is now gone — the
  class-scoped shape is collision-free by construction, closing #1115.
- **S3 / Vctrs**: idiomatic S3 dispatch — an S3 generic plus a `method.Type`
  method, called as `method(obj)`. Static methods and consts live in the
  `Type$Trait$name` namespace.
- **S4**: an S4 generic named `s4_trait_<Trait>_<method>`, registered via
  `setGeneric` / `setMethod` and called as `s4_trait_<Trait>_<method>(obj)`.
  Static methods and consts are flat, class-qualified `Type_Trait_method()`
  functions (S4 objects intercept `$<-`, so `Type$Trait$` can't be assigned onto
  them).
- **S7**: an S7 generic named `s7_trait_<Trait>_<method>` (via `S7::new_generic`),
  plus an optional per-method fast-path shortcut `Type_method(obj)` that bypasses
  `S7::S7_dispatch()` (opt out with `#[miniextendr(s7(no_shortcut))]`). Static
  methods and consts are assigned into a local env attached to the class via
  `attr()`, so they stay reachable as `Type$Trait$name` (S7's `$` falls through to
  attributes).

`-> Self` / `Result<Self, E>` / `Option<Self>` trait methods re-wrap their return
into a classed object of the implementing type on every system (via the shared
`MethodReturnBuilder`, also from #1141) rather than leaking an unclassed pointer.

## Decision trees

### Which class system should I use?

Start with the intended R API surface:

- **R6**: when you want reference semantics, active bindings (computed properties),
  private methods, or method chaining. The most commonly used system for stateful
  Rust structs.
- **S7**: when you want R 7-compatible OOP with declared properties, class
  constraints, or inheritance. Requires the S7 package (`Suggests`).
- **S3**: when you want idiomatic R dispatch (`generic(obj)`) and interoperability
  with the broader S3 ecosystem. No extra package dependency.
- **S4**: when you need formal multiple dispatch or are integrating with an
  existing S4-heavy package.
- **Env**: minimal overhead, no package dependencies, `obj$method()` syntax. Good
  for internal helpers or when you want the simplest possible wrapper.
- **Vctrs**: when the Rust type represents a vector or record and you want it to
  work with vctrs-aware tidyverse functions (`dplyr`, `tidyr`, etc.).

### How do I expose a Rust struct as an R6 class?

1. Write `#[miniextendr(r6)] impl MyStruct { ... }` in your Rust crate.
2. Ensure the impl block is in a module reachable from `lib.rs`.
3. Include a `pub fn new(...) -> Self` method (or mark a method with
   `#[miniextendr(constructor)]`).
4. Run `just configure && just rcmdinstall && just force-document`.
5. In R: `obj <- MyStruct$new(...)`.

The generated class is named after the Rust type by default. Override with
`#[miniextendr(r6, class = "OverrideName")]`.

### How do I implement a trait for multiple class systems?

1. Declare the trait with `#[miniextendr] pub trait MyTrait { ... }`. This
   generates the vtable, view struct, and shims in `miniextendr_trait.rs`.
2. For each implementing type, add `#[miniextendr] impl MyTrait for MyType { ... }`.
   This generates the vtable registration and R-side trait wrappers.
3. Only Env gets true dual calling (`MyType$MyTrait$method(obj)` **and**
   `obj$MyTrait$method()`). The other systems surface the method through their own
   dispatch conventions: R6 as `MyType$MyTrait$method(obj)` (a class-scoped
   namespace, single-form); S3/Vctrs as an S3 generic (`method(obj)`); S4 as
   `s4_trait_<MyTrait>_<method>(obj)`; S7 as `s7_trait_<MyTrait>_<method>(obj)`
   (plus an optional `MyType_method(obj)` fast-path). See "Trait methods across
   class systems" above for the full shape table.

### How do I build a chainable builder-pattern API?

Use a `&mut self -> &mut Self` method signature (not `-> ()`) for each
mutator step — see "In-place builder idiom" above. Works the same way
regardless of which class system you pick; choose the class system by your
desired call syntax (`obj$step()` chains for R6/Env, `obj |> step()` pipes
for S3/S4/S7).

### My constructor panics but R doesn't see an error — what went wrong?

The constructor's `.Call()` result must be validated before wrapping into the
R object. All six generators do this via `condition_check_lines()`. If you are
manually writing constructor wrappers (rare), you must include the guard:

```r
.val <- .Call(C_Type__new, ...)
if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
  return(.miniextendr_raise_condition(.val, sys.call()))
```

If this guard is absent, a panicking constructor silently returns a corrupted
SEXP as the object value. There is no runtime crash — the corruption shows up
later, unpredictably.

## Key files

- `miniextendr-macros/src/miniextendr_impl.rs` — impl block parsing: `ImplAttrs`,
  `ParsedImpl`, `ParsedMethod`, `ReceiverKind`; architecture diagram and class
  system dispatch table.
- `miniextendr-macros/src/miniextendr_impl/env_class.rs` — Env generator:
  `generate_env_r_wrapper`. Env dispatch, `$.Type` method, `local()` binding.
- `miniextendr-macros/src/miniextendr_impl/r6_class.rs` — R6 generator:
  `generate_r6_r_wrapper`. Active bindings, finalizer, deep_clone, static methods.
- `miniextendr-macros/src/miniextendr_impl/s3_class.rs` — S3 generator:
  `generate_s3_r_wrapper`. Conditional generic guard, `new_<class>` constructor.
- `miniextendr-macros/src/miniextendr_impl/s4_class.rs` — S4 generator:
  `generate_s4_r_wrapper`. `setClass`/`setGeneric`/`setMethod`, `x@ptr` extraction.
- `miniextendr-macros/src/miniextendr_impl/s7_class.rs` — S7 generator:
  `generate_s7_r_wrapper`. Properties, fallback methods, `x@.ptr`, convert methods.
- `miniextendr-macros/src/miniextendr_impl/vctrs_class.rs` — Vctrs generator:
  `generate_vctrs_r_wrapper`. `new_vctr`/`new_rcrd`/`new_list_of`, protocol boilerplate.
- `miniextendr-macros/src/miniextendr_impl_trait.rs` — Trait impl codegen:
  vtable registration and R-side trait wrappers; the per-class namespace shape is
  owned by `miniextendr-macros/src/miniextendr_impl_trait/method_context.rs`
  (`trait_namespace_target`).
- `miniextendr-macros/src/r_class_formatter.rs` — Shared utilities: `ClassDocBuilder`,
  `MethodDocBuilder`, `MethodContext`, `emit_s3_generic_guard`, `should_export_from_tags`.
- `miniextendr-macros/src/method_return_builder.rs` — `condition_check_lines`,
  `condition_check_inline_block`, `ReturnStrategy`; `build_r6_body` (`invisible(self)`).
- `miniextendr-macros/src/r_wrapper_builder.rs` — `DotCallBuilder` at ~L390;
  `.null_call_attribution()` for lambda contexts.
- `miniextendr-api/src/registry.rs` — `RWrapperPriority` enum (L210), `collect_r_wrappers`,
  `sort_s7_classes` (L584).

## Common pitfalls

- **Missing `condition_check_lines` in constructor**: if you copy-paste a
  constructor wrapper and omit the tagged-SEXP guard, panics silently corrupt
  the returned object. All six generators include it; only risk arises in
  hand-crafted wrappers.

- **S7 parent undefined at load time**: S7's `parent = X` requires `X` to be
  evaluated before the child class definition. `collect_r_wrappers()` topologically
  sorts S7 classes in the generated file. If you have a manual S7 wrapper mixed
  into the file, ensure the parent appears first.

- **S3 `@export` on conditional generic**: `if (!exists("generic", mode = "function")) { generic <- function(x, ...) UseMethod("generic") }` is not introspectable by roxygen2. Adding `#' @export` directly above it causes roxygen to drift the export onto the next function. Use `#' @export generic_name` (explicit target) instead. The macro generator handles this correctly.

- **Vctrs DLL not loaded**: `R_GetCCallable("vctrs", ...)` longjmps (throws R
  error) if the vctrs DLL is not loaded. `@importFrom vctrs ...` in NAMESPACE
  forces load order. Verify the `importFrom vctrs` line exists in `NAMESPACE`
  when debugging vctrs dispatch failures.

- **S4 helpers need `methods` namespace**: `slot()` and `slot<-()` live in the
  `methods` package, not `R_BaseEnv`. Access via `getNamespace("methods")`. The
  S4 generator emits `@importFrom methods ...` to ensure the package is attached.

- **R6 `invisible(self)` for void methods**: instance methods that return `()`
  in Rust emit `invisible(self)` in R for method chaining. This is correct and
  intentional. Do not replace it with `invisible(NULL)` — that would break
  chaining syntax (`obj$method1()$method2()`).

- **Sidecar accessors must precede class definitions**: `#[r_data]` getter/setter
  wrappers (`RWrapperPriority::Sidecar`) must appear before class definitions that
  reference them. This ordering is automatic when using `collect_r_wrappers()`.
  It becomes a risk only if wrapper fragments are assembled manually.

- **MXL111 — `s4_` prefix on S4 method names**: a method named `s4_something` on
  a `#[miniextendr(s4)]` impl block generates `s4_s4_something` in R (the lint
  fires). Drop the `s4_` prefix from the Rust method name.

- **`DotCallBuilder` vs sidecar wrappers**: `DotCallBuilder` (in `r_wrapper_builder.rs`)
  emits `.call = match.call()`. Sidecar accessors from `externalptr_derive.rs` do
  NOT use `DotCallBuilder` and have no call slot. Adding `.call = match.call()`
  to a hand-written sidecar wrapper causes "Incorrect number of arguments" at runtime.
  See the `miniextendr-macros` skill for details on the two codegen paths.

## Related skills

- `miniextendr-macros` — the full `#[miniextendr]` codegen pipeline: C wrapper
  synthesis, attribute reference, tagged-SEXP error transport, trait ABI shims.
- `miniextendr-externalptr` — `Box<Box<dyn Any>>` storage, TypedExternal display
  tags, pointer provenance. The `.ptr` / `private$.ptr` / `x@ptr` fields are all
  `ExternalPtr` values.
- `miniextendr-ffi` — `#[r_ffi_checked]`, `_unchecked` variants, MXL300/MXL301.
- `miniextendr-altrep` — for compute-on-access vectors; separate from impl-block
  class systems.
- `miniextendr-lint` — MXL111 (s4 prefix) and other rules that affect class codegen.
