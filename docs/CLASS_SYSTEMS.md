# Class Systems in miniextendr

miniextendr supports five R class systems. This guide helps you choose the right one for your use case.

## Quick Comparison

| Feature | Env | R6 | S3 | S4 | S7 |
|---------|-----|----|----|----|----|
| **Attribute** | `#[miniextendr]` | `#[miniextendr(r6)]` | `#[miniextendr(s3)]` | `#[miniextendr(s4)]` | `#[miniextendr(s7)]` |
| **Method Call** | `obj$method()` | `obj$method()` | `generic(obj)` | `generic(obj)` | `generic(obj)` |
| **Encapsulation** | Weak | Strong | None | Moderate | Strong |
| **Dependencies** | None | R6 package | None | methods package | S7 package |
| **Active Bindings** | No | Yes | No | No | Yes (computed/dynamic properties) |
| **Inheritance** | No | Limited | S3 dispatch | S4 dispatch | S7 dispatch |
| **Best For** | Simple APIs | Complex state | Tidyverse compat | Bioconductor | Modern OOP |

## Choosing a Class System

```text
                         ┌─────────────────────────────────────┐
                         │  Do you need method dispatch on     │
                         │  object type (polymorphism)?        │
                         └─────────────────────────────────────┘
                                         │
                    ┌────────────────────┴────────────────────┐
                    │ No                                      │ Yes
                    ▼                                         ▼
         ┌──────────────────┐              ┌──────────────────────────────┐
         │   Use Env style  │              │  Do you need tidyverse       │
         │   (simplest)     │              │  compatibility?              │
         └──────────────────┘              └──────────────────────────────┘
                                                        │
                                   ┌────────────────────┴────────────────────┐
                                   │ Yes                                     │ No
                                   ▼                                         ▼
                        ┌───────────────────┐           ┌──────────────────────────────┐
                        │  Use S3           │           │  Need reference semantics    │
                        │  (generic.class)  │           │  (modify in place)?          │
                        └───────────────────┘           └──────────────────────────────┘
                                                                     │
                                              ┌──────────────────────┴────────────────────┐
                                              │ Yes                                       │ No
                                              ▼                                           ▼
                                   ┌───────────────────┐                   ┌───────────────────────┐
                                   │  Use R6           │                   │  Modern or legacy?    │
                                   │  (encapsulation)  │                   │                       │
                                   └───────────────────┘                   └───────────────────────┘
                                                                                     │
                                                             ┌───────────────────────┴───────────────┐
                                                             │ Modern                                │ Legacy
                                                             ▼                                       ▼
                                                  ┌───────────────────┐                 ┌───────────────────┐
                                                  │  Use S7           │                 │  Use S4           │
                                                  │  (new standard)   │                 │  (Bioconductor)   │
                                                  └───────────────────┘                 └───────────────────┘
```

---

## Environment Style (Default)

The simplest approach. Methods are functions attached to an environment.

### Rust Code

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter {
    value: i32,
}

#[miniextendr]  // env is default
impl Counter {
    /// Create a new counter.
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }

    /// Get the current value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Increment by one.
    pub fn inc(&mut self) {
        self.value += 1;
    }
}
```

### Generated R Code

The `.Call()` symbols are prefixed with your crate's name (`mypkg` in these
examples) so that two packages loaded into the same webR session can never
collide on a C symbol (see `docs/WEBR.md`). The prefix is invisible to package
users — R-facing function and class names are unchanged.

```r
Counter <- new.env(parent = emptyenv())

Counter$new <- function(initial) {
  if (!isTRUE(is.integer(initial))) .miniextendr_arg_error("initial", "must be integer")
  if (!isTRUE(length(initial) == 1L)) .miniextendr_arg_error("initial", "must have length 1")
  .val <- .Call(C_mypkg_Counter__new, .call = sys.call(), initial)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))) return(.miniextendr_raise_condition(.val, sys.call()))
  self <- .val
  class(self) <- "Counter"
  self
}

Counter$value <- function() {
  .val <- .Call(C_mypkg_Counter__value, .call = sys.call(), .mx_self)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))) return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

Counter$inc <- function() {
  .val <- .Call(C_mypkg_Counter__inc, .call = sys.call(), .mx_self)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))) return(.miniextendr_raise_condition(.val, sys.call()))
  .mx_self
}

`$.Counter` <- function(self, name) {
  .miniextendr_env_method(self, name, Counter)
}
`[[.Counter` <- `$.Counter`
```

The object *is* the external pointer: the constructor takes the `EXTPTRSXP`
returned by `.Call()`, sets its class attribute, and returns it. There is no
list and no `.ptr` field. The methods live in the `Counter` environment and
take no receiver formal; each body refers to `.mx_self`. No Rust parameter
can produce a dotted R name, so a method may take arguments named `self` or
`x`. An R formal is the Rust parameter name without leading underscores, so
`_self: i32` becomes `self`; clippy's `duplicate_underscore_argument` lint
flags `_self` beside `&self`, so allow it where you use one. `.mx_self` resolves
through the `$.Counter` dispatcher: `c$value()` calls `$.Counter(c, "value")`,
which calls `.miniextendr_env_method`, a helper the wrapper file defines (its
source is in `miniextendr-api/src/registry.rs`). The helper fetches
`Counter$value` and returns it with a new enclosing environment that holds
only `.mx_self`. That environment's parent is the method's original environment,
the package namespace, so package objects used by method bodies and parameter
defaults keep resolving there. The method stored in `Counter` is unchanged, and each
returned closure keeps its own receiver alive. `[[` is aliased to the same
dispatcher. A `&mut self` method returns `.mx_self`, so calls
chain. Errors raised from Rust come back as a tagged value and are re-raised by
`.miniextendr_raise_condition` with the call attributed to the user's frame
(see `docs/CALL_ATTRIBUTION.md`).

An Env object's flat trait lookup (`obj$method()`) requires one matching
trait. If several traits provide the method, it raises
`miniextendr_ambiguous_method` with the method name in `condition$method` and
the matching trait names in `condition$traits`, reporting the call as written
(`obj$method`). Select the trait explicitly
with `obj$Trait$method()`. An inherent method of that name takes precedence.
The same rule applies to trait static methods and associated constants.

### Usage

```r
c <- Counter$new(0L)
c$value()      # 0
c$inc()        # returns the object, so c$inc()$inc() also works
c$value()      # 1
c[["value"]]() # same dispatch as `$`
```

Trait methods implemented for an env class sit in a namespace named after the
trait: `c$MyTrait$method()` binds the receiver for you, and
`Counter$MyTrait$method(c)` is the standalone form. Its receiver formal is
`.mx_self`, as for R6 trait methods (`Type$Trait$method(obj, ...)`), so trait
methods may take arguments named `x`, `self` or `private`; pass the object by
position. Calling an inherent method without an object (`Counter$value()`)
fails with `object '.mx_self' not found`, because nothing bound `.mx_self`.

### When to Use

- Simple APIs with few methods
- No need for method dispatch
- Minimal dependencies
- Quick prototyping

---

## R6 Style

Full-featured reference classes with encapsulation.

### Rust Code

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Rectangle {
    width: f64,
    height: f64,
}

#[miniextendr(r6)]
impl Rectangle {
    pub fn new(width: f64, height: f64) -> Self {
        Rectangle { width, height }
    }

    pub fn get_width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width;
    }

    /// Active binding for computed property.
    #[miniextendr(r6(active))]
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// Private method.
    fn validate(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }

    /// Static method.
    pub fn square(size: f64) -> Self {
        Rectangle { width: size, height: size }
    }
}
```

### Generated R Code

```r
Rectangle <- R6::R6Class("Rectangle",
    public = list(
        initialize = function(width, height, .ptr = NULL) {
            if (!is.null(.ptr)) {
                private$.ptr <- .ptr
            } else {
                private$.ptr <- .Call(C_mypkg_Rectangle__new, width, height)
            }
        },
        get_width = function() {
            .Call(C_mypkg_Rectangle__get_width, private$.ptr)
        },
        set_width = function(width) {
            .Call(C_mypkg_Rectangle__set_width, private$.ptr, width)
        }
    ),
    private = list(
        .ptr = NULL,
        validate = function() {
            .Call(C_mypkg_Rectangle__validate, private$.ptr)
        }
    ),
    active = list(
        area = function() {
            .Call(C_mypkg_Rectangle__area, private$.ptr)
        }
    ),
    lock_objects = TRUE,
    lock_class = FALSE,
    cloneable = FALSE
)

# Static method
Rectangle$square <- function(size) {
    Rectangle$new(.ptr = .Call(C_mypkg_Rectangle__square, size))
}
```

### Usage

```r
r <- Rectangle$new(3, 4)
r$get_width()    # 3
r$area           # 12 (active binding, no parens!)
r$set_width(5)
r$area           # 20

# Static method
s <- Rectangle$square(5)
s$area           # 25
```

### When to Use

- Complex state management
- Need private methods
- Active bindings (computed properties)
- Reference semantics (modify in place)

### Extending a generated R6 class from R

When you need to add R-only methods to a miniextendr-scaffolded R6 class (e.g. convenience
wrappers, formatting helpers, or methods that delegate to other R packages), roxygen2 8.0.0
lets you document those additions outside the generated class body.

**Primary form** — place a roxygen block directly above a `$set()` call; roxygen2
auto-associates it with the class:

```r
#' @description Return a formatted label for the object.
#' @return A character string.
Rectangle$set("public", "label", function() {
  paste0("Rectangle(", self$get_width(), " x ", private$.height, ")")
})
```

**Fallback form** — when the `$set()` call escapes roxygen2's source-tracing (e.g. it lives
inside a helper function or a conditional block), attach a `@R6method` tag to a bare `NULL`:

```r
#' @R6method Rectangle$label
#' @description Return a formatted label for the object.
#' @return A character string.
NULL

Rectangle$set("public", "label", function() {
  paste0("Rectangle(", self$get_width(), " x ", private$.height, ")")
})
```

The `@R6method ClassName$method_name` tag tells roxygen2 where to anchor the documentation
even when it cannot trace the code attachment itself.

### Field Access via Sidecar

For R6 classes, `#[r_data]` sidecar fields become active bindings with
`r6(r_data_accessors)`:

```rust
#[derive(ExternalPtr)]
pub struct MyStruct {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub name: String,
    #[r_data]
    pub value: f64,
}

#[miniextendr(r6(r_data_accessors))]
impl MyStruct {
    pub fn new(name: String, value: f64) -> Self {
        MyStruct { _r: RSidecar, name, value }
    }
}
```

This generates `obj$name` and `obj$value` active bindings, with setters
(`obj$name <- "new"`). See "Direct Field Access via Sidecar" below.

---

## S3 Style

Traditional R generic function dispatch.

### Rust Code

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Person {
    name: String,
    age: i32,
}

#[miniextendr(s3)]
impl Person {
    pub fn new(name: String, age: i32) -> Self {
        Person { name, age }
    }

    /// Implements print.Person - `&mut self` hands back `x`; the
    /// `Invisible<()>` marker makes that `invisible(x)` (#1213)
    #[miniextendr(s3(generic = "print"))]
    pub fn show(&mut self) -> Invisible<()> {
        println!("Person: {}, age {}", self.name, self.age);
        Invisible(())
    }

    /// Implements format.Person
    #[miniextendr(s3(generic = "format"))]
    pub fn fmt(&self) -> String {
        format!("{} ({})", self.name, self.age)
    }

    pub fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name)
    }
}
```

### Generated R Code

```r
#' @export
new_person <- function(name, age) {
    ptr <- .Call(C_mypkg_Person__new, name, age)
    structure(ptr, class = "Person")
}

#' @export
print.Person <- function(x, ...) {
    .Call(C_mypkg_Person__show, x)
    invisible(x)
}

#' @export
format.Person <- function(x, ...) {
    .Call(C_mypkg_Person__fmt, x)
}

#' @export
greet <- function(x, ...) UseMethod("greet")

#' @export
greet.Person <- function(x, ...) {
    .Call(C_mypkg_Person__greet, x)
}
```

### Usage

```r
p <- new_person("Alice", 30)
print(p)         # Person: Alice, age 30
format(p)        # "Alice (30)"
greet(p)         # "Hello, I'm Alice!"

# Works with tidyverse
tibble::tibble(person = list(p))
```

### Base non-generic collisions

A method name whose only existing binding is a **plain (non-generic) base or
stats closure** — `var`, `get`, `row`, `col`, `diag`, `reshape`, … — is
classified and shadowed automatically (#1248): the generated code checks
`utils::isS3stdGeneric()` / `methods::isGeneric()` / `is.primitive()` on the
existing binding, and if none apply, defines a package-local `UseMethod`
generic in its place. Ordinary (non-dispatching) calls to the masked
function, e.g. `var(1:10)`, keep working — a default method wrapping the
original closure is registered via `base::registerS3method()`, so it lives
only in the namespace's S3 methods table (never as a `name.default`
namespace binding, which would trip roxygen2's dynamic S3-export scan).
This mirrors the S7 class system's #1114 classifier.

`utils::isS3stdGeneric()` / `methods::isGeneric()` are called unconditionally
by every generated S3 (and vctrs) generic guard, so packages using the S3 or
vctrs class systems must import `methods` and `utils`:
`minirextendr::use_s3()` (or `use_vctrs()`) declares both.

### When to Use

- Tidyverse integration
- Extending existing generics (print, format, etc.)
- vctrs-compatible types
- Simple polymorphism

---

## S4 Style

Formal class system with slots and method signatures.

### Rust Code

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Gene {
    symbol: String,
    chromosome: i32,
}

#[miniextendr(s4)]
impl Gene {
    pub fn new(symbol: String, chromosome: i32) -> Self {
        Gene { symbol, chromosome }
    }

    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    pub fn chromosome(&self) -> i32 {
        self.chromosome
    }

    /// `Invisible<()>`: a void method returns the object, and `show()` at
    /// the console would print it again (#1213)
    #[miniextendr(s4(generic = "show", receiver = "object", no_dots))]
    pub fn display(&self) -> Invisible<()> {
        println!("Gene {} on chr{}", self.symbol, self.chromosome);
        Invisible(())
    }
}
```

### Generated R Code

```r
setClass("Gene", contains = "externalptr")

#' @export
Gene <- function(symbol, chromosome) {
    ptr <- .Call(C_mypkg_Gene__new, symbol, chromosome)
    new("Gene", ptr)
}

setGeneric("symbol", function(object) standardGeneric("symbol"))
setMethod("symbol", "Gene", function(object) {
    .Call(C_mypkg_Gene__symbol, object)
})

setGeneric("chromosome", function(object) standardGeneric("chromosome"))
setMethod("chromosome", "Gene", function(object) {
    .Call(C_mypkg_Gene__chromosome, object)
})

setMethod("show", "Gene", function(object) {
    .Call(C_mypkg_Gene__display, object)
})
```

### Usage

```r
g <- Gene("TP53", 17L)
symbol(g)       # "TP53"
chromosome(g)   # 17
show(g)         # Gene TP53 on chr17
```

### Methods on an existing generic (S3 and S4)

`s3(generic = "...")` and `s4(generic = "...")` add a method to a generic that
already exists: base's `print`, `format` or `summary`, `methods::show`, the
primitive `length`, or a generic of an imported package. The method's receiver
formal is `x`; when the generic names its first argument differently, say so
with `receiver`, and drop the `...` the wrapper otherwise appends with
`no_dots` when the generic has none:

```rust
#[miniextendr(s3(generic = "summary", receiver = "object"))]
pub fn summarise(&self) -> String { ... }  // summary.Gene <- function(object, ...)

#[miniextendr(s4(generic = "show", receiver = "object", no_dots))]
pub fn display(&self) -> Invisible<()> { ... }  // setMethod("show", c(object = "Gene"), function(object) ...)
```

The receiver is what the wrapper reads the pointer from and what a void or
`&mut self` method hands back. The generic's other formals are ordinary Rust
parameters (`all.equal(target, current, ...)` is `receiver = "target"` plus a
`current` parameter); a parameter whose R name is the receiver is a compile
error. A `&Dots` parameter still supplies `...` under `no_dots`. `receiver` and
`no_dots` need `generic` and an instance method; trait impl methods accept all
three.

An explicit S3 `generic` never defines a generic. An explicit S4 `generic` is
defined only when no generic or function of that name is visible from the
package, so the method attaches to `show` or `length` instead of masking them
with a generic of the package's own. A generic from an installed copy of the
same package does not count, so `load_all()` with that copy attached still
defines it. A name nothing defines is created as
`setGeneric(name, function(<receiver>, ...))`, but gets no generated help page
(#2039); without `generic`, an S4 method uses the documented
`s4_<method>` generic.

### When to Use

- Bioconductor packages
- Formal class hierarchies
- Strict type checking
- Legacy S4 codebases

---

## S7 Style

Modern OOP system (successor to S3/S4).

### Rust Code

```rust
use miniextendr_api::ExternalPtr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct Point {
    x: f64,
    y: f64,
}

#[miniextendr(s7)]
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// `&mut self` hands back `x`; `Invisible<()>` makes it `invisible(x)` (#1213)
    #[miniextendr(s7(generic = "base::print"))]
    pub fn show(&mut self) -> Invisible<()> {
        println!("Point({}, {})", self.x, self.y);
        Invisible(())
    }
}
```

### Generated R Code

```r
Point <- S7::new_class("Point",
    properties = list(
        .ptr = S7::class_any
    ),
    constructor = function(x, y, .ptr = NULL) {
        if (!is.null(.ptr)) {
            S7::new_object(S7::S7_object(), .ptr = .ptr)
        } else {
            S7::new_object(S7::S7_object(),
                .ptr = .Call(C_mypkg_Point__new, x, y))
        }
    }
)

S7::method(x, Point) <- function(x) {
    .Call(C_mypkg_Point__x, x@.ptr)
}

S7::method(y, Point) <- function(x) {
    .Call(C_mypkg_Point__y, x@.ptr)
}

S7::method(distance, Point) <- function(x, other) {
    .Call(C_mypkg_Point__distance, x@.ptr, other@.ptr)
}

S7::method(print, Point) <- function(x, ...) {
    .Call(C_mypkg_Point__show, x@.ptr)
    invisible(x)
}
```

### Usage

```r
p1 <- Point(0, 0)
p2 <- Point(3, 4)
x(p1)              # 0
distance(p1, p2)   # 5
print(p1)          # Point(0, 0)
```

### Fast-path dispatch shortcuts

Every non-fallback S7 instance method also gets a plain function
`<ClassName>_<method>(.mx_self, ...)` whose body calls the underlying Rust routine
directly, bypassing `S7::S7_dispatch()` (the class walk + method-table lookup).
On hot loops this is several times faster than calling the generic. Trait-impl
S7 methods (`#[miniextendr(s7)] impl Trait for Type`) get the same shortcuts.
The dotted receiver leaves an argument named `self` free; pass the object by
position. The generic's dispatch arguments are unchanged.

```r
# Generic (full S7 dispatch — honours subclass overrides)
distance(p1, p2)
# Shortcut (direct .Call — no dispatch)
Point_distance(p1, p2)
```

**Footgun:** the shortcut does not perform subclass dispatch — a method
override defined on a child class will *not* be honoured. Use the generic when
subclassing is possible.

Opt a method out with `s7(no_shortcut)` (the generic + method registration are
unaffected):

```rust
#[miniextendr(s7(no_shortcut))]
pub fn get(&self) -> i32 { ... }
```

Shortcut names share one R namespace with the `<ClassName>_<static_method>`
functions emitted for static methods. A collision inside one impl block
(typically via an `r_name` alias) is a compile error suggesting a rename or
`s7(no_shortcut)`. The wrapper writer checks across separate expansions too,
including `#[derive(ExternalPtr)]` sidecar accessors
(`<ClassName>_get_<field>` / `<ClassName>_set_<field>`), free functions and
other class impl blocks. Rename the colliding method or field, or use
`s7(no_shortcut)` (see #991).

The writer's other name checks cover every class system; see
[Names the wrapper writer rejects](#names-the-wrapper-writer-rejects).

### Which generic a method attaches to

The generic is the method's R name (`r_name`, or the Rust name), unless
`s7(generic = "...")` names another one. Three cases:

- **The package's own generic**: any unqualified name, such as `area` or a new
  operator `%scale%`. If no usable generic of that name exists when the package
  loads, a package-local `S7::new_generic()` is defined (shadowing a plain,
  non-generic function of the same name); otherwise the existing S7, S3, S4 or
  primitive generic is reused. The generic is exported and gets a standalone
  help page listing the classes with methods for it.
- **Another package's generic**: `s7(generic = "pkg::name")`, such as
  `base::print` or `generics::tidy`. The generic is bound with
  `S7::new_external_generic("pkg", "name", <dispatch args>)` and is neither
  exported nor documented here. The package does not have to import `pkg`, or
  even list more than `Suggests:` for it: when `pkg` is not loaded, S7 attaches
  the method once it loads. That needs `S7::methods_register()` in the
  package's `.onLoad()`, as S7 recommends for every package (see
  "Registering methods when the package loads" below).
- **A base operator** (`[`, `[[`, `$`, the `Ops` group, `%*%`): the method
  attaches to base's operator directly. Nothing is defined, exported or
  documented as a package generic. `` S7::method(`[[`, Bag) <- f `` is a
  replacement call, so R binds `[[` in the package namespace; the generated
  `base::rm(list = "[[")` on the next line removes that binding. With it, R
  would take `[[` for a generic of the package's own and register the
  package's S3 `[[` methods in the namespace's own table, where a user's
  `x[[i]]` never finds them.

### Multiple dispatch

`s7(dispatch = "x, other")` makes the generic dispatch on several arguments.
The first name is the receiver (the object the method is defined on, `x` by
default); the others are the method's leading parameters, in order:

```rust
#[miniextendr(s7)]
impl Left {
    /// Pair `x` with `other`.
    #[miniextendr(s7(dispatch = "x, other"))]
    pub fn pair(&self, other: f64, sep: String) -> String { ... }
}
```

```r
pair <- S7::new_generic("pair", c("x", "other"), function(x, other, ...) S7::S7_dispatch())
S7::method(pair, list(Left, S7::class_any)) <- function(x, other, sep, ...) {
  .Call(C_mypkg_Left__pair, .call = sys.call(), x@.ptr, other, sep)
}
```

The receiver is registered for the class, and every other dispatch argument for
`S7::class_any`. The method's own argument checks and Rust conversions are the
type check, and their errors name the parameter. No single S7 class matches
what a parameter accepts in general (a `&T` takes the class object, a bare
external pointer or a list with `.ptr`; `coerce` widens numeric
parameters), so a narrower class would turn accepted values into "Can't find
method" errors. A more specific method registered in R,
such as `S7::method(pair, list(Left, Right))`, still takes precedence over the
generated one for those argument classes.

The names are checked at compile time: they must be distinct, syntactic
argument names; the receiver name cannot also be a parameter name; the others
must be the method's first parameters in the same order and cannot have
defaults (S7 rejects defaults on dispatch arguments). With `s7(no_dots)` the
generic has no `...`, so S7 requires every formal of the method to be a
dispatch argument.

### Operator methods

An instance method can be called through R's operator syntax by giving it an
operator as its generic:

```rust
#[miniextendr(s7)]
impl Bag {
    /// `bag[i]`, registered on base's `[`.
    #[miniextendr(s7(generic = "["))]
    pub fn subset(&self, i: Vec<i32>) -> Vec<f64> { ... }

    /// `bag[[i]]`
    #[miniextendr(r_name = "[[")]
    pub fn at(&self, i: i32) -> f64 { ... }

    /// `bag %scale% k`, a package-local generic.
    #[miniextendr(r_name = "%scale%")]
    pub fn scale(&self, k: f64) -> Vec<f64> { ... }

    /// `bag %cat% other`, also a package-local generic.
    #[miniextendr(s7(generic = "%cat%"))]
    pub fn concat(&self, other: Vec<f64>) -> Vec<f64> { ... }
}
```

The generated R backtick-quotes the operator wherever it is an R symbol
(`` S7::method(`[[`, Bag) <- function(x, i, ...) ``) and exports a new
generic such as `%scale%` as `export("%scale%")`. An operator `r_name` gets no
fast-path shortcut, because `Bag_[[` is not a syntactic R name; the
`s7(generic = ...)` spelling on an ordinary Rust name keeps one (`Bag_subset()`
and `Bag_concat()` above).

R's `Ops` group (`+`, `-`, `*`, `/`, `^`, `%%`, `%/%`, `==`, `!=`, `<`, `<=`,
`>=`, `>`, `&`, `|`) and `%*%` are dispatched by S7 on both operands, through
its own `(e1, e2, ...)` and `(x, y, ...)` generics. A method on one of them
dispatches on that pair without an `s7(dispatch = ...)`: the receiver becomes
`e1` (`x` for `%*%`), and the method takes exactly one parameter, the right
operand, which must be named `e2` (`y`):

```rust
#[miniextendr(s7)]
impl Money {
    /// `a + b`
    #[miniextendr(s7(generic = "+"))]
    pub fn add(&self, e2: &Money) -> Self { ... }

    /// `a * k`
    #[miniextendr(r_name = "*")]
    pub fn times(&self, e2: f64) -> Self { ... }
}
```

```r
S7::method(`+`, list(Money, S7::class_any)) <- function(e1, e2, ...) { ... }
```

A wrong operand name, a second parameter, a conflicting `s7(dispatch = ...)` or
`s7(fallback)` are compile errors, and so is `!`, which S7 does not dispatch.
Two S7 limitations apply to these methods: only `Money + x` is registered, so
`1 + money` needs a reversed method in R
(`` S7::method(`+`, list(S7::class_any, Money)) ``), and unary `-money` fails
inside S7, which always calls the method with two operands.

### Registering methods when the package loads

A package with S7 classes needs an `.onLoad()` that calls
`S7::methods_register()`:

```r
.onLoad <- function(libname, pkgname) {
  suppressMessages(S7::methods_register())
}
```

S7 records a method for a generic that belongs to another package when the
package is built: a base operator (`bag[i]`, `bag[[i]]`, `money + 1`), a base
generic (`format()`, `print()`) or another package's generic
(`s7(generic = "pkg::name")`). Only `S7::methods_register()` registers those
methods again in a new R session. Without it they work in the session that
installed the package and fail in every later one; `bag[i]` then errors with
"S7 objects are not subsettable". Methods on the package's own generics are
not affected.

`minirextendr::use_s7()` writes this hook to `R/zzz.R` when the package has no
`.onLoad()`, and names the line to add when it has one.
`minirextendr::miniextendr_doctor()` warns when an S7 package's `.onLoad()` does
not call it. The `suppressMessages()` hides the "Overwriting method" messages
S7 prints under `devtools::load_all()`, which registers such methods twice.

### When to Use

- New packages without legacy constraints
- Clean, modern OOP design
- Computed and dynamic properties (see below)
- S7 ecosystem integration

### S7 Computed and Dynamic Properties

S7 supports properties that are computed from Rust methods. Use `#[miniextendr(s7(getter))]` for read-only computed properties and add `#[miniextendr(s7(setter, prop = "name"))]` for read-write dynamic properties.

#### Rust Code

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Range {
    start: f64,
    end: f64,
}

#[miniextendr(s7)]
impl Range {
    pub fn new(start: f64, end: f64) -> Self {
        Range { start, end }
    }

    /// Computed property (read-only): length of the range.
    /// Accessed as obj@length in R.
    #[miniextendr(s7(getter))]
    pub fn length(&self) -> f64 {
        self.end - self.start
    }

    /// Dynamic property getter: read the midpoint.
    #[miniextendr(s7(getter, prop = "midpoint"))]
    pub fn get_midpoint(&self) -> f64 {
        (self.start + self.end) / 2.0
    }

    /// Dynamic property setter: set the midpoint.
    /// Adjusts start and end to maintain length while centering on new midpoint.
    #[miniextendr(s7(setter, prop = "midpoint"))]
    pub fn set_midpoint(&mut self, value: f64) {
        let half = (self.end - self.start) / 2.0;
        self.start = value - half;
        self.end = value + half;
    }

    /// Regular method (not a property).
    pub fn start(&self) -> f64 {
        self.start
    }
}
```

#### Generated R Code

```r
Range <- S7::new_class("Range",
    properties = list(
        .ptr = S7::class_any,
        length = S7::new_property(
            getter = function(self) .Call(C_mypkg_Range__length, self@.ptr)
        ),
        midpoint = S7::new_property(
            getter = function(self) .Call(C_mypkg_Range__get_midpoint, self@.ptr),
            setter = function(self, value) {
                .Call(C_mypkg_Range__set_midpoint, self@.ptr, value)
                self
            }
        )
    ),
    constructor = function(start, end, .ptr = NULL) { ... }
)

# Regular method as S7 generic
S7::method(start, Range) <- function(x, ...) .Call(C_mypkg_Range__start, x@.ptr)
```

#### Usage

```r
r <- Range(0, 10)

# Computed property (read-only)
r@length         # 10

# Dynamic property (read-write)
r@midpoint       # 5
r@midpoint <- 10 # Adjusts start/end
r@midpoint       # 10
start(r)         # 5 (new start after midpoint shift)
r@length         # 10 (length preserved)
```

#### Property Attributes

| Attribute | Description |
|-----------|-------------|
| `#[miniextendr(s7(getter))]` | Read-only computed property. Property name = method name. |
| `#[miniextendr(s7(getter, prop = "name"))]` | Getter with custom property name. |
| `#[miniextendr(s7(setter, prop = "name"))]` | Setter for a dynamic property. Must match a getter's `prop` name. |

**Rules:**
- A getter without a setter creates a computed (read-only) property
- A getter + setter with the same `prop` name creates a dynamic (read-write) property
- Property methods are NOT exposed as S7 generics (accessed via `@` only)
- Setters must take exactly one parameter (the new value)

---

## Feature Comparison Matrix

### Constructor Patterns

| System | Constructor Name | Returns |
|--------|------------------|---------|
| Env | `TypeName$new()` | Environment with class |
| R6 | `TypeName$new()` | R6 object |
| S3 | `new_typename()` | Object with class attribute |
| S4 | `TypeName()` | S4 object |
| S7 | `TypeName()` | S7 object |

The constructor is decided for the Rust type, across all its impl blocks:

1. The method marked `#[miniextendr(<system>(constructor))]`
   (`r6(constructor)`, `s7(constructor)`, ...).
2. Otherwise a receiver-less method named `new` that returns `Self`. A vctrs
   `new` is the constructor whatever it returns, since it returns the payload.
3. Otherwise the type's one receiver-less method returning `Self`, whatever its
   name. `Result<Self, E>` and `Option<Self>` don't make a method a candidate:
   a fallible constructor is marked with rule 1. An S7 `convert_from` method
   isn't a candidate either, and a vctrs class never infers its constructor.

```rust
#[miniextendr(r6)]
impl Reading {
    pub fn from_value(value: f64) -> Self { Reading { value } }   // the constructor: Reading$new(value), and Reading$from_value(value)
    pub fn parse(text: &str) -> Result<Self, String> { /* ... */ } // a static method: Reading$parse(text)
    pub fn value(&self) -> f64 { self.value }
}
```

A constructor whose R name isn't `new`, marked (rule 1) or inferred (rule 3),
keeps that name too: it is also a static method under it, which builds the
same object (`Reading$from_value()` above, `typename_from_value()` for S3,
`TypeName_from_value()` for S4 and S7). A vctrs class has only its `new`.

Every other receiver-less method returning `Self`, such as a `from_file`
factory next to a `new`, is a static method. A type that has several such
methods and no declared constructor is refused, at compile time when they are
in one impl block and when the wrapper writer runs when they are in several.
The error lists them: mark one with `<system>(constructor)` or name it `new`,
and the others stay static methods.

#### A class without a constructor

A type with no constructor by these rules (no marked method, no `new`, and no
receiver-less method returning bare `Self`) is defined without one. Its
objects come from its factories: static methods returning `Result<Self, E>` or
`Option<Self>`, which wrap the pointer they return in the class directly
(`Class$new(.ptr = ...)` for R6, `Class(.ptr = ...)` for S7,
`methods::new("Class", ptr = ...)` for S4), or functions of other classes that
return one. A type with factories returning bare `Self` has candidates, so it
has a constructor: the one candidate, or the one marked among several.

Nothing is generated in place of the constructor. R6's `TypeName$new()` and
S7's `TypeName()`, which exist anyway to wrap the factories' pointers, raise
an error naming the type when called without a `.ptr`, rather than build an
object holding a `NULL` pointer. Env and S3 leave `TypeName$new` `NULL`, and
S4 and vctrs define no `TypeName()` or `new_typename()`: a package that builds
its objects in R may define its own under that name (#2069). An abstract S7
class (`s7(abstract)`) keeps S7's own refusal to construct it. A vctrs class
without a constructor gets its vectors from `vctrs::new_vctr()`
(`new_rcrd()`, `new_list_of()`) with `class = "TypeName"`.
When nothing creates the type, neither a constructor nor a `#[miniextendr]`
function or method returning it, nor code of the crate building an
`ExternalPtr<TypeName>` itself (e.g. a wrapper's hand-written `IntoR`), the
build warns (lint MXL121, see [Macro Errors](MACRO_ERRORS.md)). The lint sees
`ExternalPtr::new` of a `TypeName { .. }` literal, directly or through a `let`
in the same function, or of a local written `let x: TypeName = ..`, and any
`ExternalPtr<TypeName>` written as a type; it misses a pointer built from a
value from a call whose type isn't written.
An S7 property whose class is a class without a constructor needs a
`default`, since S7 builds a property's default object by calling the class's
constructor.

The class keeps the doc comments of its impl blocks in every system. A class
named as its Rust type has its class page, `TypeName.Rd`, titled by default.
An S3 or vctrs class the package names itself (`class = "..."`) has no class
environment either (#1918), so its impl docs go in a roxygen block of their
own:

- Without `@rdname` in the impl docs, the block is on a page named after the
  class (`engine.Rd`, aliased `engine`), titled `engine S3 Class` (`engine
  vctrs S3 Class`) unless the impl docs give a `@title`.
- With `@rdname topic`, the block joins `topic` and adds no alias and no
  title: the topic's own blocks name and title the page. A package that
  documents the class on a topic of its own, even one named after the class,
  puts `@rdname <that topic>` on the impl docs. Without it, the generated
  page and the package's own would both title the page.
- Without impl docs there is no block.

### Method Access

| System | Instance Method | Static Method |
|--------|-----------------|---------------|
| Env | `obj$method()` | `TypeName$method()` |
| R6 | `obj$method()` | `TypeName$method()` |
| S3 | `method(obj)` | `typename_method()` |
| S4 | `method(obj)` | `TypeName_method()` |
| S7 | `method(obj)` | `TypeName_method()` |

### Mutable Receivers (`&mut self`)

All class systems support mutable receivers. The Rust method:

```rust
pub fn increment(&mut self) {
    self.value += 1;
}
```

Modifies the underlying data in place. The R object reference remains valid.

### Consuming Receivers (`self`)

Methods taking `self` by value are supported on every class system; what the
wrapper does depends on the return type:

| Signature | Wrapper behaviour | R result |
|---|---|---|
| `fn step(self, ..) -> Self` | moves the value out, calls, writes the result back into the **same** handle | the receiver object (identity preserved, like `&mut Self`) |
| `fn step(self, ..) -> Result<Self, E>` / `-> Option<Self>` | calls on a **clone** (`T: Clone` required), overwrites the handle only on `Ok` / `Some` | the receiver object; `Err` / `None` raise and leave the object untouched |
| `fn finish(self, ..) -> T` (anything else) | moves the value out and converts `T` as usual; the handle is left **consumed** | the converted value; later use of the object errors with "was consumed" |

```rust
#[derive(Clone, ExternalPtr)]
pub struct Pipeline { steps: Vec<String> }

#[miniextendr(s3)]
impl Pipeline {
    pub fn new() -> Self { Pipeline { steps: vec![] } }
    pub fn with_step(mut self, name: String) -> Self { self.steps.push(name); self }
    pub fn try_step(mut self, name: String) -> Result<Self, String> {
        if name.is_empty() { return Err("empty step".into()); }
        self.steps.push(name);
        Ok(self)
    }
    pub fn run(self) -> i32 { self.steps.len() as i32 }
}
```

```r
p <- new_pipeline() |> with_step("load") |> try_step("fit")
try_step(p, "")     # Error: empty step   (p still has two steps)
run(p)              # 2; p is now consumed
with_step(p, "x")   # Error: this `Pipeline` object was consumed ...
```

`self: Self` is the same as `self`. `self: Box<Self>` / `Rc<Self>` and the
other smart-pointer receivers are rejected (the handle stores the value
itself). The former `#[miniextendr(constructor)]` escape hatch on a `self`
method is an error: a constructor has no receiver. The R6 finalizer is never
inferred from a `self` receiver any more; mark it with `r6(finalize)`.

Fallible in-place steps, `&mut self -> Result<&mut Self, E>` and
`-> Option<&mut Self>`, are recognised alongside `&mut self -> &mut Self`:
success hands back the same handle, failure raises through the normal error
paths.

---

## Conditional Methods

Methods and associated items in a `#[miniextendr]` impl block can carry
`#[cfg(...)]` / `#[cfg_attr(...)]`. A gated-off method has no C wrapper,
registration, R binding or docs, and constructor inference, S7 property pairing
and name checks see only the enabled methods. See
[Conditional Compilation](MINIEXTENDR_ATTRIBUTE.md#conditional-compilation) for
every place a gate can sit.

## Multiple Impl Blocks

A type's methods can be split over several `#[miniextendr]` impl blocks, in one
file or across modules, in every class system. The blocks need no names: the
wrapper writer merges them per Rust type, defines the class once and attaches
the methods of every block to it (#1776).

```rust
/// A counter.
#[miniextendr(r6)]
impl Counter {
    pub fn new(start: i32) -> Self { Counter { value: start } }
    pub fn value(&self) -> i32 { self.value }
}

#[miniextendr(r6)]
impl Counter {
    pub fn add(&mut self, by: i32) { self.value += by }

    #[miniextendr(r6(active))]
    pub fn doubled(&self) -> i32 { self.value * 2 }

    #[miniextendr(r6(private))]
    pub fn checked(&self) -> bool { self.value >= 0 }

    pub fn origin() -> i32 { 0 }
}
```

```r
c <- Counter$new(5L)
c$add(2L)
c$value()       # 7
c$doubled       # 14
Counter$origin() # 0
```

One block defines the class: it writes the class object (`new.env()`,
`R6::R6Class()`, `methods::setClass()`, `S7::new_class()`), the constructor,
and what the class options add (the `$` / `[[` field methods of
`r_data_accessors`, R6's `$lock()`). Every block adds its methods, static
methods and `as.<class>()` coercions, in source order. R6 private methods,
`finalize` and `deep_clone` are `$set("private", ...)` calls, so any block can
have them; an R6 active binding's `@field` and a lifecycle import of any block
go to the class's roxygen block.

Every block's doc comment documents the class. The class's roxygen block has
the defining block's impl doc comment (for S3 and vctrs, with the
constructor's), then each other block's, in source order:

- Prose and the tags roxygen2 collects (`@description`, `@details`,
  `@section`, `@seealso`, `@family`, `@importFrom`, ...) add up; each block's
  prose becomes another `@description` paragraph. A tag repeated exactly, such
  as one `@importFrom` or `@family` on two blocks, is written once. (`@return`,
  `@examples` and `@export` belong on the methods, as for one block, and so
  does `@param` outside R6.)
- A generated line gives way to the same tag on another block: the
  `@title <Class> <system> Class` default, `@name`, `@rdname`, `@export`, a
  constructor `@param` filler and an R6 active binding's generated `@field`. So
  when the defining block has no doc comment, another block can title the class.
- A class has one title: two blocks with different `@title`s stop the install.
- The defining block decides the class's page. A `@name` or `@rdname` on
  another block must repeat the defining block's (its own, or the default
  `<Class>`); a different one stops the install, because the defining block's
  generated documentation was written for its page. Put page tags on the
  defining block.

Each method's doc comment documents that method, whichever block has it. An R6
impl-level `@param`, which roxygen2 hands down to the class's methods, spares
the methods of its own block their `(no documentation available)` placeholder;
a method in another block documents the argument itself (#2066).

The blocks of a type follow four rules, checked when the wrapper writer
assembles `R/<pkg>-wrappers.R` during `R CMD INSTALL`; a violation stops the
install with an error naming the blocks:

- **One block defines the class.** Only one block may have the constructor and
  S7 properties (`s7(getter)`, `s7(setter)`, `s7(validate)`). That block
  defines the class; without one, the first block in source order does. The
  constructor is decided for the type
  ([Constructor Patterns](#constructor-patterns)): a block's one method
  returning `Self` is the constructor when no block declares one, and a static
  method when another block has a `new` or a marked constructor. Methods
  returning `Self` in two blocks, with none declared, are refused.
- **The doc comments agree on the title and page**: one `@title` value, and a
  `@name` / `@rdname` only as the defining block has it (see above).
- **The blocks agree on the class settings**: the class system with its options
  (`r6(lock_class = TRUE)`, `s3(r_data_accessors)`, `vctrs(kind = ...)`, ...),
  `class = "..."`, `internal` / `noexport`, and the `@noRd` /
  `@keywords internal` tags. These decide how every block's methods are
  generated, documented and exported, so each block spells them; the class
  docs have them once. `strict` and `preconditions` are per-block method
  defaults and may differ.
- **No two blocks bind one member**: an env or R6 member, an R6 static method,
  an S4 or S7 method of the class. Rust already rejects two methods of one name
  (E0592); this catches two R names made equal with `r_name = "..."`.

## Names the wrapper writer rejects

Each class, generic and function the macros generate is bound by name in the
package namespace, and a later binding of a name replaces an earlier one when
the package loads. A collision between two macro expansions is only visible
when the wrapper writer assembles `R/<pkg>-wrappers.R` during `R CMD INSTALL`,
so the install stops there, with an error naming both producers (the Rust impl,
function or sidecar field, with its source position):

- **Two Rust types with one R class name**, through `class = "..."` or two
  types of one name in different modules. Their class objects and methods
  would overwrite each other.
- **A class object and a function of the same name**: the Env or S3 class
  environment (`Type <- new.env(...)`), `R6::R6Class(...)` or
  `S7::new_class(...)`, against a free `#[miniextendr]` function, an S7
  shortcut or a sidecar accessor. S4's `methods::setClass()` binds no name, so
  the S4 constructor function shares the class name as intended.
- **Impl blocks of one type that break the rules** of
  [Multiple Impl Blocks](#multiple-impl-blocks): two blocks that each define
  the class, blocks with different class settings, or a member bound by two
  blocks.
- **A generic and a plain binding of the same name.** The S3, S4 and S7
  generics that methods attach to are created inside `if (...)` guards. Several
  classes may declare the same generic so their methods share dispatch, but a
  function or class object with the generic's name would replace it or be
  replaced by it.
- **Two functions of the same name**, such as an S7 shortcut and a sidecar
  accessor or a free function (#991), including two blocks of one type whose
  S3 methods or static methods get one name.

Rename the function or method with `r_name = "..."`, or the class with
`class = "..."`.

---

## Cross-Type Class Name References

Several attributes let one `#[miniextendr]` type reference another by name -
`s7(parent = ...)`, `r6(inherit = ...)`, `s7(convert_from = ...)`, and
`s7(convert_to = ...)`. miniextendr resolves these at wrapper-gen time using
the Rust type name, not the R class name, so you can stay consistent with your
Rust source and still get the correct R-visible name in generated wrappers.

```rust
#[miniextendr(s7, class = "Shape")]
impl S7Shape {
    pub fn new() -> Self { S7Shape }
}

#[miniextendr(s7(parent = "S7Shape"))]   // Rust name on the left
impl S7Circle {
    pub fn new() -> Self { S7Circle }
}
```

Generated R uses the registered class name:

```r
S7Circle <- S7::new_class("S7Circle", parent = Shape, ...)
```

The same placeholder (`.__MX_CLASS_REF_<Type>__`) powers R6 `inherit =` and S7
`convert_from` / `convert_to`. Unregistered names produce a compile-time warning
and fall through to the bare identifier - which will fail at R load.

### Explicit Return Class Systems

A return can choose its class system directly with `WrapAsR6<Board>` or
`#[miniextendr(wrap = "r6")]` on a function or method returning `Board`.
The corresponding markers for S7, S4, S3, Env, and Vctrs use the same shared
wrapping decision. For example, an Env factory returning an R6 board emits
`Board$new(.ptr = .val)`, and a trait factory can use `WrapAsR6<Self>`.

Explicit wrapping uses the final Rust type-name segment as the R class name
(`Self` uses the implementing class name), without a registry lookup. The
selected system and name must match the target's R definition. See
[explicit cross-class return wrapping](MINIEXTENDR_ATTRIBUTE.md#explicit-cross-class-return-wrapping)
for both spellings, container/error behavior, visibility composition, and
vctrs class preservation.

### S7 Property Class Constraints

When an S7 `getter` method returns another `#[miniextendr]` type, the generated
property declares that class as its constraint:

```rust
#[miniextendr(s7)]
impl S7PropOuter {
    pub fn new(inner: S7PropInner) -> (Self, /* sidecar */) { ... }

    #[miniextendr(s7(getter))]
    pub fn inner(&self) -> S7PropInner { ... }
}
```

```r
S7PropOuter <- S7::new_class("S7PropOuter",
    properties = list(
        inner = S7::new_property(
            class = S7PropInner,                          # resolved class name
            getter = function(self) .Call(..., self@.ptr)
        )
    )
)
```

Property-class resolution uses a sibling placeholder
(`.__MX_CLASS_REF_OR_ANY_<Type>__`) with a **silent** fallback to
`S7::class_any` in two cases, so property getters returning "foreign" types
don't break package load:

| Getter return type | Emitted class |
|--------------------|---------------|
| Registered S7 class | `<R class name>` |
| Registered non-S7 class (R6 / S3 / S4 / Env / vctrs) | `S7::class_any` |
| Unregistered / primitive / `SEXP` / `PathBuf` | `S7::class_any` |

The other four reference sites (`parent`, `inherit`, `convert_from`,
`convert_to`) keep the loud fallback - an unresolved name there almost always
means a typo worth surfacing.

---

## Help Pages and Doc Tags

### One page per class, `@rdname` to split

Every generated wrapper for a class (constructor, instance method, static
method, `as.*` coercion, S7 `convert`, trait-impl method) is documented with
`@rdname <Class>`, so a class renders as a single help page listing all of its
methods. That is the right default for a handful of accessors; for a
15-verb builder it means one long page.

A method-level `@rdname` splits a method onto its own page. The generator
honours it wherever it would otherwise inject the class default, and never
emits both:

```rust
#[miniextendr(s3)]
impl Pipeline {
    /// Run the pipeline.
    ///
    /// Long-form documentation that deserves its own page.
    /// @rdname run_pipeline
    /// @param inputs Numeric inputs.
    pub fn run(&self, inputs: Vec<f64>) -> Result<Output, String> { /* ... */ }
}
```

renders `man/run_pipeline.Rd` for `run.Pipeline` while the rest of the class
stays on `man/Pipeline.Rd`. Applies to all class systems and to
`#[miniextendr] impl Trait for Type` blocks. The S4/S7 *generics* the
generator emits alongside a method stay on the class page, as do trait
consts. The S3 generic guard is documented under the method's own
`generic.Class` name (so classes sharing a generic don't collide on
`\alias{generic}`), which is the same alias the method produces; it therefore
moves to the split page with the method, otherwise `R CMD check` reports the
alias as duplicated across the two pages. A method-level `@name` is honoured
the same way.

Method prose is emitted as `@description` (the class page supplies the
`@title`), so a split page would have no title and roxygen2 would skip it.
The generator therefore adds `@title <R name>` (`run.Pipeline` above) to a
split method unless the doc comment carries its own `@title`.

Exception: R6 *instance* methods. They are emitted as `Class$set("public",
...)` blocks, which roxygen2 folds into the class block (its R6 support
introspects the generator and renders every method in the class page's
Methods section). A method-level `@rdname` there would fail with "Block must
contain only one @rdname", so the generator drops it; R6 static methods
(`Class$fn`) split like any other.

### Impl-block doc tags

Roxygen tags on the **impl block's own** `///` comment describe the class, so
the following are stripped from that comment (with a build-time deprecation
warning) because they only make sense on a method:

- `@param`: a class has no arguments; document constructor arguments on the
  constructor (`new`) method, or at the class level for R6 where roxygen2
  8.0.0 inherits class-level `@param` into every method (so it is *kept* for
  R6 impl blocks).
- `@return` / `@returns`: an impl block returns nothing.
- `@examples`: examples belong on the method they demonstrate.
- `@export`: export visibility is controlled by `#[miniextendr(internal)]` /
  `#[miniextendr(noexport)]`, not by roxygen.

Tags on **method** doc comments are not stripped: `@param`, `@return`,
`@examples`, `@rdname`, `@name`, `@seealso`, and so on flow through to the
generated wrapper verbatim (undocumented parameters get an auto-generated
`@param name (undocumented)` line, except when the method's own `@rdname`,
`@describeIn` or `@inheritParams` sends it to a page that documents them; see
[Parameters on shared pages](S3_METHODS.md#parameters-on-shared-pages); env
methods, which have no usage section, get no filler, and their choice
parameters get a `\describe` item instead). A
method that joins another topic also drops the generator's fixed `x` / `...`
lines and sorts after that topic's own block. `@describeIn` works on the
methods whose wrapper is a plain R function (S3 instance methods, static
methods, S4 constructors) and is a compile error elsewhere; see the same
section. Trait-impl methods (`impl Trait for Type`) forward only their
`@param` lines and these page tags: `@describeIn`, `@rdname`, `@name`,
`@order`, `@inheritParams`, `@inherit` and `@inheritDotParams`. Env trait
methods, the `Type$Trait$method` statics of S3 and vctrs classes and the
`attr(Type, "Trait")$method` statics of S7 classes are documented like env
methods: every tag but `@title` and a bare `@export` (which would export the
member itself) flows through, and their `@param` lines and choice text go into
a `\describe` list. S4 statics
(`Type_Trait_method()`) get `@param` lines, generated ones included. R6
constructor and instance methods are documented on the class page whatever
their tags, so they keep the generated line. The one exception is class
systems whose
methods are registered by assignment (S4 `setMethod()`, S7 `S7::method()`),
where per-method `@param` would document arguments roxygen2 cannot see in
`\usage`; those generators drop method `@param` tags to keep `R CMD check`
quiet.

A tag that flows through keeps its blank lines and indentation, as on a
function; see
[Doc comments to roxygen](MINIEXTENDR_ATTRIBUTE.md#doc-comments-to-roxygen).

S4's structural tags (`@exportClass`, `@exportMethod`, `@exportPattern`) pass
through unchanged. Tag-name matching is exact on the first
whitespace-delimited token.

---

## Generated Wrapper Annotations

Each generated R wrapper (constructor, instance method, or static method)
carries a `# Type::method (line:col)` source-location comment just above
the `.Call()` line, pointing back at the Rust declaration. This is emitted
for all six class systems (env, R6, S3, S4, S7, vctrs) and makes
`rpkg/R/*-wrappers.R` diffs easy to trace when macro changes shift
generated code around.

---

## Trait Implementations

For cross-package interoperability:

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}

#[miniextendr]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.count }
    fn increment(&mut self) { self.count += 1; }
}
```

This enables type-erased dispatch across package boundaries.

---

## Direct Field Access via Sidecar

Sidecar fields (`#[r_data]` + `RSidecar`) are the way to expose struct fields
to R. Every public field gets generated accessors, in a form that depends on
the class system.

### How It Works

1. On the `#[derive(ExternalPtr)]` struct, add an `#[r_data] _r: RSidecar`
   selector and mark each exposed field `#[r_data]`. Fields without it stay
   Rust-only.
2. Each `pub` `#[r_data]` field gets `Type_get_<field>(x)` /
   `Type_set_<field>(x, value)`. They are exported, documented on a "Type
   Sidecar Accessors" page, unless the class uses the S3, S4 or env field
   syntax below: then `$` / `[[` are the class's R API and the standalone
   accessors are internal, for the package's own R code.
3. `r_data_accessors` on the impl also wires the fields into the class:
   `#[miniextendr(r6(r_data_accessors))]` makes them active bindings,
   `#[miniextendr(s7(r_data_accessors))]` S7 properties, and
   `s3(r_data_accessors)`, `s4(r_data_accessors)` or `env(r_data_accessors)`
   `$` / `[[` methods that read them, `$<-` / `[[<-` methods that write
   them, and `.DollarNames()` for completion; an S3 class also gets
   `names()` and `as.list()`. On S3, S4 and env, `r_data_accessors = "get"`
   generates the readers only.

### Rust Code

```rust
use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar};

#[derive(ExternalPtr)]
#[externalptr(s3)]
pub struct MyConfig {
    // Rust-only internal state
    cache: Vec<u8>,

    #[r_data]
    _r: RSidecar,

    // Fields exposed to R
    #[r_data]
    pub name: String,
    #[r_data]
    pub score: f64,
    #[r_data(name = "table")]
    pub r_table: Sidecar<SEXP>, // any R value, e.g. a data frame; `table` in R
    #[r_data(get = "Self::cache_size")]
    pub cache_size: Computed, // read-only, computed from the Rust value
}

impl MyConfig {
    fn cache_size(&self) -> i32 {
        i32::try_from(self.cache.len()).unwrap_or(i32::MAX)
    }
}

#[miniextendr(s3(r_data_accessors))]
impl MyConfig {
    pub fn new(name: String, score: f64) -> Self {
        MyConfig {
            cache: vec![],
            _r: RSidecar,
            name,
            score,
            r_table: Sidecar::new(SEXP::nil()),
            cache_size: Computed,
        }
    }

    pub fn has_table(self: ExternalPtrRef<'_, Self>) -> bool {
        !Self::r_table(&self).is_nil()
    }
}
```

`name` and `score` live in the Rust struct: each read converts the Rust value
to R, and each write converts into Rust or raises an error. `r_table` is a
`Sidecar<T>`: its value lives in the external pointer's protection list, which
roots it, and Rust reads and writes it through the `MyConfig::r_table(&ptr)` /
`MyConfig::set_r_table(&mut ptr, v)` accessors the derive generates, so a
method that needs it takes a view as its receiver (`self:
ExternalPtrRef<'_, Self>`, as `has_table` does). The field types, the `Sidecar<T>`
lifecycle and what survives `saveRDS()` are in `EXTERNALPTR.md`, "RSidecar (R
Data Fields)".

#### R names

`#[r_data(name = "table")]` gives a field its R name. Everything R sees uses
it: the field list and the `$` / `[[` labels, the R6 active binding or S7
property, the standalone `MyConfig_get_table()` / `MyConfig_set_table()` and
the setter's `'table' must be ...` message. The Rust side keeps the Rust
identifier: `MyConfig::r_table(&ptr)`, and the C symbols. The name must be a
syntactic R name (`n.rows` is fine, `n rows` and `for` are not), unique among
the type's R names, and not `.ptr`, which is the element that carries the
pointer in a list or environment receiver.

#### Computed fields

A `Computed` field is a read-only R field computed from the Rust value by the
function `get` names, a plain `fn(&Self) -> impl IntoR` (`Self` is the
struct; a free function of the module works too). It needs no
`#[miniextendr]`, and the field holds nothing: it is a zero-sized marker that
gives the computed field its place in the field order, so every struct
literal writes `cache_size: Computed` once, as it writes `_r: RSidecar`.

Computed fields take part in everything the other fields do, at their
declared place: the standalone `MyConfig_get_cache_size(x)`, and `x$cache_size`
/ `x[["cache_size"]]` on S3, S4 and env classes with `r_data_accessors`. They
have no setter: no `MyConfig_set_cache_size()`, and under the get/set form
`x$cache_size <- v` raises `miniextendr_read_only_field` (`` `cache_size` of
`MyConfig` is computed from the Rust value and can't be assigned ``, naming
the object's R class). A
computed field exists only for R, so it must be `pub`; and it can't be an R6
active binding or an S7 property, which need a setter (#1888), so it is
refused under `#[externalptr(r6)]` / `#[externalptr(s7)]`; the example above
is an S3 class for that reason. The getter's errors are its own: a
`rust_error!(class = "pkg_error", "...")` in it reaches R with that class and
the call as written (`x$cache_size`). It takes `&Self`, so it can't read
`Sidecar` values, which are R fields already; it returns a value, not a
`Result`.

### R Behavior by Class System

| System | Get | Set | Computed fields | A restored pointer (`readRDS()`) |
|--------|-----|-----|-----|-----|
| **R6** with `r6(r_data_accessors)` | `obj$name` (active binding) | `obj$name <- "new"` | refused at compile time (#1888) | the bindings read and write the save's `Sidecar` values |
| **S7** with `s7(r_data_accessors)` | `obj@name` (S7 property) | `obj@name <- "new"` | refused at compile time (#1888) | the properties read and write the save's `Sidecar` values |
| **S3** with `s3(r_data_accessors)` | `obj$name`, `obj[["name"]]` | `obj$name <- "new"`, `obj[["name"]] <- "new"` | `obj$cache_size`; `obj$cache_size <- v` raises `miniextendr_read_only_field` | the field methods refuse it |
| **S4** with `s4(r_data_accessors)` | `obj$name` | `obj$name <- "new"` | `obj$cache_size`; the assignment raises `miniextendr_read_only_field` | the field methods refuse it |
| **Env** with `env(r_data_accessors)` | `obj$name`, `obj[["name"]]` | `obj$name <- "new"`, `obj[["name"]] <- "new"` | `obj$cache_size`; the assignment raises `miniextendr_read_only_field` | the field methods refuse it |
| **S3, S4, env** with `r_data_accessors = "get"` | as above | the class's own `$<-`, which can call the internal `MyConfig_set_name(x, "new")` | `obj$cache_size`; no generated assignment | the field methods refuse it |
| **vctrs**, and any class without the option | `MyConfig_get_name(x)` | `MyConfig_set_name(x, "new")` | `MyConfig_get_cache_size(x)` | the standalone accessors read and write the save's `Sidecar` values |

The standalone functions take the external pointer, or any object that
carries it: an environment or list with a `.ptr` element, an R6 object, an
S4 object with a `ptr` slot, or an object with a `.ptr` attribute (an S7
object's `.ptr` property). They exist for every class system. On an S3, S4
or env class with the field syntax they are internal: no `export()` and no
help page, since the generated `$<-` of an immutable class would otherwise
sit next to an exported setter that writes through every copy, and a getter
that reads the saved object the class refuses. The package's own R code
still calls them, e.g. to read a saved object's `Sidecar` values (below).

The field methods are documented on the class's page, or on the topic an
`@rdname` in the impl block's doc comment names. On the author's topic they
add no `@param` lines (the topic's own block documents `x`, `name`, `i`,
`...`, `value` and `pattern`) and sort after its block (`@order NaN`).

That topic also receives the constructor, methods, statics and generated
protocol helpers of the class, including members declared in other impl
blocks. The block that defines the class chooses the default topic. A
method's explicit `@rdname` or supported `@describeIn` overrides it;
generated argument placeholders do not overwrite the author topic's
parameter descriptions.

#### Field syntax on S3, S4 and env classes

The generated methods call the same accessors as the standalone functions, so
a value is converted, checked and stored the same way, and a failed
conversion raises the setter's argument error with the call as written
(`x$score <- value`). They differ from list fields in two ways:

- **A write changes the object in place.** The field lives in the Rust value
  or the pointer's protection list, and every copy of the R object shares the
  pointer, so `y <- x; y$score <- 2` also changes `x$score`. That is the
  semantics of an R6 or env object, not of a list. A class that needs a
  copy-on-modify `$<-` takes `r_data_accessors = "get"` and writes its own
  `$<-` method that returns a new object.
- **No read matches a prefix of a name.** `x$ta` never reads `x$tags`, on
  any class system and on any receiver: a pointer is not a list, and a
  prefix match hides a typo (#1922).
- **An S3 bare pointer refuses a name that is not a field.** `x$tags` and
  `x[["tags"]]` take an exact field name; `x$nope`, `x$ta` and
  `x[["nope"]]` raise `miniextendr_no_field`, naming the R class and its
  fields, as a write does. `x[[2]]` takes a whole-number position in the
  field order; `x[[9]]` past the last field raises
  `subscriptOutOfBoundsError` with the call `x[[9]]`, and so do `0` and a
  negative position, which on a list would raise R's own `get1index` error
  or drop an element. `x[[TRUE]]`, `x[[NA]]`, `x[[1.5]]` and any other
  index (a vector of another length, a list) raise `notSubsettableError`
  from the class's own `[[`, on the call as written (`x[[1:2]]`). All of
  these carry the type's `field_error(class = ...)` in front. The writes
  take an exact name only, and a position goes to R's own `[[<-`.
- **`lookup = "list"` keeps a list's other lenience.** With
  `s3(r_data_accessors, lookup = "list")` (or `= "get"`) the bare pointer
  reads `NULL` for an unknown name, `""` and `NA`, and `[[` takes a
  position as a list's does (`2.9` is the second field, a logical is a
  position, `NA` is `NULL`; out of range is still
  `subscriptOutOfBoundsError`). Names are still matched exactly.
  `lookup = "strict"` is the default spelled out.
- **A name that is not a field** goes to the object's own elements when it
  is a list or an environment carrying the handle in `.ptr`, by the exact
  name: `$` reads a list's element with `.subset2()` (R's own `$` on a list
  would match a prefix) and an environment's binding with R's `$`, and `[[`
  goes to R's own `[[` with `exact = TRUE` whatever the call passed. On an
  S4 object it raises `miniextendr_no_field`. On an env class, `$` looks the
  name up among the fields first, then among the methods, and gives `NULL`
  for neither.

The rule for each receiver:

| Receiver | `$` | `[[` by name | `[[` by position | `$<-` / `[[<-`, unknown name |
|---|---|---|---|---|
| S3 bare pointer | exact, else `miniextendr_no_field` | exact, else `miniextendr_no_field` | a whole number in range; out of range, `subscriptOutOfBoundsError`; a logical, `NA` or fraction, `notSubsettableError` | `miniextendr_no_field` |
| S3 bare pointer, `lookup = "list"` | exact, else `NULL` | exact, else `NULL` | field order, as a list's (truncated, a logical too, `NA` is `NULL`); a position that picks no field, `subscriptOutOfBoundsError` | `miniextendr_no_field` |
| S3 list or environment carrying `.ptr` | an exact field, else its own element by the exact name | an exact field, else its own `[[` with `exact = TRUE` | its own `[[` | its own |
| env class | a field, then a method, else `NULL` | the same | none | `miniextendr_no_field` |
| S4 | an exact field, else `miniextendr_no_field` | none | none | `miniextendr_no_field` |

**`names()`, `as.list()` and `.DollarNames()`.** On an S3 bare pointer,
`names(x)` is the field names in declared order (computed fields included)
and `as.list(x)` a named list of every field read through the getters, so
`lapply(x, f)` and `vapply(x, f, ...)` iterate the fields; a `NULL` field is
kept (`length(x)` stays a pointer's `1`). On a list or environment receiver
both are R's own, so `names(x)` agrees with `length(x)` and `x[[i]]` there
(`.DollarNames()` still lists both); an environment gets
`as.list.environment()` by name, since its class attribute would otherwise
send `as.list()` to the default method, which can't coerce an environment.
`.DollarNames(x, pattern)`, which R's
completion engine calls for `x$<Tab>`, completes the fields on every system,
plus a list's or environment's own names on an S3 receiver and the methods
on an env object. It reads only names, so it never raises. It is registered
as `S3method(utils::.DollarNames, Class)` (roxygen2's `@exportS3Method
utils::.DollarNames`): the unqualified `S3method(.DollarNames, Class)` fails
to load with only base attached, which is how `R CMD check` loads a
namespace. `utils` dispatches it as an S3 generic, so an S4 class takes the
same S3 method.

| | `$` | `[[` | `names()` | `as.list()` | `.DollarNames()` | `$<-` | `[[<-` |
|-|-----|------|-----------|-------------|------------------|-------|--------|
| S3 | `$.Class` | `[[.Class` | `names.Class` | `as.list.Class` | `.DollarNames.Class` | `$<-.Class` | `[[<-.Class` |
| S4 | `setMethod("$")` | (none) | (none) | (none) | `.DollarNames.Class` | `setMethod("$<-")` | (none) |
| env | `$.Class` (fields, then methods) | `[[.Class` (same) | (none) | (none) | `.DollarNames.Class` (fields, then methods) | `$<-.Class` | `[[<-.Class` |

The class can't define these methods itself next to the option: the impl
block is refused at compile time when one of its methods is a method the
option generates (`$`, `[[`, `names`, `as.list`, `.DollarNames`, `$<-` and
`[[<-` on S3; `$`, `.DollarNames` and `$<-` on S4). A class that writes its
own `$<-` takes `r_data_accessors = "get"`, which generates the readers
only. The impl block needs no constructor: a package that builds its
objects itself (`structure(make_engine(), class = "engine")`) writes
`#[miniextendr(s3(r_data_accessors = "get"), class = "engine")] impl Engine
{}` and gets the readers for the pointer it classes by hand. Such an impl
emits no class environment (`engine <- new.env(...)`), so the package's own
constructor can be named `engine()` without a second binding. The environment
exists only when it can hold something: `Class$new()` for an impl with a
constructor, or a trait's `Type$Trait` namespace for a class named as its
Rust type. The readers are documented on the class's topic (or the impl's
`@rdname`) under `#[miniextendr(internal)]` too, which only drops the exports
(#1918).

#### A restored pointer

`saveRDS()` keeps a pointer's `Sidecar` values but not its Rust value
(`EXTERNALPTR.md`, "Serialization"). The generated field methods refuse a
pointer `readRDS()` brought back, whatever the field: `x$keys`, `x[["keys"]]`,
`x$keys <- v` and `x[["keys"]] <- v` on an S3, S4 or env object raise
`miniextendr_restored_no_value` (a save by this version of the package) or
`miniextendr_restored_other_version` (a save by another), both with the class
`miniextendr_restored`. A reloaded object is refused, not recovered; the
class re-creates it. The standalone `Type_get_f()` / `Type_set_f()`, the R6
active bindings and the S7 properties keep reading and writing a same-version
save's `Sidecar` values, as the way to reach a saved object's data. The
methods pass a `require_live` flag to the same per-field accessor, so a read
costs no extra `.Call()`, and `as.list()` refuses because its first read
does. Only a read of a field reaches Rust: on a restored bare pointer
`names()`, `.DollarNames()` and an unknown name (`NULL`) still answer,
unless the type asks for `refuse = "every_read"` (below).

A type names its own condition classes and message for every one of its
restored refusals, the field methods and the methods that take the handle
alike:

```rust
#[derive(ExternalPtr)]
#[externalptr(s3, restored(
    class = ["pkg_error_saved_object", "pkg_error"],
    message = "this engine was saved; build a new one with engine()",
))]
pub struct Engine { /* ... */ }
```

The classes go in front of the `miniextendr_restored*` classes, which stay,
in the given order (one string, or a list); the message replaces
miniextendr's in both the no-value and the other-version case, and the
condition's `saved_version` / `current_version` fields stay. The message is
a static string per type: one type maps to one R class, so the package can
name it literally.

A class whose contract is that a reloaded object is refused on every use adds
`refuse = "every_read"` to `restored(...)`. Then every S3 reader (`$`, `[[`,
`names()`, `as.list()`) checks a bare pointer, and a list or environment
receiver, before it looks at the name or index, so `names(x)`, an unknown
name, a position past the last field and an index that is neither a name
nor a position all raise the type's restored condition. A list or
environment of the class that carries no pointer (the shape an earlier
release of the package may have saved) is refused on every read too, with
`miniextendr_no_handle` behind the `restored(class = ...)` and
`field_error(class = ...)` classes. A list carrying a live `.ptr` answers
as before. `.DollarNames(x)` returns `character(0)` for a restored pointer
(it serves completion, so it never raises). The check is one more `.Call()`
per read, paid only by the types that opt in.

#### The package's classes on the readers' own errors

A package that raises every error under its own classes names them for the
field readers' own errors too:

```rust
#[derive(ExternalPtr)]
#[externalptr(s3, field_error(class = ["pkg_error_field", "pkg_error"]))]
pub struct Engine { /* ... */ }
```

The classes go in front of each error's own class, which stays: a position
that picks no field (`subscriptOutOfBoundsError`), an index that is neither
a name nor a position (`notSubsettableError`), a name that is not a field
(`miniextendr_no_field`), a pointer of another type under the class, and a
receiver of the class that carries no pointer (`miniextendr_no_handle`). A
list or environment without a pointer, e.g. an older format, also gets the
`restored(class = ...)` classes first, then these, each class once at its
last place, so a parent class both lists end in follows both specific
classes, and the `restored(message = ...)` text when the type sets one; an
atomic vector given the class was never saved, so it gets these alone.

The receiver refusals name the class the user sees: `this "engine" object
holds a pointer of another type`, `... holds no pointer`, `... holds a null
pointer` (the receiver's first class; a receiver without one is named by
the Rust type). `field_error(message = "...")` replaces that text with the
package's own, for every one of them except a list or environment without
a pointer under a `restored(message = "...")`, which reads as a save:

```rust
#[derive(ExternalPtr)]
#[externalptr(s3, field_error(
    class = ["pkg_error_field", "pkg_error"],
    message = "not an engine; build one with engine()",
))]
pub struct Engine { /* ... */ }
```

A name that is no field is refused with `miniextendr_no_field`, whose
message lists the fields: ``` `value` is not a field of `engine`; its
fields are `input`, `table` ```. `field_error(no_field_hint = "...")` adds
the package's own sentence after that list, each `%s` in it replaced by the
name asked for. Any other `%` in it is written as given. It applies to every
name the field methods refuse: `$` and `[[`, and `$<-` / `[[<-` where they
are generated. Without it the message is unchanged.

```rust
#[derive(ExternalPtr)]
#[externalptr(s3, field_error(
    class = ["pkg_error_field", "pkg_error"],
    no_field_hint = "A column of the table is `as.data.frame(x)$%s` or `x$table$%s`.",
))]
pub struct Engine { /* ... */ }
```

```r
conditionMessage(tryCatch(x$value, error = identity))
#> [1] "`value` is not a field of `engine`; its fields are `input`, `table`. A column of the table is `as.data.frame(x)$value` or `x$table$value`."
```

#### Conditions the generated accessors raise

| Error | Class | Raised by |
|---|---|---|
| a restored pointer | `miniextendr_restored_no_value` or `miniextendr_restored_other_version`, both with `miniextendr_restored`, behind the type's `restored(class = ...)` | the field methods, `as.list()` included; a struct or computed field's standalone getter; a `Sidecar` field's standalone accessors for another version's save |
| `[[` with a position that picks no field (past the last, `0`, negative) | `subscriptOutOfBoundsError` (R's own class), behind the type's `field_error(class = ...)` | an S3 bare pointer |
| a pointer of another type under the class | `rust_error`, behind the type's `field_error(class = ...)` | every accessor |
| a receiver of the class with no pointer | `miniextendr_no_handle`, behind the type's `field_error(class = ...)`, and `restored(class = ...)` for a list or environment | every accessor; every S3 reader under `refuse = "every_read"` |
| `[[` with an index that is neither a name nor a whole-number position (under `lookup = "list"`, a logical or a fraction is a position) | `notSubsettableError` (R's own class), behind the type's `field_error(class = ...)` | an S3 bare pointer |
| no such field (the message lists the fields, then the type's `field_error(no_field_hint = ...)`) | `miniextendr_no_field`, behind the type's `field_error(class = ...)` | S4 `$`; `$`, `[[`, `$<-` and `[[<-` on an S3 bare pointer (`lookup = "list"` reads `NULL` instead) |
| an assignment to a computed field | `miniextendr_read_only_field` | `$<-` / `[[<-` in the get/set form |
| a failed conversion | `rust_error` plus the crate's `conversion_error_class` | the setters, with `'name' must be ...` and `e$param == "value"` |
| a computed field's getter | whatever the getter raises (`rust_error!(class = ...)`) | the computed field's reads |

### When to Use Sidecar vs Manual Getters

- **Use sidecar** when you have fields to expose and want zero-boilerplate
  accessors; a `Computed` field covers a value derived from the Rust value.
- **Use manual getters** when a read needs arguments, validation of the
  receiver beyond the type, or side effects. Manual getters work identically
  across all class systems and are straightforward to write.

---

## Export Control

Control R export visibility with `#[miniextendr(internal)]` and `#[miniextendr(noexport)]`.
These work consistently across standalone functions, class impl blocks, and
trait impls. Both suppress NAMESPACE `export()` (the function stays callable
via `:::`); they differ at the **documentation** level.

### `#[miniextendr(internal)]`

Adds `@keywords internal` to roxygen and suppresses `@export`. The function is
still **documented**: it contributes an `\alias{}` and usage entry to its `.Rd`
page (including shared/grouped pages), which is hidden from the package index
via `\keyword{internal}` but reachable with `?name`.

```rust
#[miniextendr(internal)]
pub fn helper_function(x: i32) -> i32 { x * 2 }

#[miniextendr(s3, internal)]
impl InternalType {
    pub fn new() -> Self { ... }
}
```

### `#[miniextendr(noexport)]`

Emits `@noRd`: **no Rd contribution at all** — no alias, no usage entry, not
even on a shared/grouped page it would otherwise land on (e.g. a page its
`@rdname` names, or a class's `@rdname` page for methods). `?name` finds
nothing. No `@keywords internal`.

For S3 class impls, `noexport` also drops the `S3method()` dispatch
registration from NAMESPACE — zero observable trace. (A hand-written `@noRd`
doc comment without the `noexport` attribute keeps dispatch registered:
undocumented but dispatchable.)

```rust
#[miniextendr(noexport)]
pub fn private_helper(x: i32) -> i32 { x * 2 }
```

### Comparison

| Attribute | `@export` | `@keywords internal` | Man page / alias |
|-----------|-----------|---------------------|------------------|
| (default) | Yes | No | Yes |
| `internal` | No | Yes | Yes — alias + usage, hidden from index |
| `noexport` | No | No | No (`@noRd`) — no alias anywhere |

`internal` and `noexport` together on one function is a compile error (on impl
blocks, `internal` wins).

---

## S4 Helpers Module

miniextendr provides Rust helpers for interoperating with existing S4 objects (e.g., Bioconductor). These are for **reading/writing S4 objects passed as arguments**, not for generating S4 classes (use `#[miniextendr(s4)]` for that).

```rust
use miniextendr_api::s4_helpers;

unsafe {
    // Check if an object is S4
    if s4_helpers::s4_is(obj) {
        // Get the class name
        let class = s4_helpers::s4_class_name(obj); // Option<String>

        // Check and access slots
        if s4_helpers::s4_has_slot(obj, "data") {
            let data = s4_helpers::s4_get_slot(obj, "data")?; // Result<SEXP, REvalError>
        }

        // Set a slot value
        s4_helpers::s4_set_slot(obj, "label", new_value)?;
    }
}
```

| Function | Purpose |
|----------|---------|
| `s4_is(obj)` | Check if SEXP is an S4 object |
| `s4_class_name(obj)` | Get the S4 class name as `Option<String>` |
| `s4_has_slot(obj, name)` | Check if a slot exists |
| `s4_get_slot(obj, name)` | Get a slot value as `Result<SEXP, REvalError>` |
| `s4_set_slot(obj, name, value)` | Set a slot value; `Err(REvalError)` carries R's condition (an invalid value, say) |

All functions require the R main thread and operate on raw SEXP values.

---

## Enums: value vs instance

A fieldless enum can be exposed to R in **two mutually-exclusive** ways. Pick one.

**By value** — the enum crosses the boundary as an R character or factor and is
reconstructed on each call; there is no persistent R object holding a Rust
instance:

- `#[derive(MatchArg)]` → R character scalar validated with `match.arg`.
- `#[derive(RFactor)]` → R `factor` with integer codes and level labels.

This is the right choice for a closed set of options passed as arguments. See
[ENUMS_AND_FACTORS.md](ENUMS_AND_FACTORS.md).

**By reference** — the enum is wrapped exactly like a struct: `#[derive(ExternalPtr)]`
plus a class-system `impl` block. R holds an opaque pointer to a boxed instance,
and the impl block's methods become R6/S3/S4/S7/env methods. The variants are
internal state, invisible to R:

```rust
#[derive(Clone, Copy, ExternalPtr)]
pub enum DType { F32, F64 }

#[miniextendr(r6)]
impl DType {
    pub fn new_f32() -> Self { DType::F32 }
    pub fn is_float(&self) -> bool { matches!(self, DType::F32 | DType::F64) }
    pub fn size_bytes(&self) -> i32 { match self { DType::F32 => 4, DType::F64 => 8 } }
}
```

Use this when the enum needs methods or object identity.

### You cannot have both

Deriving both `ExternalPtr` and `MatchArg`/`RFactor` on one type is a hard
compile error:

```
error[E0119]: conflicting implementations of trait `IntoR` for type `DType`
   = note: conflicting implementation in crate `miniextendr_api`:
           - impl<T> IntoR for T where T: IntoExternalPtr;
   = note: this error originates in the derive macro `MatchArg`
```

`ExternalPtr` supplies a blanket `impl<T> IntoR for T where T: IntoExternalPtr`,
while `MatchArg`/`RFactor` generate their own `impl IntoR` (and a conflicting
`TryFromSexp`). A type is therefore *either* a by-value enum *or* a by-reference
instance — never both.

If you want both a friendly string API *and* methods, keep the enum by-value and
put the behaviour on a separate wrapper struct (or expose free functions that
take the by-value enum).

---

## Recommendations

1. **Start with Env** for simple cases
2. **Use R6** when you need encapsulation or active bindings
3. **Use S3** for tidyverse compatibility
4. **Use S4** for Bioconductor integration
5. **Use S7** for new packages wanting modern OOP

When in doubt, start with the default (Env) and migrate if needed.
