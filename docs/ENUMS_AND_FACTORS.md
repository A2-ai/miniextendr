# Enums and Factors Guide

How to map Rust enums to R factors and character strings.

miniextendr provides two complementary systems for enum-like types:

| System | R Representation | Partial Match | Default | Use Case |
|--------|-----------------|---------------|---------|----------|
| `RFactor` | Factor (integer + levels) | No | - | Categorical data for `table()`, `lm()`, etc. |
| `MatchArg` | Character scalar | Yes | First choice | Parameter validation (`match.arg()` style) |

Both systems pass the enum **by value** (as a factor/string). If instead you want
the enum to be a methods-bearing **instance** — an `ExternalPtr` wrapped with a
class-system `impl` block — see
[CLASS_SYSTEMS.md § Enums: value vs instance](CLASS_SYSTEMS.md#enums-value-vs-instance).
Note the two are **mutually exclusive**: deriving `ExternalPtr` alongside
`MatchArg`/`RFactor` is a compile error (`E0119`, conflicting `IntoR`).

## RFactor: enum as R Factor

Maps a Rust enum to an R factor with levels. Each variant becomes a level.

```rust
#[derive(Copy, Clone, RFactor)]
pub enum Color {
    Red,    // level index 1
    Green,  // level index 2
    Blue,   // level index 3
}
```

Use in functions:

```rust
#[miniextendr]
pub fn describe(color: Color) -> &'static str {
    match color {
        Color::Red => "warm",
        Color::Green => "cool",
        Color::Blue => "cool",
    }
}

#[miniextendr]
pub fn favorite() -> Color {
    Color::Blue
}
```

From R:

```r
describe(factor("Red", levels = c("Red", "Green", "Blue")))
# [1] "warm"

favorite()
# [1] Blue
# Levels: Red Green Blue
```

### Rename Variants

```rust
#[derive(Copy, Clone, RFactor)]
#[r_factor(rename_all = "snake_case")]
pub enum Status {
    InProgress,   // level: "in_progress"
    Completed,    // level: "completed"
    NotStarted,   // level: "not_started"
}

#[derive(Copy, Clone, RFactor)]
pub enum Priority {
    #[r_factor(rename = "lo")]
    Low,
    #[r_factor(rename = "med")]
    Medium,
    #[r_factor(rename = "hi")]
    High,
}
```

Supported `rename_all` values: `snake_case`, `kebab-case`, `lower`, `upper`.

### Factor Vectors

Use `FactorVec<T>` for vectors and `FactorOptionVec<T>` for vectors with NA:

```rust
use miniextendr_api::{FactorVec, FactorOptionVec};

#[miniextendr]
pub fn all_colors() -> FactorVec<Color> {
    FactorVec(vec![Color::Red, Color::Green, Color::Blue])
}

#[miniextendr]
pub fn parse_colors(input: FactorOptionVec<Color>) -> Vec<&'static str> {
    input.0.iter().map(|c| match c {
        Some(Color::Red) => "red",
        Some(Color::Green) => "green",
        Some(Color::Blue) => "blue",
        None => "NA",
    }).collect()
}
```

From R:

```r
all_colors()
# [1] Red   Green Blue
# Levels: Red Green Blue

x <- factor(c("Red", NA, "Blue"), levels = c("Red", "Green", "Blue"))
parse_colors(x)
# [1] "red" "NA"  "blue"
```

### Caching

The `#[derive(RFactor)]` macro generates a `OnceLock`-cached levels STRSXP. The
levels string vector is allocated once and reused for all subsequent conversions,
giving ~4x speedup for single-value conversions.

### Via `#[miniextendr]`

Instead of `#[derive(RFactor)]`, you can use the attribute macro:

```rust
#[miniextendr]
#[derive(Copy, Clone)]
pub enum Color { Red, Green, Blue }
```

These are equivalent. `#[miniextendr]` on a fieldless enum dispatches to the same
RFactor derive internally.

---

## MatchArg: enum as string parameter

Maps a Rust enum to R character strings with `match.arg()` validation. Supports
partial matching and defaults to the first variant when `NULL` is passed (an
`Option<T>` parameter is the exception; see "Optional Choice" below).

```rust
#[derive(Copy, Clone, MatchArg)]
pub enum Mode {
    Fast,    // choice: "Fast"
    Safe,    // choice: "Safe"
    Debug,   // choice: "Debug"
}
```

Use in functions:

```rust
#[miniextendr]
pub fn run(#[miniextendr(match_arg)] mode: Mode) -> String {
    match mode {
        Mode::Fast => "running fast".into(),
        Mode::Safe => "running safe".into(),
        Mode::Debug => "running debug".into(),
    }
}
```

The generated R wrapper shows the choice list directly as the formal default
and validates through a helper defined once at the top of the wrappers file:

```r
run <- function(mode = c("Fast", "Safe", "Debug")) {
  mode <- .miniextendr_match_arg(mode, c("Fast", "Safe", "Debug"), "mode")
  .Call(C_mypkg_run, mode)
}
```

`.miniextendr_match_arg()` follows `base::match.arg()`: an omitted argument
(or `NULL`) selects the first choice, a single string is matched exactly or as
a unique prefix, and a factor is read as its labels. Unlike `match.arg()` the
message names the argument (`'mode' should be one of "Fast", "Safe", "Debug"`)
and the choice list is spelled out in the call rather than read off the formal
(#1552).

The enum's `CHOICES` are spliced in at wrapper-gen time (not stored in an R
variable), so `?run` and tab-completion both show the real options. If you set
an explicit `default = "\"Safe\""`, the splice rotates that value to position 1
of the vector — `match.arg(arg)` returns `arg[1]` when `arg` matches the formal
default, so the rotated value becomes the effective default while the rest of
the choices remain visible: `function(mode = c("Safe", "Fast", "Debug"))`. The
default value must be one of the enum's choices; otherwise wrapper generation panics
at write time. When another parameter is named `c`, the formal default and the
usage line read `mode = base::c("Fast", "Safe", "Debug")`
([MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md#parameters-named-like-a-base-function)).

From R:

```r
run("Fast")       # exact match
run("F")          # partial match → "Fast"
run()             # NULL → default (first choice: "Fast")
run("Saf")        # partial match → "Safe"
run("X")          # Error: 'mode' should be one of "Fast", "Safe", "Debug"
```

A `MatchArg` enum parameter without `#[miniextendr(match_arg)]` gets no R-side
`match.arg()` check. Its Rust conversion reads the value as the check does
(`NULL` or the whole choice vector selects the first choice, a factor is read as
its labels, a unique prefix matches) and refuses a value with the argument
error the check raises, word for word: `'mode' should be one of "Fast",
"Safe", "Debug"`, `'mode' must be of length 1`, `'mode' must be NULL or a
character vector` (#1767). Only `e$rust_type` tells the two apart.

### Matching a Raw Argument in the Body

Some arguments are only a choice on some paths: an argument the body hands on
to another function, or one whose meaning depends on another argument. Take
it as `SEXP` and match it in the body with `match_arg_param::<T>(value, "mode")`.
It follows the wrapper's `.miniextendr_match_arg()` exactly, results and
messages alike (#1741), and its error raises as the wrapper's argument error
(`kind = "conversion"`, `e$param`, the crate's `conversion_error_class`, the
wrapper's call):

```rust
use miniextendr_api::{MatchArg, SEXP, match_arg_param, miniextendr};

#[derive(Copy, Clone, MatchArg)]
#[match_arg(rename_all = "snake_case")]
pub enum Fill {
    Drop,
    Draw,
    Error,
}

#[miniextendr]
pub fn fill_with(fill: SEXP) -> String {
    let fill: Fill = match_arg_param(fill, "fill").unwrap_or_else(|e| e.raise());
    fill.to_choice().to_string()
}
```

| `fill` | Result |
|---|---|
| `NULL`, or the whole choice vector `c("drop", "draw", "error")` | `Drop` (the first choice) |
| `"error"`, `"e"` (a unique prefix), `factor("draw")` | that choice |
| `"dr"` (an ambiguous prefix), `"zzz"`, `""`, `NA_character_` | `'fill' should be one of "drop", "draw", "error"` |
| `c("drop", "draw")`, `character(0)` | `'fill' must be of length 1` |
| `1L`, `TRUE`, `list("drop")` | `'fill' must be NULL or a character vector` |

As in `match.arg()`, `NA_character_` is matched as the string `"NA"`, so it
selects a choice spelled `"NA"` or with that prefix, and the whole choice
vector selects the first choice only when it has no attributes (a named
vector is a vector of length 3). The returned `ArgError` carries the parameter
name and message (`e.param()`, `e.message()`) for a caller that words its own
error; `arg_error!` raises any other argument error from a body
([CONDITIONS.md](CONDITIONS.md#argument-errors-from-a-body)).

`match_arg_param` matches against `T::CHOICES` in declaration order, the
formal of a plain `match_arg` parameter. A parameter with `default = "..."`
moves the default to the front of its formal and its check
(`c("Safe", "Fast", "Debug")` for `default = "\"Safe\""`), which changes the
first choice, the vector that counts as the whole choice vector, and the order
of the choices in the message. A body that matches a value forwarded from such
a formal passes the same default to `match_arg_param_with_default` (#1767):

```rust
let mode: Mode = match_arg_param_with_default(mode, "mode", Mode::Safe)
    .unwrap_or_else(|e| e.raise());
```

The default is a `T`, not a string, so it is always one of the choices. Both
this function and the wrappers writer order the choices with the same code.

### Optional Choice: `Option<T>`

With a plain `T`, an omitted argument and an explicit `NULL` both resolve to
the first choice, so "no choice was made" cannot be expressed. Declare the
parameter as `Option<T>` for that (#1473):

```rust
#[miniextendr]
pub fn run(#[miniextendr(match_arg)] mode: Option<Mode>) -> String {
    match mode {
        Some(mode) => format!("running {mode:?}"),
        None => "engine decides".into(),
    }
}
```

The R formal defaults to `NULL` instead of the choice vector, the prelude
skips the check for `NULL`, and the auto-generated `@param` line ends in
", or NULL for no choice". `NULL` arrives as `None`; any other value is
matched exactly as for the plain type:

```r
run <- function(mode = NULL) {
  if (!is.null(mode)) mode <- .miniextendr_match_arg(mode, c("Fast", "Safe", "Debug"), "mode")
  .Call(C_mypkg_run, mode)
}

run()             # None
run(NULL)         # None
run("Sa")         # Some(Safe)
run("X")          # Error: 'mode' should be one of "Fast", "Safe", "Debug"
```

The same works for `choices(...)` on an `Option<String>` / `Option<&str>`
parameter. A `default = "..."` on an `Option<T>` choice parameter is a compile
error (the formal is `NULL` by definition; drop the `Option` to make a choice
the default). The generated C wrapper decodes the argument with
`match_arg_option_from_sexp`, so any `MatchArg` type works, derived or
hand-written.

### Omitted Choice

`Option<T>` gives up the choice vector in the usage line: the formal has to be
`NULL`. To keep `mode = c("Fast", "Safe", "Debug")` visible in `?run` and still
learn that the caller made no choice, wrap the type in `Missing<..>`. Omission
and nullability stay separate contracts: `Missing` reports the first, `Option`
the second, and neither turns into the other (#1551).

```rust
use miniextendr_api::Missing;

#[miniextendr]
pub fn run(#[miniextendr(match_arg)] mode: Missing<Option<Mode>>) -> String {
    match mode {
        Missing::Absent => "inherit the mode from elsewhere".into(),
        Missing::Present(None) => "explicitly no mode".into(),
        Missing::Present(Some(mode)) => format!("running {mode:?}"),
    }
}
```

```r
run <- function(mode = c("Fast", "Safe", "Debug")) {
  if (!missing(mode) && !is.null(mode)) mode <- .miniextendr_match_arg(mode, c("Fast", "Safe", "Debug"), "mode")
  .Call(C_mypkg_run, .call = sys.call(), if (missing(mode)) quote(expr=) else mode)
}

run()             # Missing::Absent
run(NULL)         # Missing::Present(None)
run("Sa")         # Missing::Present(Some(Safe))
run("X")          # Error: 'mode' should be one of "Fast", "Safe", "Debug"
```

The wrapper skips the check for an omitted argument and forwards R's
missing-argument sentinel to Rust (the same `quote(expr=)` forwarding every
`Missing<T>` parameter uses), so the formal default is only documentation.
Explicit `NULL` is `Present(None)`, exactly as for any `Missing<Option<T>>`;
collapse the two with `mode.into_option().flatten()` when the difference does
not matter. The auto-generated `@param` line ends in ", or NULL; omitting the
argument means no choice".

The other forms follow the same rule, `Missing<X>` behaves like `X` for every
supplied value and adds `Absent` for an omitted one:

| Parameter type | Formal | Omitted | `NULL` | `"Sa"` |
|----------------|--------|---------|--------|--------|
| `Mode` | choices | `Fast` (first) | `Fast` | `Safe` |
| `Option<Mode>` | `NULL` | `None` | `None` | `Some(Safe)` |
| `Missing<Mode>` | choices | `Absent` | `Present(Fast)` | `Present(Safe)` |
| `Missing<Option<Mode>>` | choices | `Absent` | `Present(None)` | `Present(Some(Safe))` |
| `Missing<Vec<Mode>>` (`several_ok`) | choices | `Absent` | `Present` (all) | `Present(vec![Safe])` |
| `Either<Mode, R>` | choices | `Left(Fast)` (first) | `Right(..)` (converted to `R`; a `DataFrame` refuses it) | `Left(Safe)` |
| `Missing<Either<Mode, R>>` | choices | `Absent` | `Present(Right(..))` (converted to `R`) | `Present(Left(Safe))` |
| `Missing<Either<Vec<Mode>, R>>` (`several_ok`) | choices | `Absent` | `Present(Right(..))` (converted to `R`) | `Present(Left(vec![Safe]))` |

To refuse an explicit `NULL` in any of these forms, with your own message, see
[Refusing an Explicit `NULL`](#refusing-an-explicit-null).

`choices(...)` accepts the same wrappers around `String` / `&str`. `Missing<..>`
has to be the outermost wrapper (`Option<Missing<T>>` is a compile error), and
it cannot carry a `default = "..."`. Impl methods of every class system take
the same types through the method-level `match_arg(p)` /
`match_arg_several_ok(p)` / `choices(p = "...")` attributes; trait methods
accept `choices(p = "...")` / `choices_several_ok(p = "...")` only, so they
take the string forms (`Missing<String>`, `Missing<Option<String>>`,
`Missing<Vec<String>>`, and with the `either` feature
`Missing<Either<String, R>>` / `Missing<Either<Vec<String>, R>>`). An omitted
argument crosses the trait ABI of a `#[miniextendr]` trait as R's
missing-argument sentinel, so a cross-package call sees `Absent` too.

A hand-written R function in front of such a wrapper passes omission on only
if its own formal has no default: `run <- function(mode) run_impl(mode)`
reaches Rust as `Absent` when called as `run()`, while
`function(mode = c("Fast", "Safe")) run_impl(mode)` hands the default vector
over as a supplied value (R's `missing()` does not see through a formal with a
default). To give the generated wrapper itself a bare formal, use
`no_default` (below).

#### Without a Default: `no_default`

The choice vector in the formal tells an R user that omitting the argument
picks the first choice (every choice under `several_ok`). A function that
refuses an omitted argument has no such default. Add `no_default` and the
formal is the bare name (#1828):

```rust
use miniextendr_api::{arg_error, miniextendr, Missing};

#[miniextendr]
pub fn run(#[miniextendr(match_arg, no_default)] mode: Missing<Mode>) -> String {
    match mode {
        Missing::Absent => arg_error!(param = "mode", "'mode' is required: there is no mode to assume"),
        Missing::Present(mode) => format!("running {mode:?}"),
    }
}
```

```r
run <- function(mode) {
  if (!missing(mode)) mode <- .miniextendr_match_arg(mode, c("Fast", "Safe", "Debug"), "mode")
  .Call(C_mypkg_run, .call = sys.call(), if (missing(mode)) quote(expr=) else mode)
}

run()             # Error: 'mode' is required: there is no mode to assume
run("Sa")         # Missing::Present(Safe)
run("X")          # Error: 'mode' should be one of "Fast", "Safe", "Debug"
```

Only the formal changes, and with it the usage line (`run(mode)`) and
`args(run)`. The prelude, the choice list, partial matching and factor input
stay as they were, since the helpers take the choice list as an argument. An
omitted `Missing<..>` argument still reaches Rust as `Absent`, so the function
words its own refusal (here with `arg_error!`, the argument error the
wrapper's own checks raise; see
[CONDITIONS.md](CONDITIONS.md#argument-errors-from-a-body)). Any other type (`Mode`, `Option<Mode>`, an `Either`, a
`several_ok` list) behaves like any parameter without a default: the prelude's
first use of the omitted argument raises R's `argument "mode" is missing, with
no default`.

The auto-generated `@param` line still lists the choices and the other
accepted values (`, or NULL`, `, or a data frame`), but it drops "omitting the
argument means no choice": what omission means is yours to document. The
keyword works with `choices(...)` and `several_ok` as well
(`#[miniextendr(match_arg, several_ok, no_default)]`). It is a compile error
together with `default = "..."` and on a parameter that is neither `match_arg`
nor `choices`. Impl and trait methods name the parameter at method level,
`no_default(p)` (or `no_default(p, q)`), next to `match_arg(p)` /
`choices(p = "...")`.

### Choice or Another Value: `Either<T, R>`

With the `either` feature, a choice parameter can also take a value of a
different kind: a named route or a data frame of doses, a level name or a
number. Declare it as `Either<T, R>` with the choice type on the left:

```rust
use miniextendr_api::either_impl::Either;
use miniextendr_api::DataFrame;

#[derive(Copy, Clone, Debug, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum Route { Oral, Bolus, Infusion }

#[miniextendr]
pub fn set_route(#[miniextendr(match_arg)] route: Either<Route, DataFrame>) -> String {
    match route {
        Either::Left(route) => format!("{route:?}"),
        Either::Right(doses) => format!("{} dose rows", doses.nrow()),
    }
}
```

The formal default is `T`'s choice vector, and the prelude matches the
argument only when it is a form `match.arg()` reads (character or factor);
anything else reaches Rust unchanged:

```r
set_route <- function(route = c("oral", "bolus", "infusion")) {
  if (is.character(route) || is.factor(route)) route <- .miniextendr_match_arg(route, c("oral", "bolus", "infusion"), "route")
  .Call(C_mypkg_set_route, .call = sys.call(), route)
}

set_route()                         # Left(Oral), the first choice
set_route("inf")                    # Left(Infusion)
set_route(data.frame(amt = 1:2))    # Right(<data frame>)
set_route("iv")                     # Error: 'route' should be one of "oral", "bolus", "infusion"
```

The C wrapper decodes with `match_arg_either_or`: character or factor input
becomes `Left(T)` (matched against `MatchArg::CHOICES`), everything else is
converted to `R` with its `TryFromSexp` impl and becomes `Right`. The split
follows the R prelude exactly, so a data frame is never tried as a choice and
a misspelled choice never falls through to `R`. An explicit `NULL` is not a
choice either: it goes to `R` (and fails for `DataFrame`). The auto-generated
`@param` line names the other kind: `One of "oral", "bolus", "infusion", or a
data frame.` A value `R` refuses is refused in the same words:

```r
set_route(1:3)   # Error: 'route' must be one of "oral", "bolus", "infusion", or a data frame: got integer
```

On `Either<T, R>`, `NULL` goes to `R` (a `DataFrame` arm refuses it). To
accept `NULL` as well, declare `Option<Either<T, R>>` (a `NULL` formal) or
`Missing<Option<Either<T, R>>>` (the choice vector stays the formal; omitted
is `Absent`, `NULL` is `Present(None)`).

The layers of the previous sections compose with it, outermost first:
`Option<Either<T, R>>` has a `NULL` formal and turns `NULL` into `None`,
`Missing<Either<T, R>>` and `Missing<Option<Either<T, R>>>` keep the choice
vector and report an omitted argument as `Absent`. `choices("a", "b")` works
the same way on `Either<String, R>`. Impl methods take all of these through
`match_arg(p)` / `choices(p = "...")`, trait methods the `choices` forms
(`choices(p = "...")` on `Either<String, R>` or `Missing<Either<String, R>>`).
The parameter types of a `#[miniextendr]` trait also cross its trait ABI,
which converts each one with `TryFromSexp` / `IntoR`; `Option<Either<..>>`
has a `TryFromSexp` impl (`NULL` → `None`, otherwise the left-first `Either`
conversion, not the choice decoding) but no `IntoR`, so the `Option` layer is
not available on a trait method.
The other arm's name in
the `@param` line comes from its Rust type (`DataFrame` is "a data frame",
`List` "a list", `f64` "a number", `Vec<String>` "a character vector"; a type
R has no name for is shown in code format). A layer inside the left arm
(`Either<Option<T>, R>`) is a compile error: the choice type has to be the left
arm.

The other arm has to read something other than character or factor input,
since every such argument goes to the choice. An arm that reads only strings or
factors could receive at most `NULL`, so it is a compile error that names the
parameter and the arm: `String`, `&str`, `Vec<String>`, `PathBuf`, a type alias
or `#[derive(TryFromSexp)]` newtype of one, your own `MatchArg` or `RFactor`
enum, and any type whose `TryFromSexp::CHARACTER_ONLY` is `true` (see
[Extending miniextendr](EXTENDING_MINIEXTENDR.md#option-2-direct-tryfromsexp-intor-implementation)).
The check reads the arm's conversion, not its name. An `Option<String>` arm is
refused too, since only `NULL` could reach it: for a `NULL` alternative,
declare the parameter as `Option<Either<T, R>>`. `AsCharacter` and
`AsCharacterVec` are allowed, because they also read numbers the way
`as.character()` does (`101L` reaches the arm as `"101"`).

#### Several Choices or Another Value

`several_ok` takes the same split with a list on the left: `Either<Vec<T>, R>`
or `Either<Box<[T]>, R>` accepts one or more choices or a value of another kind
(#1612):

```rust
#[miniextendr]
pub fn set_routes(
    #[miniextendr(match_arg, several_ok)] routes: Either<Vec<Route>, DataFrame>,
) -> String {
    match routes {
        Either::Left(routes) => format!("{} routes", routes.len()),
        Either::Right(doses) => format!("{} dose rows", doses.nrow()),
    }
}
```

```r
set_routes <- function(routes = c("oral", "bolus", "infusion")) {
  if (is.character(routes) || is.factor(routes)) routes <- .miniextendr_match_arg_several(routes, c("oral", "bolus", "infusion"), "routes")
  .Call(C_mypkg_set_routes, .call = sys.call(), routes)
}

set_routes()                        # Left([Oral, Bolus, Infusion]), every choice
set_routes(c("inf", "or"))          # Left([Infusion, Oral]), in the order given
set_routes(data.frame(amt = 1:2))   # Right(<data frame>)
set_routes(c("oral", "iv"))         # Error: 'routes' element 2 ("iv") should be one of ...
set_routes(character(0))            # Error: 'routes' must be of length >= 1
```

Character or factor input is matched element by element, as for a plain
`several_ok` list (see [Multiple Choices with
`several_ok`](#multiple-choices-with-several-ok)), and an omitted argument
selects every choice as `Left`. Anything else goes to `R`, and a value `R`
refuses reads `'routes' must be one or more of "oral", "bolus", "infusion", or
a data frame: got integer`. An explicit `NULL`
goes to `R` too: that is the one difference from `several_ok` on `Vec<T>`,
where `NULL` selects every choice. Only the owned containers decode under
`Either`: `Either<[T; N], R>` and `Either<&[T], R>` are compile errors, and so
is an `R` that reads only character input (`Either<Vec<T>, Vec<String>>`), as
for a scalar choice.

The layers stack as for a scalar `Either`: `Missing<Either<Vec<T>, R>>` keeps
the choice vector and reports an omitted argument as `Absent` (an explicit
`NULL` is `Present` of the `R` conversion), `Option<Either<Vec<T>, R>>` has a
`NULL` formal and reads both an omitted argument and `NULL` as `None`, and
`Missing<Option<Either<Vec<T>, R>>>` keeps the choice vector with `Absent` for
an omitted argument and `Present(None)` for `NULL`. A bare `Option<Vec<T>>`
stays a compile error, since `NULL` already means every choice there.
`choices("a", "b"), several_ok` works the same way on `Either<Vec<String>, R>`.
Impl methods take all of these through `match_arg_several_ok(p)` /
`choices_several_ok(p = "...")`; trait methods take
`choices_several_ok(p = "...")` on `Either<Vec<String>, R>` or
`Missing<Either<Vec<String>, R>>`. The auto-generated `@param` line reads
`One or more of "oral", "bolus", "infusion", or a data frame.`

### Refusing an Explicit `NULL`

A plain choice reads an explicit `NULL` as its first choice, as `match.arg()`
does, and an `Either<T, R>` choice hands it to `R`, which may refuse it in
generic words (`'route' must be one of "oral", "bolus", "infusion", or a data
frame: got NULL`). When the first choice is a setting rather than a neutral
default, or when `NULL` deserves its own hint, refuse it with `not_inherits`:

```rust
#[miniextendr]
pub fn set_rule(
    #[miniextendr(
        match_arg,
        not_inherits("NULL", message = "`rule` can't be NULL; use \"auto\" to reset it.")
    )]
    rule: Missing<Either<Rule, DataFrame>>,
) -> String { /* ... */ }
```

```r
set_rule <- function(rule = c("include", "exclude", "auto")) {
  if (!isTRUE(missing(rule) || !inherits(rule, "NULL"))) .miniextendr_arg_error("rule", message = "`rule` can't be NULL; use \"auto\" to reset it.")
  if (!missing(rule) && (is.character(rule) || is.factor(rule))) rule <- .miniextendr_match_arg(rule, c("include", "exclude", "auto"), "rule")
  .Call(C_mypkg_set_rule, .call = sys.call(), if (missing(rule)) quote(expr=) else rule)
}
```

`NULL`'s implicit class is `"NULL"`, so `inherits(NULL, "NULL")` is `TRUE`.
The class checks run before the choice is matched, so `NULL` is refused before
it can become the first choice or reach `R`. The refusal raises the same
condition as a failed choice (`rust_error`, `kind = "conversion"`,
`e$param`), and an omitted `Missing<..>` argument still passes as `Absent`.
It works on every form in the table above, `several_ok` included, and on impl
and trait methods through `not_inherits(p(class = "NULL", message = "..."))`.
On an `Option<..>` parameter the check lets `NULL` through, since `NULL` is
that parameter's `None`; drop the `Option` instead. See [Refusing
classes](MINIEXTENDR_ATTRIBUTE.md#refusing-classes) for the attribute itself.

### Rename Variants

Same syntax as RFactor but with `#[match_arg(...)]`:

```rust
#[derive(Copy, Clone, MatchArg)]
#[match_arg(rename_all = "snake_case")]
pub enum BuildStatus {
    InProgress,    // choice: "in_progress"
    Completed,     // choice: "completed"
}

#[derive(Copy, Clone, MatchArg)]
pub enum Priority {
    #[match_arg(rename = "lo")]  Low,
    #[match_arg(rename = "med")] Medium,
    #[match_arg(rename = "hi")]  High,
}
```

### Via `#[miniextendr]`

```rust
#[miniextendr(match_arg)]
#[derive(Copy, Clone)]
pub enum Mode { Fast, Safe, Debug }
```

### For an Enum You Do Not Own (Newtype)

`#[derive(MatchArg)]` has to sit on the enum's declaration, so it cannot be
attached to an enum from a crate that does not depend on miniextendr (the
usual shape when an R package wraps an existing Rust library). Wrap the
foreign enum in a newtype and implement the three traits the derive would
have emitted. `MatchArg` (`miniextendr_api::match_arg::MatchArg`) is
public, and so are the helpers the derive leans on:

```rust
use miniextendr_api::match_arg::MatchArg;
use miniextendr_api::{IntoR, SEXP, SexpError, TryFromSexp};

// Owned by the wrapped library; no miniextendr types anywhere near it.
use wrapped::Interp;

#[derive(Copy, Clone)]
pub struct InterpChoice(pub Interp);

impl MatchArg for InterpChoice {
    const CHOICES: &'static [&'static str] = &["linear", "cubic", "nearest"];

    fn from_choice(choice: &str) -> Option<Self> {
        Some(InterpChoice(match choice {
            "linear" => Interp::Linear,
            "cubic" => Interp::Cubic,
            "nearest" => Interp::Nearest,
            _ => return None,
        }))
    }

    fn to_choice(self) -> &'static str {
        match self.0 {
            Interp::Linear => "linear",
            Interp::Cubic => "cubic",
            Interp::Nearest => "nearest",
        }
    }
}

impl TryFromSexp for InterpChoice {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        miniextendr_api::match_arg_from_sexp(sexp).map_err(Into::into)
    }
}

impl IntoR for InterpChoice {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> { Ok(self.into_sexp()) }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> { self.try_into_sexp() }
    fn into_sexp(self) -> SEXP { self.to_choice().into_sexp() }
}

#[miniextendr]
pub fn interpolate(#[miniextendr(match_arg)] method: InterpChoice) -> f64 {
    wrapped::run(method.0)
}
```

Everything downstream of the trait is shared with derived enums: the
`match.arg()` prelude, the choices spliced into the formal default
(`function(method = c("linear", "cubic", "nearest"))`), partial matching,
factor input, `several_ok` (`Vec<InterpChoice>`), the `Vec<T>` return path
(via the blanket `IntoRVecElement` bridge), impl-block `match_arg(param)`
attributes, and the auto-injected `@param` choices text. `to_choice` is an
exhaustive `match`, so adding a variant to the wrapped enum without updating
the newtype is a compile error rather than silent drift.
`CHARACTER_ONLY = true` says the conversion reads only character or factor
input, as a derived enum's does, so `Either<InterpChoice, String>` is refused
like `Either<Route, String>` (see [Choice or Another
Value](#choice-or-another-value-either-t-r)).
`rpkg/src/rust/match_arg_foreign_tests.rs` is the reference fixture.

### Inline String Choices

For simple cases where you don't need an enum, use `choices(...)` on a `&str` parameter:

```rust
#[miniextendr]
pub fn correlate(
    x: f64, y: f64,
    #[miniextendr(choices("pearson", "kendall", "spearman"))] method: &str,
) -> String {
    format!("method={}, cor={}", method, x * y)
}
```

### Multiple Choices with `several_ok`

R's `match.arg(..., several.ok = TRUE)` accepts multiple values from the choice
list and returns a character vector. miniextendr exposes this with
`several_ok`, which is valid on both `match_arg` and `choices`:

```rust
// Enum: Vec<Mode> - each element validated against MatchArg::CHOICES
#[miniextendr]
pub fn pick_modes(#[miniextendr(match_arg, several_ok)] modes: Vec<Mode>) -> String { ... }

// Inline: Vec<String> - each element validated against the inline list
#[miniextendr]
pub fn pick_metrics(
    #[miniextendr(choices("mean", "median", "sd", "var"), several_ok)] metrics: Vec<String>,
) -> String { ... }
```

Accepted container shapes: `Vec<T>`, `Box<[T]>`, `&[T]`, and `[T; N]`; under
`Missing<..>` or `Either<.., R>`, `Vec<T>` and `Box<[T]>` only.
`several_ok` without `match_arg` or `choices` is a compile error (no choice
list to validate against). `several_ok` on a scalar type (e.g. `Mode` without
a `Vec`) is also a compile error, and so is `Option<Vec<T>>`.
`Either<Vec<T>, R>` takes one or more choices or a value of another kind (see
[Several Choices or Another Value](#several-choices-or-another-value)).

An omitted argument, and an explicit `NULL`, select the full choice list
(under `Either`, `NULL` goes to the `R` arm instead).
Pass a single string to get partial matching, or a character vector to select
several choices; each element is matched exactly or as a unique prefix.
`Missing<Vec<T>>` (or `Missing<Box<[T]>>`) keeps the choice vector as the
formal and reports an omitted argument as `Missing::Absent` instead; `NULL`
still selects every choice (see [Omitted Choice](#omitted-choice)).

Validation is stricter than `base::match.arg(several.ok = TRUE)`, which keeps
the elements that match and silently drops the rest as long as one element
matched (so `f(c("alpha", "zzz"))` would behave like `f("alpha")`). The
generated prelude routes `several_ok` parameters through an internal helper
that requires every element to match and reports the first that does not,
with its position (#1472):

```r
pick_modes <- function(modes = c("Fast", "Safe", "Debug")) {
  modes <- .miniextendr_match_arg_several(modes, c("Fast", "Safe", "Debug"), "modes")
  .Call(C_mypkg_pick_modes, modes)
}

pick_modes(c("Fast", "zzz"))
#> Error in pick_modes(c("Fast", "zzz")) :
#>   'modes' element 2 ("zzz") should be one of "Fast", "Safe", "Debug"
```

The Rust side (`match_arg_vec_from_sexp`) still checks every element against
`MatchArg::CHOICES`, so a value that bypasses the wrapper is caught too.

### On Impl-Block Methods

Rust rejects attribute macros on function parameters inside impl items, so
`match_arg` / `choices` / `several_ok` on impl methods use **method-level**
attributes that name the parameter. This works for all class systems
(`r6`, `env`, `s3`, `s4`, `s7`, `vctrs`) on both constructors and instance
methods:

```rust
#[miniextendr(r6)]
impl Counter {
    // Constructor: first choice becomes the R default.
    #[miniextendr(match_arg(mode))]
    pub fn new(mode: Mode) -> Self { ... }

    // several_ok variant - note the distinct attribute name.
    #[miniextendr(match_arg_several_ok(modes))]
    pub fn reset(&mut self, modes: Vec<Mode>) -> i32 { ... }

    // Inline string choices - pass the list as a comma-separated string.
    #[miniextendr(choices(level = "low, medium, high"))]
    pub fn describe(level: String) -> String { ... }
}
```

Each form generates the same R `match.arg` prelude you get on standalone
functions, including the choices vector as the formal default.

The vctrs class system accepts `match_arg` on its `fn new()` constructor
even though vctrs constructors return a data vector (`Vec<T>`) rather than
`Self`. The vctrs generator recognizes a receiverless `new` as the
constructor regardless of return type.

### Auto-Injected `@param` Docs

When you leave a `match_arg` parameter undocumented, miniextendr fills in
the roxygen `@param` line at write time using the enum's `CHOICES`; a
`choices(...)` parameter gets the same line from its literal list:

```r
#' @param mode One of "Fast", "Safe", "Debug".
```

The line also names the other accepted values and what omitting the argument
means (see the sections above). This runs for standalone functions and for
impl-block and trait methods across every class system, with one exception:
R6 trait methods. They live in `Type$Trait$method`, not among the R6
generator's public methods, so roxygen2 has no method section to list their
arguments in; a plain `@param` there lands in a top-level `\arguments` of the
class page, detached from any usage. Env classes have no usage section for
their methods, inherent or trait, and neither do the `Type$Trait$method`
statics of S3 and vctrs classes or the `attr(Type, "Trait")$method` statics
of S7 classes, so there the text goes into a
`\describe` list under the method's description instead of an `@param`
line. Explicit `@param` lines you
write yourself are preserved verbatim; only missing entries are
auto-generated. A
block with `@describeIn`, `@inheritParams`, or an `@rdname` naming another
page gets no generated line: the page it joins or the topic it inherits from
documents the parameter. On a file-stem page several functions opt into
(`@rdname <file stem>`) the line is kept only when no function there
documents the parameter (see
[Parameters on shared pages](S3_METHODS.md#parameters-on-shared-pages)).

#### One entry per parameter name on a shared page

Functions documented on one Rd page (a shared `@rdname`, such as the
functions of one source file that opt into its file-stem page, or the methods
of one class) get one `\item` per parameter name. roxygen2 merges the blocks
of a page in file order, and when
two blocks document the same name the later block's line replaces the
earlier one. On a class page that includes a method's generated line
replacing an explicit `@param` written on an earlier method's block. On a
file-stem page the generated line is left out when another function there
documents the parameter, but when none does, every function's generated
line is written. Two functions on one page that take a `mode` of different
types (`Mode` and `Missing<Option<Mode>>`) therefore show only the last
one's text.

miniextendr does not merge the texts into one line. Give such parameters
distinct names, or leave such functions on pages of their own (the default
for a free function without `@rdname`).

---

### Returning `Vec<Enum>`

Functions can return `Vec<T>` for any `MatchArg` enum. Each variant round-trips
to its choice string:

```rust
#[miniextendr]
pub fn all_modes() -> Vec<Mode> {
    vec![Mode::Fast, Mode::Safe, Mode::Debug]
}
```

```r
all_modes()
# [1] "Fast"  "Safe"  "Debug"
```

This is provided by a blanket `impl<T: MatchArg> IntoR for Vec<T>` in
`miniextendr-api`. No extra derive is required.

---

## MatchArg as Base Trait

`MatchArg` is the base trait for all enum-like types. `RFactor` requires `MatchArg`
as a supertrait, so any `RFactor` type also has `MatchArg::CHOICES`, `from_choice()`,
and `to_choice()`. It can also be implemented by hand on a newtype (see
[For an Enum You Do Not Own](#for-an-enum-you-do-not-own-newtype)). Use `MatchArg`
as a bound for generic code over both systems:

```rust
use miniextendr_api::MatchArg;

fn describe_choices<T: MatchArg>() -> String {
    T::CHOICES.join(", ")
}

fn lookup<T: MatchArg>(choice: &str) -> Option<T> {
    T::from_choice(choice)
}
```

---

## Comparison Table

| Feature | RFactor | MatchArg |
|---------|---------|----------|
| R storage | `factor(1, levels=c(...))` | `"Fast"` (character) |
| Validation | Type check (is factor with correct levels) | `match.arg()` with partial matching |
| Default on NULL | Error | First choice (`Option<T>`: `None`; `several_ok`: all choices; under `Either`: the `R` arm) |
| Omitted argument | Error | First choice (`Option<T>`: `None`; `Missing<..>`: `Absent`) |
| Vec support | `FactorVec<T>`, `FactorOptionVec<T>` | `Vec<T>` return + `several_ok` inputs |
| Partial matching | No | Yes (`"F"` → `"Fast"`) |
| Factor input | Native | Converted to character first |
| Use case | Categorical data | Parameter selection |

## When to Use Which

**RFactor** when:
- Data is categorical (colors, species, status codes)
- Working with R functions expecting factors (`table()`, `lm()`, `ggplot2`)
- Need vector support with NA handling
- Factor level ordering matters

**MatchArg** when:
- Building an API with string-based options
- Want R's `match.arg()` partial matching and error messages
- Want a default value when the argument is omitted
- Validating user input parameters

## See Also

- [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md): `#[miniextendr]` on enums
- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md): full type conversion reference
