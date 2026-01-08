# Adapter Traits: Exporting External Traits to R

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

// 4. Register in module
miniextendr_module! {
    mod mymath;
    impl RNum for MyNumber;
}
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
| `RFromStr` | `FromStr` | `r_from_str(s) -> Option<Self>` |
| `RHash` | `Hash` | `r_hash() -> i64` |
| `ROrd` | `Ord` | `r_cmp(&self, other) -> i32` |
| `RPartialOrd` | `PartialOrd` | `r_partial_cmp(&self, other) -> Option<i32>` |
| `RError` | `Error` | `error_message()`, `error_chain()`, `error_chain_length()` |
| `RClone` | `Clone` | `r_clone() -> Self` |
| `RCopy` | `Copy` | `r_copy() -> Self`, `is_copy() -> bool` |
| `RDefault` | `Default` | `r_default() -> Self` |

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

miniextendr_module! {
    mod mymod;
    impl RDebug for MyData;
    impl RClone for MyData;
}
```

## Trait ABI Constraints

When designing adapter traits, keep these limitations in mind:

| Feature | Supported? | Notes |
|---------|------------|-------|
| Generic parameters on trait | No | `trait Foo<T>` not allowed |
| Generic methods | No | `fn bar<T>()` not allowed |
| Async methods | No | `async fn` not allowed |
| Associated types | No | `type Item` not allowed |
| Self by value | Yes | `fn consume(self)` works |
| &self / &mut self | Yes | Standard receivers |
| Static methods | Yes | But don't go through vtable |

**Method arguments and return types** must implement:

- `TryFromSexp` for parameters (R → Rust conversion)
- `IntoR` for return values (Rust → R conversion)

## Example: Exposing Iterator-like Behavior

External trait `Iterator` has associated types, so we create a simpler adapter:

```rust
#[miniextendr]
pub trait RIterator {
    /// Get the next item, or NULL if exhausted
    fn next_item(&mut self) -> Option<i32>;

    /// Collect remaining items into a vector
    fn collect_rest(&mut self) -> Vec<i32>;
}

// For any Iterator<Item = i32>
impl<T> RIterator for T
where
    T: Iterator<Item = i32>,
{
    fn next_item(&mut self) -> Option<i32> {
        self.next()
    }

    fn collect_rest(&mut self) -> Vec<i32> {
        self.collect()
    }
}
```

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

miniextendr_module! {
    mod producer;
    impl RNum for BigInt;
}
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

## More Adapter Trait Examples

### Display and FromStr Adapters

Expose string formatting and parsing from external types:

```rust
use std::fmt::Display;
use std::str::FromStr;

/// Adapter for Display - convert any Display type to R character
#[miniextendr]
pub trait RDisplay {
    fn as_r_string(&self) -> String;
}

impl<T: Display> RDisplay for T {
    fn as_r_string(&self) -> String {
        self.to_string()
    }
}

/// Adapter for FromStr - parse R character into Rust types
#[miniextendr]
pub trait RFromStr: Sized {
    fn from_r_string(s: &str) -> Result<Self, String>;
}

// Example impl for a specific type (can't blanket impl due to Sized constraint)
impl RFromStr for MyType {
    fn from_r_string(s: &str) -> Result<Self, String> {
        MyType::from_str(s).map_err(|e| e.to_string())
    }
}
```

### Debug Adapter

Expose debug formatting for inspection in R:

```rust
use std::fmt::Debug;

#[miniextendr]
pub trait RDebug {
    fn debug_string(&self) -> String;
    fn debug_pretty(&self) -> String;
}

impl<T: Debug> RDebug for T {
    fn debug_string(&self) -> String {
        format!("{:?}", self)
    }

    fn debug_pretty(&self) -> String {
        format!("{:#?}", self)
    }
}
```

### Comparison Adapters

Expose ordering for R sort operations:

```rust
use std::cmp::Ordering;

#[miniextendr]
pub trait ROrd {
    /// Compare two values: returns -1, 0, or 1
    fn r_cmp(&self, other: &Self) -> i32;
}

impl<T: Ord> ROrd for T {
    fn r_cmp(&self, other: &Self) -> i32 {
        match self.cmp(other) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }
}

#[miniextendr]
pub trait RPartialOrd {
    /// Compare two values: returns -1, 0, 1, or NA for incomparable
    fn r_partial_cmp(&self, other: &Self) -> Option<i32>;
}

impl<T: PartialOrd> RPartialOrd for T {
    fn r_partial_cmp(&self, other: &Self) -> Option<i32> {
        self.partial_cmp(other).map(|ord| match ord {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        })
    }
}
```

### Hash Adapter

Expose hashing for R deduplication or environments:

```rust
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[miniextendr]
pub trait RHash {
    /// Get a 64-bit hash of the value
    fn r_hash(&self) -> i64;
}

impl<T: Hash> RHash for T {
    fn r_hash(&self) -> i64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as i64
    }
}
```

### Serde Adapters (with `serde` feature)

Expose JSON serialization for R interop:

```rust
use serde::{Serialize, Deserialize};

#[miniextendr]
pub trait RSerialize {
    /// Serialize to JSON string
    fn to_json(&self) -> Result<String, String>;

    /// Serialize to pretty-printed JSON
    fn to_json_pretty(&self) -> Result<String, String>;
}

impl<T: Serialize> RSerialize for T {
    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

#[miniextendr]
pub trait RDeserialize: Sized {
    /// Deserialize from JSON string
    fn from_json(s: &str) -> Result<Self, String>;
}

// Example for a specific type
impl RDeserialize for MyConfig {
    fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
```

### Error Adapter

Expose error chains for rich R error reporting:

```rust
use std::error::Error;

#[miniextendr]
pub trait RError {
    /// Get the error message
    fn error_message(&self) -> String;

    /// Get the full error chain as a vector of messages
    fn error_chain(&self) -> Vec<String>;
}

impl<T: Error> RError for T {
    fn error_message(&self) -> String {
        self.to_string()
    }

    fn error_chain(&self) -> Vec<String> {
        let mut chain = vec![self.to_string()];
        let mut current: &dyn Error = self;
        while let Some(source) = current.source() {
            chain.push(source.to_string());
            current = source;
        }
        chain
    }
}
```

### IO Adapters (with `connections` feature)

Expose Rust IO traits for R connection interop:

```rust
use std::io::{Read, Write, BufRead};

#[miniextendr]
pub trait RRead {
    /// Read up to n bytes, returns raw vector
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>, String>;

    /// Read all remaining bytes
    fn read_to_end(&mut self) -> Result<Vec<u8>, String>;
}

impl<T: Read> RRead for T {
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; n];
        let bytes_read = self.read(&mut buf).map_err(|e| e.to_string())?;
        buf.truncate(bytes_read);
        Ok(buf)
    }

    fn read_to_end(&mut self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        Read::read_to_end(self, &mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }
}

#[miniextendr]
pub trait RWrite {
    /// Write bytes, returns number written
    fn write_bytes(&mut self, data: Vec<u8>) -> Result<usize, String>;

    /// Flush the writer
    fn flush(&mut self) -> Result<(), String>;
}

impl<T: Write> RWrite for T {
    fn write_bytes(&mut self, data: Vec<u8>) -> Result<usize, String> {
        self.write(&data).map_err(|e| e.to_string())
    }

    fn flush(&mut self) -> Result<(), String> {
        Write::flush(self).map_err(|e| e.to_string())
    }
}

#[miniextendr]
pub trait RBufRead {
    /// Read a single line (without newline)
    fn read_line(&mut self) -> Result<Option<String>, String>;
}

impl<T: BufRead> RBufRead for T {
    fn read_line(&mut self) -> Result<Option<String>, String> {
        let mut line = String::new();
        match BufRead::read_line(self, &mut line) {
            Ok(0) => Ok(None),  // EOF
            Ok(_) => {
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Ok(Some(line))
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
```

## See Also

- [SAFETY.md](SAFETY.md) - Thread safety for trait dispatch
- [ENTRYPOINT.md](ENTRYPOINT.md) - Trait ABI initialization requirements
- `miniextendr-api/src/trait_abi/` - Trait ABI implementation
