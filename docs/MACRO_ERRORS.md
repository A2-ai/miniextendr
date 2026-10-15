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

The build script prints every diagnostic as a cargo warning
(`warning: <crate>@<version>: [MXL008] <file>:<line>: ...`) and never fails the build;
`just lint` lists them without failing either. The groups below are the
rules' severities: an error-severity rule flags code that will not work as
written.

### Errors

| Code | Description | Fix |
|------|-------------|-----|
| **MXL008** | Trait impl class system incompatible | Ensure trait impl uses same class system as inherent impl |
| **MXL110** | Parameter name is an R reserved word | Rename the parameter — generated R wrapper would fail to parse |
| **MXL120** | vctrs constructor returns `Self` / a named type, or the vctrs impl has an instance-method receiver | Return the plain vector payload; turn instance methods into static methods that take the vector data as a parameter |

### Warnings (P0: high impact)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL106** | Non-`pub` function carries a `/// @export` tag | Make it `pub fn`, or drop the tag to keep it private |
| **MXL111** | `s4_*` instance method on `#[miniextendr(s4)]` impl | Drop the `s4_` prefix — codegen auto-prepends it (you'd get `s4_s4_*`) |
| **MXL121** | R code can never create the type: no impl block of it has a constructor, no `#[miniextendr]` function or method returns it (`T`, `Self`, `ExternalPtr<T>`, bare or in `Result` / `Option`), and the crate builds no `ExternalPtr<T>` itself (`ExternalPtr::new(T { .. })` / `ExternalPtr::new(T(..))`, also of a local bound earlier in the same function by `let x = T { .. }` or `let x: T = ..`, or `ExternalPtr<T>` written as a type or turbofish anywhere outside a parameter, as in a wrapper whose hand-written `IntoR` builds the pointer); vctrs classes are skipped | Mark a constructor with `#[miniextendr(<system>(constructor))]`, or add a `#[miniextendr]` function returning the type. A factory the lint cannot see (one returning a bare `SEXP` without naming `ExternalPtr<T>`, or `ExternalPtr::new(value)` of a value from a call whose type isn't written, as in `let record = make_record(inner); ExternalPtr::new(record)`, or code inside a macro call such as `vec![..]`) is the case for `let value: T = ...` or `// mxl::allow(MXL121)` on the impl |

### Warnings (P1: important)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL203** | `internal` + `noexport` redundancy | Remove `noexport` and keep `internal`, which already suppresses `@export` (the function macro rejects the pair too) |
| **MXL204** | Doc link `[pkg::topic]` into an R package that is neither the package itself, a `DESCRIPTION` dependency, nor shipped with R | For a Rust item, write `[text][crate::path]`; for an R package, add it to `DESCRIPTION` |

### Warnings (P2: safety)

| Code | Description | Fix |
|------|-------------|-----|
| **MXL300** | Direct `Rf_error`/`Rf_errorcall` call | Use `panic!()` or return `Err(...)` instead |
| **MXL301** | `_unchecked` FFI call outside guard context | Use the checked wrapper, or ensure you're inside `with_r_unwind_protect` |
| **MXL302** | `into_sexp()` inside a `vec!`/array literal (unprotected SEXPs across allocations) | Protect each element through a `ProtectScope`, or use the `IntoList` / `DataFrameRow` derives |
| **MXL304** | Call to, or declaration of, R's non-API `ATTRIB` / `SET_ATTRIB` | `SexpExt::has_attributes()`, `is_identical()`, `get_attr()` / `set_attr()`, or `DUPLICATE_ATTRIB` |
| **MXL305** | `Rf_eval` / `R_forceAndCall` / `Rf_applyClosure` / `R_CheckUserInterrupt` in a `#[miniextendr]` fn or method body: an R error or interrupt there jumps over the body's Rust frames and leaves the receiver borrowed | Evaluate through `RCall::eval` / `eval_with_handlers` / `try_eval_with_handlers` (or `Quoted::eval`); check interrupts with `#[miniextendr(check_interrupt)]` |

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

#### MXL305: Raw R evaluation in a `#[miniextendr]` body

The generated wrapper runs a `#[miniextendr]` fn or method body inside one `R_UnwindProtect`, and for a method it holds the receiver's borrow (see [Reentrant Calls](EXTERNALPTR.md#reentrant-calls)) until the body returns. `Rf_eval`, `R_forceAndCall` and `Rf_applyClosure` run R code; if that code raises an R error, R jumps straight to the wrapper's `R_UnwindProtect`, over every Rust frame of the body. No destructor in those frames runs, and the receiver stays borrowed: every later call on the object raises `miniextendr_borrow_conflict` for the rest of the session. `R_CheckUserInterrupt()` does the same with a pending interrupt. The lint flags these calls (and their `_unchecked` forms) in `#[miniextendr]` fns, in the methods of `#[miniextendr]` impl blocks, and in the default methods of `#[miniextendr]` traits.

Wrapping the call in `with_r_unwind_protect` does not help: it resumes the R error with `R_ContinueUnwind`, which jumps over the body's frames all the same.

**Preferred alternatives** (`miniextendr_api::expression`):

- `RCall::eval` / `eval_as` / `r_eval_str`: an R error comes back as `Err(REvalError)`; the caller's condition handlers don't see the evaluation.
- `eval_with_handlers` (or `RCall::eval_with_handlers`): the caller's handlers see every condition, and an R error or other exit leaves as a Rust unwind, so the borrow is released before R carries on.
- `try_eval_with_handlers`: the caller's handlers see warnings and messages, and an R error comes back as `Err`.
- `Quoted::eval()` for an unevaluated argument.
- For `R_forceAndCall` / `Rf_applyClosure`, build the call with `RCall::from_sexp(f).arg(..)` and evaluate it with one of the above.

For interrupts, `#[miniextendr(check_interrupt)]` checks once, before the arguments are converted and the receiver is borrowed. There is no safe check for inside a long loop yet ([#2101](https://github.com/A2-ai/miniextendr/issues/2101)).

**When the jump is intentional** (a fixture that tests R's own longjmp), suppress with `// mxl::allow(MXL305)`.

## Inline Suppression

MXL300, MXL301, MXL302, MXL304 and MXL305 support inline suppression via `// mxl::allow(...)` comments. The suppression comment can appear:

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
- Only `MXL300`, `MXL301`, `MXL302`, `MXL304` and `MXL305` are suppressible this way; `MXL121` takes the comment on the impl block's header instead (see its row above)

## Common Proc-Macro Errors

### "a function takes at most one `...`"

The parameter of type `&Dots` is R's `...`, at any position, and an R function has one `...`. Two `&Dots` parameters, Rust `...` next to an explicit `&Dots`, or a `&Dots` next to a `LazyDots` (the dots passed unforced), are refused:

```rust
// Wrong: two dots parameters
#[miniextendr]
fn bad(x: i32, a: &Dots, b: &Dots) -> i32 { x }

// Wrong: `...` and `rest: &Dots` are both the dots
#[miniextendr]
fn also_bad(rest: &Dots, x: i32, more: ...) -> i32 { x }

// Wrong: `&Dots` and `LazyDots` are both the dots; keep one
#[miniextendr]
fn mixed(a: &Dots, b: LazyDots) -> i32 { 0 }

// Correct: one dots parameter; `flag` follows it and is matched by name in R
#[miniextendr]
fn good(x: i32, rest: &Dots, flag: bool) -> i32 { x }
```

### "`LazyDots` must be the parameter's whole type, taken by value"

A `LazyDots` parameter is R's `...`, which the wrapper passes as its own frame; it is always present. `&LazyDots`, `Option<LazyDots>` and `Missing<LazyDots>` are refused (take `rest: LazyDots`), and so are a per-parameter option on it (`default`, `coerce`, `match_arg`, `no_na`, ...), `dots = typed_list!(..)` (which validates the forced `list(...)` of a `&Dots` parameter), a trait method taking it (its arguments cross the trait ABI as converted values) and an `extern "C-unwind"` function taking it (no generated wrapper passes the frame). See [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md#dots-left-unforced-lazydots).

### "dots expects `typed_list!(...)`, `empty`, `empty(allow = [...])`, `forward(allow = [...])` or `data`"

`#[miniextendr(dots = ...)]` takes `typed_list!(..)` (validate the forced `list(...)`) or the role of `...` (#2157): `empty`, `empty(allow = ["name", ...])`, `forward(allow = ["name", ...])` or `data`. `empty(..)` and `forward(..)` take `allow` (a string or an array of distinct, non-empty strings, not `"..."`) and `class` (the package's classes for the refusal), each once. `forward` needs `allow`: with nothing listed it would take nothing, which is `empty`; `data` takes every argument, unchecked, and has no options. Every problem in one attribute is reported at once. See [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md#refusing-unused-arguments-the-dots-role).

### "`dots = empty` checks the R wrapper's `...`, and this one has none"

`empty` checks the wrapper's `...` formal. A standalone function has one only when it takes a dots parameter (`_: ...`, `&Dots` or `LazyDots`). An S3, S4 or S7 instance method also has the generic's dispatch `...`, unless `s3(no_dots)` / `s7(no_dots)` removes it, and so does a vctrs protocol method. R6 and env methods, and static methods, have only their Rust parameters. Without a `...`, R already refuses an argument the function has no formal for. Similarly, "`dots = forward` (or `data`) passes arguments to the function, which takes none": both hand the arguments to the function, so add a dots parameter.

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

### "function `f` has `/// @export` but is not `pub`"

Only `pub` functions get an automatic `@export` in R wrappers, so a private
function with a hand-written `/// @export` tag is contradictory (MXL106; the tag
still exports it). Make the function `pub`, or drop the tag. To keep a `pub`
function unexported, use `#[miniextendr(noexport)]` or `#[miniextendr(internal)]`.

### "unknown impl block option `label`"

Impl blocks need no labels: the wrapper writer merges the `#[miniextendr]` impl
blocks of a type into one class (#1776). Remove `label = "..."`.

### Impl blocks of one type that don't merge

`R CMD INSTALL` stops in the wrapper writer, naming the impl blocks, when the
blocks of a type break a rule of
[Multiple Impl Blocks](CLASS_SYSTEMS.md#multiple-impl-blocks):

- ``two impl blocks of `MyType` each define the class``: only one block may have
  the constructor (declared, or inferred from the block's one method returning
  `Self`) and S7 properties. The message names what each block has; move the
  later block's into the earlier one. Doc comments don't count: every block's
  doc comment documents the class.
- ``` `MyType` has several methods that could be its constructor ```: no block
  declares a constructor, and the blocks have more than one receiver-less method
  returning `Self` between them. See the compile-time form of this error below.
- ``the impl blocks of `MyType` give the class two titles``: two blocks' doc
  comments have different `@title`s. The message shows both; keep one.
- ``the impl blocks of `MyType` put the class on two help pages``: a block that
  does not define the class has a `@name` or `@rdname` other than the defining
  block's (its own tag, or the default `<Class>`). Move the tag to the block
  with the constructor, or remove it.
- ``the impl blocks of `MyType` disagree on the class settings``: spell the same
  class system and options, `class = "..."`, `internal` / `noexport` and
  `@noRd` / `@keywords internal` on every block. The message shows both sides.
- ``two impl blocks of `MyType` define ...``: two methods get one R member name
  (an env or R6 member, an R6 static method, an S4 or S7 method). Rename one
  with `r_name = "..."`.
- ``R class `MyType` is claimed by two Rust types``: two types (for example two
  `MyType`s in different modules) get one R class name. Give one a distinct
  `class = "..."`.

```rust
#[miniextendr(r6)]
impl MyType {
    pub fn new() -> Self { MyType }
    fn method_a(&self) -> i32 { 0 }
}

#[miniextendr(r6)]
impl MyType {
    fn method_b(&self) -> String { String::new() }
}
```

### "trait impl class system incompatible"

A trait impl's class system must match the inherent impl's class system (MXL008).
An `s3` trait impl works with every inherent class system. A trait impl does not
inherit the inherent impl's system: a bare `#[miniextendr]` counts as `env`.

```rust
#[miniextendr(s7)]
impl MyType { /* ... */ }

#[miniextendr]  // MXL008: env-style trait impl on an s7 type; write #[miniextendr(s7)] or (s3)
impl Display for MyType { /* ... */ }
```

### "`MyType` is not a type an `ExternalPtr` stores"

Types used in `#[miniextendr]` impl blocks, and the `T` of a `&T` / `&mut T`
parameter, are values R holds in an external pointer:

```rust
#[derive(ExternalPtr)]
struct MyType { /* ... */ }

#[miniextendr]
impl MyType { /* ... */ }
// Registration is automatic via #[miniextendr].

#[miniextendr]
pub fn read(x: &MyType) -> i32 { /* ... */ }
```

A `&T` parameter borrows an external value for the call (`EXTERNALPTR.md`,
"Arguments"). Only the reference types the macro reads from an R vector
(`&str`, `&[T]`, `&i32`, `&f64`, `&RLogical`, ...) mean something else; for
any other `T`, derive `ExternalPtr` or take the argument by value. If the
reference already implements `TryFromSexp` (including a type alias such as
`type Text = str`), use `#[miniextendr(convert)]` on the parameter instead.
On an inherent or trait impl method, use `#[miniextendr(convert(param))]`
on the method. See [explicit reference conversions](EXTERNALPTR.md#arguments).

### "takes `self` and is marked `constructor`"

`#[miniextendr(<system>(constructor))]` describes a receiverless `fn new(...) ->
Self`. A consuming builder step needs no marker: `fn step(self, ...) -> Self`
writes its result back into the same R object, `-> Result<Self, E>` /
`Option<Self>` run on a clone and overwrite on success (the type must be
`Clone`). Drop the attribute. See
[CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#consuming-receivers-self).

### "`MyType` has several methods that could be its constructor, and none is declared"

Without a declared constructor (`#[miniextendr(<system>(constructor))]`, or a
receiver-less `new` returning `Self`), a type's one receiver-less method
returning `Self` is its constructor
([Constructor Patterns](CLASS_SYSTEMS.md#constructor-patterns)). With several,
which one is meant is a guess, so the impl is refused: E0277 at compile time
when they are in one impl block, pointing at each candidate, and an install
error naming each with its block when they are in several.

```rust
#[miniextendr(r6)]
impl Shape {
    #[miniextendr(r6(constructor))]   // Shape$new(radius), and Shape$circle(radius); `square` stays Shape$square(side)
    pub fn circle(radius: f64) -> Self { /* ... */ }
    pub fn square(side: f64) -> Self { /* ... */ }
}
```

Mark one, or name it `new`: the others stay static methods. A marked method
keeps its own name as a static method too.

### "`a` and `b` are both marked `#[miniextendr(<system>(constructor))]`; a class has one constructor"

Drop the marker from one; it becomes a static method.

### R error: "`MyType` has no constructor: its objects come from the functions that return one."

A type without a constructor (no marked method, no `new`, and no
receiver-less method returning bare `Self`) has none to call. An R6 class's
`MyType$new()` and an S7 class's `MyType()` raise this error when called
without a `.ptr`, rather than build an object holding no pointer. Env and S3
leave `MyType$new` `NULL` (R's "attempt to apply non-function"), and S4 and
vctrs define no `MyType()` or `new_mytype()` (#2069). Call a factory instead:
a static method returning `Result<Self, E>` or `Option<Self>`, or another
function returning the type. To give the type a constructor, name one method
`new` or mark it `#[miniextendr(<system>(constructor))]`
([Constructor Patterns](CLASS_SYSTEMS.md#constructor-patterns)). When nothing
returns the type at all and the crate builds no `ExternalPtr` of it, the build
warns with MXL121.

### "Env member `Class$method` is defined by both ..."

Env constructors, instance methods and static methods share the class
namespace. A constructor always defines `Class$new`, and when its Rust name
is `create`, `Class$create` as well. Rename the other method with
`#[miniextendr(r_name = "...")]`.
A non-constructor `new` is allowed when the class has no constructor. The
check sees one impl block; a member bound by two impl blocks of a type is
reported when the package installs (see "Impl blocks of one type that don't
merge" above).

### "R6 member `initialize` conflicts with the generated constructor"

Every generated R6 class has an `initialize` member, even without a Rust
constructor. An instance method or active binding cannot reuse that name.
Rename the method, set `#[miniextendr(r_name = "reset")]`, or choose another
`r6(prop = "...")` for an active binding. A static method or a method marked
`constructor` may still be named `initialize` in Rust.

### "R6 static method `Class$new` would replace the R6 generator's own `new`"

R6 static methods are assigned onto the generator object that
`R6::R6Class()` returns, next to R6's own bindings (`new`, `set`, `lock`,
`classname`, `public_methods`, ...). The assignment replaces R6's binding
without a warning, so `Class$new()` would stop creating objects. Rename the
static method with `#[miniextendr(r_name = "...")]`. A constructor named
`new`, and instance methods or active bindings under these names, are fine. A
constructor under another name is also a static method under that name, so a
constructor named `set` is refused too: rename it, or give it
`r_name = "new"`, which leaves it only `Class$new()`.

### "S4 generic `s4_x` gets a method for `Class` from both `a` and `b`"

Each S4 instance method becomes a `setMethod()` on its generic for the class:
`s4(generic = "...")` when given, else `s4_` followed by the R method name
(`r_name`, `postfix`). Two methods of one impl block on the same generic would
register the same signature twice, and the second would replace the first.
Rename one with `#[miniextendr(r_name = "...")]`, or give it another
`s4(generic = "...")`. Two impl blocks of a type with a method on one generic
are reported when the package installs.

### "unsupported receiver type"

The R handle stores the value itself, so a method can take `self`,
`self: Self`, `&self`, `&mut self`, `self: ExternalPtrRef<'_, Self>`,
`self: ExternalPtrMut<'_, Self>` or `self: ExternalPtr<Self>`.
`self: Box<Self>`, `Rc<Self>`, `Arc<Self>` and friends cannot be handed over;
unwrap to `self` or take `&self`.

### "a method takes its object as `self: ExternalPtrRef<'_, Self>`"

`self: &ExternalPtr<Self>` and `self: &mut ExternalPtr<Self>` receivers are
gone (#2106). Take `&self` / `&mut self` when the method needs only the
value, `self: ExternalPtrRef<'_, Self>` / `mut self: ExternalPtrMut<'_, Self>`
when it also needs the R object (sidecar accessors, identity), or
`self: ExternalPtr<Self>` for the handle the wrapper doesn't borrow.

### "this parameter borrows its object for the call and cannot be `'static`"

A `&T` / `&mut T` / view parameter is borrowed for the call only. Elide the
lifetime, or take `ExternalPtr<T>` to keep a handle.

### "parameter `_x` becomes the R argument `x`, which ..."

Every parameter becomes an argument of the generated R function under its own
name with the leading underscores dropped (`_x` becomes `x`, `r#in` becomes
`in`). Callers pass arguments by name, so the macro never renames one. Instead
it rejects the names R cannot use:

- an R reserved word (`if`, `in`, `for`, `function`, `TRUE`, `NA`, ...) or a
  name that starts with a digit: the wrappers file would not parse;
- two parameters with the same R name (`x` and `_x`): R rejects a repeated
  argument name;
- a name the class system's wrapper binds itself: the receiver of an S3 or S4
  instance method (`x`, or the `receiver = "..."` of a method on an existing
  generic) or of an S7 one (its first dispatch argument, `x` by default), and
  R6's `self` and `private` inside the class (`initialize` and every method).

Rename the Rust parameter, or give it another R name with `formal = "..."`
(see below). Env methods, Env and R6 trait methods and S7 fast-path shortcuts
take their receiver as `.mx_self`, which no Rust parameter can spell and no
`formal` may take, so they reserve nothing; neither do static methods and
constructors outside R6, which are plain functions.

### "parameter `conf_int` is declared as the R argument `conf int`, which ..."

`formal = "..."` (an impl method's `formal(p = "...")`) names the parameter's R
formal ([Declared R formals](MINIEXTENDR_ATTRIBUTE.md#declared-r-formals-formal)).
The error points at the string, and names what is wrong with it:

- it is not a syntactic R name (letters, digits, `.` and `_`, starting with a
  letter or with a `.` not followed by a digit), or it is a reserved word;
- it is `...` or `..1`: R's dots, which only the dots parameter becomes;
- the generated wrapper uses the name itself: `.call`, `.val`, `.ptr`, an S3
  dispatch variable (`.Generic`, `.Class`, ...), a name starting `.mx_`,
  `.miniextendr_`, `.rdata_` or `.__MX_`, or a method's receiver.

A declared formal equal to another formal is the "both become the R argument"
error above, with "(a declared `formal` names the R argument)". Every problem
of a signature is reported at once. Declare another name.

### "`formal` is given twice on this parameter"

A parameter has one R formal: keep one `formal = "..."` (or one
`formal(p = "...")` entry for `p` on a method).

### "`formal` takes the R argument name as a string, e.g. `formal = "conf.int"`"

Quote the name: `formal = "na.rm"`, not `formal = na.rm`.

### "`formal` on the variadic (...) parameter `rest`: its R formal is always `...`"

The dots parameter (`...`, `&Dots`, `LazyDots`) is R's `...`. An argument with
a dotted name is a parameter of its own, with `formal`.

### "`formal(conf_levl = "conf.level")` names a parameter this method doesn't have"

The key of a method-level `formal(...)` entry is the Rust parameter's name, as
in `defaults(...)`. Fix the spelling.

### "`formal(...)` on an S7 property getter, setter or validator"

S7 calls a property's getter, setter and validator with fixed arguments
(`self`, `value`), so a declared formal would name nothing. Drop it.

### "`formal(...)` is not supported on trait methods"

A trait method's R formals are its declaration's parameter names, which every
implementing type shares and the trait ABI's metadata records; neither the
`#[miniextendr]` trait nor a trait impl takes `formal`. Declare the dotted name
on a standalone function or an inherent impl method.

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

### "a method registers on an exported generic; name it `pkg::generic`", "a generic takes one `pkg::` qualifier", "`pkg` is not an R package name"

A `generic = "..."` value may name the generic's package, `pkg::generic`
(#2154). The package must be a valid R package name (ASCII letters, digits and
`.`; at least two characters; starts with a letter; doesn't end in `.`),
followed by one `::` and the generic's name. `pkg:::generic`, `a::b::c`,
`::generic` and `pkg::` are refused. See
[S3_METHODS.md](S3_METHODS.md#generics-from-a-suggests-package).

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

### "`call = ...` accepts `wrapper` (the call as written, the default), `caller` (...) or `none` (...)"

`call = parent`, `call = self` & co. name no attribution. The three values are
`wrapper` (the default), `caller` and `none`, as a path or a string; see
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#choosing-the-attribution-marker-attribute-crate-default).

### "`call = none` passes no call, so a `Call` parameter has nothing to receive"

`#[miniextendr(call = none)]` makes every condition of the function call-less
(`.call = FALSE`), and a `Call` / `CallerCall` parameter is bound from the
call the wrapper passes. Drop the parameter, or drop the option and use
`call = wrapper` / `call = caller` to choose the call it receives. See
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#no-call-at-all-none).

### "`call_arg` cannot be combined with `call = none`"

`call_arg` gives the wrapper a `.call` formal naming the call its conditions
report, and `call = none` reports none. Keep one of them. `call = none` is
also refused on an `extern "C-unwind"` function, which has no call slot.

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

### "per-parameter options (...) do not apply to the `NArgs` parameter `n`"

An `NArgs` parameter is no R argument: the generated wrapper passes `nargs()`
in its place (#1860). A `default`, a coercion or a check on it, written on the
parameter or named by a method's `defaults(n = ...)` / `no_na(n)` / ..., has
no formal to act on. Drop the option. The same family covers `a #[miniextendr]
function takes at most one NArgs parameter` (both would receive the same
count) and `` `NArgs` on an `extern "C-unwind"` function `` (an extern
function has no generated R wrapper to pass `nargs()`; drop the `extern`, or
take a `SEXP` and pass the count from your own R code). See
[S3_METHODS.md](S3_METHODS.md#subscript-forms-and-the-argument-count).

### "Cargo.toml: [package.metadata.miniextendr] `call_attribution` must be one of ..."

The crate-wide default takes the same three values as the attribute, as a
string: `call_attribution = "wrapper"`, `"caller"` or `"none"`, set once. A
`"caller"` default applies to `noexport` / `internal` free functions only;
exported functions keep `wrapper`. `"none"` applies to every free function.

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
`E: serde::Serialize + Display` (and without an `RConditionData` impl) is
already classed from its serde shape, so the bare `#[miniextendr(serde_error)]`
flag, `serde_error = true`, `serde_error = false` and an empty `serde_error()`
would switch nothing on or off. The attribute only carries options: write
`serde_error(shape = ENGINE)`, options such as
`serde_error(prefix = "..", skip(..), as_character(..))`, or drop it. To give
every function returning a type the same options, declare them once with
`serde_error_shape!(Error = ENGINE)` instead. See
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

### "unknown serde_error option; expected `shape`, `tag`, `prefix`, `skip(...)`, `rename(...)`, `as_character(...)`, `as_integer(...)`, `as_double(...)` or `as_logical(...)`"

`serde_error(...)` takes `shape = PATH`, `tag = "..."` and `prefix = "..."` as
name-value pairs and `skip("a", "b")`, `rename(a = "b")` and
`as_character("a")` / `as_integer(..)` / `as_double(..)` / `as_logical(..)`
as nested lists. The list options are the payload-field controls;
`skip = "a"`, `rename = "b"` and `as_integer = "n"` are rejected with a
message pointing at the list form. See
[CONDITIONS.md](CONDITIONS.md#payload-fields-named-message) and
[CONDITIONS.md](CONDITIONS.md#field-types-and-shared-shapes).

### "serde_error rename target `message` is reserved"

`rename(from = "to")` may not target `message`, `call` or `kind`: those are
the condition's own slots, and the rename would recreate the collision the
option exists to avoid. Pick another name. The same family covers
`serde_error skip names `a` twice`, `serde_error rename names `a` twice`, and
`serde_error names `a` in both skip and rename`: each field appears in at most
one place.

### "serde_error names `n` in both as_integer and as_double; a field has one R type"

Each `as_*` option gives a payload field one atomic R type, so a field may be
named by only one of them. Keep the one you want. Within one function every
conflict is reported at once, each on the name that repeats an earlier option.
To give a field a different type than a declared or named shape does, name
it in the function's own options: those apply after the shapes and win
([CONDITIONS.md](CONDITIONS.md#field-types-and-shared-shapes)).

### "serde_error names `message` in both skip and as_character; a skipped field has no R type"

`skip` drops the field, so a type for it has nothing to act on. Drop one of
the two. `serde_error as_integer names `n` twice` and
`serde_error as_double takes a list of field names` belong to the same family.

### "serde_error shape takes the path of a `SerdeErrorShape` const: `shape = ENGINE`"

`shape = ` names a `const` (or `static`) of type
`miniextendr_api::condition::SerdeErrorShape` by path, as Rust code would
refer to it from the function's module: `shape = ENGINE`,
`shape = crate::errors::ENGINE`, or on a method `shape = Self::SHAPE`. A string, a list (`shape(ENGINE)`), the bare
word and a qualified path (`<T as Tr>::X`) are rejected, as is `shape` given
twice. A path that does not resolve, or names a value of another type, is
rustc's own error at the path.

### "evaluation panicked: SerdeErrorShape: field `n` is in both `as_integer` and `as_double`; a field has one R type"

error[E0080] on a `SerdeErrorShape` const: the builder methods check their
fields against the options already set and panic during const evaluation, so
a contradictory shape does not compile. The message names the field and the
two options; the span is the const. The checks mirror the attribute's: a
field in two `as_*` lists, a field skipped and also typed or renamed, a field
listed or renamed twice, two renames onto one name, a reserved `rename`
target, an empty list or name, and an option set twice (list every field in
one call).

### "conflicting implementations of trait `SerdeErrorShapeFor<__MiniextendrInitMarker>` for type `engine::Error`"

error[E0119]: a second `serde_error_shape!` for a type the crate already
declared a shape for. A declaration is a trait impl keyed by the crate's
marker type, so a type gets one per crate; the first declaration is marked
"first implementation here". Keep one. A function that needs other options
names them itself, `serde_error(as_double("n"))` or
`serde_error(shape = OTHER)`; they apply after the declared shape
([CONDITIONS.md](CONDITIONS.md#field-types-and-shared-shapes)).

### "the trait bound `engine::Error: serde::Serialize` is not satisfied" at `serde_error_shape!`

error[E0277] on the type in a `serde_error_shape!` declaration, "required by a
bound in `SerdeErrorShapeFor`". The declared shape is applied on the serde
path, so the type must be `serde::Serialize + Display`; the declaration checks
both. Implement them for the type, or class it with `RConditionData`
instead.

### "`serde_error_shape!` needs the `serde` feature of miniextendr-api; enable it to class errors through serde"

Without the API crate's `serde` feature nothing is classed through serde, so
a declared shape would silently do nothing. Enable the feature, or drop the
declaration.

### "`serde_condition!` needs the `serde` feature of miniextendr-api; enable it to class errors through serde"

`serde_condition!(&err)` builds the condition a function returning
`Err(err)` raises through serde, which needs the API crate's `serde` feature
(#2217). Enable the feature, or build the `RCondition` another way: from an
`RConditionData` type, return it as `Err` directly.

### "the trait bound `MyError: serde::Serialize` is not satisfied" at `serde_condition!`

error[E0277], "required by a bound in `__mx_serde_condition_arg`": the error
given to `serde_condition!` must be `serde::Serialize + Display`, the serde
path the macro runs. An "expected `&_`, found `MyError`" mismatch there means
the error was passed by value: the macro takes a reference,
`serde_condition!(&err)`.

### "cannot find type `__MiniextendrInitMarker` in the crate root"

Under the API crate's `serde` feature, the `Err` arm of every
`#[miniextendr]` function or method returning a `Result` looks up a shape
declared with `serde_error_shape!`, keyed by a marker type that
`miniextendr_init!` defines at the crate root; `serde_error_shape!` and
`serde_condition!` name the marker too, and so does a rebuild hook's `Err`
(`#[externalptr(rebuild = path)]`). Call `miniextendr_api::miniextendr_init!()`
at the crate root (`lib.rs`), as every R package does, not inside a module. A
doctest with such a function needs it as well, at the top level next to a
`fn main() {}`.

### "tuple field 0 of variant `E::Raw` needs a `data` name"

`#[derive(RCondition)]` turns every field into a `data` entry named after
the field, and a tuple field has no name. Give it one with
`#[condition(rename = "name")]`, or drop it from the payload with
`#[condition(skip)]`. Named fields never hit this. See
[CONDITIONS.md](CONDITIONS.md#deriving-rcondition).

### "`kind` is one of the condition's own slots (`message`, `call`, `kind`)"

A `#[derive(RCondition)]` field (or its `rename` target) may not be
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

### "`f` defines the `$<-` method of `Class`, which `s3(r_data_accessors)` generates for the `#[r_data]` fields"

`s3(r_data_accessors)` generates `$`, `[[`, `$<-` and `[[<-` methods for the
class, and `s4(r_data_accessors)` `$` and `$<-`, so an impl method on one of
those generics would define the class's method a second time. Remove the
method or the option. To keep your own `$<-` / `[[<-` (a copy-on-modify
setter, say), use the getters-only form `s3(r_data_accessors = "get")`, which
generates only the reading methods. A method on another class
(`s3(generic = "$", class = "other")`) is no collision.

### "the getters-only form `r_data_accessors = "get"` applies to `s3`, `s4` and `env` classes"

R6 active bindings and S7 properties always come with their setters, so
`r6(r_data_accessors = "get")` and `s7(r_data_accessors = "get")` are
refused; use the bare `r_data_accessors`. The same family covers `unknown
r_data_accessors value` (the only value is `"get"`) and `vctrs classes have
no r_data_accessors` (a vctrs class uses the standalone
`Type_get_<field>()` / `Type_set_<field>()`).

### "`s3(r_data_accessors)` on `Type` needs the field helpers `#[derive(ExternalPtr)]` emits"

This one is raised when the package is installed, not at compile time: the
impl block asked for field methods, but the type has no `#[r_data]
RSidecar` selector or no `pub` `#[r_data]` field, so `#[derive(ExternalPtr)]`
emitted no field list for them to use. Add the selector and the fields, or
remove the option.

### "`reexport!` takes one or more `pkg::name` items, e.g. `reexport!(generics::tidy)`"

`reexport!()` with nothing in it re-exports nothing. List the other packages'
exports to re-export, or remove the invocation. See
[Re-exporting the generic](S3_METHODS.md#re-exporting-the-generic).

### "expected `::` after the package `generics`: an item is `pkg::name`"

Each item of `reexport!` names a package and one of its exports. The same
family covers the other shapes that aren't one `pkg::name` item, each at its
token:

- `expected the package name of pkg::name`: an item starts with something other
  than an identifier (`"generics"::tidy`). Write the package as identifiers
  joined by `.`.
- `expected the name of pkg::name, ...` and `expected an identifier after .`: a
  name that isn't identifiers joined by `.` (`pkg::name.2`). Write it as a
  string literal, `pkg::"name.2"`.
- `an item names a package and one of its exports, pkg::name; separate items
  with ,` (`generics::tidy::x`) and `unexpected token after pkg::name; separate
  items with ,` (`generics::tidy generics::glance`): put a `,` between items.
- `expected a pkg::name item before this ,`: two commas in a row. A trailing
  comma is fine.

### "`x` is not an R package name: ASCII letters, digits and `.`, at least two characters, starting with a letter and not ending in `.`"

The part before `::` is an R package, and R's rule for a package name
(Writing R Extensions, "The DESCRIPTION file") applies: `my_pkg` and `x` are
refused, `data.table` and `R6` are fine.

### "`_hidden` is not a syntactic R name; write a non-syntactic name as a string literal, `pkg::"_hidden"`"

A name written as identifiers must be a syntactic R name: ASCII letters,
digits, `.` and `_`, starting with a letter or a `.` not followed by a digit,
and not a reserved word (`function`, `if`, `TRUE`, ...). Rust keywords are
fine (`base::as.numeric`). Write any other name as a string literal:
`magrittr::"%>%"`, `base::"[.data.frame"`. The R code backticks it.

### "the re-exported name `a b` can't hold whitespace, control characters, `` ` ``, `\` or `"`"

A name written as a string literal goes verbatim into the generated R code
(between backticks) and into roxygen2's `@importFrom` tag (between quotes),
so it can't hold whitespace, control characters or those quoting characters.
The same family covers `a re-exported name can't be empty`, `` `café` is not
ASCII; R code in a package must be ASCII `` (`R CMD check` warns about
non-ASCII R code) and `a re-exported name written as a string literal takes no
suffix` (`"x"suffix`).

### "`tidy` is already re-exported from `generics` in this `reexport!`; a package exports one object under a name"

One invocation names the same name from two packages
(`reexport!(generics::tidy, broom::tidy)`). `export(tidy)` can export only one
object; keep the item of the package whose function you mean. Naming the same
`pkg::name` twice is fine. Across invocations the wrapper writer refuses it
(below).

### Install error: "`reexport!` re-exports `tidy` from 2 packages: ..."

`R CMD INSTALL` stops in the wrapper writer when the `reexport!` invocations of
the crate name one name from two packages. The message lists every item with
its `(file.rs:line:column)`; keep the items of one package.

### Install error: "`reexport!` re-exports `generics::tidy` (...), and the package's wrappers also bind `tidy` (...)"

The generated wrappers define the re-exported name too: a `#[miniextendr]`
function, a class, or the S3 generic of an `s3` impl block's method (a method
without `generic = "..."` defines a generic of its own name). The package's
own `tidy` would hide the imported one in its namespace, and `export(tidy)`
would export it instead. Remove the re-export, or rename the package's binding
(`r_name = "..."`, `class = "..."`). For an S3 method on the other package's
generic, give the method `s3(generic = "tidy")`: an explicit generic is
neither defined nor exported by the package.

The same error with "also export `tidy`" means a doc comment exports the name
itself (`@export tidy`, `@rawNamespace export(tidy)`, the workaround before
`reexport!`). `reexport!` writes the export; drop the other one.

Hand-written R files are not read: a function of the same name there is not
refused.

### "`rng(seed = sed)` names no parameter of this function"

`rng(seed = <param>)` (#2198) names the parameter whose value the wrapper
passes to `set.seed()`, by its Rust name: `rng(seed = _seed)` for a parameter
`_seed` (the R formal `seed`). Fix the name, or add the parameter. See
[Seeded Simulation](RNG.md#seeded-simulation-rng-seed).

### "`rng(seed = rng)` needs an R argument the wrapper evaluates"

The seed parameter must be an R formal the wrapper evaluates, since it passes
the value to `set.seed()`: not the `...`, an `NArgs` or a `Call` / `CallerCall`
parameter, the `&RngGuard`, or a `Quoted` / `Quosure` argument (`set.seed()`
would force it). Name the seed argument itself.

### "`rng(seed = ..)` records the seed in the result's `seed` attribute, and a unit return is `NULL`, which can't carry one"

The seed contract gives the result a `seed` attribute, and a function
returning `()` (or `Result<(), E>`, `Option<()>`, `Invisible<()>`) returns
`NULL`. Return the simulated values, or use plain `rng`. Reported with the
other `rng(seed = ..)` problems of the function at once.

### "`rng(seed = ..)` is for standalone functions and their `s3(...)` methods, not an impl method"

The seed contract is written into the R wrapper of a standalone function or an
`s3(...)` method; impl methods (every class system) and trait methods ("not a
trait method") refuse it. Write `rng` on the method, or give the seeded draws a
standalone function, for a `simulate()` method
`#[miniextendr(rng(seed = seed), s3(generic = "stats::simulate", class = ".."))]`.

### "unknown `rng(...)` option; expected `seed = <parameter>`"

`rng(...)` takes one option, `seed`. The same family covers `` `rng(...)`
takes `seed = <parameter>` `` (`rng()` is empty: write `rng` alone for the RNG
state without the seed contract), `` `rng(seed = ..)` names the seed parameter
by its Rust name, as an identifier `` (`rng(seed = "seed")`: drop the quotes)
and `` `seed` is set more than once ``.

## Debugging Tips

1. **Run [`just lint`](https://github.com/A2-ai/miniextendr/blob/main/justfile)** before building: it catches attribute issues earlier than compile errors
2. **Check NAMESPACE**: if a function exists in Rust but not in R, run [`just rcmdinstall && just devtools-document`](https://github.com/A2-ai/miniextendr/blob/main/justfile)
3. **Feature-gated modules**: use `#[cfg]` on `mod` declarations for conditional compilation
