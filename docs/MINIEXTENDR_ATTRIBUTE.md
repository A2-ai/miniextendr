# `#[miniextendr]` Attribute Reference

Complete reference for `#[miniextendr]` on every Rust item type.

## Dispatch Overview

`#[miniextendr]` adapts its behavior based on the item it's applied to:

| Item | Default Behavior | Example |
|------|-----------------|---------|
| `fn` | Generate C wrapper + R wrapper | `#[miniextendr] pub fn add(x: i32, y: i32) -> i32` |
| `impl` (inherent) | Env-class methods | `#[miniextendr] impl Counter { ... }` |
| `impl Trait for Type` | Trait ABI shims | `#[miniextendr(s3)] impl Display for Counter { ... }` |
| `trait` | Vtable + ABI generation | `#[miniextendr] pub trait Counter { ... }` |
| `struct` (any field count) | ExternalPtr | `#[miniextendr] struct Point { x: f64, y: f64 }` |
| fieldless `enum` | RFactor | `#[miniextendr] enum Color { Red, Green, Blue }` |

---

## Functions

```rust
#[miniextendr]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

Generates:
- C wrapper (`C_mypkg_greet` — prefixed with your crate's name for webR cross-package uniqueness, see `docs/WEBR.md`) handling SEXP conversion
- R wrapper (`greet <- function(name) { .Call(C_mypkg_greet, name) }`)
- `pub` functions get `@export`; non-pub get `@noRd`
- Each function documents its own page, `man/greet.Rd`, as in roxygen2; a
  block without a `@title` is titled with the function name. Grouping is
  opt-in: `@rdname topic` puts the function on `topic`'s page (give the
  functions of `zero_copy_tests.rs` `@rdname zero_copy_tests` to share
  `zero_copy_tests.Rd`), and `@describeIn` / `@name` work as in roxygen2
  (#1289).
- Each parameter the doc comment does not document gets a generated `@param`
  line (`(no documentation available)`, or the choice list of a `choices` /
  `match_arg` parameter), except in a block with `@describeIn`,
  `@inheritParams`, or an `@rdname` naming another page than its own or its
  file-stem page: there the page it joins, or the topic it inherits from,
  documents the arguments. On a file-stem page (`@rdname <file stem>`) the
  line is kept only when no function on the page documents that argument. A
  block that joins another topic also gets `@order NaN`, so that topic's own
  block names and titles the page whatever the R file order. See
  [Documentation pages for standalone methods](S3_METHODS.md#documentation-pages-for-standalone-methods)
  and [Parameters on shared pages](S3_METHODS.md#parameters-on-shared-pages).

### Doc comments to roxygen

The `///` comment is read line by line into the wrapper's `#'` block:

- Text before the first `@tag` line is leading prose. It becomes
  `@description` unless the comment has one, with its lines, blank lines and
  indentation, so a markdown list or a fenced block renders as one. Rustdoc
  intra-doc links (`` [`Foo`] ``) lose their brackets, outside code, and so
  does a roxygen2 link such as `[other_fn()]`, unless the crate sets
  `roxygen_prose_links = "keep"` ([below](#links-in-leading-prose)). A link
  that only rustdoc can resolve, such as `[crate::Foo]`, loses them under
  either setting and in tag text too
  ([rustdoc-only links](#rustdoc-only-links)).
- A tag runs from its `@tag` line to the next one.
- A multi-line tag (`@description`, `@details`, `@param`, `@return`,
  `@examples`, `@examplesIf`, `@section`, ...) keeps its blank lines, which
  start a new roxygen2 paragraph (or leave a blank line in an example). It
  also keeps the indentation of its continuation lines, less the one space
  after `///`. Trailing blank lines are dropped.
- roxygen2 markdown reads a continuation indented 4 or more spaces after a
  blank line as a code block, as rustdoc does.
- `@title`, `@keywords`, `@concept` and `@aliases` fold a wrapped line onto
  one line.
- A single-line tag (`@export`, `@noRd`, `@rdname topic`, ...) ends at its
  line. The lines after it, up to the next tag, stay in rustdoc only.
- rustdoc keeps the leading prose and those rustdoc-only lines, and drops
  every tag with its text.

```rust
/// @description First paragraph.
///
/// Second paragraph.
/// @examples
/// x <- c(1, 2) |>
///   sum()
```

gives `#' @description First paragraph.`, `#'`, `#' Second paragraph.`,
`#' @examples`, `#' x <- c(1, 2) |>`, `#'   sum()`.

#### Links in leading prose

rustdoc and roxygen2 (markdown on) share the `[...]` link syntax:
`[other_fn()]`, `[pkg::fn()]` and `[Topic]` are intra-doc links to Rust items
for `cargo doc` and `\link{}`s to R help topics for roxygen2. rustdoc resolves
`[other_fn()]` to an in-scope Rust fn and warns
`rustdoc::broken_intra_doc_links` when there is none, so the text cannot tell
which reader it was written for. By default leading prose is taken as written
for rustdoc: its links lose their brackets (`[other_fn()]` becomes
`other_fn()`, `` [`Foo`][crate::Foo] `` becomes `` `Foo` ``), so roxygen2 never
tries to resolve a Rust item as an R topic. Markdown links `[text](url)`, code
spans and fenced blocks are left alone, except rustdoc's inline form
`[text](crate::x)` (see [rustdoc-only links](#rustdoc-only-links)). The text
of an explicit tag (`@description`, `@details`, `@param`, ...) keeps its
links, except the [rustdoc-only](#rustdoc-only-links) ones.

A crate whose doc comments are written for R first sets the default once:

```toml
[package.metadata.miniextendr]
roxygen_prose_links = "keep"    # or "strip", the default
```

Leading prose then keeps its links, like an explicit `@description`:

```rust
/// `summary()`: 50 unless [set_threshold()] set another; see [stats::median()].
/// @param x An object.
#[miniextendr(s3(generic = "summary", class = "thing"))]
fn summary_thing(x: List) -> List { /* ... */ }
```

renders `\code{\link[=set_threshold]{set_threshold()}}` and
`\code{\link[stats:median]{stats::median()}}` in the Rd. Every link in the
crate's leading prose but the [rustdoc-only](#rustdoc-only-links) ones is then
roxygen2's to resolve, so a rustdoc link written in one of roxygen2's own
forms (`` [`RustType`] ``, `` [`Type::method`] ``) is a roxygen2 warning there
("could not resolve link", "refers to un-installed package"). Give it a
rustdoc-only target or move it to a rustdoc-only line (both below). To keep
the links of one block without the crate setting, write its prose under an
explicit `@description`. The key must be `"strip"` or `"keep"`, set once;
anything else is a compile error (see [MACRO_ERRORS.md](MACRO_ERRORS.md)).

A block that renders no help page (`noexport` without `internal`, or an
explicit `@noRd`; on an impl block, its methods too) keeps no links whatever
the setting: every link in its leading prose and in its non-code tag text loses
its brackets, since none could render and roxygen2 would still try to resolve
each one (rustdoc keeps them).

#### Rustdoc-only links

roxygen2 reads a link target `pkg::topic` as a link into the R package `pkg`,
everything before the last `::`. When `pkg` is a Rust path root or cannot be
an R package name, no R package can resolve the link, so it was written for
rustdoc. Such a link loses its brackets wherever it is: in leading prose under
either `roxygen_prose_links` setting, and in the text of every explicit tag
except the code ones (`@examples`, `@examplesIf`, `@usage`, `@eval`,
`@evalRd`, `@evalNamespace`, `@rawRd`, `@rawNamespace`). The target, bracketed
(with or without backticks), after `[text]`, or in rustdoc's inline form
`[text](target)`, is rustdoc-only when its `pkg` part is:

- `crate`, `self` or `Self`;
- not a valid R package name (ASCII letters, digits and `.`, at least two
  characters, starting with a letter and not ending in `.`): a path with two
  or more `::` (`a::b::c`), or a module with a `_` (`my_mod::f()`).

| Written | roxygen2 gets |
|---|---|
| `` [`crate::Foo`] `` | `` `crate::Foo` `` |
| `[Self::new()]` | `Self::new()` |
| `[the registry][crate::registry]` | `the registry` |
| `` [`Sources::prepare`][crate::Sources::prepare] `` | `` `Sources::prepare` `` |
| `` [`a::b::c`] ``, `[my_mod::f()]` | `` `a::b::c` ``, `my_mod::f()` |
| `[text](crate::x)`, `` [`Foo`](crate::Foo) ``, `[new](Self::new())` | `text`, `` `Foo` ``, `new` |

Everything else stays as written outside the default `"strip"` prose, because
roxygen2 may read it as its own link: `[fn()]`, `[pkg::fn()]`, `[topic]`,
`` [`topic`] ``, `` [`pkg::topic`] ``, `[text][topic]`. That includes
`` [`Type::method`] `` and `[Type::method]`, since R package names can be
capitalised (`R6`, `S7`, `Matrix`), and `[super::x]`, since `super` is a CRAN
package. Code spans and fenced blocks are never rewritten.

An inline link `[text](target)` loses its link when the target is a Rust path:
identifiers joined by at least one `::`, with an optional `()` or `!` suffix
and an optional rustdoc disambiguator prefix (`fn@`, `struct@`, `enum@`,
`trait@`, `macro@`, `mod@`, `type@`, `const@`, `static@`, `method@`, `field@`,
`variant@`, `union@`, `value@`, `prim@`, ...). Unlike the bracketed forms, the
root does not matter: roxygen2 never reads an inline destination as an R
topic, only as the web link `\href{crate::x}{text}` (dead, and without a
warning). Every other markdown link stays: any URL (`[text](https://a.b/c::d)`,
`[text](./x.html)`, since a `/`, `.`, `#`, `?` or lone `:` is never part of a
Rust path), and a bare `[text](Foo)`, which may be a relative URL.

| Written | roxygen2 gets |
|---|---|
| ``[`Serialize`](serde::Serialize)``, `[text](fn@crate::x)` | `` `Serialize` ``, `text` |
| `[text](Foo)` | `[text](Foo)` |

A bracketed ``[`Type::method`]`` or `[Type::method]` in tag text (or in leading
prose under `roxygen_prose_links = "keep"`) is the case the macro cannot
classify: it is a roxygen2 link into the R package `Type` when that package
exists. The lint rule MXL204 reports each `pkg::` link (outside a block that
renders no page, whose links never reach roxygen2) whose `pkg` is not the
package itself, not declared in `DESCRIPTION` (Depends, Imports, Suggests,
Enhances, LinkingTo) and not a base or recommended R package, and names the
`[text][crate::path]` rewrite below. It reads `DESCRIPTION` from
`<crate dir>/../../DESCRIPTION` and skips when there is none.

So to link a Rust item from text that reaches R without the link reaching
roxygen2, root its target at the crate: `` [`Sources::prepare`][crate::Sources::prepare] ``
is a link in rustdoc and `` `Sources::prepare` `` on the R page. Inside an
`impl`, `` [`Self::prepare`] `` does the same.

To keep a whole line out of R help, put it after a single-line tag
(`@export`, `@noRd`, `@rdname topic`, ...): the lines after such a tag, up
to the next tag, stay in rustdoc only.

```rust
/// Prepares the sources.
///
/// @export
///
/// Rust callers: see [`Sources::prepare`] and [`Prepared`].
#[miniextendr]
pub fn prepare_sources() { /* ... */ }
```

The R page gets "Prepares the sources." and rustdoc gets both paragraphs.

### Function Attributes

#### Visibility & Export

| Attribute | Effect |
|-----------|--------|
| `internal` | Add `@keywords internal`, suppress `@export` |
| `noexport` | Suppress `@export` only |
| `invisible` | Wrap R return in `invisible()` (same as an `Invisible<T>` return type) |
| `visible` | Force visible return (same as a `Visible<T>` return type) |
| `call = wrapper \| caller` | Which call conditions are attributed to (same as a `Call` / `CallerCall` parameter); `caller` needs `noexport` / `internal`. See [below](#condition-call-markers-and-defaults) |
| `doc = "..."` | Custom roxygen block (replaces auto-generated) |

```rust
#[miniextendr(internal)]
pub fn helper() -> i32 { 42 }

#[miniextendr(invisible)]
pub fn set_option(key: String, value: i32) -> i32 { /* ... */ }
```

##### Return visibility: markers and defaults

Nothing is invisible unless it says so, with one carve-out: a bare function
whose R value is `NULL` (no return type, `-> ()`, `Option<()>`,
`Result<(), E>` on success) returns `invisible(NULL)`, because a side-effect
call should not print `NULL`. Everything else is visible, **including the
receiver a method hands back for chaining** (`&mut self -> ()`,
`&mut self -> &mut Self`, `self -> Self`): `obj$mutate()` prints the object
at the console, like an R function returning `self` would; `obj$a()$b()` and
`obj |> a() |> b()` keep working either way.

Two spellings change the decision and do the identical codegen: the return
type markers `Invisible<T>` / `Visible<T>` (`miniextendr_api::{Invisible,
Visible}`, transparent newtypes around the real value, `IntoR` forwards to
`T`) and the attribute `invisible` / `visible` (on a function, or on a method
as `#[miniextendr(invisible)]` / `#[miniextendr(r6(invisible))]` & co.). A
marker and an attribute that disagree are a compile error; markers are
return-position only and cannot be nested.

```rust
use miniextendr_api::{Invisible, Visible, miniextendr};

#[miniextendr]
pub fn quiet_value() -> Invisible<i32> { Invisible(42) }   // withVisible()$visible == FALSE

#[miniextendr]
pub fn loud_null() -> Visible<()> { Visible(()) }          // prints NULL

#[miniextendr(r6)]
impl Counter {
    pub fn tick(&mut self) {}                                 // chainable, prints the counter
    pub fn tick_quietly(&mut self) -> Invisible<()> { Invisible(()) } // chainable, silent
    pub fn add(&mut self, k: i32) -> Invisible<&mut Self> { self.n += k; Invisible(self) }
    pub fn copy(&self) -> Invisible<Self> { Invisible(self.clone()) } // invisible classed copy
}
```

The marker is peeled before any other analysis, so `Invisible<Option<T>>`
keeps `Option`'s `None`-raises rule, `Invisible<Result<T, E>>` still raises
on `Err`, and `Invisible<Self>` still wraps the handle in the class. Trait
methods take the marker in the impl's signature (which must match the trait's
declaration): `fn poke(&mut self) -> Invisible<()>`.

Generated standalone sidecar setters and R6 active-binding setters return the
receiver invisibly by default. On a public sidecar field, use
`#[r_data(setter = "visible")]` or `#[r_data(setter = "invisible")]` to choose
explicitly; this can share the attribute with `prop_doc = "..."`. It controls
both the standalone setter and the class-integrated setter. S7 property setters
keep their existing visible default when the field option is absent.

For an inherent R6 setter method, the existing `Visible<T>` / `Invisible<T>`
return marker or `#[miniextendr(visible)]` / `#[miniextendr(invisible)]` option
also controls the generated active-binding setter branch. A direct call via
`activeBindingFunction("field", obj)(value)` returns `obj` with that visibility.
An unmarked binding setter stays invisible even though an ordinary unmarked
Rust method call is visible. R assignment (`obj$field <- value`) is always
invisible, regardless of the setter's choice.

```rust
use miniextendr_api::{ExternalPtr, miniextendr};
use miniextendr_api::externalptr::RSidecar;

#[derive(ExternalPtr)]
#[externalptr(r6)]
pub struct Settings {
    #[r_data]
    sidecar: RSidecar,
    #[r_data(setter = "visible")]
    pub label: String,
}

#[miniextendr(r6(r_data_accessors))]
impl Settings {
    pub fn new(label: String) -> Self { Self { sidecar: RSidecar, label } }
}
```

```r
obj <- Settings$new("before")
set_label <- activeBindingFunction("label", obj)
withVisible(set_label("after"))$visible  # TRUE; returns obj
withVisible(obj$label <- "assigned")$visible  # FALSE: R assignment
```

Two syntactic limits, shared with `Dots` and `Missing<T>`: a type alias or a
`use Invisible as Quiet` rename defeats the last-segment detection. The miss
is fail-safe (the value converts through the marker's `IntoR`; only the
visibility falls back to the default).

**Migrating from the old default.** Before #1213, R6 and Env chainable
tails and every trait-impl void method were `invisible(self)` /
`invisible(x)` implicitly. A package that relied on the silence adds
`#[miniextendr(invisible)]` or returns `Invisible<()>` /
`Invisible<&mut Self>` on those methods; pipes and `$`-chains need no change.

##### Explicit cross-class return wrapping

Use `WrapAsR6<T>`, `WrapAsS7<T>`, `WrapAsS4<T>`, `WrapAsS3<T>`,
`WrapAsEnv<T>`, or `WrapAsVctrs<T>` when a return needs a specific R class
system. The equivalent attribute is `#[miniextendr(wrap = "r6")]`, with
`"s7"`, `"s4"`, `"s3"`, `"env"`, and `"vctrs"` selecting the other systems.
Both spellings work on free functions, inherent methods, and trait impl
methods. Nested method syntax such as `#[miniextendr(env(wrap = "r6"))]`
is also accepted.

For example, an Env factory can return a usable R6 board:

```rust
use miniextendr_api::{WrapAsR6, miniextendr};

#[miniextendr(env)]
impl Factory {
    pub fn build(&self, w: i32, h: i32) -> WrapAsR6<Board> {
        WrapAsR6(Board::new(w, h))
    }

    #[miniextendr(wrap = "r6")]
    pub fn build_attr(&self, w: i32, h: i32) -> Board {
        Board::new(w, h)
    }
}
```

Both R methods finish with `Board$new(.ptr = .val)`, so callers can use
`factory$build(3L, 4L)$dimensions()`. The explicit system takes precedence
over automatic return-class detection. A trait factory can similarly return
`WrapAsR6<Self>` (in both the trait declaration and implementation), or put
`#[miniextendr(wrap = "r6")]` on an implementation returning bare `Self`.

The macro emits the chosen wrapping expression directly:

| Selection | R expression |
|---|---|
| R6 | `T$new(.ptr = .val)` |
| S7 | `T(.ptr = .val)` |
| S4 | `methods::new("T", ptr = .val)` |
| S3 / Env | `structure(.val, class = "T")` |
| Vctrs | Prepend `T` to the existing class vector, retaining vctrs parent classes |

The marker forwards ordinary `IntoR` conversion to its payload; it does not
turn an arbitrary external pointer into vctrs vector data. A vctrs payload
must already provide a valid vctrs representation, for example through
`#[derive(Vctrs, PreferVctrs)]`.

Put containers outside the marker: `Option<WrapAsR6<Board>>`,
`Result<WrapAsR6<Board>, Error>`, `Vec<WrapAsR6<Board>>`,
`Option<Vec<WrapAsR6<Board>>>`, or `Result<Vec<WrapAsR6<Board>>, Error>`.
The attribute accepts the same shapes with bare `Board` payloads. `Err`
retains normal error transport; vectors wrap each element. Explicit optional
class returns raise on `None`, including free functions and optional vectors:
the boundary unwraps the container before converting its class payload.
Ordinary free functions without explicit wrapping keep their existing
`Option<T>: IntoR` rules (including `NULL` for supported optional vectors). `Result<T, ()>` is not supported by explicit wrapping.
Put visibility outside the complete return, e.g.
`Invisible<Result<WrapAsR6<Board>, Error>>`.

The payload must be an owned named type. The final Rust path segment names
the R class; `Self` uses the implementing class name. No registry lookup or
cross-check is added: the selected class system and name must match the R
class definition, and a mismatch fails when R constructs or uses the object.
As with visibility markers, aliases and renamed imports do not select the
marker behavior. Different marker/attribute systems, nested wrapping markers,
argument-position markers, and combinations with `serialize` or `unwrap_in_r`
are compile errors. Raw `extern "C-unwind"` functions cannot use `wrap`.

##### S7 conversion return markers

`ConvertTo<T>` and `ConvertFrom<T>` add the type spellings of
`s7(convert_to = "Target")` and `s7(convert_from = "Source")`. They are
restricted to inherent S7 methods because these attributes register methods
with `S7::convert()` as well as wrapping the returned class.

```rust
use miniextendr_api::{ConvertFrom, ConvertTo, ExternalPtr, miniextendr};

#[miniextendr(s7)]
impl Fahrenheit {
    pub fn from_celsius(source: ExternalPtr<Celsius>) -> ConvertFrom<Self> {
        ConvertFrom(Self { value: source.value * 9.0 / 5.0 + 32.0 })
    }

    pub fn to_celsius(&self) -> ConvertTo<Celsius> {
        ConvertTo(Celsius { value: (self.value - 32.0) * 5.0 / 9.0 })
    }
}
```

`ConvertTo<T>` selects `T` as the target of an instance method with no
additional parameters. `ConvertFrom<T>` keeps `T` as the returned payload
(`Self` or the enclosing class type); its static method's sole source
parameter supplies the source class name. That parameter may be a named
class, a reference to it, or `ExternalPtr<Source>`. The equivalent attributes
keep the plain return types and state the target/source name explicitly.

Both spellings feed the existing S7 class-reference resolver, so registered
R class renames work for conversion methods. Ordinary method calls and
`S7::convert(from, to)` use the same return wrapping. This also distinguishes
conversion markers from `WrapAsS7<T>`, which explicitly wraps a return without
registering an S7 conversion.

`Result<ConvertTo<T>, E>` / `Result<ConvertFrom<Self>, E>` preserve error
transport; `Option` variants raise on `None`. Put `Invisible` or `Visible`
outside the complete return. Visibility applies to both ordinary method
calls and `S7::convert()`. Conversion returns represent one class instance,
so vectors of class instances are rejected. Conflicting marker/attribute
names, incompatible class systems, wrong source/receiver arity, and
`serialize`/`unwrap_in_r` combinations are compile errors. As with the other
return markers, renamed imports and type aliases do not select the syntax.

#### Condition call: markers and defaults

Every generated wrapper passes a call object to its C entry point
(`.Call(C_pkg_f, .call = <call>, ...)`), and conditions raised from Rust are
attributed to it, as written. Which call it is has two attributions and three
equivalent spellings (#1566), most specific first:

| | `wrapper` (default) | `caller` |
|-|---------------------|----------|
| `.call =` | `sys.call()` | `.mx_call`, the caller's call, or the frame / call passed as `.call` |
| Marker parameter | `call: Call` | `call: CallerCall` |
| Attribute | `call = wrapper` | `call = caller` |
| `Cargo.toml` default | `call_attribution = "wrapper"` | `call_attribution = "caller"` |

```rust
use miniextendr_api::{Call, CallerCall, miniextendr};

#[miniextendr]
pub fn scale(x: f64, call: Call) -> f64 {      // R wrapper: scale <- function(x)
    let _ = call.sexp();                        // the wrapper's sys.call()
    x * 2.0
}

#[miniextendr(noexport)]                        // behind a hand-written scale2()
pub fn scale2_impl(x: f64, _call: CallerCall) -> f64 { x * 2.0 }
```

The marker (`miniextendr_api::{Call, CallerCall}`, `repr(transparent)` over
`SEXP`) is not an R formal: the C wrapper binds it from its hidden call slot,
so the body sees exactly the call the wrapper attributed to. A function taking
one runs on the main thread. `caller`, in either spelling, requires
`noexport` / `internal`; a crate default of `"caller"` applies to those
functions only and leaves exported ones at `wrapper`. Two spellings on one
function must agree, a function takes at most one marker, per-parameter
options do not apply to it, and class / trait methods accept none of the three
(they keep the wrapper's own call). A `caller` standalone wrapper (S3 methods
aside) ends its formals with `.call = NULL`, so a hand-written helper in
between can pass on its caller's frame (`call = parent.frame()`) or a call
([A helper in between](CALL_ATTRIBUTION.md#a-helper-in-between-call)). The
attribution is independent of `no_preconditions`, which on an impl block
applies to trait impls too. Details and the fixtures:
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#choosing-the-attribution-marker-attribute-crate-default).

#### Unevaluated arguments: `Quoted` / `Quosure`

A `Quoted` parameter stays an R formal that the wrapper passes unevaluated, as
`list(substitute(x), parent.frame())`; a `Quosure` parameter passes
`rlang::enquo(x)` (the package then needs rlang in `Imports:`). Rust reads the
expression and its environment and evaluates it in the caller's R context,
against a data frame's columns if it likes. Both work under `Missing<..>`, on
standalone functions and standalone S3 methods only, without per-parameter
options, and keep the function on the main thread. See
[QUOTED_ARGUMENTS.md](QUOTED_ARGUMENTS.md).

#### R-side preconditions: markers and defaults

A generated wrapper checks each argument's R type before the `.Call()`, one
guard per check (`if (!isTRUE(is.integer(n))) .miniextendr_arg_error("n",
"must be integer")`), so a wrong argument fails in R with a message naming it.
These are the *type-derived* checks. Without them the Rust conversion still
refuses the argument with the same condition (`kind = "conversion"`,
`e$param`, plus `e$rust_type`); only the message is the conversion's (`'n'
must be a single integer: got character`). Each parameter keeps or drops them
as the most specific spelling says (#1566):

| Rank | Keeps | Drops | Where |
|------|-------|-------|-------|
| 1 | `n: Checked<T>` | `n: Unchecked<T>` | a parameter of a function or inherent-impl method |
| 1 | `#[miniextendr(preconditions)] n: T` | `#[miniextendr(no_preconditions)] n: T` | a function parameter |
| 1 | `preconditions(n, m)` | `no_preconditions(n)` | a method attribute (inherent and trait impls) |
| 2 | `preconditions` (or `= true`) | `no_preconditions` (or `= true`) | the function attribute; bare on a method |
| 3 | `preconditions` | `no_preconditions` | the impl-block attribute (trait impls too) |
| 4 | `preconditions = true` | `preconditions = false` | `[package.metadata.miniextendr]` in `Cargo.toml` |
| 5 | | the `no-preconditions-default` cargo feature | the build ([FEATURE_DEFAULTS.md](FEATURE_DEFAULTS.md#no-preconditions-default)) |
| 6 | the framework default | | |

```toml
[package.metadata.miniextendr]
preconditions = false        # every wrapper of the crate drops its type checks
```

```rust
use miniextendr_api::{Checked, List, Unchecked, miniextendr};

#[miniextendr(noexport)]
pub fn fit_impl(data: List, n_iter: Checked<i32>, tol: f64) -> f64 {
    let n = *n_iter;                            // or n_iter.into_inner()
    /* ... */
}

#[miniextendr]
pub fn scale_by(#[miniextendr(no_na)] factor: f64, xs: Unchecked<Vec<f64>>) -> Vec<f64> {
    xs.into_inner().into_iter().map(|x| x * factor).collect()
}
```

Under that crate default `fit_impl` keeps `n_iter`'s two guards and none for
`data` or `tol`; `scale_by` (in any crate) drops `is.double(xs)` and keeps
`factor`'s guards and its `no_na`. The per-parameter spelling
`#[miniextendr(no_preconditions)] xs: Vec<f64>` generates the same R. On a
class, the method-level forms do the same:

```rust
#[miniextendr(r6, no_preconditions)]
impl Sampler {
    #[miniextendr(preconditions(n))]         // n keeps its guards, scale does not
    pub fn draw(&self, n: i32, scale: f64) -> Vec<f64> { /* ... */ }
    pub fn reseed(&mut self, seed: Checked<i32>) { /* ... */ }
}
```

The switch covers the type-derived checks only: `inherits`, `not_inherits`, `no_na`, the
`match_arg` / `choices` validation, the `.call` validation and the Rust
conversion stay whatever the spelling. The markers
(`miniextendr_api::{Checked, Unchecked}`, `repr(transparent)`, `Deref` to
`T`) are peeled before any R-side analysis: the formals, `@param` lines and
the conversion see `T` (`e$rust_type` says `i32`), and the C wrapper wraps the
converted value before the call. They implement neither `TryFromSexp` nor
`IntoR`, so a type alias of a marker does not compile (the guards are R text
written from the syntax, so an alias could only silently follow the default).

Two keywords follow the pair rule, the last one written wins (`preconditions,
no_preconditions` at one level, a list naming one parameter twice). A marker
and a keyword on one parameter that disagree are a compile error, as are two
markers on a parameter, a marker not outermost (`Option<Checked<T>>`; write
`Checked<Option<T>>`), a rank-1 spelling on a parameter with no type-derived
check (`SEXP`, `Missing<T>`, `ExternalPtr<T>`, `&Dots`, a type the check table
does not know) or on a `match_arg` / `choices` parameter, a marker in a
`#[miniextendr]` trait's method signature (spell it on the impl), a marker
on an `extern "C-unwind"` function, a marker as a return type, and the
function attribute's `preconditions = true | false` on a parameter (write it
bare there). `Checked<u16>` with `coerce` keeps the
widened guard; `Checked<&str>` and `Checked<&[f64]>` work on both thread
paths. `Checked` means the R-side guard is kept: it is not an
overflow-checked conversion, nor the thread-checked FFI variants.

#### Threading

| Attribute | Effect |
|-----------|--------|
| `worker` | Run on worker thread (default if `worker-default` feature) |
| `no_worker` | Run on main R thread |

Functions taking or returning `!Send` framework types automatically run on the
main thread — the generated wrapper can only execute there. No attribute is
needed. This covers `SEXP` (parameters and returns), the R-backed views
(`AltrepSexp`, `RDVector`, `RDMatrix`, `RndVec`, `RndMat`, `ProtectedStrVec`),
and the GC-rooted owned handles (`BuiltDataFrame`, `DataFrameShape`) anywhere
in the return type, including nested inside `Result`, `Option`, or containers.
Arbitrary user `!Send` types can't be detected syntactically — mark those
functions `no_worker` explicitly.

```rust
#[miniextendr(no_worker)]
pub fn fast_add(x: i32, y: i32) -> i32 { x + y }

#[miniextendr]
pub fn inspect_sexp(x: miniextendr_api::sys::SEXP) -> i32 { /* ... */ }
```

#### Type Conversion

| Attribute | Effect |
|-----------|--------|
| `coerce` | Auto-coerce R types (e.g., double → int) |
| `no_coerce` | Reject type mismatches |
| `wrap = "r6"` | Explicitly wrap the return in R6 (also `s7`, `s4`, `s3`, `env`, `vctrs`); see explicit cross-class return wrapping above |
| `serialize` | Return the complete value through serde, equivalent to `AsSerialize<T>`; requires the `serde` feature (see [serde return transport](SERDE_R.md#returning-serde-values)) |
| `strict` | Panic on lossy conversions (i64/u64 overflow) |
| `no_strict` | Allow lossy conversions |
| `prefer = "..."` | Return type preference: `"list"`, `"externalptr"`, `"vector"`, `"native"`, `"auto"` |

```rust
#[miniextendr(strict)]
pub fn exact_value(x: i64) -> i64 { x }

#[miniextendr(prefer = "list")]
pub fn get_record() -> MyStruct { /* ... */ }
```

#### Parameter Attributes

Written on a single parameter of a standalone function:

| Attribute | Effect |
|-----------|--------|
| `coerce` | Coerce this argument only (see [COERCE.md](COERCE.md)) |
| `default = "..."` | R formal default (an R expression) |
| `match_arg` | Validate against the parameter type's `MatchArg` choices (`Option<T>`: `NULL` is no choice; `Missing<..>`: the formal keeps the choices (none under `no_default`) and an omitted argument is `Absent`; `Either<T, R>` / `Either<Vec<T>, R>` (`several_ok`): non-character input goes to `R`, so `R` must read more than character or factor input; see [ENUMS_AND_FACTORS.md](ENUMS_AND_FACTORS.md#omitted-choice)) |
| `choices("a", "b")` | Validate a string against a literal choice list (the formal is `c("a", "b")`; the same type layers and `no_default` as `match_arg`) |
| `several_ok` | With `match_arg` / `choices`: accept several values (`Either<Vec<T>, R>`: several values or a value of another kind; see [ENUMS_AND_FACTORS.md](ENUMS_AND_FACTORS.md#several-choices-or-another-value)) |
| `no_default` | With `match_arg` / `choices`: the R formal has no default (the bare name instead of the choice vector or `NULL`), and the generated `@param` line drops its omission note. An omitted `Missing<..>` argument is still `Absent`; any other type gets R's missing-argument error. Not with `default` (see [ENUMS_AND_FACTORS.md](ENUMS_AND_FACTORS.md#without-a-default-no-default)) |
| `inherits = "cls"` / `inherits("a", "b")` | R check `inherits(x, c(...))`: the argument must inherit from one of the classes |
| `inherits(class = "cls", message = "...")` / `inherits("a", "b", message = "...")` | The same check, failing with your message |
| `inherits(class = "cls", when(class = "data.frame", message = "..."))` | The same check; a refused value of a `when` class gets that message instead (see [Hints for common wrong classes](#hints-for-common-wrong-classes)) |
| `not_inherits = "cls"` / `not_inherits("a", "b")` | R check `!inherits(x, c(...))`: the argument must inherit from none of the classes (see [Refusing classes](#refusing-classes)) |
| `not_inherits(class = "cls", message = "...")` / `not_inherits("a", "b", message = "...")` | The same check, failing with your message |
| `no_na` | R check `!anyNA(x)`: the argument must not be (or contain) `NA`; `NaN` is refused too. On a type that reads more values as missing than `anyNA()` sees (`AsNumeric*`, `AsCharacter*`, and aliases or derived newtypes of them), the converted value is checked too. On an `Either` written out in the signature (not behind an alias), the check runs only after the conversion, for the arm taken (see below) |
| `no_na(message = "...")` | The same check, failing with your message |
| `preconditions` / `no_preconditions` | Keep or drop this argument's type-derived R checks, whatever the function, impl or crate says (the `Checked<T>` / `Unchecked<T>` spelling; see [R-side preconditions](#r-side-preconditions-markers-and-defaults)) |

```rust
#[miniextendr]
pub fn obj_summary(#[miniextendr(inherits = "pkg_obj")] x: List) -> String { /* ... */ }

#[miniextendr]
pub fn scale_by(#[miniextendr(no_na)] factor: f64, #[miniextendr(no_na)] xs: Vec<f64>) -> Vec<f64> { /* ... */ }
```

```r
if (!isTRUE(inherits(x, "pkg_obj"))) .miniextendr_arg_error("x", "must inherit from 'pkg_obj'")
if (!isTRUE(is.list(x))) .miniextendr_arg_error("x", "must be a list")
```

`inherits` and then `not_inherits` run before the parameter's type checks and
`no_na` after them, one guard per check (under `call = caller`, the same
guards raising with the caller's call).
A failure raises the same condition as a failed Rust conversion: the crate's
`conversion_error_class`, `rust_error`, `kind = "conversion"` and `e$param`,
with the message `'x' must inherit from 'pkg_obj'`
([ERROR_HANDLING.md](ERROR_HANDLING.md#type-conversion-errors)). The
`match_arg` / `choices` validation raises it too
(`'mode' should be one of "fast", "slow"`). An `Option<T>` parameter passes
`NULL` and a `Missing<T>`
parameter an omitted argument. Unlike the type checks, they stay under
`Unchecked` / `no_preconditions` / the crate's `preconditions = false`: the
Rust conversion does not repeat them. A
plain `f64` accepts `NA_real_` (it is a valid double; `Option<f64>` is the
NA-carrying form), so `no_na` is the way to refuse it before Rust sees it.
The `no_na` message says `'x' must not contain NA` for an argument that holds
several values (`Vec<T>`, slices, arrays, maps and lists, and the vector
markers such as `AsNumericVec` or `AsFromStrVec<T>`) and `'x' must not be NA`
for a scalar. It looks through `Option<T>`, `Missing<T>` and `Result<T, _>` to
`T`, and an `Either<L, R>` holds several values when either arm does.

The reading markers read some inputs as missing that `anyNA()` passes, so on
them `no_na` also checks the converted value, in the C wrapper right after the
conversion. `AsNumeric` / `AsNumericVec` refuse the text `"NA"` and blank
strings (and factor labels among those), and `NaN` read from the text `"NaN"`,
as the R guard refuses a numeric `NaN`. `AsCharacter` / `AsCharacterVec`
refuse a factor `NA` level (`factor(x, exclude = NULL)`) and an `NA` returned
by a class's `as.character()` method; `"NA"` and `""` stay values there. The
check follows the type, not its name, so a type alias (`type Dose =
AsNumeric`) or a `#[derive(TryFromSexp)]` newtype of a marker is checked too.
The condition is the R guard's (same classes, `kind`, `e$param`, message,
your `message` when given, no `e$rust_type`, the same call: the caller's
under `call = caller`). It stays under `no_preconditions`. On
`Option<AsNumeric>`, `NULL` is still "not given" and passes as `None`.

On a list or map, `no_na` checks the top-level elements only: R's `anyNA()`
on the list, and for a map of reading markers (`HashMap<String, AsNumeric>`)
each value as the marker reads it. `list(a = c(1, NA))` passes.

On an `Either<L, R>` (also under `Missing` or `Option`), `no_na` emits no R guard. The
check runs in the C wrapper after the conversion, for the arm the value
converted to. That arm makes `anyNA()`'s check on the input in Rust, then its
marker check, so every arm refuses what the guard refused. The exception is a
`DataFrame` arm, which refuses nothing: a data frame whose cells are `NA`
reaches it. A plain `DataFrame` parameter keeps the R guard, which refuses
any `NA` cell. With no guard in front, an input that neither arm converts gets
the conversion's refusal (`'x' must be raw or a single number: got length 2`)
before any NA check. A `match_arg` / `choices` parameter typed `Either` keeps
the R guard, because its conversion has no Rust check.

```rust
#[miniextendr]
pub fn value_or_table(#[miniextendr(no_na)] x: Either<AsNumeric, DataFrame>) -> String { /* ... */ }
```

```r
value_or_table(NA)                                    # Error: 'x' must not be NA
value_or_table(data.frame(id = 1:2, v = c(1, NA)))    # reaches the DataFrame arm
```

An arm can be optional. For an optional number or a table, write
`#[miniextendr(no_na, default = "NULL")] x: Either<Option<AsNumeric>, DataFrame>`.
The left arm is tried first, so `NULL` (given or by default) converts to
`Left(None)` and passes as not given; it never reaches the `DataFrame` arm.
`NA` and `"NA"` are still refused, with the same condition as on
`Either<AsNumeric, DataFrame>`: the check reads the input, so an `NA` that the
arm converts to `None` (as `Option<f64>` does) is refused too.

The whole `Either` can be optional instead: `Option<Either<AsNumericVec,
DataFrame>>` reads `NULL` as `None` before either arm is tried, and `None`
passes. A given value is checked by the arm it converted to, as above, so a
data frame with `NA` cells still reaches the `DataFrame` arm. An
`Option<DataFrame>` parameter is not an `Either`: it keeps the R guard
(`is.null(x) || !anyNA(x)`), which refuses any `NA` cell.

The macro finds an `Either` by the type as written. An `Either` behind a type
alias (`type NumberOrTable = Either<AsNumeric, DataFrame>`), a newtype or
`Result<Either<..>, ()>` keeps the R guard, so a data frame with `NA` cells is
refused again: write the outer `Either` out in the signature. The arms
themselves can be wrapped. A `#[derive(TryFromSexp)]` newtype arm or a
`Result<T, ()>` arm is checked as the `T` it wraps: with
`struct Table(DataFrame)`, `Either<AsNumeric, Table>` lets a data frame with
`NA` cells through, and a `Result<AsNumeric, ()>` arm still refuses `NA`.

The generated message states the rule (`'model' must inherit from
'pkg_model'`). To say where the object comes from instead, give the check a
`message`:

```rust
#[miniextendr]
pub fn fit_summary(
    #[miniextendr(inherits(
        class = "pkg_model",
        message = "`model` must be a `pkg_model` object; create one with `pkg_model()`."
    ))]
    model: List,
    #[miniextendr(no_na(message = "`weight` must be a number, not NA"))] weight: f64,
) -> String { /* ... */ }
```

```r
if (!isTRUE(inherits(model, "pkg_model"))) .miniextendr_arg_error("model", message = "`model` must be a `pkg_model` object; create one with `pkg_model()`.")
if (!isTRUE(is.list(model))) .miniextendr_arg_error("model", message = "`model` must be a `pkg_model` object; create one with `pkg_model()`.")
if (!isTRUE(is.double(weight))) .miniextendr_arg_error("weight", "must be double")
if (!isTRUE(length(weight) == 1L)) .miniextendr_arg_error("weight", "must have length 1")
if (!isTRUE(!anyNA(weight))) .miniextendr_arg_error("weight", message = "`weight` must be a number, not NA")
```

The `inherits` message also replaces the messages of the parameter's type
checks. Any value that is not a list of that class gets it: a number, `NULL`,
or a classed double. A parameter without a message keeps the generated type
messages. Under `no_preconditions`, and on a type without an R type check
(`Missing<T>`, `DataFrame`, custom newtypes), no type check carries the
message: a value of the right class that the Rust conversion refuses gets the
conversion's message. `no_na` keeps its own message.

The message becomes the condition message as written, without a `'model'`
prefix. Everything else stays the same: the classes, `kind = "conversion"`,
`e$param` and the call (under `call = caller`, the caller's). Inside
`inherits(...)` and `not_inherits(...)`, each string literal and each
`class = "..."` names one class, and one `message` covers all of them. Each
check takes at most one message, and it must not be empty. The macro escapes
it for the R string literal: quotes, backslashes, newlines and other control
characters, and non-ASCII text as `\u{..}` (R code in a package must be
ASCII).

Impl and trait methods cannot carry parameter attributes, so the same options
are method-level and name the parameter: `match_arg(p)`,
`match_arg_several_ok(p)`, `choices(p = "a, b")`, `choices_several_ok(p = "a, b")`,
`no_default(p, q)`, `inherits(p = "cls_a, cls_b")`, `not_inherits(p = "cls_a, cls_b")`,
`no_na(p, q)`. A message goes in parentheses after the parameter:
`inherits(p(class = "cls_a, cls_b", message = "..."))`,
`not_inherits(p(class = "cls_a, cls_b", message = "..."))`,
`no_na(p(message = "..."), q)`. At method level the classes are one
comma-separated string, as in `choices(p = "a, b")`, because a nested option
cannot hold a list of literals. At parameter level no class name is split, so
`inherits(class = "a, b")` names one class, `a, b`.

#### Hints for common wrong classes

Some wrong values are common enough to deserve their own advice: a data frame
passed where a model object is expected should be told which function takes a
data frame, but `NULL` or a number should not. A `when(...)` inside
`inherits(...)` gives a refused value of the listed classes its own message:

```rust
#[miniextendr]
pub fn set_dose(
    #[miniextendr(inherits(
        class = "pkg_model",
        message = "`model` must be a `pkg_model` object; start from `pkg_model()`.",
        when(
            class = "data.frame",
            message = "`model` must be a `pkg_model` object; start from `pkg_model()`. To name the dose column of a data frame, use `col_dose()`."
        ),
        when("pkg_fit", "pkg_results", message = "`model` is a fit; pass its `$model`."),
    ))]
    model: List,
) -> List { /* ... */ }
```

```r
if (!isTRUE(inherits(model, "pkg_model") || !inherits(model, "data.frame"))) .miniextendr_arg_error("model", message = "`model` must be a `pkg_model` object; start from `pkg_model()`. To name the dose column of a data frame, use `col_dose()`.")
if (!isTRUE(inherits(model, "pkg_model") || !inherits(model, c("pkg_fit", "pkg_results")))) .miniextendr_arg_error("model", message = "`model` is a fit; pass its `$model`.")
if (!isTRUE(inherits(model, "pkg_model"))) .miniextendr_arg_error("model", message = "`model` must be a `pkg_model` object; start from `pkg_model()`.")
if (!isTRUE(is.list(model))) .miniextendr_arg_error("model", message = "`model` must be a `pkg_model` object; start from `pkg_model()`.")
```

Each hint is one more guard ahead of the `inherits` guard, so the hints are
tried in the order written and the first one naming a class of the value is
raised. A value that passes `inherits` passes every hint: an object whose
class is `c("pkg_model", "data.frame")` is accepted. A hint changes the
message of a refusal, never what the function accepts. A value of no hinted
class gets the check's own message, or the generated
`'model' must inherit from 'pkg_model'` without one.

A hint raises the condition the check raises (the classes,
`kind = "conversion"`, `e$param`, the call, under `call = caller` the
caller's), its message is escaped like any other, and an `Option<T>` /
`Missing<T>` parameter still passes `NULL` / an omitted argument. Inside
`when(...)` the classes are spelled as in `inherits(...)` (each string
literal and each `class = "..."` names one class), and `message` is
required. At method level the classes are one comma-separated string:
`inherits(model(class = "pkg_model", when(class = "pkg_fit, pkg_results", message = "...")))`.

The hints of several `inherits` attributes on one parameter run in the order
written, and each one accepts every required class. A `when` class that
`inherits` requires is a compile error, since a value of that class passes
the check and never sees the hint. `when(...)` belongs to `inherits` only:
`not_inherits` has one message for every class it refuses. A class may be
both a hint class and a `not_inherits` class: a value of that class alone gets
the hint, and one that also inherits from a required class gets the
`not_inherits` refusal.

#### Refusing classes

`not_inherits` is the negative of `inherits`: the argument must inherit from
none of the listed classes. It takes the same spellings
(`not_inherits = "cls"`, `not_inherits("a", "b", message = "...")`, method
level `not_inherits(p = "a, b")` / `not_inherits(p(class = "a, b", message = "..."))`),
raises the same condition, lets `NULL` through on an `Option<T>` and an
omitted argument on a `Missing<T>`, and stays under `no_preconditions`.

Use it to keep a marker's checks while refusing the classes whose value the
marker would read wrongly. `AsNumeric` reads a `difftime` as its bare number,
which drops the unit, and a `Date` or date-time as days or seconds since 1970:

```rust
#[miniextendr]
pub fn set_interval(
    #[miniextendr(
        no_na,
        default = "NULL",
        not_inherits(
            "difftime",
            "Date",
            "POSIXt",
            message = "`tau` must be a plain number in the time unit of the data"
        )
    )]
    tau: Option<AsNumeric>,
) { /* ... */ }
```

```r
if (!isTRUE(is.null(tau) || !inherits(tau, c("difftime", "Date", "POSIXt")))) .miniextendr_arg_error("tau", message = "`tau` must be a plain number in the time unit of the data")
if (!isTRUE(is.null(tau) || is.numeric(tau) || is.logical(tau) || is.character(tau) || is.factor(tau))) .miniextendr_arg_error("tau", "must be NULL or numeric, logical, character, or factor")
if (!isTRUE(is.null(tau) || length(tau) == 1L)) .miniextendr_arg_error("tau", "must be NULL or have length 1")
if (!isTRUE(is.null(tau) || !anyNA(tau))) .miniextendr_arg_error("tau", "must not be NA")
```

The refusal runs before the type checks (after `inherits`, when both are
given). Here that order matters: `is.numeric()` is `FALSE` for a `difftime`, a
`Date` and a `POSIXt`, so the type check would refuse them first, with its own
message. A value of any other class passes the refusal and meets the type
checks as before: `list(1)` still gets `'tau' must be NULL or numeric,
logical, character, or factor`. So, unlike the `inherits` message, a
`not_inherits` message does not replace the type checks' messages. Without a
message, a failure reads `'tau' must not inherit from 'difftime', 'Date' or
'POSIXt'`.

The check is plain R on the argument, so it needs no type check: it also runs
on an `Either` parameter, which has none, and under `no_preconditions`.
`inherits` and `not_inherits` can share a parameter
(`inherits = "pkg_model", not_inherits = "pkg_model_v1"`); naming the same
class in both is a compile error.

`not_inherits("NULL")` refuses an explicit `NULL`, whose implicit class is
`"NULL"`. On a `match_arg` / `choices` parameter it runs before the choice is
matched, so `NULL` gets your message instead of becoming the first choice; see
[Refusing an Explicit `NULL`](ENUMS_AND_FACTORS.md#refusing-an-explicit-null).

#### Parameters named like a base function

A parameter may share its name with a base function that generated wrappers
call, such as `length`, `c`, `list`, `missing`, `attr` or `inherits`. The
wrapper then writes each call to that function as `base::length(...)`, so
omitting the argument, or passing a function there, affects only that argument:

```rust
#[miniextendr]
pub fn resample(overwrite: bool, length: f64) -> f64 { /* ... */ }
```

```r
if (!isTRUE(base::length(overwrite) == 1L)) .miniextendr_arg_error("overwrite", "must have length 1")
if (!isTRUE(is.double(length))) .miniextendr_arg_error("length", "must be double")
if (!isTRUE(base::length(length) == 1L)) .miniextendr_arg_error("length", "must have length 1")
```

Your own R code in the wrapper (`r_entry`, `r_on_exit`, `r_post_checks`, a
`default = "..."`) gets the same treatment, and so does the usage line: a
choice formal next to a parameter named `c` reads `mode = base::c("a", "b")`.
In an impl block, one method's parameter qualifies the calls of the whole
class. Only these calls are qualified: `base::` costs time on every call, so
wrappers without such a parameter keep the unqualified form.

#### Error Handling

| Attribute | Effect |
|-----------|--------|
| (default) | `Result::Err`, panics, and `Option::None` ride the tagged-condition transport: R wrapper raises a structured `rust_*` condition |
| `unwrap_in_r` | Return `Result<T, E>` to R as a list with an `$error` slot — `Err` is delivered as a value, not as an R error |

```rust
// Default: Err becomes a `rust_error` R condition
#[miniextendr]
pub fn parse_data(s: String) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| e.to_string())
}

// `unwrap_in_r`: Err is delivered as `list(value = NULL, error = "...")`
#[miniextendr(unwrap_in_r)]
pub fn parse_data_unwrap(s: String) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| e.to_string())
}
```

#### Miscellaneous

| Attribute | Effect |
|-----------|--------|
| `check_interrupt` | Insert `R_CheckUserInterrupt()` before call |
| `rng` | Manage R's RNG state (`GetRNGstate`/`PutRNGstate`) |
| `c_symbol = "..."` | Custom C function name |
| `lifecycle = "..."` | Mark as deprecated/experimental/superseded |
| `dots = typed_list!(...)` | Validate `...` arguments (see [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md)) |

```rust
#[miniextendr(check_interrupt, rng)]
pub fn long_simulation(n: i32) -> Vec<f64> { /* ... */ }

#[miniextendr(lifecycle = "deprecated")]
pub fn old_api() -> i32 { 0 }
```

#### Rust keywords as R names

Rust keywords are ordinary R names (`type`, `where`, `match`, `mod`, `ref`, ...).
Spell them as raw identifiers on the Rust side; the `r#` prefix is Rust syntax
only and is dropped from every generated name — R function and method names,
R formals, `@param` names, list element and data-frame column names, C symbols:

```rust
/// @param where Filter expression.
/// @param type Output type.
#[miniextendr]
pub fn r#match(r#where: &str, r#type: i32) -> String { /* ... */ }
// R: match(where, type)

#[derive(DataFrameRow)]
pub struct Row { pub r#type: String, pub r#where: i32 }
// R: data.frame(type = ..., where = ...)
```

This works in every name position: `#[miniextendr]` fn and method names,
parameters, named dots (`r#dyn: ...`), trait methods, struct fields in the
`IntoList` / `TryFromList` / `DataFrameRow` / `ExternalPtr` derives, and the
`typed_list!` / `typed_dataframe!` DSLs. Names that R itself reserves (`if`,
`for`, `in`, `while`, `break`) are still not valid R formals — use `r_name` or
a different name for those.

#### R Wrapper Customization

These attributes inject custom R code into the generated wrapper function, giving
fine-grained control over the R-side behavior without touching the Rust logic.

| Attribute | Effect |
|-----------|--------|
| `r_name = "..."` | Override R function name (e.g., `"is.widget"`) |
| `r_entry = "..."` | Inject R code at function entry (before all checks) |
| `r_post_checks = "..."` | Inject R code after checks (before `.Call()`) |
| `r_on_exit = "..."` | Register `on.exit()` cleanup (short form, `add = TRUE`) |
| `r_on_exit(expr = "...", add = bool, after = bool)` | Long form with full `on.exit()` control |

Generated wrapper layout:

```r
fn_name <- function(formals) {
  # r_entry code
  on.exit(...)         # r_on_exit
  # missing defaults, lifecycle, preconditions, match.arg
  # r_post_checks code
  .Call(C_mypkg_fn_name, ...)
}
```

```rust
// Rename R function (C symbol still derived from Rust name)
#[miniextendr(r_name = "is.widget")]
pub fn is_widget(x: i32) -> bool { x > 0 }

// Coerce input before Rust sees it
#[miniextendr(r_entry = "x <- as.integer(x)")]
pub fn process(x: i32) -> i32 { x * 2 }

// Validate after built-in checks
#[miniextendr(r_post_checks = "stopifnot(x > 0L)")]
pub fn positive_only(x: i32) -> i32 { x }

// Register cleanup code
#[miniextendr(r_on_exit = "message(\"done\")")]
pub fn with_cleanup(x: i32) -> i32 { x + 1 }

// Full on.exit control (LIFO order)
#[miniextendr(r_on_exit(expr = "close(con)", after = false))]
pub fn with_connection(x: i32) -> i32 { x }

// Combine all four
#[miniextendr(
    r_name = "widget.create",
    r_entry = "n <- as.integer(n)",
    r_on_exit = "message(\"cleanup\")",
    r_post_checks = "stopifnot(n > 0L)",
)]
pub fn create_widget(n: i32) -> i32 { n * 10 }
```

`r_on_exit` defaults: `add = TRUE`, `after = TRUE` (composable, FIFO, following standard R convention).
When `add = FALSE`: omits both `add` and `after` (R ignores `after` when `add = FALSE`).

#### S3 Standalone Functions

Functions can be standalone S3 methods without an impl block:

```rust
#[miniextendr(s3(generic = "format", class = "percent"))]
pub fn format_percent(x: SEXP, _dots: ...) -> Vec<String> { /* ... */ }
```

---

## Impl Blocks (Inherent)

```rust
#[derive(ExternalPtr)]
pub struct Counter { value: i32 }

#[miniextendr]       // default: env-class
impl Counter {
    pub fn new(initial: i32) -> Self { Counter { value: initial } }
    pub fn value(&self) -> i32 { self.value }
    pub fn increment(&mut self) { self.value += 1; }
}
```

### Class System Selection

| Syntax | System | R Pattern |
|--------|--------|-----------|
| `#[miniextendr]` | Env (default) | `obj$method()` environment dispatch |
| `#[miniextendr(r6)]` | R6 | `R6Class` with `$new()` |
| `#[miniextendr(s3)]` | S3 | `generic.Class(x, ...)` |
| `#[miniextendr(s4)]` | S4 | `setClass`/`setMethod` formal OOP |
| `#[miniextendr(s7)]` | S7 | `new_class`/`new_generic` modern OOP |
| `#[miniextendr(vctrs)]` | vctrs | vctrs-compatible S3 vector class |

### Impl-Level Attributes

| Attribute | Applies To | Effect |
|-----------|-----------|--------|
| `class = "..."` | All systems | Custom R class name |
| `label = "..."` | All systems | Distinguish multiple impl blocks on same type |
| `strict` / `no_strict` | All systems | Strict type conversion for all methods |
| `internal` | All systems | `@keywords internal` on class |
| `noexport` | All systems | Suppress `@export` on class |
| `preconditions` / `no_preconditions` | All systems, trait impls too | Keep or drop the type-derived R checks of every method whose parameters and own attribute say nothing (see [R-side preconditions](#r-side-preconditions-markers-and-defaults)) |
| `blanket` | Trait impls | Skip trait ABI (for blanket impls) |

```rust
// Two impl blocks need labels
#[miniextendr(s3, label = "core")]
impl Counter { /* constructors + getters */ }

#[miniextendr(s3, label = "mutations")]
impl Counter { /* mutating methods */ }
```

### R6-Specific Options

```rust
#[miniextendr(r6(
    inherit = "BaseClass",
    portable = true,
    cloneable = true,
    lock_objects = false,
    lock_class = false,
    r_data_accessors,
))]
impl MyClass { /* ... */ }
```

### S7-Specific Options

```rust
#[miniextendr(s7(
    parent = "ParentClass",
    abstract = true,
    r_data_accessors,
))]
impl MyClass { /* ... */ }
```

### vctrs-Specific Options

```rust
#[miniextendr(vctrs(
    kind = "vctr",          // vctr | rcrd | list_of
    base = "double",        // underlying R type
    inherit_base_type = true,
    ptype = "double(0)",    // prototype R expression
    abbr = "pct",           // vec_ptype_abbr
))]
impl Percent { /* ... */ }
```

### Method-Level Attributes

Methods inside impl blocks can have per-method attributes nested under the class
system keyword:

```rust
#[miniextendr(s3)]
impl Person {
    // Override the S3 generic name
    #[miniextendr(s3(generic = "print"))]
    pub fn show(&mut self) { println!("{}", self.name); }

    // Override the class suffix for double-dispatch
    #[miniextendr(s3(generic = "vec_ptype2", class = "my_vctr.my_vctr"))]
    pub fn ptype2_self(&self) -> SEXP { /* ... */ }

    // Generate as.data.frame.Person
    #[miniextendr(as = "data.frame")]
    pub fn as_df(&self) -> List { /* ... */ }

    // Skip this method
    #[miniextendr(ignore)]
    pub fn internal_helper(&self) { /* ... */ }

    // Parameter defaults
    #[miniextendr(defaults(n = "1"))]
    pub fn increment_by(&mut self, n: i32) { self.value += n; }
}
```

#### Shared Method Attributes (all class systems)

| Attribute | Effect |
|-----------|--------|
| `ignore` | Don't generate R wrapper for this method |
| `constructor` | Mark as constructor (factory method) |
| `generic = "..."` | Override S3/S4/S7 generic name |
| `class = "..."` | Override S3 class suffix |
| `as = "..."` | Generate `as.<target>()` coercion method |
| `defaults(p = "val")` | R-side parameter defaults |
| `worker` / `no_worker` | Thread override |
| `check_interrupt` | Insert interrupt check |
| `coerce` / `no_coerce` | Type coercion override |
| `wrap = "r6"` | Explicitly wrap the return in R6 (also `s7`, `s4`, `s3`, `env`, `vctrs`); see explicit cross-class return wrapping above |
| `serialize` | Convert the complete return value through `AsSerialize<T>` |
| `rng` | RNG state management |
| `unwrap_in_r` | Return `Result<T, E>` as a list with `$value`/`$error` instead of raising on `Err` |
| `r_name = "..."` | Override R method name |
| `r_entry = "..."` | Inject R code at method entry |
| `r_post_checks = "..."` | Inject R code after checks |
| `r_on_exit = "..."` | Register `on.exit()` cleanup |
| `match_arg(p)` / `choices(p = "a, b")` | Validate `p` with `match.arg()` (see [Parameter Attributes](#parameter-attributes)) |
| `no_default(p, q)` | The R formals of the choice parameters `p` and `q` have no default ([Parameter Attributes](#parameter-attributes)) |
| `inherits(p = "cls_a, cls_b")` | R check `inherits(p, c(...))` |
| `inherits(p(class = "cls_a, cls_b", message = "..."))` | The same check, failing with your message |
| `inherits(p(class = "cls", when(class = "df_a, df_b", message = "...")))` | The same check; a refused value of a `when` class gets that message instead ([Hints for common wrong classes](#hints-for-common-wrong-classes)) |
| `not_inherits(p = "cls_a, cls_b")` | R check `!inherits(p, c(...))`: `p` must inherit from none of the classes |
| `not_inherits(p(class = "cls_a, cls_b", message = "..."))` | The same check, failing with your message |
| `no_na(p, q)` | R check `!anyNA(p)`; on a type that reads more values as missing than `anyNA()` sees (`AsNumeric*`, `AsCharacter*`, and aliases or derived newtypes of them), the converted value is checked too; on an `Either`, only after the conversion, for the arm taken ([Parameter Attributes](#parameter-attributes)) |
| `no_na(p(message = "..."))` | The same check, failing with your message |
| `preconditions` / `no_preconditions` | Keep or drop the type-derived R checks of this method's parameters, over the impl block's (see [R-side preconditions](#r-side-preconditions-markers-and-defaults)) |
| `preconditions(p, q)` / `no_preconditions(p)` | The same for the named parameters only, over the method's own |

Valid `as = "..."` targets: `data.frame`, `list`, `character`, `numeric`, `double`,
`integer`, `logical`, `matrix`, `vector`, `factor`, `Date`, `POSIXct`, `complex`,
`raw`, `environment`, `function`.

#### R6-Specific Method Attributes

```rust
#[miniextendr(r6)]
impl MyClass {
    // Private method (R6 convention: prefixed with .)
    #[miniextendr(private)]
    pub fn internal_calc(&self) -> f64 { /* ... */ }

    // Active binding getter
    #[miniextendr(active, prop = "count")]
    pub fn get_count(&self) -> i32 { self.count }

    // Active binding setter
    #[miniextendr(setter, prop = "count")]
    pub fn set_count(&mut self, value: i32) { self.count = value; }

    // Destructor
    #[miniextendr(finalize)]
    pub fn cleanup(&mut self) { /* ... */ }
}
```

#### S7-Specific Method Attributes

```rust
#[miniextendr(s7)]
impl MyClass {
    // Computed property getter
    #[miniextendr(s7(getter, prop = "area"))]
    pub fn get_area(&self) -> f64 { self.width * self.height }

    // Property setter
    #[miniextendr(s7(setter, prop = "area"))]
    pub fn set_area(&mut self, value: f64) { /* ... */ }

    // Property validator
    #[miniextendr(s7(validate, prop = "width"))]
    pub fn validate_width(value: f64) -> Result<(), String> {
        if value < 0.0 { Err("width must be non-negative".into()) } else { Ok(()) }
    }

    // Property with defaults + constraints
    #[miniextendr(s7(getter, required))]      // no default, must be provided
    pub fn name(&self) -> String { self.name.clone() }

    #[miniextendr(s7(getter, frozen))]        // immutable after creation
    pub fn id(&self) -> i32 { self.id }

    #[miniextendr(s7(getter, default = "0"))] // R expression for default
    pub fn score(&self) -> f64 { self.score }

    // Remove ... from generic signature
    #[miniextendr(s7(no_dots))]
    pub fn length(&self) -> i32 { self.len }

    // Multiple dispatch: the receiver `x`, then the leading parameter `other`
    #[miniextendr(s7(dispatch = "x, other"))]
    pub fn combine(&self, other: &Self) -> Self { /* ... */ }

    // Ops operator: dispatches on (e1, e2); the operand is named `e2`
    #[miniextendr(s7(generic = "+"))]
    pub fn add(&self, e2: &Self) -> Self { /* ... */ }

    // Another package's generic, attached when that package loads
    #[miniextendr(s7(generic = "generics::tidy"))]
    pub fn tidy(&self) -> BuiltDataFrame { /* ... */ }

    // Type conversion methods
    #[miniextendr(s7(convert_from = "OtherClass"))]
    pub fn from_other(other: OtherClass) -> Self { /* ... */ }

    #[miniextendr(s7(convert_to = "OtherClass"))]
    pub fn to_other(&self) -> OtherClass { /* ... */ }
}
```

---

## Trait Definitions

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn default_initial() -> i32 { 0 }  // static method
}
```

Generates cross-package ABI:
- `TAG_COUNTER`: type tag constant
- `CounterVTable`: function pointer table
- Method shims: `extern "C"` trampolines with `with_r_unwind_protect`
- `CounterView`: runtime dispatch wrapper

**No attributes accepted** on the `#[miniextendr]` for traits (the attr parameter is unused).

**Constraints:**
- Methods must take `&self` or `&mut self` (not `self` by value)
- No async methods
- No generic method parameters (trait-level generics are OK)
- Static methods work (resolved at compile time, not in vtable)

### Trait Impl Blocks

```rust
#[miniextendr(s3)]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.val }
    fn increment(&mut self) { self.val += 1; }
}
```

Use `blanket` to skip ABI emission for blanket impls:

```rust
#[miniextendr(blanket)]
impl<T: AsRef<str>> Display for T { /* ... */ }
```

---

## Structs

### ALTREP via `#[miniextendr]` Is Removed

`#[miniextendr]` no longer generates ALTREP classes. Applying the old ALTREP
attributes (`class`, `base`) to a 1-field struct is a compile error with
migration guidance. Use the per-family ALTREP derives instead:

```rust
use miniextendr_api::prelude::*;

#[derive(AltrepInteger)]
#[altrep(len = "len", elt = "value", class = "MyInts")]
pub struct LazyInts {
    value: i32,
    len: usize,
}
```

See [ALTREP.md](ALTREP.md) for the full derive surface.

### Struct → ExternalPtr (default)

```rust
#[miniextendr]
pub struct Point { x: f64, y: f64 }
```

Generates `ExternalPtr` + `TypedExternal` derives. The struct lives as an opaque
R external pointer. This is the default for **any** field count — a bare 1-field
struct is ExternalPtr too, not ALTREP.

Override with an explicit mode:

```rust
#[miniextendr(externalptr)]
pub struct Wrapper(Vec<i32>);   // ExternalPtr (same as the default)

#[miniextendr(list)]
pub struct Wrapper(Vec<i32>);   // List conversion instead of ExternalPtr
```

### Struct Mode Overrides

`prefer = "..."` is not always marker-only. When no explicit mode attribute
(`list` / `dataframe` / `externalptr`) is also present, `prefer = "list"` and
`prefer = "dataframe"` resolve to the **same full mode** as their explicit
counterparts — not a lightweight ExternalPtr-plus-marker combination. Only
`prefer = "native"` has no explicit-mode counterpart, so it stays
ExternalPtr plus a marker derive. See `expand_struct` in
`miniextendr-macros/src/struct_enum_dispatch.rs` for the resolution logic.

| Syntax | Result |
|--------|--------|
| `#[miniextendr]` (no mode attr) | ExternalPtr |
| `#[miniextendr(list)]` | `IntoList` + `TryFromList` + `PreferList` |
| `#[miniextendr(dataframe)]` | `IntoList` + `DataFrameRow` + companion type |
| `#[miniextendr(externalptr)]` | `ExternalPtr` + `TypedExternal` |
| `#[miniextendr(prefer = "list")]` | Same as `#[miniextendr(list)]` — full list mode (`IntoList` + `TryFromList` + `PreferList`), not just a marker |
| `#[miniextendr(prefer = "dataframe")]` | Same as `#[miniextendr(dataframe)]` — full dataframe mode (`IntoList` + `DataFrameRow` + companion type), not just a marker |
| `#[miniextendr(prefer = "externalptr")]` | ExternalPtr (explicit; identical to the no-mode-attr default) |
| `#[miniextendr(prefer = "native")]` | `ExternalPtr` + `PreferRNativeType` marker (the only `prefer` value that stays marker-only) |

To keep the `PreferRNativeType` derive's concrete `IntoR` (which routes through
`AsRNative`) from colliding with the blanket `impl<T: IntoExternalPtr> IntoR`
(E0119, #1283), a struct-level `prefer = "native"` type deliberately does **not**
implement `IntoExternalPtr`. As a result the `AsExternalPtr` call-site wrapper
does not apply to such a type — its default `IntoR` is the native representation.

#### List Mode

`#[miniextendr(list)]` makes a struct round-trip with an R **list**. It expands
to three derives:

| Derive | Direction | Role |
|--------|-----------|------|
| `IntoList` | Rust → R | `struct` → R list (one element per field) |
| `TryFromList` | R → Rust | R list → `struct` (one field per element) |
| `PreferList` | Rust → R | marker so the struct can be **returned directly** from a `#[miniextendr]` fn — `IntoR` routes through `IntoList` |

```rust
#[miniextendr(list)]
pub struct Record {
    pub name: String,
    pub value: i32,
}

#[miniextendr]
pub fn make_record() -> Record {
    Record { name: "test".into(), value: 42 }
}
// R: make_record()  ->  list(name = "test", value = 42L)
```

**Field shape determines list shape:**

| Struct kind | R list produced |
|-------------|-----------------|
| Named (`struct R { a, b }`) | named list — `list(a = …, b = …)` |
| Tuple (`struct R(A, B)`)    | unnamed list — `list(…, …)` (positional) |
| Unit (`struct R`)           | empty list — `list()` |

**Field type requirements.** Each non-ignored field's type must implement
`IntoR` (for `IntoList`) and `TryFromSexp` (for `TryFromList`). Heterogeneous
fields are fine — the list holds each element at its natural R type. This
includes scalars (`i32`, `f64`, `String`, `bool`), vectors (`Vec<f64>`,
`Vec<i32>`, `Vec<String>`), and `Option<T>` (→ `NULL` when `None`):

```rust
#[miniextendr(list)]
pub struct FitResult {
    pub estimate: Vec<f64>,   // numeric vector element
    pub sigma: f64,           // length-1 numeric element
}
```

**Skipping fields.** `#[into_list(ignore)]` drops a field from the list. On the
way back (`TryFromList`) an ignored field is filled with `Default::default()`,
so its type must implement `Default`:

```rust
#[miniextendr(list)]
pub struct Cfg {
    pub label: String,
    #[into_list(ignore)]
    cache: RefCell<Option<Vec<u8>>>,  // not in the R list; Default on the way back
}
```

**Return-only by default.** List mode generates `PreferList` (return path) but
**not** `TryFromSexp` on the struct, so a list-mode struct is *not* usable as a
`#[miniextendr]` function argument directly. To reconstruct one from R, take a
`List` parameter and call `try_from_list` yourself:

```rust
use miniextendr_api::list::{List, TryFromList};

#[miniextendr]
pub fn use_record(x: List) -> i32 {
    let r = Record::try_from_list(x).expect("record list");
    r.value
}
```

`TryFromList` reports a `MissingField` error when a named element is absent, a
`DuplicateName` error when a field's name appears more than once in the list
(the struct could keep only one of the values; other repeated names are
ignored, as unknown names are), and the field's own conversion error (e.g. a
type mismatch) when an element is present but the wrong type.

#### DataFrame Mode

```rust
#[miniextendr(dataframe)]
pub struct Obs {
    pub id: i32,
    pub score: f64,
}

#[miniextendr]
pub fn make_obs() -> ObsDataFrame {  // companion type
    Obs::to_dataframe(vec![
        Obs { id: 1, score: 0.5 },
        Obs { id: 2, score: 0.8 },
    ])
}
// R: data.frame(id = c(1L, 2L), score = c(0.5, 0.8))
```

---

## Enums (Fieldless)

### Default → RFactor

```rust
#[miniextendr]
#[derive(Copy, Clone)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}
```

Generates `MatchArg` + `RFactor` + `IntoR` + `TryFromSexp`. The enum maps to
an R factor with levels matching variant names.

```r
s <- get_season()  # factor("Summer", levels = c("Spring", "Summer", "Autumn", "Winter"))
```

### MatchArg Mode

```rust
#[miniextendr(match_arg)]
#[derive(Copy, Clone)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}
```

Like RFactor but uses R's `match.arg()` semantics with partial matching. The enum
maps to a character scalar in R, validated against the allowed choices.

### Explicit Factor

```rust
#[miniextendr(factor)]  // same as default, but explicit
pub enum Color { Red, Green, Blue }
```

---

## Automatic Registration

All `#[miniextendr]` items are automatically registered via linkme distributed slices. No manual module declarations are needed -- simply annotate your items with `#[miniextendr]` and they will be available to R at load time.

---

## Derive Macros

These are separate from `#[miniextendr]` but complementary:

| Derive | What It Generates |
|--------|-------------------|
| `ExternalPtr` | Type-safe external pointer wrapper |
| `Altrep` | Full ALTREP class (IntoR + TryFromSexp + registration) |
| `AltrepInteger` / `AltrepReal` / ... | ALTREP for specific vector types |
| `IntoList` | Struct → R list |
| `TryFromList` | R list → Struct |
| `DataFrameRow` | Struct → columnar data.frame + companion type |
| `RFactor` | Enum ↔ R factor |
| `MatchArg` | Enum ↔ R character with `match.arg()` |
| `RNativeType` | Newtype for native R types |
| `PreferList` | Marker: route `IntoR` through list |
| `PreferDataFrame` | Marker: route `IntoR` through data.frame |
| `PreferExternalPtr` | Marker: route `IntoR` through ExternalPtr |
| `PreferRNativeType` | Marker: route `IntoR` through native SEXP |
| `Vctrs` | vctrs-compatible S3 class (feature-gated) |

The `#[miniextendr]` attribute on structs/enums is a convenience that calls the
appropriate derives internally. Both paths produce identical code.

## See Also

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md): all 6 class systems with examples
- [S3_METHODS.md](S3_METHODS.md): detailed S3 print/format guide
- [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md): dots and typed_list validation
- [ALTREP.md](ALTREP.md): ALTREP deep dive
- [DATAFRAME.md](DATAFRAME.md): DataFrame conversion (derive + serde + columnar)
- [SERDE_R.md](SERDE_R.md): serde integration for direct Rust-R serialization
- [ERROR_HANDLING.md](ERROR_HANDLING.md): tagged-condition transport, `unwrap_in_r`, panic handling
- [LIFECYCLE.md](LIFECYCLE.md): deprecation/experimental lifecycle attributes
- [TRAIT_ABI.md](TRAIT_ABI.md): cross-package trait dispatch
