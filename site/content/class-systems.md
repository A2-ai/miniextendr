+++
title = "Class Systems"
weight = 4
description = "Env, R6, S3, S4, and S7 -- choosing the right class system"
+++

miniextendr supports five R class systems. Switch between them with a single attribute change.

## Quick Comparison

| Feature | Env | R6 | S3 | S4 | S7 |
|---------|-----|----|----|----|----|
| **Attribute** | `#[miniextendr]` | `#[miniextendr(r6)]` | `#[miniextendr(s3)]` | `#[miniextendr(s4)]` | `#[miniextendr(s7)]` |
| **Method Call** | `obj$method()` | `obj$method()` | `generic(obj)` | `generic(obj)` | `generic(obj)` |
| **Encapsulation** | Weak | Strong | None | Moderate | Strong |
| **Dependencies** | None | R6 package | None | methods package | S7 package |
| **Active Bindings** | No | Yes | No | No | Yes |
| **Inheritance** | No | Limited | S3 dispatch | S4 dispatch | S7 dispatch |
| **Best For** | Simple APIs | Complex state | Tidyverse compat | Bioconductor | Modern OOP |

## Choosing a Class System

- **Simple internal API?** Use **Env** (default) -- no dependencies, `$` dispatch
- **Complex stateful objects?** Use **R6** -- active bindings, encapsulation
- **Tidyverse / generic functions?** Use **S3** -- `print.MyClass`, `format.MyClass`
- **Bioconductor ecosystem?** Use **S4** -- formal classes, multiple dispatch
- **Modern R OOP?** Use **S7** -- best of S3 + S4, computed properties

## Environment Style (Default)

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter { value: i32 }

#[miniextendr]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }

    pub fn value(&self) -> i32 { self.value }

    pub fn increment(&mut self) { self.value += 1; }
}
```

```r
c <- Counter$new(0L)
c$value()      # 0
c$increment()
c$value()      # 1
```

## R6 Style

```rust
#[miniextendr(r6)]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }

    pub fn value(&self) -> i32 { self.value }

    // Active binding (property-like access)
    #[miniextendr(r6(active))]
    pub fn current(&self) -> i32 { self.value }
}
```

```r
c <- Counter$new(0L)
c$value()    # Method call
c$current    # Active binding (no parens)
```

## S3 Style

```rust
#[miniextendr(s3)]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }

    pub fn value(&self) -> i32 { self.value }
}
```

```r
c <- new_Counter(0L)
value(c)     # S3 generic dispatch
print(c)     # Uses print.Counter if defined
```

## S4 Style

```rust
#[miniextendr(s4)]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }

    pub fn value(&self) -> i32 { self.value }
}
```

## S7 Style

```rust
#[miniextendr(s7)]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }

    pub fn value(&self) -> i32 { self.value }
}
```

## Trait Methods

Trait implementations can be exposed to R as method groups:

```rust
trait Describable {
    fn describe(&self) -> String;
}

#[miniextendr]
impl Describable for Counter {
    fn describe(&self) -> String {
        format!("Counter({})", self.value)
    }
}
```

In R, trait methods are accessible via `Type$Trait$method(obj)` (standalone) or `obj$Trait$method()` (`$` dispatch).

Trait dispatch works cross-package: a consumer package can call trait methods registered by a producer package through the trait ABI.

## Multiple Impl Blocks

If a type has more than one `#[miniextendr]` impl block, add `#[miniextendr(label = "...")]` to disambiguate them. Without a label, the second block causes a lint error (MXL009).

```rust
#[miniextendr(label = "counter_core")]
impl Counter { /* ... */ }

#[miniextendr(label = "counter_ext")]
impl Counter { /* additional methods */ }
```
