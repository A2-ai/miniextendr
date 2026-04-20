+++
title = "serde_r: Direct Rust-R Serialization"
weight = 36
description = "The serde_r feature provides direct serialization between Rust types and native R objects without going through an intermediate format like JSON. This enables efficient, type-preserving conversions that respect R's native data structures."
+++

The `serde_r` feature provides direct serialization between Rust types and native R objects without going through an intermediate format like JSON. This enables efficient, type-preserving conversions that respect R's native data structures.

## Overview

| Feature | `serde` (JSON) | `serde_r` (Native) |
|---------|----------------|-------------------|
| Intermediate format | JSON string | None |
| Type preservation | No (all numbers → f64) | Yes (i32 stays i32) |
| NA handling | Limited | Full support via `Option<T>` |
| Performance | Extra parse/stringify | Direct conversion |
| Smart Vec dispatch | No | Yes (Vec<i32> → integer vector) |

## Enabling the Feature

```toml
# Cargo.toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["serde_r"] }

# Or for both JSON and native R serialization:
miniextendr-api = { version = "0.1", features = ["serde_full"] }
```

## Type Mappings

### Serialization (Rust → R)

| Rust Type | R Type | Notes |
|-----------|--------|-------|
| `bool` | `logical(1)` | Scalar |
| `i8/i16/i32` | `integer(1)` | Widened to i32 |
| `i64/u64/f32/f64` | `numeric(1)` | Converted to f64 |
| `String/&str` | `character(1)` | UTF-8 preserved |
| `Option<T>::Some(v)` | T | Transparent |
| `Option<T>::None` | `NULL` | |
| `Vec<i32>` | `integer` vector | Smart dispatch |
| `Vec<f64>` | `numeric` vector | Smart dispatch |
| `Vec<bool>` | `logical` vector | Smart dispatch |
| `Vec<String>` | `character` vector | Smart dispatch |
| `Vec<struct>` | `list` of lists | Heterogeneous |
| `HashMap<String, T>` | named `list` | Keys become names |
| `BTreeMap<String, T>` | named `list` | Sorted keys |
| `struct { fields }` | named `list` | Field names preserved |
| `()` / unit struct | `NULL` | |
| unit enum variant | `character(1)` | Variant name |
| newtype variant | `list(Variant = value)` | Tagged |
| tuple variant | `list(Variant = list(...))` | Tagged list |
| struct variant | `list(Variant = list(a=..., b=...))` | Tagged named list |

### Deserialization (R → Rust)

| R Type | Rust Type | Notes |
|--------|-----------|-------|
| `logical(1)` | `bool` | NA → error or `Option::None` |
| `integer(1)` | `i32` | NA → error or `Option::None` |
| `numeric(1)` | `f64` | NA → error or `Option::None` |
| `character(1)` | `String` | NA → error or `Option::None` |
| `integer` vector | `Vec<i32>` | |
| `numeric` vector | `Vec<f64>` | |
| `logical` vector | `Vec<bool>` | |
| `character` vector | `Vec<String>` | |
| `raw` vector | `Vec<u8>` / `&[u8]` | |
| named `list` | struct / `HashMap` | Field matching |
| unnamed `list` | `Vec<T>` / tuple | Positional |
| `NULL` | `()` / `Option::None` | |

## Basic Usage

### Defining Serializable Types

```rust
use serde::{Serialize, Deserialize};
use miniextendr_api::{miniextendr, ExternalPtr};
use miniextendr_api::serde_r::{RSerializeNative, RDeserializeNative};

#[derive(Serialize, Deserialize, Clone, ExternalPtr)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[miniextendr]
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

// Register the adapter traits
#[miniextendr]
impl RSerializeNative for Point {}

#[miniextendr]
impl RDeserializeNative for Point {}

// Registration is automatic via #[miniextendr].
```

### Using from R

```r
# Create a Point
p <- Point$new(1.0, 2.0)

# Serialize to R list
data <- p$to_r()
# list(x = 1.0, y = 2.0)

# Access fields
data$x  # 1.0
data$y  # 2.0

# Deserialize from R list
p2 <- Point$from_r(list(x = 3.0, y = 4.0))
p2$x  # 3.0
p2$y  # 4.0

# Round-trip
original <- Point$new(5.0, 6.0)
restored <- Point$from_r(original$to_r())
identical(original$x, restored$x)  # TRUE
```

## Smart Vec Dispatch

One of the key features of `serde_r` is smart vector dispatch. When serializing `Vec<T>`, the serializer automatically chooses the most efficient R representation:

```rust
// Vec<i32> -> integer vector (atomic)
let ints = vec![1, 2, 3, 4, 5];
// Serializes to: c(1L, 2L, 3L, 4L, 5L)

// Vec<f64> -> numeric vector (atomic)
let floats = vec![1.1, 2.2, 3.3];
// Serializes to: c(1.1, 2.2, 3.3)

// Vec<String> -> character vector (atomic)
let strings = vec!["a".to_string(), "b".to_string()];
// Serializes to: c("a", "b")

// Vec<Point> -> list of lists (heterogeneous)
let points = vec![Point { x: 1.0, y: 2.0 }];
// Serializes to: list(list(x = 1.0, y = 2.0))
```

## NA/NULL Handling

### Option<T> for NA Support

Use `Option<T>` to represent potentially missing values:

```rust
#[derive(Serialize, Deserialize, ExternalPtr)]
pub struct Record {
    pub id: i32,                    // Required
    pub name: Option<String>,       // Optional (can be NULL)
    pub value: Option<f64>,         // Optional (can be NULL)
}
```

From R:
```r
# Create with all values
r1 <- Record$from_r(list(id = 1L, name = "test", value = 3.14))

# Create with missing values
r2 <- Record$from_r(list(id = 2L, name = NULL, value = NULL))

# Serialize back
r2$to_r()
# list(id = 2L, name = NULL, value = NULL)
```

## Nested Structures

`serde_r` handles arbitrarily nested structures:

```rust
#[derive(Serialize, Deserialize)]
pub struct Level3 {
    pub data: Vec<f64>,
    pub flag: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Level2 {
    pub level3: Level3,
    pub values: Vec<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct Level1 {
    pub level2: Level2,
    pub name: String,
}

#[derive(Serialize, Deserialize, ExternalPtr)]
pub struct DeepNest {
    pub level1: Level1,
}
```

From R:
```r
# Create from deeply nested R list
deep <- list(
  level1 = list(
    level2 = list(
      level3 = list(
        data = c(1.0, 2.0, 3.0),
        flag = TRUE
      ),
      values = c(10L, 20L, 30L)
    ),
    name = "nested"
  )
)

dn <- DeepNest$from_r(deep)
```

## Enum Serialization

### Unit Variants

Unit enum variants serialize to character strings:

```rust
#[derive(Serialize, Deserialize)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}
```

From R:
```r
# Unit variant -> character
status <- "Active"  # Deserializes to Status::Active
```

### Data Variants

Data-carrying variants serialize to tagged lists:

```rust
#[derive(Serialize, Deserialize)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}
```

From R:
```r
# Circle { radius: 5.0 } serializes to:
list(Circle = list(radius = 5.0))

# Rectangle { width: 10.0, height: 20.0 } serializes to:
list(Rectangle = list(width = 10.0, height = 20.0))
```

## HashMap/BTreeMap

Maps with string keys become named R lists:

```rust
use std::collections::HashMap;

#[derive(Serialize, Deserialize, ExternalPtr)]
pub struct Config {
    pub settings: HashMap<String, i32>,
    pub metadata: HashMap<String, String>,
}
```

From R:
```r
cfg <- Config$from_r(list(
  settings = list(timeout = 30L, retries = 3L),
  metadata = list(author = "test", version = "1.0")
))

data <- cfg$to_r()
data$settings$timeout  # 30L
data$metadata$author   # "test"
```

## Standalone Functions

For one-off conversions without registering types:

```rust
use miniextendr_api::serde_r::{to_r, from_r};

#[miniextendr]
pub fn convert_to_r() -> SEXP {
    let data = vec![1, 2, 3, 4, 5];
    to_r(&data).expect("serialize")
}

#[miniextendr]
pub fn convert_from_r(sexp: SEXP) -> Vec<i32> {
    from_r(sexp).expect("deserialize")
}
```

## Columnar `data.frame` Assembly

For `&[T: Serialize]`, `ColumnarDataFrame::from_rows` (alias
`vec_to_dataframe`) produces a column-oriented R `data.frame` where each
field of `T` becomes one atomic column. Nested structs are recursively
flattened into prefixed columns (`point_x`, `point_y`); `#[serde(flatten)]`
fields appear without a prefix; `#[serde(skip_serializing_if)]` fills NA.
`Option<Struct>` fills NA across all sub-columns when `None`.

```rust
use miniextendr_api::serde_r::{ColumnarDataFrame, vec_to_dataframe};

#[derive(Serialize)]
struct Row {
    id: i32,
    point: Point,           // flattened to point_x, point_y
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,   // NA when None
}

#[miniextendr]
pub fn rows_as_df(rows: Vec<Row>) -> ColumnarDataFrame {
    ColumnarDataFrame::from_rows(&rows).unwrap()
        .rename("point_x", "x")
        .rename("point_y", "y")
        .drop("note")
}
```

`ColumnarDataFrame` implements `IntoR`, so return it directly from a
`#[miniextendr]` function - no explicit `.build()` or `into_sexp()` call
needed. Builder methods (`rename`, `strip_prefix`, `drop`, `select`) are
chainable and run before the SEXP reaches R.

## Error Handling

Deserialization can fail for various reasons:

```rust
use miniextendr_api::serde_r::from_r;

#[miniextendr]
pub fn safe_deserialize(sexp: SEXP) -> Result<Point, String> {
    from_r::<Point>(sexp).map_err(|e| e.to_string())
}
```

Error types include:
- `TypeMismatch` - Wrong R type for target Rust type
- `MissingField` - Required struct field not in list
- `InvalidVariant` - Unknown enum variant name
- `LengthMismatch` - Wrong length for tuple/array
- `UnexpectedNa` - NA where not allowed
- `Overflow` - Numeric overflow in conversion

## Integration with R Object Systems

### With R6

```r
library(R6)

MyClass <- R6Class("MyClass",
  public = list(
    x = NULL,
    y = NULL,
    initialize = function(x, y) {
      self$x <- x
      self$y <- y
    },
    to_list = function() list(x = self$x, y = self$y)
  )
)

obj <- MyClass$new(1.0, 2.0)
point <- Point$from_r(obj$to_list())
```

### With S4

```r
setClass("S4Point", slots = c(x = "numeric", y = "numeric"))
s4obj <- new("S4Point", x = 3.0, y = 4.0)

# Extract slots as list
point <- Point$from_r(list(x = s4obj@x, y = s4obj@y))
```

### With S7

```r
library(S7)

S7Point <- new_class("S7Point",
  properties = list(x = class_double, y = class_double)
)

s7obj <- S7Point(x = 5.0, y = 6.0)
point <- Point$from_r(list(x = prop(s7obj, "x"), y = prop(s7obj, "y")))
```

### With Environments

```r
e <- new.env()
e$x <- 7.0
e$y <- 8.0

point <- Point$from_r(as.list(e))
```

## Comparison with IntoList Derive

miniextendr also provides `#[derive(IntoList)]` for simpler struct-to-list conversion. Here's how they compare:

| Feature | `IntoList` | `serde_r` |
|---------|-----------|-----------|
| Derive macro | Yes | Needs serde derives |
| Deserialization | No (one-way) | Yes (bidirectional) |
| Enum support | No | Yes |
| Smart Vec dispatch | No | Yes |
| HashMap/BTreeMap | No | Yes |
| Option/NA | No | Yes |
| Nested structs | Yes | Yes |

Use `IntoList` for simple one-way struct-to-list conversion. Use `serde_r` when you need full bidirectional serialization, enum support, or smart vector handling.
