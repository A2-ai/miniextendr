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

## Two Paths

**Field-based derive**: `#[altrep(len = "field", elt = "field", class = "Name")]` on `#[derive(AltrepInteger)]` (or any other type). miniextendr generates everything from the struct fields -- no trait impls needed.

**Manual**: `#[altrep(manual, class = "Name")]` on `#[derive(AltrepInteger)]`. miniextendr generates the registration and boilerplate; you implement `AltrepLen` and `AltIntegerData` (or the matching `Alt*Data` trait) by hand. Use this when element values require computation beyond what a single field provides.

ALTREP callbacks run on R's main thread (they receive `SEXP` arguments, which are not `Send`).

## Quick Start

A constant integer vector (manual path):

```rust
use miniextendr_api::miniextendr;
use miniextendr_api::altrep_data::{AltrepLen, AltIntegerData};

// 1. Define your data type and register it as an ALTREP integer class.
//    `#[altrep(manual, class = "ConstantInt")]` tells the derive to register
//    the class but let you write AltrepLen + AltIntegerData yourself.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(class = "ConstantInt", manual)]
pub struct ConstantIntData {
    value: i32,
    len: usize,
}

// 2. Implement the required data traits.
impl AltrepLen for ConstantIntData {
    fn len(&self) -> usize { self.len }
}

impl AltIntegerData for ConstantIntData {
    fn elt(&self, _i: usize) -> i32 { self.value }
}

// 3. Export a constructor. Returning ConstantIntData directly wraps it as ALTREP.
#[miniextendr]
pub fn constant_int(value: i32, len: i32) -> ConstantIntData {
    ConstantIntData { value, len: len as usize }
}
```

```r
x <- constant_int(42L, 1000000L)
length(x)  # 1000000 (O(1))
x[1]       # 42 (computed on demand)
```

## Derive Options

Field-based derive attributes (non-manual path only):

- `len = "field"` -- derive length from a struct field
- `elt = "field"` -- all positions return the same field value (constant vector)
- `elt_delegate = "field"` -- delegate `elt(i)` to an inner type's implementation

Common attributes for both paths:

- `class = "ClassName"` -- R class name for registration (required)
- `manual` -- opt out of field-based codegen; you implement `AltrepLen` and `Alt*Data`
- `dataptr` -- enable `DATAPTR` (pointer to materialized buffer); requires implementing `AltrepDataptr`
- `serialize` -- enable custom serialization; requires implementing `AltrepSerialize`

## Guard Modes

Control how panics and R errors are handled in ALTREP callbacks:

- **`#[altrep(rust_unwind)]`** (default) -- `catch_unwind` for Rust panics
- **`#[altrep(r_unwind)]`** -- `with_r_unwind_protect` for callbacks that call R API
- **`#[altrep(unsafe)]`** -- no protection (fastest, requires manual safety guarantee)

## Supported Vector Types

Each type has a corresponding derive (`#[derive(AltrepInteger)]`, `#[derive(AltrepReal)]`, etc.) and a `manual` fallback (`#[derive(Altrep)]`).

| Derive | Data Trait | R Type | Element |
|--------|-----------|--------|---------|
| `AltrepInteger` | `AltIntegerData` | integer | `i32` |
| `AltrepReal` | `AltRealData` | numeric | `f64` |
| `AltrepLogical` | `AltLogicalData` | logical | `Logical` |
| `AltrepComplex` | `AltComplexData` | complex | `Rcomplex` |
| `AltrepRaw` | `AltRawData` | raw | `u8` |
| `AltrepString` | `AltStringData` | character | `Option<&str>` |
| `AltrepList` | `AltListData` | list | `SEXP` |
| `Altrep` | (manual) | any | (user-defined) |

## Materialization

When R needs the full data (e.g., for `DATAPTR`), ALTREP vectors materialize. miniextendr caches the materialized result in the ALTREP `data2` slot.

## Serialization

ALTREP objects serialize by materializing to standard R vectors. Custom serialization can be implemented via the `Serialize` ALTREP method.
