# S3 Methods Guide

How to implement S3 generics (`print`, `format`, etc.) with `#[miniextendr(s3)]`.

## Quick Example

```rust
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct Person {
    name: String,
    age: i32,
}

#[miniextendr(s3)]
impl Person {
    /// @param name Name of the person.
    /// @param age Age of the person.
    pub fn new(name: String, age: i32) -> Self {
        Person { name, age }
    }

    /// Implements format.Person - returns a formatted string.
    #[miniextendr(s3(generic = "format"))]
    pub fn fmt(&self) -> String {
        format!("{} (age {})", self.name, self.age)
    }

    /// Implements print.Person - prints and returns self invisibly.
    #[miniextendr(s3(generic = "print"))]
    pub fn show(&mut self) {
        println!("Person: {}, age {}", self.name, self.age);
    }

    /// Custom method: greet.Person
    pub fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name)
    }
}

// Registration is automatic via #[miniextendr].
```

Generated R code (`.Call()` symbols are prefixed with your crate's name —
`mypkg` here — for webR cross-package uniqueness, see `docs/WEBR.md`):

```r
# Constructor
new_person <- function(name, age) {
  structure(.Call(C_mypkg_Person__new, name, age), class = "Person")
}

# format.Person - returns the string directly
format.Person <- function(x, ...) {
  .Call(C_mypkg_Person__fmt, x)
}

# print.Person - calls Rust, then returns invisible(x)
print.Person <- function(x, ...) {
  .Call(C_mypkg_Person__show, x)
  invisible(x)
}

# greet generic + method
greet <- function(x, ...) UseMethod("greet")
greet.Person <- function(x, ...) {
  .Call(C_mypkg_Person__greet, x)
}
```

## Key Concepts

### Generic Override with `#[miniextendr(s3(generic = "..."))]`

By default, the Rust method name becomes the S3 generic name. Use `s3(generic = "...")`
on the method to override this, mapping to a different R generic:

```rust
// Rust method is `fmt`, but R generic is `format`
#[miniextendr(s3(generic = "format"))]
pub fn fmt(&self) -> String { ... }

// Rust method is `show`, but R generic is `print`
#[miniextendr(s3(generic = "print"))]
pub fn show(&mut self) { ... }
```

This is essential for base R generics like `print` and `format` where you want your Rust
method to have a more descriptive name.

### Dots (`...`)

All S3 method signatures include `...` automatically. You don't need to declare them
in your Rust signature. The generated R wrapper adds `...` to the parameter list:

```r
# Generated: always has (x, ...) or (x, other_params, ...)
format.Person <- function(x, ...) { ... }
greet.Person <- function(x, ...) { ... }
```

If your Rust method takes extra parameters, they appear between `x` and `...`:

```rust
pub fn describe(&self, verbose: bool) -> String { ... }
// → describe.Person <- function(x, verbose, ...) { ... }
```

A method for a replacement generic (`s3(generic = "[[<-")`, any name ending
in `<-`) gets the `...` before its last parameter, which must be `value`:
`` `[[<-.Person` <- function(x, i, ..., value) ``. See
[Replacement and extraction generics](#replacement-and-extraction-generics).

### Constructor

The constructor is always the method returning `Self`. It generates a function named
`new_<classname_lowercase>()` that wraps the result with `structure(..., class = "ClassName")`:

```rust
pub fn new(name: String, age: i32) -> Self { ... }
// → new_person <- function(name, age) {
//     structure(.Call(C_mypkg_Person__new, name, age), class = "Person")
//   }
```

### Static Methods

Methods without `self` become standalone functions prefixed with the lowercase class name:

```rust
pub fn species() -> String { "Homo sapiens".into() }
// → person_species <- function() { .Call(C_mypkg_Person__species) }
```

## Functional builders (native pipe `|>`)

An idiomatic Rust builder mutates the receiver in place and returns `&mut Self`
so steps chain: `b.set_a(1).set_b(2)`. On the S3 path this maps to a
**pipe-friendly free function**: the generated generic takes the object as its
first argument and returns the (same, mutated) object, so it composes under R's
native pipe operator `|>` (R 4.1+).

```rust
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct GreetingBuilder {
    name: String,
    punctuation: String,
    loud: bool,
}

#[miniextendr(s3)]
impl GreetingBuilder {
    pub fn new() -> Self { /* empty defaults */ }

    // Builder steps: `&mut self -> &mut Self`.
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn set_loud(&mut self, loud: bool) -> &mut Self {
        self.loud = loud;
        self
    }

    // Terminal step: `&self -> T` (a different type), converted to R via `IntoR`.
    pub fn build(&self) -> String { /* render the greeting */ }
}
```

In R, every step is a free function whose first argument is the object, so the
whole pipeline reads naturally:

```r
new_greetingbuilder() |>
  set_name("World") |>
  set_loud(TRUE) |>
  build()
#> [1] "HELLO, WORLD."
```

**Value semantics.** A `&mut self -> &mut Self` step mutates the underlying Rust
value *in place* and the wrapper returns the **same** `ExternalPtr` handle —
there is no clone and no re-wrap, so object identity is preserved
(`identical(x, set_name(x, "a"))` is `TRUE`). This is the same reference
semantics as `&mut self -> ()` mutators (which return the object via the
`Chainable Mutation` strategy); the only difference is that returning
`&mut Self` lets the same method *also* chain in Rust.

The terminal `build()` takes `&self` so the R object stays valid afterwards,
and returns a different type (`String` here) through the usual `IntoR`
conversion.

**Consuming receivers.** Builders written as `fn step(mut self, ..) -> Self`
work too: the wrapper moves the value out of the R handle, calls the step, and
writes the result back into the *same* handle, so identity is preserved
exactly as for `&mut Self`. `fn step(mut self, ..) -> Result<Self, E>` /
`Option<Self>` run on a clone (the type must be `Clone`) and overwrite the
handle only on success, so a failed step leaves the R object as it was. A
consuming method with any other return type (`fn finish(self) -> T`) is a
terminal consume: the value is moved out and converted, and later use of the
handle errors with "was consumed". See
[CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#consuming-receivers-self).

**Fallible in-place steps.** `&mut self -> Result<&mut Self, E>` and
`-> Option<&mut Self>` are recognised as in-place builders as well: `Ok` /
`Some` hands back the same handle, `Err` / `None` raises through the normal
`Result` / `Option` error paths.

> The generated generic is named after the Rust method (`set_name`,
> `set_loud`, `build`), so pick method names that don't collide with existing
> generics or base-R functions in the importing package.

All of the above works on every impl-block class system (S3, Env, R6, S4,
S7): each returns the receiver from the generated generic or method, visibly
unless the method is marked `Invisible<..>` / `#[miniextendr(invisible)]`
(see [Return visibility](MINIEXTENDR_ATTRIBUTE.md#return-visibility-markers-and-defaults)).
`rpkg/src/rust/pipe_builder_tests.rs` is the cross-system fixture.

## Implementing `print`

R convention: `print()` displays output and returns `invisible(x)` so the object
can be used in pipelines without double-printing.

**Recommended pattern: `&mut self` returning `Invisible<()>`:**

```rust
use miniextendr_api::Invisible;

#[miniextendr(s3(generic = "print"))]
pub fn show(&mut self) -> Invisible<()> {
    println!("Person: {}, age {}", self.name, self.age);
    Invisible(())
}
```

This generates the `ChainableMutation` return strategy with the invisible tail:

```r
print.Person <- function(x, ...) {
  .Call(C_mypkg_Person__show, x)
  invisible(x)
}
```

`&mut self` + void return hands back `x` after the `.Call()`; the
`Invisible<()>` marker (or `#[miniextendr(s3(invisible))]`) makes that tail
`invisible(x)`, matching the R convention where `print()` returns the object
invisibly. Without the marker the method returns `x` visibly, which for a
`print` method means the object prints twice at the console.

**Why `&mut self`?** The receiver tail is only generated for `&mut self`
methods returning `()`. With `&self` returning `()`, the generated code would be
a bare `.Call(...)` returning `NULL`, which is functional but doesn't follow R convention.
If your print method doesn't actually mutate, using `&mut self` is a pragmatic
choice to get correct R behavior.

**Alternative: `&self` returning `()`:**

```rust
#[miniextendr(s3(generic = "print"))]
pub fn show(&self) {
    println!("Person: {}, age {}", self.name, self.age);
}
```

Generates:

```r
print.Person <- function(x, ...) {
  .Call(C_mypkg_Person__show, x)
}
```

This works (the `println!` output appears), but the function returns `NULL` instead
of `invisible(x)`. For most interactive use this is fine, but `y <- print(x)` gives
`NULL` rather than `x`.

## Implementing `format`

R convention: `format()` returns a character vector representation. Many R functions
use `format()` internally (e.g., `paste()`, `cat(format(x))`).

**Recommended pattern: `&self` returning `String`:**

```rust
#[miniextendr(s3(generic = "format"))]
pub fn fmt(&self) -> String {
    format!("{} (age {})", self.name, self.age)
}
```

Generates:

```r
format.Person <- function(x, ...) {
  .Call(C_mypkg_Person__fmt, x)
}
```

The string is returned directly (visible), which is correct for `format()`.

**Tip:** Implement `format` even if you also implement `print`. Other R code may call
`format()` on your object (e.g., when building error messages or tibble displays).

## Implementing Both `print` and `format`

A complete type should implement both. The standard R pattern is for `print` to call
`format` internally, but with miniextendr each method dispatches to separate Rust code:

```rust
#[derive(ExternalPtr)]
pub struct Temperature {
    celsius: f64,
}

#[miniextendr(s3)]
impl Temperature {
    pub fn new(celsius: f64) -> Self {
        Temperature { celsius }
    }

    #[miniextendr(s3(generic = "format"))]
    pub fn fmt(&self) -> String {
        format!("{:.1}°C", self.celsius)
    }

    #[miniextendr(s3(generic = "print"))]
    pub fn show(&mut self) {
        println!("{:.1}°C", self.celsius);
    }
}
```

Usage in R:

```r
t <- new_temperature(36.6)
print(t)    # 36.6°C  (returns invisible(t))
format(t)   # "36.6°C" (returns the string)
cat(format(t), "\n")  # 36.6°C
```

## Handles with R-visible state

When the R object needs fields that R code reads and writes next to the Rust
value, declare them as sidecar fields: `#[r_data]` fields on the
`#[derive(ExternalPtr)]` struct, next to an `#[r_data] _r: RSidecar` selector.
Each public field gets `Type_get_<field>(x)` and `Type_set_<field>(x, value)`;
a `Sidecar<T>` field holds a value, such as a data frame in a
`Sidecar<SEXP>`, that the pointer roots. The fields live with the handle, so
every method that takes the handle (`self: &ExternalPtr<Self>`) sees them and
nothing has to be re-wrapped. With `#[miniextendr(s3(r_data_accessors))]` on
the impl, the class also gets `$` / `[[` methods that read the fields and
`$<-` / `[[<-` methods that write them:

```rust
#[derive(ExternalPtr)]
#[externalptr(s3)]
pub struct Engine {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub rate: f64,
    #[r_data]
    pub keys: Sidecar<Vec<String>>,
}

#[miniextendr(s3(r_data_accessors))]
impl Engine {
    pub fn new(rate: f64) -> Self {
        Engine { _r: RSidecar, rate, keys: Sidecar::new(Vec::new()) }
    }
}
```

```r
e <- new_engine(0.5)
e$rate                 # 0.5
e$keys <- c("a", "b")  # validated as Vec<String>, stored in the pointer
e[["keys"]][2] <- "z"  # read, edit, write back
e$speed                # error of class `miniextendr_no_field`, naming the fields
e$rate <- "fast"       # the setter's argument error, call `e$rate <- value`
```

A write changes the object in place: the field lives behind the pointer, and
every copy of the R object shares the pointer, so `f <- e; f$rate <- 1` also
changes `e$rate`. That is R6 or environment semantics, not list semantics. A
class whose `$<-` must copy takes the getters-only form,
`s3(r_data_accessors = "get")`, and writes its own replacement method
(`#[miniextendr(s3(generic = "$<-"))]`, see "Replacement and extraction
generics" below), which returns a new object. The impl block is refused at
compile time when one of its methods is a `$` / `[[` / `$<-` / `[[<-` method
the option generates.

On the bare classed pointer, a name that is not a field is an error. On a list
or an environment that carries the handle in `.ptr` (the interop shape below),
it goes to R's own `$` / `[[`, partial matching included, so the object keeps
its other elements; a numeric index always does. The walkthrough and the
per-class-system accessor table are in `CLASS_SYSTEMS.md`, "Direct Field
Access via Sidecar". The sidecar is the supported answer for state the
package owns; the shape below is for interop.

### Interop: an existing list shape carrying the handle

Some R code already keeps its state in a list, or the class is defined in R (or
in a package you do not control) as a list. Such an object can carry the handle
in a `.ptr` element and still dispatch into `#[miniextendr(s3)] impl` methods:

```r
p <- structure(list(.ptr = new_person("Alice", 30), notes = character()), class = "Person")
greet(p)      # greet.Person -> Rust; the receiver prelude resolves the pointer from p$.ptr
show(p)       # `&mut self`: mutates the Rust value behind the shared pointer
p$notes       # list fields are R's business and are untouched
```

Every instance-method wrapper passes the object R dispatched with to `.Call()`,
and the generated prelude resolves the receiver through the same class-handle
unwrap that `ExternalPtr<T>` arguments use (`EXTERNALPTR.md`, "Passing Class
Objects to Standalone Functions"): a bare pointer, an R6 / S4 / S7 handle, an
environment or a list with `.ptr`, or an object with a `.ptr` attribute. The
`.ptr` element is found by name, so its position does not matter, and a `.ptr`
value that is not an external pointer counts as no handle. A receiver that
yields no pointer raises `expected a `Person` object (an external pointer, or an
R6/S4/S7 handle, environment, or list carrying one in `.ptr`), got VECSXP`; a
pointer to a different Rust type still fails the downcast with
`expected ExternalPtr<Person>, found `Other``.

This is a compatibility path, not a way to add state. A method returning `Self`
(or `Result<Self, E>` / `Option<Self>`) re-wraps its result as a fresh classed
pointer, `structure(<ptr>, class = "Person")`, so the list fields are gone on
the returned value, and a `self`-by-value method consumes the shared handle.
State the package owns belongs in sidecar fields.

## Standalone S3 methods (class defined elsewhere)

The `#[miniextendr(s3)]`-on-impl pattern above owns both the constructor and the methods: the Rust type `Person` *is* the S3 class. That is the right tool when Rust holds the canonical data. It is the wrong tool when:

- The class is defined in R (an existing `structure(list(), class = "my_thing")` or a package you don't control).
- The class is a vctrs type where the "object" is an R vector with a class attribute, not a Rust struct.
- You want to provide a method for a base R class (`print.data.frame`, `format.POSIXct`) from a Rust extension.

For those cases, drop the impl block and write a plain function:

```rust
use miniextendr_api::{miniextendr, SEXP};

/// Implements `format.percent` in Rust.
#[miniextendr(s3(generic = "format", class = "percent"))]
pub fn format_percent(x: SEXP, _dots: ...) -> Vec<String> {
    let data: &[f64] = unsafe { x.as_slice::<f64>() };
    data.iter().map(|v| format!("{:.1}%", v * 100.0)).collect()
}
```

What the macro does:

1. Emits an R wrapper named `format.percent` (the `<generic>.<class>` convention).
2. Adds `#' @method format percent` to the roxygen block. `devtools::document()` picks that up and writes `S3method(format, percent)` into `NAMESPACE`. You do not call `.S3method()` yourself.
3. Registers the underlying C entry point via `distributed_slice`, same as every other `#[miniextendr]` function.

Your Rust function receives exactly the arguments R dispatches with. For most S3 generics that is `(x, ...)`, so the first param is typed (`SEXP`, or a concrete type with `TryFromSexp`) and the last is `_dots: ...` to absorb the R-level `...`. If the generic takes more positional arguments (`vec_cast(x, to, ...)`), list them in order.

### Operator generics (`[`, `[[`, `$`, `==`, ...)

The generic name can be any R generic, including the operators:

```rust
#[miniextendr(s3(generic = "[", class = "percent"))]
pub fn subset_percent(x: Vec<f64>, i: Vec<i32>, _dots: ...) -> Vec<f64> { ... }

#[miniextendr(s3(generic = "$", class = "percent"))]
pub fn dollar_percent(x: Vec<f64>, name: String) -> Result<f64, String> { ... }
```

The wrapper name `[.percent` is not a syntactic R name, so the generated
definition is backtick-quoted, `` `[.percent` <- function(x, i, ...) ``, and
the roxygen line stays `#' @method [ percent` (which roxygen2 turns into
`S3method("[", percent)`). The same quoting applies to an impl-block method
that overrides its generic, `#[miniextendr(s3(generic = "[["))]` (#1475). Group
generics (`Ops`, `Math`) are not special-cased: write a method per operator,
or one on the group name (`s3(generic = "Ops", class = "thing")`, formals
`e1`, `e2`), which R calls for every operator in the group; a condition it
raises names the operator call, `x + 1` (#1851).

`$` methods receive the field name as a character string (`x$n` dispatches as
`` `$.percent`(x, "n") ``), so type that parameter `String` or `&str`.

### Replacement and extraction generics

A class built in R as a list (`structure(list(...), class = "thing")`) can
take its `$`, `[[`, `$<-`, `[[<-`, `[<-` and `names<-` methods from Rust:

```rust
use miniextendr_api::dots::Dots;
use miniextendr_api::{List, SEXP, miniextendr};

#[miniextendr(s3(generic = "$<-", class = "thing"))]
pub fn dollar_assign_thing(x: List, name: &str, value: SEXP) -> SEXP { /* … */ }

#[miniextendr(s3(generic = "[[<-", class = "thing"))]
pub fn element_assign_thing(x: List, i: SEXP, value: SEXP) -> SEXP { /* … */ }

#[miniextendr(s3(generic = "[<-", class = "thing"))]
pub fn subset_assign_thing(x: List, i: SEXP, _dots: &Dots, value: SEXP) -> SEXP { /* … */ }

#[miniextendr(s3(generic = "$", class = "thing"))]
pub fn dollar_thing(x: List, name: &str) -> SEXP { /* … */ }
```

The generated methods take the formals R passes:

```r
`$<-.thing` <- function(x, name, value) { ... }
`[[<-.thing` <- function(x, i, value) { ... }
`[<-.thing` <- function(x, i, ..., value) { ... }
`$.thing` <- function(x, name) { ... }
```

roxygen2 registers each one (`S3method("$<-",thing)`), and
`tools::checkS3methods()` and `tools::checkReplaceFuns()`, which `R CMD check`
runs, report nothing for them. `rpkg/src/rust/s3_replacement_tests.rs` is the
working fixture, and `rpkg/tests/testthat/test-s3-replacement.R` tests each
point below.

**How R calls a replacement method.** R turns an assignment into a getter
call, an edit of the value it got, and a replacement call with the result:

- `x$f$col[i] <- v` first calls `` `$.thing`(x, "f") ``, changes `col[i]` in
  that copy of `f`, then calls `` `$<-.thing`(`*tmp*`, "f", value = <the new f>) ``.
  The method sees the whole new element, never the changed cell.
- `x[["f"]]$col <- v` reads through `[[` and writes back through `[[<-`.
  `modifyList(x, list(f = v))` does the same for each name in its list.
- `x["f"] <- list(v)` goes through `[<-`.

**`name` is a string.** `x$f` and `x$f <- v` pass the name as a character
string (`"f"`), so a `&str` or `String` parameter fits `$` and `$<-`.

**`[[` and `[[<-` take `i: SEXP`.** `x[["f"]]` passes a string, `x[[2]]` a
double and `lapply()` an integer. A `&str` parameter's generated check refuses
the number before the method runs (`'<param>' must be character`, the error
`$<-` gives for a number as its `name`), so take a `SEXP` and branch on its
type, or use a type that converts from both.

**`[<-` takes `&Dots` before `value`.** `x[i, j] <- v` passes `j` as a further
positional argument, which the `...` collects. Without a `&Dots` parameter,
`x[1, 2] <- v` fails in R with `unused argument (2)`.

**`value` comes last.** R passes the new value by name, `value = v`, and
`R CMD check` (`tools::checkReplaceFuns()`) requires the last formal of every
replacement function, registered S3 methods included, to be `value`. A method
for a generic whose name ends in `<-` must therefore name its last parameter
`value` (`_value` counts, since the R name drops leading underscores);
anything else is a compile error:

```text
error: an S3 method for the replacement generic `$<-` must take the new value
as its last parameter, named `value`, but the last R argument is `new_value`.
R passes the new value as `value = ` (...), and `R CMD check` requires the
last argument of a replacement function to be named `value`.
```

The same holds for an impl-block method (`#[miniextendr(s3(generic = "[[<-"))]`
in a `#[miniextendr(s3)] impl`), whose generated `...` goes before `value`
(`x, i, ..., value`), so a trailing `&Dots` parameter is refused too.

**Return the object.** R assigns whatever the method returns to the variable:
a `$<-` method that returns `NULL` leaves `x` as `NULL`. Change a copy and
return it (`SexpExt::shallow_duplicate()`, rooted with a `ProtectScope` while
you allocate), since R may still share `x`'s elements with other bindings. An
impl-block method on an `ExternalPtr` class that changes `&mut self` and
returns `()`, `&mut Self` or `Result<&mut Self, E>` already returns `x`.

**Some forms reach a method only if one exists.** `names(x) <- v` uses the
default unless the class has a `names<-` method, and `names(x)[2] <- "y"`
calls that method with the whole new names vector. `attr<-` and `class<-` are
not generics: `attr(x, "a") <- v` and `class(x) <- v` never reach a method.

**`lapply()` and `str()` call `[[`.** `lapply()`, `sapply()`, `vapply()`,
`Map()` and `str()` read each element through `x[[i]]` with an integer `i`,
so a `[[` method runs for them too and must accept positions. A `for` loop,
`as.list()` and `unlist()` do not call it.

**The generated checks apply.** Each parameter keeps its R-side check, which
runs before the Rust function: `is.list(x)` for a `List` receiver
(`'x' must be a list`), a length-1 character vector for `&str`
(`'name' must be character`, `'name' must have length 1`). A `SEXP`
parameter has none.

**The call a condition reports is the assignment.** R calls a replacement
method with the temporary it assigns through and the whole new value,
`` `$<-.thing`(`*tmp*`, f, value = <the new f>) ``, and that is the call a
hand-written method's `stop()` would name. A generated method reports
`x$f <- value` instead (`x[["f"]] <- value`, `x["f"] <- value`,
`names(x) <- value`): compact, and never holding the value, which for
`x$f$col <- big` is the evaluated inner value inlined. The same rewrite gives
every generated S3 method the generic's call, `summary(x)` rather than
`summary.thing(x)` and `x + 1` from an `Ops` method, for every condition it
raises, the R-side checks above included; a method called directly by its
name reports that call. See
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md#s3-methods-the-generic-call)
(#1851).

### Subscript forms and the argument count

`x[i]`, `x[i, ]`, `x[, j]`, `x[i, j]` and `x[i, j, drop = FALSE]` all call
the same `[` method. Two things decide what a method can do with them.

**Declare `i`, `j` and `drop` as `Missing`.** An empty subscript is an empty
argument: `x[1, ]` binds `j` to it, and so does `x[, 2]` for `i`. The
wrapper reads a plain parameter (in its generated check, or for a `SEXP` in
the `.Call()`), which fails before the method runs with R's
`argument "j" is missing, with no default`. A
`Missing<T>` parameter takes it as `Missing::Absent`. A method that does not
name `j` at all gets `unused argument` for `x[1, 2]` unless it takes `...`
(`_dots: ...` or `&Dots`). Name `drop` too if `x[i, j, drop = FALSE]` should
reach the method.

**Take `NArgs` to tell `x[i]` from `x[i, ]`.** Both give the same `i` and a
missing `j`; only R's `nargs()` tells them apart, and `[.data.frame` uses it
to choose between list-style and matrix-style subscripting. A parameter of
type `miniextendr_api::NArgs` is filled with that count (#1860):

```rust
use miniextendr_api::{Missing, NArgs, SEXP, miniextendr};

#[miniextendr(s3(generic = "[", class = "mx_vec1"))]
pub fn mx_vec1_subset(
    x: Vec<f64>,
    i: Missing<Vec<i32>>,
    _j: Missing<SEXP>,
    drop: Missing<SEXP>,
    nargs: NArgs,
) -> Result<Vec<f64>, Vec1SubscriptError> {
    // `x` and a named `drop` count; every other argument is a subscript.
    let subscripts = nargs.get() - 1 - usize::from(drop.is_present());
    if subscripts > 1 {
        return Err(/* ... */);
    }
    /* x[i] or x[] */
}
```

`NArgs` is no R formal. The method above is
`` `[.mx_vec1` <- function(x, i, j, drop) ``, and its wrapper passes
`nargs()` where the parameter sits in the `.Call()`. A function without an
`NArgs` parameter keeps its wrapper unchanged. The count is the one `nargs()`
gives in the method, the call as typed:

| Typed | `nargs()` |
|---|---|
| `x[1:3]` | 2 |
| `x[1:3, ]` | 3 |
| `x[, 1:3]` | 3 |
| `x[1, 2]` | 3 |
| `x[1, , drop = FALSE]` | 4 |
| `x[]` | 2 |
| `x[i] <- v`, `x[] <- v` | 3 |
| `x[i, ] <- v`, `x[1, 2] <- v` | 4 |

`x` counts, every empty argument counts, and a named `drop = FALSE` counts;
the subscript slots are `nargs - 1`, less one when `drop` was given. In a
replacement method put the `NArgs` parameter anywhere, after `value`
included: `value` stays the last formal.

**An impl-block method has `...`.** The `[[` or `[` of a
`#[miniextendr(s3)] impl` gets `(x, i, ...)`, so `h[[i, j]]` and `h[[i, ]]`
put the extra subscript in the `...`, which the method never evaluates. It
runs as if `h[[i]]` had been typed; an `NArgs` parameter tells the forms apart
(`MxBagHandle::at` in `rpkg/src/rust/s3_nonsyntactic_tests.rs` refuses
`h[[i, ]]` this way).

**Forwarding to `[.data.frame`.** A class built on a data frame can hand
every form on as typed: build the call with an empty argument
(`SEXP::missing_arg()`) for each absent subscript within the slot count, add
`drop` when given, and evaluate it in base:

```rust
let slots = nargs.get() - 1 - usize::from(drop.is_present());
let mut call = RCall::new("[.data.frame").quoted_arg(x);
if slots >= 1 { call = subscript(call, i); } // Absent -> SEXP::missing_arg()
if slots >= 2 { call = subscript(call, j); }
if let Missing::Present(drop) = drop {
    call = call.named_quoted_arg("drop", drop);
}
call.eval_with_handlers(R_BaseEnv)
```

Call it by name, as above: the list-style branch of `[.data.frame` calls
`NextMethod()`, which fails when the call's head is the function itself.

`rpkg/src/rust/s3_subscript_tests.rs` has the fixtures: `[.mx_vec1` refuses
every two-subscript form with a classed error, `[.mx_frame` forwards each form
and gives results `identical()` to `[.data.frame`'s, `[<-.mx_nargs` records
the count of a replacement call, and an environment-class trait method takes
`NArgs` too. `rpkg/tests/testthat/test-s3-subscript.R` tests them. See
[MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md#argument-count-nargs) for
where `NArgs` is accepted.

### Double dispatch (vctrs)

A few generics dispatch on two arguments. vctrs calls `vec_ptype2(x, y, ...)` and resolves the method as `vec_ptype2.<class_of_x>.<class_of_y>`. Encode that as a dotted class string:

```rust
#[miniextendr(s3(generic = "vec_ptype2", class = "percent.percent"))]
pub fn vec_ptype2_percent_percent(_x: SEXP, _y: SEXP, _dots: ...) -> SEXP { ... }

#[miniextendr(s3(generic = "vec_cast", class = "percent.double"))]
pub fn vec_cast_percent_double(x: SEXP, _to: SEXP, _dots: ...) -> SEXP { ... }
```

The wrapper name becomes `vec_ptype2.percent.percent`, and the roxygen becomes `#' @method vec_ptype2 percent.percent`. That is what vctrs expects.

### vctrs `@importFrom`

The macro recognizes the vctrs generic names (`vec_ptype_abbr`, `vec_proxy`, `vec_restore`, `vec_ptype2`, `vec_cast`, etc.) and auto-injects `#' @importFrom vctrs <generic>` into the roxygen block. This forces vctrs to load before your method is registered, which is necessary for `R_GetCCallable` lookups (vctrs registers its native generics via ccallable).

For non-vctrs generics from other packages (e.g., `tibble::tbl_sum`), add the import manually:

```rust
/// @importFrom tibble tbl_sum
#[miniextendr(s3(generic = "tbl_sum", class = "my_tbl"))]
pub fn tbl_sum_my_tbl(x: SEXP, _dots: ...) -> Vec<String> { ... }
```

### Documentation pages for standalone methods

A standalone function without a page tag documents its own page, named after
the function (`man/<fn>.Rd`), as in plain roxygen2. A block without a
`@title` gets the function name as its title. Grouping functions on one page
is opt-in (#1289), through one of these tags:

- `/// @rdname topic` puts the function on `topic`'s page. To group the
  functions of one Rust file, give each of them `/// @rdname <file stem>`
  (`@rdname setters` in `setters.rs`).
- `/// @name topic` documents the function on its own `topic.Rd`, exactly as in
  plain roxygen2. A block that wants a custom topic name *and* a shared page
  spells out the `@rdname` as well.
- `/// @describeIn topic Short description.` puts it on `topic`'s page *and*
  lists it in that page's "Functions" section with the description. The
  description may wrap onto following `///` lines; they stay attached (#1476).
  roxygen2 rejects `@describeIn` next to `@rdname`.
  roxygen2 resolves `topic` to the destination *object's own* page
  (`topic.Rd`), not to wherever that object's block was sent with `@rdname`.
  So the destination must be documented under its own name: a function
  without a page tag already is; one sent elsewhere with `@rdname` needs
  `/// @rdname topic` back, otherwise the `@describeIn` block ends up alone
  on `topic.Rd`.

Grouping is opt-in because a shared page has one title: roxygen2 drops the
other blocks' titles without a warning, and their functions become aliases on
someone else's page. An `@inheritSection` between two blocks of one page also
resolves to that page itself, and roxygen2 copies the section once per block,
copies already inherited included, so a family of ten such functions can grow
one page to megabytes and stall `R CMD INSTALL`. On pages of their own,
`@inheritSection first_fn Section` inherits from `first_fn`'s page as intended
(`rpkg/src/rust/own_page_docs.rs` is the fixture).

A `@title` wrapped onto the next `///` line is folded back onto one line, and
a bare `@name` / `@rdname` takes the next `///` line as its topic, as roxygen2
itself would read it.

Functions are emitted in source order within a file (priority group first,
then file, then line), so the `@description` paragraphs and `\usage` entries
of a shared page appear in the order the Rust file defines them.

#### Parameters on shared pages

A parameter the Rust doc comment does not document gets a generated `@param`
line: `(no documentation available)`, or the choice list of a `choices` /
`match_arg` parameter. That keeps every argument of a function documented on
its own page, including a file-stem page it opts into. The line is left out
when the block takes its arguments from elsewhere:

- it has `@describeIn topic ...`, or an `@rdname topic` naming another page
  than its own or its file-stem page, so `topic`'s page documents them;
- it has `@inheritParams source` (or `@inherit source` including params),
  which fills in exactly the arguments the block leaves out.

A family of functions can then share one page whose block, often in R,
documents each argument once:

```r
# R/range_summaries.R
#' Range summaries
#'
#' @description
#' Summaries of where values sit relative to a range.
#'
#' @param values A numeric vector.
#' @param lower,upper Lower and upper bound of the range.
#' @name range_summaries
NULL
```

```rust
/// Width of the values' range.
/// @rdname range_summaries
#[miniextendr]
pub fn range_width(values: Vec<f64>) -> f64 { /* ... */ }

/// Values clamped into the range.
/// @rdname range_summaries
#[miniextendr]
pub fn range_clamp(values: Vec<f64>, lower: f64, upper: f64) -> Vec<f64> { /* ... */ }
```

roxygen2 keeps a single entry per argument name on a merged page, the one
from the block it reads last, so a generated line would replace the shared
description (next to a grouped `@param lower,upper` it would also add
separate `lower` and `upper` entries). `rpkg/src/rust/shared_param_docs.rs`
is the working fixture.

The same holds for the fixed lines a class generator adds for the formals it
injects: `@param x` / `@param ...` on an S3 method and its generic, `self` /
`...` on an S7 trait method's shortcut, and `.ptr` on an S7 class block.
They are left out when the method (or, for `.ptr`, the impl block) joins
another topic with `@rdname` or `@describeIn`: that topic's block documents
`x` and `...`. A method split onto a brand-new page with `@rdname` therefore
documents them in its own doc comment (``/// @param x A `Counter`.``), as
`GreetingBuilder::build` does in `rpkg/src/rust/pipe_builder_tests.rs`. On
the class page (no page tag, or `@rdname <Class>`) the lines stay. An
`@inheritParams` alone keeps the method on the class page, so it keeps them
too.

A file-stem page is shared by the functions of a Rust file that opt into it
with `@rdname <file stem>` (one of them may add `@name <file stem>` to give
the page a custom title). The registry treats that page as the file's own,
not as a topic defined elsewhere, and decides each generated line when it
writes `R/miniextendr-wrappers.R`: the line stays only when no function on
the page documents that argument itself, so a shared argument shows the one
real description once. `rpkg/src/rust/stem_page_docs.rs` is the fixture. An
`@rdname` naming any other topic joins it as described above.

The registry sees only the generated wrappers, not your R files. An R-file
block on a file-stem page (typically `#' @name <file stem>` on `NULL`, to
give the page its title and description) does not count as documenting an
argument, so a function on the page that leaves the argument out still gets
its generated line, and roxygen2 keeps the line of whichever block it reads
last. If the R file sorts before `R/<pkg>-wrappers.R`, the page takes the R
block's name and title, but `(no documentation available)` silently replaces
the R block's description of that argument. If it sorts after, the
description is kept, but the page is named and titled after the first
generated block. Document the argument in the doc comment of a function on
the page instead: the registry then drops the generated lines for it, and
the R block keeps the page's name, title and description.

An argument that no block documents is reported by `R CMD check`
("Undocumented arguments in Rd file"), so document it on the shared block or
in the function's own doc comment (a `@param` the function writes itself is
kept as is).

The shared block may live in any R file. roxygen2 gives a merged page the
`\name` and `\title` of the first block it reads, and it reads every block
sorted by `@order` (`Inf` when absent, so file order among unordered blocks).
Every generated block that joins another topic (`@describeIn`, an `@rdname`
naming another page, the S3 generic block that follows such a method)
carries `@order NaN`, and R's `order()` sorts `NaN` after `Inf`: the topic's
own block is read first wherever its file sorts, and the generated blocks
keep their source order among themselves. A topic made only of generated
blocks is named and titled by the first of them, as before. An `@order` you
write in the Rust doc comment is kept instead. The fixture's R file,
`R/range_summaries.R`, sorts after `R/miniextendr-wrappers.R` on purpose,
and `test-doc-shared-pages.R` checks the page name and title. Writing
`@order 1` on the shared R block has the same effect without relying on the
generated tag.

A method-level `@describeIn topic ...` works on the class methods whose R
wrapper is a function roxygen2 can list: S3 instance methods (listed as
`generic(Class)`), the static methods of S3, S4, S7 and vctrs classes, S4
constructors, and `as.<target>()` coercions. The method block then carries
neither `@name` nor `@rdname` (roxygen2 rejects both next to `@describeIn`),
and the S3 generic block, whose `@name generic.Class` is the method's alias,
follows it to `topic`. On S4 and S7 instance methods (registered by
`setMethod()` / `S7::method<-`), on Env and R6 methods (`Class$method <-`),
and on the S3, vctrs and S7 constructors (documented by the class block),
`@describeIn` is a compile error that points at `@rdname`.

Trait-impl methods (`impl Trait for Type`) take the same page tags on their
own wrapper block: `@describeIn`, `@rdname`, `@name`, `@order`, and the
inheritance tags `@inheritParams`, `@inherit` and `@inheritDotParams`. Their
prose, `@examples` and other tags are not forwarded, except on env trait
methods, the `Type$Trait$method` statics of S3 and vctrs classes and the
`attr(Type, "Trait")$method` statics of S7 classes, which are documented
like env methods (see
[Impl-block doc tags](CLASS_SYSTEMS.md#impl-block-doc-tags)). `@describeIn` works
where that block documents an R function or method: S3 and vctrs instance
methods (`generic.Type`), S4 instance methods (the block sits on the
method's own `setMethod()` call, so unlike an inherent S4 method it is
listed as `generic(Type)`), S4 static methods (`Type_Trait_method()`), and
S7 instance methods through their fast-path shortcut (`Type_method()`). The
S4 and S7 generics stay on the type page, and on a joined page the S7
shortcut keeps only the first line of its advisory text, as its title. On
the trait namespace members (Env and R6 methods, and the static methods of
S3, vctrs and S7) and on S7 methods with
`s7(no_shortcut)`, `@describeIn` is a compile error that points at
`@rdname`. `RangeMeasure for RangeBox` in
`rpkg/src/rust/shared_param_docs.rs` is the fixture.

### Constraints

- `s3(...)` requires `class`. `generic` defaults to the Rust function name if omitted, but supplying it explicitly is the convention.
- Cannot be combined with `r_name = "..."`. The S3 naming (`<generic>.<class>`) already fixes the R wrapper name.
- The constructor for the class is out of scope for the method function. Either write it in R, or write a separate `#[miniextendr]` function that returns `structure(x, class = "my_class")` via `new_vctr(...)` or equivalent helpers in `miniextendr_api::vctrs`.

### When to pick which

| You have... | Use |
|---|---|
| A Rust struct that owns the data, methods hang off it | `#[miniextendr(s3)] impl MyType { ... }` |
| An R-side class (vctrs, base R, another package), methods are Rust logic | standalone `#[miniextendr(s3(generic = ..., class = ...))]` functions |
| A vctrs class authored here | `#[miniextendr(vctrs)]` on impl for the constructor + static methods, standalone `s3(...)` functions for the dispatch-on-class-attribute generics |

## Summary Table

| Method | Receiver | Return | Generated R | R Convention |
|--------|----------|--------|-------------|--------------|
| `format` | `&self` | `String` | `.Call(...)` | Returns formatted string (visible) |
| `print` | `&mut self` | `Invisible<()>` | `.Call(...); invisible(x)` | Returns self invisibly |
| `print` | `&mut self` | `()` | `.Call(...); x` | Returns self visibly (prints twice at the console) |
| `print` | `&self` | `()` | `.Call(...)` | Returns NULL (works but unconventional) |
| custom | `&self` | any | `.Call(...)` | Returns value directly |
| custom | `&mut self` | `()` | `.Call(...); x` | Chainable mutation |
| builder | `&mut self` | `&mut Self` | `.Call(...); x` | In-place builder; returns same handle, pipes under `\|>` |
| builder | `&self` | `&Self` | `.Call(...); x` | Read-only builder step; returns same handle |

## See Also

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md): overview of all 6 class systems (env, R6, S3, S4, S7, vctrs)
- [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md): using dots and typed_list validation
- [VCTRS.md](VCTRS.md): vctrs-compatible S3 classes with `format` methods
