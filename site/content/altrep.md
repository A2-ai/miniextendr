+++
title = "ALTREP"
weight = 5
description = "Lazy and zero-copy vectors via Alternative Representations"
+++

ALTREP (Alternative Representations) is R's system for custom vector implementations. miniextendr provides proc-macro-driven ALTREP support for lazy, compact, and zero-copy vectors.

## What is ALTREP?

ALTREP allows you to create R vectors with custom internal representations:

- **Compute elements on demand** (lazy sequences)
- **Reference external data** without copying (zero-copy views)
- **Use compact representations** (constant vectors, arithmetic sequences)
- **Provide optimized operations** (O(1) sum for arithmetic sequences)

## Quick Start

A constant integer vector:

```rust
use miniextendr_api::{miniextendr, ffi::SEXP, IntoR};
use miniextendr_api::altrep_data::{AltrepLen, AltIntegerData};

// 1. Define your data type
#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantIntData {
    value: i32,
    len: usize,
}

// 2. Implement ALTREP traits
impl AltrepLen for ConstantIntData {
    fn length(&self) -> usize { self.len }
}

impl AltIntegerData for ConstantIntData {
    fn elt(&self, _index: usize) -> i32 { self.value }
}

// 3. Export a constructor
#[miniextendr]
pub fn constant_int(value: i32, len: i32) -> impl IntoR {
    ConstantIntData { value, len: len as usize }.into_altrep()
}
```

```r
x <- constant_int(42L, 1000000L)
length(x)  # 1000000 (O(1))
x[1]       # 42 (computed on demand)
```

## Derive Options

Control ALTREP behavior with derive attributes:

- `len = "field"` -- derive `Length` from a struct field
- `elt = "field"` -- constant element (all positions return the same value)
- `elt_delegate = "field"` -- delegate `elt(i)` to an inner type's implementation

## Guard Modes

Control how panics and R errors are handled in ALTREP callbacks:

- **`#[altrep(rust_unwind)]`** (default) -- `catch_unwind` for Rust panics
- **`#[altrep(r_unwind)]`** -- `with_r_unwind_protect` for callbacks that call R API
- **`#[altrep(unsafe)]`** -- no protection (fastest, requires manual safety guarantee)

## Supported Vector Types

| Trait | R Type | Element |
|-------|--------|---------|
| `AltIntegerData` | integer | `i32` |
| `AltRealData` | numeric | `f64` |
| `AltLogicalData` | logical | `i32` |
| `AltComplexData` | complex | `Rcomplex` |
| `AltRawData` | raw | `u8` |
| `AltStringData` | character | `Option<String>` |

## Materialization

When R needs the full data (e.g., for `DATAPTR`), ALTREP vectors materialize. miniextendr caches the materialized result in the ALTREP `data2` slot.

## Serialization

ALTREP objects serialize by materializing to standard R vectors. Custom serialization can be implemented via the `Serialize` ALTREP method.
