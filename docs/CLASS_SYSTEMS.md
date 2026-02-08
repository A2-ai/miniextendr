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

```
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

```r
Counter <- local({
    e <- new.env(parent = emptyenv())

    e$new <- function(initial) {
        ptr <- .Call(C_Counter__new, initial)
        structure(
            list(.ptr = ptr),
            class = "Counter"
        )
    }

    e$value <- function(x) {
        .Call(C_Counter__value, x$.ptr)
    }

    e$inc <- function(x) {
        .Call(C_Counter__inc, x$.ptr)
        invisible(x)
    }

    e
})
```

### Usage

```r
c <- Counter$new(0L)
c$value()      # 0
c$inc()
c$value()      # 1
```

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
                private$.ptr <- .Call(C_Rectangle__new, width, height)
            }
        },
        get_width = function() {
            .Call(C_Rectangle__get_width, private$.ptr)
        },
        set_width = function(width) {
            .Call(C_Rectangle__set_width, private$.ptr, width)
        }
    ),
    private = list(
        .ptr = NULL,
        validate = function() {
            .Call(C_Rectangle__validate, private$.ptr)
        }
    ),
    active = list(
        area = function() {
            .Call(C_Rectangle__area, private$.ptr)
        }
    ),
    lock_objects = TRUE,
    lock_class = FALSE,
    cloneable = FALSE
)

# Static method
Rectangle$square <- function(size) {
    Rectangle$new(.ptr = .Call(C_Rectangle__square, size))
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

### Field Access via Sidecar

For R6 and Env classes, the sidecar pattern (`#[r_data]` + `RSidecar`) provides
zero-overhead field access as R6 active bindings:

```rust
#[r_data]
pub struct MyData {
    pub name: String,
    pub value: f64,
}

r_data_accessors!(MyStruct, MyData);
```

This generates `obj$name` and `obj$value` active bindings automatically.
See the R6 section above for a complete example.

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

    /// Implements print.Person
    #[miniextendr(generic = "print")]
    pub fn show(&self) {
        println!("Person: {}, age {}", self.name, self.age);
    }

    /// Implements format.Person
    #[miniextendr(generic = "format")]
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
    ptr <- .Call(C_Person__new, name, age)
    structure(ptr, class = "Person")
}

#' @export
print.Person <- function(x, ...) {
    .Call(C_Person__show, x)
    invisible(x)
}

#' @export
format.Person <- function(x, ...) {
    .Call(C_Person__fmt, x)
}

#' @export
greet <- function(x, ...) UseMethod("greet")

#' @export
greet.Person <- function(x, ...) {
    .Call(C_Person__greet, x)
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

    #[miniextendr(generic = "show")]
    pub fn display(&self) {
        println!("Gene {} on chr{}", self.symbol, self.chromosome);
    }
}
```

### Generated R Code

```r
setClass("Gene", contains = "externalptr")

#' @export
Gene <- function(symbol, chromosome) {
    ptr <- .Call(C_Gene__new, symbol, chromosome)
    new("Gene", ptr)
}

setGeneric("symbol", function(object) standardGeneric("symbol"))
setMethod("symbol", "Gene", function(object) {
    .Call(C_Gene__symbol, object)
})

setGeneric("chromosome", function(object) standardGeneric("chromosome"))
setMethod("chromosome", "Gene", function(object) {
    .Call(C_Gene__chromosome, object)
})

setMethod("show", "Gene", function(object) {
    .Call(C_Gene__display, object)
})
```

### Usage

```r
g <- Gene("TP53", 17L)
symbol(g)       # "TP53"
chromosome(g)   # 17
show(g)         # Gene TP53 on chr17
```

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

    #[miniextendr(generic = "base::print")]
    pub fn show(&self) {
        println!("Point({}, {})", self.x, self.y);
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
                .ptr = .Call(C_Point__new, x, y))
        }
    }
)

S7::method(x, Point) <- function(x) {
    .Call(C_Point__x, x@.ptr)
}

S7::method(y, Point) <- function(x) {
    .Call(C_Point__y, x@.ptr)
}

S7::method(distance, Point) <- function(x, other) {
    .Call(C_Point__distance, x@.ptr, other@.ptr)
}

S7::method(print, Point) <- function(x, ...) {
    .Call(C_Point__show, x@.ptr)
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
            getter = function(self) .Call(C_Range__length, self@.ptr)
        ),
        midpoint = S7::new_property(
            getter = function(self) .Call(C_Range__get_midpoint, self@.ptr),
            setter = function(self, value) {
                .Call(C_Range__set_midpoint, self@.ptr, value)
                self
            }
        )
    ),
    constructor = function(start, end, .ptr = NULL) { ... }
)

# Regular method as S7 generic
S7::method(start, Range) <- function(x, ...) .Call(C_Range__start, x@.ptr)
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

### Method Access

| System | Instance Method | Static Method |
|--------|-----------------|---------------|
| Env | `obj$method()` | `TypeName$method()` |
| R6 | `obj$method()` | `TypeName$method()` |
| S3 | `method(obj)` | `typename_method()` |
| S4 | `method(obj)` | `TypeName_method()` |
| S7 | `method(obj)` | `TypeName$method()` |

### Mutable Receivers (`&mut self`)

All class systems support mutable receivers. The Rust method:

```rust
pub fn increment(&mut self) {
    self.value += 1;
}
```

Modifies the underlying data in place. The R object reference remains valid.

---

## Multiple Impl Blocks

You can have multiple impl blocks with labels:

```rust
#[miniextendr(s3, label = "core")]
impl MyType {
    pub fn new() -> Self { ... }
    pub fn value(&self) -> i32 { ... }
}

#[miniextendr(s3, label = "math")]
impl MyType {
    pub fn add(&mut self, x: i32) { ... }
    pub fn multiply(&mut self, x: i32) { ... }
}
```

Both blocks generate methods for the same type.

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

## Recommendations

1. **Start with Env** for simple cases
2. **Use R6** when you need encapsulation or active bindings
3. **Use S3** for tidyverse compatibility
4. **Use S4** for Bioconductor integration
5. **Use S7** for new packages wanting modern OOP

When in doubt, start with the default (Env) and migrate if needed.
