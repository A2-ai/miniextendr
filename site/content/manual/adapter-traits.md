+++
title = "Adapter Traits: Exporting External Traits to R"
weight = 13
description = "This guide explains how to expose Rust traits you don't own (from external crates) to R via miniextendr's trait ABI."
+++

This guide explains how to expose Rust traits you don't own (from external
crates) to R via miniextendr's trait ABI.

## The Problem

miniextendr's `#[miniextendr]` attribute must be applied to trait definitions
to generate the ABI metadata. You cannot retroactively annotate traits from
external crates:

```rust
// This WON'T work - can't add attributes to external traits
#[miniextendr]  // ERROR: can't modify external crate
use num_traits::Num;
```

## Solution: Adapter Traits

Create a **local wrapper trait** that mirrors the methods you want to expose,
then implement it via blanket impl for types implementing the external trait.

### Basic Pattern

```rust
use miniextendr_api::prelude::*;
use num_traits::Num;

// 1. Define your local adapter trait
#[miniextendr]
pub trait RNum {
    fn add(&self, other: &Self) -> Self;
    fn mul(&self, other: &Self) -> Self;
    fn is_zero(&self) -> bool;
}

// 2. Blanket impl for all types implementing the external trait
impl<T> RNum for T
where
    T: Num + Clone,
{
    fn add(&self, other: &Self) -> Self {
        self.clone() + other.clone()
    }

    fn mul(&self, other: &Self) -> Self {
        self.clone() * other.clone()
    }

    fn is_zero(&self) -> bool {
        Num::is_zero(self)
    }
}

// 3. Implement the trait for your concrete type
#[derive(ExternalPtr)]
struct MyNumber {
    value: i64,
}

#[miniextendr]
impl RNum for MyNumber {
    // Uses the blanket impl above
}

// 4. Registration is automatic — #[miniextendr] items are registered via
//    linkme distributed slices. No manual module declaration needed.
```

### Why This Works

1. **You own the adapter trait** - `#[miniextendr]` can generate ABI metadata
2. **Blanket impl provides functionality** - Any type implementing `Num` gets `RNum`
3. **Concrete impl triggers codegen** - `impl RNum for MyNumber` generates the vtable

## Built-in Adapter Traits

`miniextendr-api` provides ready-to-use adapter traits for common std library traits.
These have blanket implementations so you just need to export them for your types:

| Trait | Wraps | Methods |
|-------|-------|---------|
| `RDebug` | `Debug` | `debug_str()`, `debug_str_pretty()` |
| `RDisplay` | `Display` | `as_r_string()` |
| `RFromStr` | `FromStr` | `from_str(s) -> Option<Self>` |
| `RHash` | `Hash` | `hash() -> i64` |
| `ROrd` | `Ord` | `cmp(&self, other) -> i32` |
| `RPartialOrd` | `PartialOrd` | `partial_cmp(&self, other) -> Option<i32>` |
| `RError` | `Error` | `error_message()`, `error_chain()`, `error_chain_length()` |
| `RClone` | `Clone` | `clone() -> Self` |
| `RCopy` | `Copy` | `copy() -> Self`, `is_copy() -> bool` |
| `RDefault` | `Default` | `default() -> Self` |

### Generic / Associated-Type Adapter Traits

These traits have generic parameters or associated types, and are supported via
concrete vtable shim generation at the impl site:

| Trait | Wraps | Methods | Notes |
|-------|-------|---------|-------|
| `RIterator` | `Iterator` (associated type `Item`) | `next_item()`, `count()`, `collect_n()`, `skip()`, `nth()` | `next` renamed to `next_item` via `r_name`; `size_hint` skipped |
| `RExtend<T>` | `Extend` | `extend_from_vec()`, `len()`, `is_empty()` | `extend_from_slice` skipped |
| `RFromIter<T>` | `FromIterator` | `from_vec(items) -> Self` | |
| `RToVec<T>` | (iterable collections) | `to_vec()`, `len()`, `is_empty()` | |
| `RMakeIter<T, I>` | (iterator factory) | `make_iter() -> I` | `I` must implement `RIterator` |

**Usage:**

```rust
use miniextendr_api::prelude::*;
use miniextendr_api::{RDebug, RDisplay, RClone};

#[derive(Debug, Clone, ExternalPtr)]
struct MyData { value: i32 }

// Expose Debug, Display (via Debug), and Clone to R
#[miniextendr]
impl RDebug for MyData {}

#[miniextendr]
impl RClone for MyData {}

// Registration is automatic via #[miniextendr].
```

## Trait ABI Constraints

When designing adapter traits, keep these limitations in mind:

| Feature | Supported? | Notes |
|---------|------------|-------|
| Generic parameters on trait | Yes | `trait Foo<T>` — concrete shims generated at impl site |
| Generic methods | No | `fn bar<T>()` not allowed |
| Async methods | No | `async fn` not allowed |
| Associated types | Yes | `type Item` — resolved to concrete type at impl site |
| Self by value | Yes | `fn consume(self)` works |
| &self / &mut self | Yes | Standard receivers |
| Static methods | Yes | But don't go through vtable |
| `#[miniextendr(skip)]` | Yes | Exclude specific methods from R wrappers |
| `#[miniextendr(r_name = "...")]` | Yes | Rename a method in R (e.g., `next` → `next_item`) |

**Method arguments and return types** must implement:

- `TryFromSexp` for parameters (R → Rust conversion)
- `IntoR` for return values (Rust → R conversion)

## Example: Using the Built-in RIterator Trait

The built-in `RIterator` trait has an associated type `Item` and uses
`#[miniextendr(skip)]` and `#[miniextendr(r_name = "...")]` to customize
its R interface:

```rust
use miniextendr_api::prelude::*;
use miniextendr_api::RIterator;

#[derive(ExternalPtr)]
struct CountUp {
    current: i32,
    max: i32,
}

// RIterator has an associated type Item, resolved to i32 here
#[miniextendr]
impl RIterator for CountUp {
    type Item = i32;

    fn next(&self) -> Option<i32> { /* ... */ }
    fn count(&self) -> i64 { /* ... */ }
    fn collect_n(&self, n: i32) -> Vec<i32> { /* ... */ }
    fn skip(&self, n: i32) -> i32 { /* ... */ }
    fn nth(&self, n: i32) -> Option<i32> { /* ... */ }
    // size_hint is #[miniextendr(skip)] — not exposed to R
    // next is #[miniextendr(r_name = "next_item")] — called next_item() in R
}
```

In R, the method is called `next_item()` (not `next()`, which would shadow
R's built-in).

## Alternative: Newtype Wrapper

When the external trait has complex signatures or you need explicit conversions,
use a newtype wrapper instead of blanket impls:

```rust
use rust_decimal::Decimal;

// Newtype wrapper
#[derive(ExternalPtr)]
pub struct RDecimal(Decimal);

impl RDecimal {
    pub fn new(s: &str) -> Result<Self, String> {
        Decimal::from_str(s)
            .map(RDecimal)
            .map_err(|e| e.to_string())
    }

    pub fn inner(&self) -> &Decimal {
        &self.0
    }
}

// Define trait on the newtype
#[miniextendr]
pub trait DecimalOps {
    fn add(&self, other: &RDecimal) -> RDecimal;
    fn to_string(&self) -> String;
}

#[miniextendr]
impl DecimalOps for RDecimal {
    fn add(&self, other: &RDecimal) -> RDecimal {
        RDecimal(self.0 + other.0)
    }

    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
```

### When to Use Newtype vs Blanket Impl

| Approach | Use When |
|----------|----------|
| Blanket impl | External trait is simple, no associated types |
| Newtype | Need explicit conversions, complex signatures, or want isolation |

## Cross-Package Trait Dispatch

Adapter traits work with miniextendr's cross-package trait ABI:

**Producer package** (defines trait + impl):

```rust
// producer/src/lib.rs
#[miniextendr]
pub trait RNum { ... }

#[derive(ExternalPtr)]
pub struct BigInt { ... }

#[miniextendr]
impl RNum for BigInt { ... }

// Registration is automatic via #[miniextendr].
```

**Consumer package** (uses trait):

```rust
// consumer/src/lib.rs
use producer::RNum;

#[miniextendr]
fn double_it(x: &dyn RNum) -> impl RNum {
    x.add(x)  // Uses trait method via vtable
}
```

The consumer calls `RNum::add` through the vtable, allowing new implementations
to be added in other packages without recompiling the consumer.

## Complete Example

See `tests/cross-package/` for a working example of:

- `producer.pkg`: Defines `Counter` trait and `SimpleCounter` impl
- `consumer.pkg`: Uses `Counter` trait objects from producer

## Tips

1. **Keep adapter traits small** - Only expose methods you actually need in R
2. **Use concrete types** - Avoid generics; use specific types like `i32`, `f64`
3. **Document the mapping** - Explain how R values map to Rust types
4. **Handle errors explicitly** - Return `Result<T, String>` for fallible operations
5. **Consider serialization** - For complex external types, `character` (JSON/string) often works

## Trait-Provided Impl Expansion (TPIE)

When an adapter trait has a blanket impl, you can write an empty `#[miniextendr] impl`
block and the trait's default implementations are automatically used:

```rust
use miniextendr_api::{RDebug, RClone};

#[derive(Debug, Clone, ExternalPtr)]
struct MyData { value: i32 }

// Empty body — uses blanket impl from RDebug
#[miniextendr]
impl RDebug for MyData {}

// Empty body — uses blanket impl from RClone
#[miniextendr]
impl RClone for MyData {}
```

This generates all the R wrappers, vtable registration, and C-callable shims
without requiring you to re-implement the methods.

## Method Attributes

### `#[miniextendr(skip)]`

Exclude a trait method from the R interface. The method exists in Rust but
gets no R wrapper:

```rust
#[miniextendr]
pub trait MyTrait {
    fn useful_method(&self) -> i32;

    #[miniextendr(skip)]
    fn internal_only(&self) -> usize;  // Not exposed to R
}
```

### `#[miniextendr(r_name = "...")]`

Rename a method in R to avoid conflicts with R keywords or built-ins:

```rust
#[miniextendr]
pub trait RIterator {
    #[miniextendr(r_name = "next_item")]
    fn next(&self) -> Option<Self::Item>;  // Called next_item() in R
}
```

## See Also

- [SAFETY.md](SAFETY.md) - Thread safety for trait dispatch
- [ENTRYPOINT.md](ENTRYPOINT.md) - Trait ABI initialization requirements
- `miniextendr-api/src/trait_abi/` - Trait ABI implementation
