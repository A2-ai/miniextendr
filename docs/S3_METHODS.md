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
    #[miniextendr(generic = "format")]
    pub fn fmt(&self) -> String {
        format!("{} (age {})", self.name, self.age)
    }

    /// Implements print.Person - prints and returns self invisibly.
    #[miniextendr(generic = "print")]
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

Generated R code:

```r
# Constructor
new_person <- function(name, age) {
  structure(.Call(C_Person__new, name, age), class = "Person")
}

# format.Person - returns the string directly
format.Person <- function(x, ...) {
  .Call(C_Person__fmt, x)
}

# print.Person - calls Rust, then returns invisible(x)
print.Person <- function(x, ...) {
  .Call(C_Person__show, x)
  invisible(x)
}

# greet generic + method
greet <- function(x, ...) UseMethod("greet")
greet.Person <- function(x, ...) {
  .Call(C_Person__greet, x)
}
```

## Key Concepts

### Generic Override with `#[miniextendr(generic = "...")]`

By default, the Rust method name becomes the S3 generic name. Use `generic = "..."` to
override this, mapping to a different R generic:

```rust
// Rust method is `fmt`, but R generic is `format`
#[miniextendr(generic = "format")]
pub fn fmt(&self) -> String { ... }

// Rust method is `show`, but R generic is `print`
#[miniextendr(generic = "print")]
pub fn show(&mut self) { ... }
```

This is essential for base R generics like `print` and `format` where you want your Rust
method to have a more descriptive name.

### Dots (`...`)

All S3 method signatures include `...` automatically. You don't need to declare them
in your Rust signature - the generated R wrapper adds `...` to the parameter list:

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

### Constructor

The constructor is always the method returning `Self`. It generates a function named
`new_<classname_lowercase>()` that wraps the result with `structure(..., class = "ClassName")`:

```rust
pub fn new(name: String, age: i32) -> Self { ... }
// → new_person <- function(name, age) {
//     structure(.Call(C_Person__new, name, age), class = "Person")
//   }
```

### Static Methods

Methods without `self` become standalone functions prefixed with the lowercase class name:

```rust
pub fn species() -> String { "Homo sapiens".into() }
// → person_species <- function() { .Call(C_Person__species) }
```

## Implementing `print`

R convention: `print()` displays output and returns `invisible(x)` so the object
can be used in pipelines without double-printing.

**Recommended pattern - use `&mut self` returning `()`:**

```rust
#[miniextendr(generic = "print")]
pub fn show(&mut self) {
    println!("Person: {}, age {}", self.name, self.age);
}
```

This generates the `ChainableMutation` return strategy:

```r
print.Person <- function(x, ...) {
  .Call(C_Person__show, x)
  invisible(x)
}
```

The `&mut self` + void return triggers `invisible(x)` after the `.Call()`. This
matches the R convention where `print()` returns the object invisibly.

**Why `&mut self`?** The `invisible(x)` pattern is only generated for `&mut self`
methods returning `()`. With `&self` returning `()`, the generated code would be
a bare `.Call(...)` returning `NULL` - functional but doesn't follow R convention.
If your print method doesn't actually mutate, using `&mut self` is a pragmatic
choice to get correct R behavior.

**Alternative - `&self` returning `()`:**

```rust
#[miniextendr(generic = "print")]
pub fn show(&self) {
    println!("Person: {}, age {}", self.name, self.age);
}
```

Generates:

```r
print.Person <- function(x, ...) {
  .Call(C_Person__show, x)
}
```

This works - the `println!` output appears - but the function returns `NULL` instead
of `invisible(x)`. For most interactive use this is fine, but `y <- print(x)` gives
`NULL` rather than `x`.

## Implementing `format`

R convention: `format()` returns a character vector representation. Many R functions
use `format()` internally (e.g., `paste()`, `cat(format(x))`).

**Recommended pattern - `&self` returning `String`:**

```rust
#[miniextendr(generic = "format")]
pub fn fmt(&self) -> String {
    format!("{} (age {})", self.name, self.age)
}
```

Generates:

```r
format.Person <- function(x, ...) {
  .Call(C_Person__fmt, x)
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

    #[miniextendr(generic = "format")]
    pub fn fmt(&self) -> String {
        format!("{:.1}°C", self.celsius)
    }

    #[miniextendr(generic = "print")]
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

## Summary Table

| Method | Receiver | Return | Generated R | R Convention |
|--------|----------|--------|-------------|--------------|
| `format` | `&self` | `String` | `.Call(...)` | Returns formatted string (visible) |
| `print` | `&mut self` | `()` | `.Call(...); invisible(x)` | Returns self invisibly |
| `print` | `&self` | `()` | `.Call(...)` | Returns NULL (works but unconventional) |
| custom | `&self` | any | `.Call(...)` | Returns value directly |
| custom | `&mut self` | `()` | `.Call(...); invisible(x)` | Chainable mutation |

## See Also

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) - Overview of all 6 class systems (env, R6, S3, S4, S7, vctrs)
- [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md) - Using dots and typed_list validation
- [VCTRS.md](VCTRS.md) - vctrs-compatible S3 classes with `format` methods
