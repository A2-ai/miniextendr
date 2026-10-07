# Proc-Macro and Lint Error Guide

This guide covers common error messages from `#[miniextendr]` proc macros and the `miniextendr-lint` static analysis tool.

## Running the Lint

```bash
just lint               # Run lint on rpkg
```

The lint also runs automatically during `cargo build`/`cargo check` via `build.rs`. To disable temporarily:

```bash
MINIEXTENDR_LINT=0 cargo check --manifest-path=rpkg/src/rust/Cargo.toml
```

A source file the lint cannot parse gets a `<file>: failed to parse: <error>` warning, and no rule checks that file. The build script watches every source file it found, so the warning goes away on the next build after the fix.

## Lint Codes Reference

### Errors (CI-blocking)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL001** | Reserved | (Legacy lint, no longer applicable) |
| **MXL002** | Reserved | (Legacy lint, no longer applicable) |
| **MXL003** | Reserved | (Legacy lint, no longer applicable) |
| **MXL004** | Reserved | (Legacy lint, no longer applicable) |
| **MXL005** | Reserved | (Legacy lint, no longer applicable) |
| **MXL006** | Reserved | (Legacy lint, no longer applicable) |
| **MXL007** | `impl Type;` requires ExternalPtr derive | Add `#[derive(ExternalPtr)]` or implement `TypedExternal` |
| **MXL008** | Trait impl class system incompatible | Ensure trait impl uses same class system as inherent impl |
| **MXL009** | Multiple impl blocks without labels | Add `#[miniextendr(label = "unique")]` to each impl block |
| **MXL010** | Duplicate labels on impl blocks | Use unique labels for each impl block |

### Warnings (P0: high impact)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL100** | Duplicate entrypoint symbol | Rename one of the conflicting entries |
| **MXL101** | Duplicate registration entries | Remove the duplicate entry |
| **MXL102** | Trait impl missing TypedExternal | Implement `TypedExternal` for the type |
| **MXL103** | Generic concrete type in trait-ABI | Use concrete (non-generic) types for trait ABI |
| **MXL104** | `#[cfg]` mismatch between item and registration | Ensure `#[cfg]` attributes match on both |
| **MXL105** | Unreachable module file | Check file paths and module hierarchy |
| **MXL106** | Registered function is not `pub` | Add `pub` to the function definition |
| **MXL107** | Missing `#[miniextendr] impl Trait for Type` | Add the attribute to the trait impl |
| **MXL108** | Missing registration for trait impl | Add `#[miniextendr]` to the trait impl |
| **MXL109** | `#[cfg]` mismatch between `mod` declarations | Ensure `#[cfg]` attributes are consistent |
| **MXL110** | Parameter name is an R reserved word | Rename the parameter — generated R wrapper would fail to parse |
| **MXL111** | `s4_*` method on `#[miniextendr(s4)]` impl | Drop the `s4_` prefix — codegen auto-prepends it (you'd get `s4_s4_*`) |
| **MXL112** | Explicit lifetime parameter on `#[miniextendr]` fn or impl | Use owned types (`Vec<T>` / `String`) — `&[T]` and `&str` arguments work without explicit lifetime annotations |

### Warnings (P1: important)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL200** | Trait tag collision preflight | Use unique trait names or labels |
| **MXL201** | Impl label mismatch | Check label spelling matches across impls |
| **MXL202** | Orphan child module reference | Remove the reference to non-existent child module |
| **MXL203** | `internal` + `noexport` redundancy | Use just `#[miniextendr(internal)]` (implies noexport) |
| **MXL204** | Multiple root-level registrations | Only one root registration per crate |

### Warnings (P2: safety)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL300** | Direct `Rf_error`/`Rf_errorcall` call | Use `panic!()` or return `Err(...)` instead |
| **MXL301** | `_unchecked` FFI call outside guard context | Use the checked wrapper, or ensure you're inside `with_r_unwind_protect` |

#### MXL300: Direct Rf_error calls

`Rf_error()` and `Rf_errorcall()` perform a C `longjmp` that skips Rust destructors. This leaks memory and can corrupt state. The lint detects these calls via text scanning.

**Preferred alternatives:**

- `panic!("message")`: caught by miniextendr's unwind protection, produces a structured R condition
- `return Err(...)`: for `Result<T, E>` return types, produces a clean R error

**When Rf_error is intentional** (e.g., inside `with_r_unwind_protect` closures or test fixtures), suppress with `// mxl::allow(MXL300)`. See [Inline Suppression](#inline-suppression) below.

#### MXL301: Unchecked FFI calls

Functions like `Rf_ScalarInteger_unchecked()` bypass miniextendr's main-thread routing check. They are only safe when you are **certain** you're on R's main thread (inside ALTREP callbacks, `with_r_unwind_protect` closures, `extern "C-unwind"` functions called by R, etc.).

**Preferred:** Use the checked wrapper (without `_unchecked` suffix). It adds a debug-mode thread assertion.

**When unchecked is intentional**, suppress with `// mxl::allow(MXL301)`.

## Inline Suppression

Both MXL300 and MXL301 support inline suppression via `// mxl::allow(...)` comments. The suppression comment can appear:

1. **On the same line** (trailing comment):

```rust
Rf_error(c"%s".as_ptr(), c"intentional".as_ptr()) // mxl::allow(MXL300)
```

2. **On the immediately preceding line** (standalone comment):

```rust
// mxl::allow(MXL300)
Rf_error(c"%s".as_ptr(), c"intentional".as_ptr())
```

Multiple codes can be suppressed in one comment:

```rust
// mxl::allow(MXL300, MXL301)
Rf_error_unchecked(c"test".as_ptr())
```

**Important:** The comment must be on the exact same line or the line directly above. A comment two lines above will not suppress the diagnostic.

```rust
// mxl::allow(MXL300)   <-- too far away (2 lines above)
unsafe {
    Rf_error(...)        <-- NOT suppressed
}
```

Move the comment inside the block:

```rust
unsafe {
    // mxl::allow(MXL300)
    Rf_error(...)        <-- suppressed
}
```

### Suppression syntax

```rust
// mxl::allow(CODE)
// mxl::allow(CODE1, CODE2)
// mxl::allow(CODE1, CODE2, CODE3)
```

- Prefix: `// mxl::allow(`
- Codes: comma-separated, whitespace around commas is ignored
- Only `MXL300` and `MXL301` are currently suppressible

## Common Proc-Macro Errors

### "a function takes at most one `...`"

The parameter of type `&Dots` is R's `...`, at any position, and an R function has one `...`. Two `&Dots` parameters, or Rust `...` next to an explicit `&Dots`, are refused:

```rust
// Wrong: two dots parameters
#[miniextendr]
fn bad(x: i32, a: &Dots, b: &Dots) -> i32 { x }

// Wrong: `...` and `rest: &Dots` are both the dots
#[miniextendr]
fn also_bad(rest: &Dots, x: i32, more: ...) -> i32 { x }

// Correct: one dots parameter; `flag` follows it and is matched by name in R
#[miniextendr]
fn good(x: i32, rest: &Dots, flag: bool) -> i32 { x }
```

### "Rust's `...` is only valid as the last parameter"

Rust's `...` parameter parses only in last position, so a function or method whose `...` has another parameter after it is not a function to `#[miniextendr]`. The parameter of type `&Dots` is R's `...` at its own position; write a dots parameter that is not last that way:

```rust
use miniextendr_api::dots::Dots;

// Wrong: `sources: ...` has a parameter after it
#[miniextendr]
pub fn summarise(sources: ..., dosing_type: &str) -> i32 { 0 }

// Correct: the R formals are `(..., dosing_type)`
#[miniextendr]
pub fn summarise(sources: &Dots, dosing_type: &str) -> i32 { 0 }
```

The error points at the `...` and links [Formals after `...`](DOTS_TYPED_LIST.md#formals-after):

```text
Rust's `...` is only valid as the last parameter; write a dots parameter that is not last as `sources: &Dots` (`miniextendr_api::dots::Dots`), which is R's `...` at that position. See https://a2-ai.github.io/miniextendr/manual/dots-typed-list/#formals-after
```

Unnamed dots (a bare `...` or `_: ...`) get `_dots: &Dots`. `miniextendr-lint` can't parse such a file either, so its "failed to parse" warning ends with `line <n>: ` and the same text.

### "expected `pub` function"

Only `pub` functions get `@export` in R wrappers. If the function is intentionally private, use `#[miniextendr(noexport)]` or `#[miniextendr(internal)]`.

### "multiple impl blocks for Type need labels"

When a type has more than one `#[miniextendr]` impl block, each needs a unique label:

```rust
#[miniextendr(label = "core")]
impl MyType {
    fn method_a(&self) -> i32 { 0 }
}

#[miniextendr(label = "extra")]
impl MyType {
    fn method_b(&self) -> String { String::new() }
}
```

### "trait impl class system incompatible"

A trait impl's class system must match the inherent impl's class system:

```rust
#[miniextendr(s3)]
impl MyType { /* ... */ }

// This will error if the trait impl uses a different class system
#[miniextendr]  // Must inherit s3 from the inherent impl
impl Display for MyType { /* ... */ }
```

### "type must derive ExternalPtr or implement TypedExternal"

Types used in `#[miniextendr]` impl blocks need pointer identity:

```rust
#[derive(ExternalPtr)]
struct MyType { /* ... */ }

#[miniextendr]
impl MyType { /* ... */ }
// Registration is automatic via #[miniextendr].
```

### "takes `self` and is marked `constructor`"

`#[miniextendr(<system>(constructor))]` describes a receiverless `fn new(...) ->
Self`. A consuming builder step needs no marker: `fn step(self, ...) -> Self`
writes its result back into the same R object, `-> Result<Self, E>` /
`Option<Self>` run on a clone and overwrite on success (the type must be
`Clone`). Drop the attribute. See
[CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#consuming-receivers-self).

### "unsupported receiver type"

The R handle stores the value itself, so a method can take `self`,
`self: Self`, `&self`, `&mut self`, or an `ExternalPtr<Self>` receiver.
`self: Box<Self>`, `Rc<Self>`, `Arc<Self>` and friends cannot be handed over;
unwrap to `self` or take `&self`.

### "parameter `_x` becomes the R argument `x`, which ..."

Every parameter becomes an argument of the generated R function under its own
name with the leading underscores dropped (`_x` becomes `x`, `r#in` becomes
`in`). Callers pass arguments by name, so the macro never renames one. Instead
it rejects the names R cannot use:

- an R reserved word (`if`, `in`, `for`, `function`, `TRUE`, `NA`, ...) or a
  name that starts with a digit: the wrappers file would not parse;
- two parameters with the same R name (`x` and `_x`): R rejects a repeated
  argument name;
- a name the class system's wrapper binds itself: the receiver of an instance
  method (`self` for Env, `x` for S3, S4 and every trait method, the first
  dispatch argument for S7, `x` by default, and `self` for the S7 fast-path
  shortcut, which `s7(no_shortcut)` drops) and
  R6's `self` and `private` inside the class (`initialize` and every method).

Rename the Rust parameter. Static methods and constructors outside R6 are plain
functions and reserve nothing.

### "`postfix` and `r_name` both set the R wrapper name"

`postfix = "_impl"` derives the R name from the Rust identifier; `r_name = "..."`
replaces it. Giving both is contradictory, so it is rejected rather than letting
one silently win. Keep `postfix` when the name should follow the Rust name,
`r_name` when it should not. The method-level variant says "R method name" and
also rejects `postfix` together with `generic = "..."`. See
[VISIBILITY.md](VISIBILITY.md#postfix-state-the-internal-entry-point-convention-once).

### "Cargo.toml: [package.metadata.miniextendr] `noexport_postfix` must be ..."

The crate-level default for internal entry points is read from the crate's
manifest at expansion time. The value must be a single-line TOML string that
is a valid R identifier fragment (letters, digits, `_`, `.`), set once, in a
`[package.metadata.miniextendr]` table (or as a dotted key under `[package]` /
`[package.metadata]`); an inline table `miniextendr = { ... }` is rejected.
The sibling key `source_tags` must be a bare `true` / `false`, set once
("`source_tags` must be `true` or `false`, found …"). The key
`conversion_error_class` must be a string or a single-line array of strings,
set once, with non-empty, distinct entries that do not name `rust_error`,
`simpleError`, `error` or `condition` ("`conversion_error_class` must be a
string or a single-line array of strings, found …", "must not name
`rust_error`", "names `a` more than once"). The key `roxygen_prose_links`
must be `"strip"` or `"keep"`, set once ("`roxygen_prose_links` must be one of
`"strip"`, `"keep"`, found …"). Any of these errors is reported
on the first `#[miniextendr]` free function in the crate. See
[VISIBILITY.md](VISIBILITY.md#crate-level-default-from-the-manifest) and, for
`conversion_error_class`,
[ERROR_HANDLING.md](ERROR_HANDLING.md#a-crate-level-class-for-every-conversion-error),
for `roxygen_prose_links`,
[MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md#links-in-leading-prose).

### "`postfix` cannot be used with `s3(generic = ..., class = ...)`"

Standalone S3 methods are always named `generic.class`, so there is nothing for
a postfix to append to. Rename through `generic` instead.

### "`call = caller` attributes conditions to the wrapper's caller, which is only meaningful for a package-internal entry point"

`#[miniextendr(call = caller)]` makes the generated wrapper report its caller's
call in conditions. That is right for a `noexport` / `internal` entry point
wrapped by a hand-written R function, and wrong for an exported function whose
caller is arbitrary user code. Add `noexport` or `internal`, or drop the option.
See [CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#internal-entry-points-caller-attribution).

### "`call_arg` cannot be combined with `call = caller`"

A `caller` wrapper already ends its formals with `.call = NULL`, where `NULL`
is its caller's call; `call_arg` gives the same formal `NULL` as the wrapper's
own call. Keep `call = caller` for an internal entry point behind a
hand-written function, or drop it for a function that reports its own call
unless told otherwise. The same applies to a `CallerCall` parameter with
`call_arg` (make it `Call`). `call_arg` is also refused with `s3(...)` (an S3
method can't take a formal its generic lacks) and on an `extern "C-unwind"`
function, which has no call slot. See
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#an-exported-function-call-arg).

### "`call = ...` accepts `wrapper` (the call as written, the default) or `caller` (...)"

`call = parent`, `call = self` & co. name no attribution. The two values are
`wrapper` (the default) and `caller`, as a path or a string; see
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#choosing-the-attribution-marker-attribute-crate-default).

### "the `Call` parameter selects `wrapper` attribution but the attribute selects `caller`"

A `Call` / `CallerCall` parameter and the `call = ...` attribute are two
spellings of one decision, so `#[miniextendr(noexport, call = caller)] fn f(x: i32, call: Call)`
has no meaning. Keep one spelling, or make them agree. The same family covers
`a #[miniextendr] function takes at most one Call / CallerCall parameter` (the
call slot is a single value), `a CallerCall parameter attributes conditions to
the wrapper's caller, which is only meaningful for a package-internal entry
point` (add `noexport` / `internal`, as for `call = caller`),
`per-parameter options (...) do not apply to a Call parameter` (it is bound
from the call slot, not from an R argument) and `Call / CallerCall parameters
are supported on standalone #[miniextendr] functions only` (class and trait
methods keep the wrapper's own call).

### "`Quoted` / `Quosure` must be the parameter's whole type, or the type argument of `Missing<..>`"

The wrapper passes an argument unevaluated (`list(substitute(x),
parent.frame())` / `rlang::enquo(x)`) only for `x: Quoted`, `x: Quosure` and
the same inside `Missing<..>`; `Option<Quoted>` or `Vec<Quosure>` would be
converted from a forced value. Use `Missing<Quoted>` for an optional argument.
The same family covers `per-parameter options (...) do not apply to a Quoted
parameter` (a default, coercion or check would force the argument; `default`
alone is accepted on `Missing<Quoted>` / `Missing<Quosure>`, where it only
writes the formal),
`` `Checked<Quoted>` on parameter `x`: `Quoted` has no R-side type check to
keep or drop `` and `` `Quoted` parameters are supported on
standalone #[miniextendr] functions only `` (a class or trait method's wrapper
forces its arguments). See [QUOTED_ARGUMENTS.md](QUOTED_ARGUMENTS.md#restrictions).

### "Cargo.toml: [package.metadata.miniextendr] `call_attribution` must be one of ..."

The crate-wide default takes the same two values as the attribute, as a
string: `call_attribution = "wrapper"` or `"caller"`, set once. A
`"caller"` default applies to `noexport` / `internal` free functions only;
exported functions keep `wrapper`.

### "the `Checked` parameter `n` keeps the R-side type checks but `no_preconditions(n)` drops them"

A `Checked<T>` / `Unchecked<T>` marker and the parameter's keyword (the
per-parameter `#[miniextendr(no_preconditions)]`, or a method's
`no_preconditions(n)`) are two spellings of one decision about `n`'s R-side
type checks. Keep one, or make them agree. Two keywords do not conflict: the
last one written wins. The same family covers:

- `at most one precondition marker per parameter` (`Checked<Unchecked<T>>`):
  one decides.
- ``put `Checked` outermost: `Checked<Option<T>>` `` (`Option<Checked<T>>`,
  `Missing<..>`, `Vec<..>`, `&..`): the marker applies to the whole
  parameter, and the conversion has no marker to unwrap inside a container.
- `` `Checked<SEXP>` on parameter `x`: `SEXP` has no R-side type check to keep
  or drop `` (also `Missing<T>`, `ExternalPtr<T>`, `&Dots`, a custom type):
  drop the spelling.
- ``a match_arg/choices parameter is validated by `match.arg()` ``:
  `match.arg()` checks a choice parameter, not the type checks, so neither
  spelling applies.
- `` `Checked<T>` in a `#[miniextendr]` trait method ``: the trait's View
  passes arguments on as R values; write `preconditions(k)` /
  `no_preconditions(k)` on the impl's method.
- `` `Checked<T>` on an `extern "C-unwind"` function ``: it takes R values as
  they are; take the inner type.
- `` `Checked<T>` marks a parameter only; it cannot be used as a return type ``:
  return the inner `T`.
- `` `preconditions = ...` on a parameter ``: the `= true | false` form is the
  function attribute's; on a parameter write the bare `preconditions` /
  `no_preconditions`, or the marker.

See [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md#r-side-preconditions-markers-and-defaults).

### "Cargo.toml: [package.metadata.miniextendr] `preconditions` must be `true` or `false`"

The crate-wide default of the R-side type checks is a boolean, set once:
`preconditions = false` drops them in every wrapper of the crate that says
nothing itself, `preconditions = true` keeps them even under the
`no-preconditions-default` feature.

### "`serde_error` is not a switch"

Under the API crate's `serde` feature every `Result<T, E>` whose
`E: serde::Serialize + Display` (and without an `RConditionError` impl) is
already classed from its serde shape, so the bare `#[miniextendr(serde_error)]`
flag, `serde_error = true`, `serde_error = false` and an empty `serde_error()`
would switch nothing on or off. The attribute only carries options: write
`serde_error(tag = "..", prefix = "..", skip(..), rename(a = ".."))`, or drop
it. See
[CONDITIONS.md](CONDITIONS.md#deriving-the-classes-from-a-serde-error-type).

### "`serde_error` cannot be used with `unwrap_in_r`"

`#[miniextendr(serde_error(..))]` classes the condition raised from a
`Result`'s `Err` arm. `unwrap_in_r` hands the whole `Result` to R as a value
and never raises, so there is nothing to class. Drop `unwrap_in_r` to raise a
classed error, or drop the options to return the `Result`. See
[CONDITIONS.md](CONDITIONS.md#deriving-the-classes-from-a-serde-error-type).

### "`#[miniextendr(serde_error(..))]` requires a `Result<T, E>` return type"

The options only change the generated `Err` arm. A function or method that
does not return `Result` has no `Err` arm, so the attribute would be a silent
no-op; it is rejected instead. Return `Result<T, E>` with
`E: serde::Serialize + Display`.

### "unknown serde_error option; expected `tag`, `prefix`, `skip(...)` or `rename(...)`"

`serde_error(...)` takes `tag = "..."` and `prefix = "..."` as name-value
pairs and `skip("a", "b")` / `rename(a = "b")` as nested lists. The two list
options are the payload-field controls; `skip = "a"` and `rename = "b"` are
rejected with a message pointing at the list form. See
[CONDITIONS.md](CONDITIONS.md#payload-fields-named-message).

### "serde_error rename target `message` is reserved"

`rename(from = "to")` may not target `message`, `call` or `kind`: those are
the condition's own slots, and the rename would recreate the collision the
option exists to avoid. Pick another name. The same family covers
`serde_error skip names `a` twice`, `serde_error rename names `a` twice`, and
`serde_error names `a` in both skip and rename`: each field appears in at most
one place.

### "tuple field 0 of variant `E::Raw` needs a `data` name"

`#[derive(RConditionError)]` turns every field into a `data` entry named after
the field, and a tuple field has no name. Give it one with
`#[condition(rename = "name")]`, or drop it from the payload with
`#[condition(skip)]`. Named fields never hit this. See
[CONDITIONS.md](CONDITIONS.md#deriving-rconditionerror).

### "`kind` is one of the condition's own slots (`message`, `call`, `kind`)"

A `#[derive(RConditionError)]` field (or its `rename` target) may not be
called `message`, `call` or `kind`: the R condition object owns those slots.
Rename the field with `#[condition(rename = "…")]` or exclude it with
`#[condition(skip)]`. The same derive also rejects duplicate `data` names,
`message = "…"` on an enum (put it on the variants), `skip` combined with
`rename` / `debug`, generic types and unions.

### "the return type's visibility marker and the `invisible` / `visible` attribute disagree"

`Invisible<T>` / `Visible<T>` on the return type and the `invisible` /
`visible` attribute (on a function, or `#[miniextendr(invisible)]` /
`#[miniextendr(r6(invisible))]` & co. on a method) are two spellings of one
decision, so `#[miniextendr(visible)] fn f() -> Invisible<i32>` has no
meaning. Keep one spelling, or make them agree. The same family covers
`visibility markers cannot be nested` (`Invisible<Visible<T>>`: use a single
marker) and `` `Invisible<T>` / `Visible<T>` mark the return type only ``
(a marker as a parameter type). See
[MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md#return-visibility-markers-and-defaults).

## Debugging Tips

1. **Run [`just lint`](https://github.com/A2-ai/miniextendr/blob/main/justfile)** before building: it catches attribute issues earlier than compile errors
2. **Check NAMESPACE**: if a function exists in Rust but not in R, run [`just rcmdinstall && just force-document`](https://github.com/A2-ai/miniextendr/blob/main/justfile) (`force-document` bypasses roxygen2's mtime cache, which can miss macro-layer wrapper changes)
3. **Feature-gated modules**: use `#[cfg]` on `mod` declarations for conditional compilation
