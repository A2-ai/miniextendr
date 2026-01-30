# Data Frame Conversion in miniextendr

miniextendr provides comprehensive support for converting between Rust types and R data frames, with three complementary approaches offering different trade-offs between ergonomics and flexibility.

## Overview

| Approach | Best For | Code Generation | Flexibility |
|----------|----------|-----------------|-------------|
| `#[derive(DataFrameRow)]` | Type-safe, ergonomic APIs | ✅ Generates DataFrame type | ⭐⭐⭐ Easy |
| `DataFrame<T>` | Generic, reusable code | ❌ No codegen | ⭐⭐ Moderate |
| `impl IntoDataFrame` | Full control, complex cases | ❌ Manual impl | ⭐ Advanced |

## Core Traits

### `IntoDataFrame`

The foundational trait for converting Rust types to R data frames.

```rust
pub trait IntoDataFrame {
    fn into_data_frame(self) -> List;
}
```

**Key Points:**
- Consumes `self` (owning conversion)
- Returns a `List` with data.frame attributes
- Used by all other approaches under the hood

**Related:**
- `AsDataFrame` (in `as_coerce` module) - S3 coercion methods for `as.data.frame()` on ExternalPtr types
- `IntoDataFrame` (this trait) - Direct conversion for return values

---

## Approach 1: Derive Macro (Recommended)

Use `#[derive(DataFrameRow)]` for the most ergonomic experience. The macro generates a companion DataFrame type and all necessary conversions.

### Basic Usage

```rust
use miniextendr_api::{miniextendr, DataFrameRow, IntoList};

#[derive(Clone, IntoList, DataFrameRow)]
struct Measurement {
    time: f64,
    value: f64,
    sensor: String,
}

// Auto-generates:
// - struct MeasurementDataFrame { time: Vec<f64>, value: Vec<f64>, sensor: Vec<String> }
// - impl IntoDataFrame for MeasurementDataFrame
// - impl From<Vec<Measurement>> for MeasurementDataFrame
// - impl IntoIterator for MeasurementDataFrame -> Measurement
// - Measurement::to_dataframe() and from_dataframe() methods

#[miniextendr]
fn get_measurements() -> MeasurementDataFrame {
    let rows = vec![
        Measurement { time: 1.0, value: 10.0, sensor: "A".into() },
        Measurement { time: 2.0, value: 20.0, sensor: "B".into() },
        Measurement { time: 3.0, value: 30.0, sensor: "C".into() },
    ];

    Measurement::to_dataframe(rows)  // or: rows.into()
}
```

### Heterogeneous Types

The derive macro fully supports different types in different fields:

```rust
#[derive(Clone, IntoList, DataFrameRow)]
struct Person {
    name: String,      // character in R
    age: i32,          // integer in R
    height: f64,       // numeric in R
    is_student: bool,  // logical in R
}

// Each field maintains its distinct type throughout conversion
```

### Collection Types

Row fields can use various collection types:

```rust
use std::collections::{HashSet, BTreeSet};

#[derive(Clone, DataFrameRow)]
struct ComplexRow {
    // Standard vectors
    measurements: Vec<f64>,

    // Fixed-size arrays
    coords: [f64; 3],

    // Boxed slices (heap-allocated)
    data: Box<[i32]>,

    // Hash sets (unordered, unique values)
    tags: HashSet<String>,

    // Tree sets (ordered, unique values)
    categories: BTreeSet<i32>,
}

// Generated DataFrame has Vec<CollectionType> for each field:
// - measurements: Vec<Vec<f64>>
// - coords: Vec<[f64; 3]>
// - data: Vec<Box<[i32]>>
// - tags: Vec<HashSet<String>>
// - categories: Vec<BTreeSet<i32>>
```

**Note:** Collection field types need manual `IntoList` implementations (see examples in `rpkg/src/rust/dataframe_collections_test.rs`)

### With Serde (when `serde` feature enabled)

```rust
use serde::Serialize;

#[derive(Serialize, DataFrameRow)]  // Serialize implies IntoList!
struct Reading {
    timestamp: f64,
    temperature: f64,
    humidity: f64,
}

#[miniextendr]
fn get_readings() -> ReadingDataFrame {
    Reading::to_dataframe(vec![
        Reading { timestamp: 1.0, temperature: 20.5, humidity: 65.0 },
        Reading { timestamp: 2.0, temperature: 21.0, humidity: 63.0 },
    ])
}
```

### Generated Methods

The derive macro adds these methods to your row type:

```rust
impl Measurement {
    /// Name of the generated companion DataFrame type
    pub const DATAFRAME_TYPE_NAME: &'static str = "MeasurementDataFrame";

    /// Transpose rows to columns
    pub fn to_dataframe(rows: Vec<Self>) -> MeasurementDataFrame;

    /// Transpose columns back to rows
    pub fn from_dataframe(df: MeasurementDataFrame) -> Vec<Self>;
}
```

### Iterating Over Rows

The generated DataFrame type implements `IntoIterator`:

```rust
let df = get_measurements();

// Iterate over rows
for measurement in df {
    println!("Time: {}, Value: {}", measurement.time, measurement.value);
}

// Or collect back to Vec
let rows: Vec<Measurement> = df.into_iter().collect();
```

### Requirements

The row type must implement `IntoList`:
- Automatically via `#[derive(IntoList)]`
- Via `#[derive(Serialize)]` when `serde` feature is enabled
- Via manual implementation using `List::from_raw_pairs()` (for heterogeneous fields)

### Attributes (TODO)

Future attributes for customization:

```rust
#[derive(DataFrameRow)]
#[dataframe(name = "Measurements")]  // Custom DataFrame name
struct Measurement { /* ... */ }
```

Note: Collection type is always `Vec<T>` currently. Future versions may support custom collection types via attributes.

---

## Approach 2: DataFrame<T>

Generic type for transposing row-oriented data. Works with any `T: IntoList`.

### With IntoList Types

```rust
#[derive(IntoList)]
struct Point {
    x: f64,
    y: f64,
}

#[miniextendr]
fn points() -> DataFrame<Point> {
    DataFrame::from_rows(vec![
        Point { x: 1.0, y: 2.0 },
        Point { x: 3.0, y: 4.0 },
    ])
}
```

### With Serialize Types

When the `serde` feature is enabled, use `from_serialize()` for the simplest experience:

```rust
use serde::Serialize;
use miniextendr_api::SerializeDataFrame;

#[derive(Serialize)]
struct Event {
    timestamp: f64,
    message: String,
}

#[miniextendr]
fn events() -> SerializeDataFrame<Event> {
    let events = vec![
        Event { timestamp: 1.0, message: "start".into() },
        Event { timestamp: 2.0, message: "end".into() },
    ];
    SerializeDataFrame::from_serialize(events)
}
```

`SerializeDataFrame<T>` is a type alias for `DataFrame<AsSerializeRow<T>>`, and `from_serialize()` handles wrapping each row automatically.

**Alternative (explicit wrapping):**

If you prefer the explicit form or need more control:

```rust
#[miniextendr]
fn events() -> DataFrame<AsSerializeRow<Event>> {
    DataFrame::from_rows(vec![
        AsSerializeRow(Event { timestamp: 1.0, message: "start".into() }),
        AsSerializeRow(Event { timestamp: 2.0, message: "end".into() }),
    ])
}
```

### Methods

```rust
impl<T: IntoList> DataFrame<T> {
    pub fn new() -> Self;
    pub fn from_rows(rows: Vec<T>) -> Self;
    pub fn push(&mut self, row: T);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

// Also implements FromIterator
let df: DataFrame<Point> = points.into_iter().collect();
```

---

## Approach 3: Manual Implementation

For full control or complex scenarios, implement `IntoDataFrame` manually.

### Column-Oriented Data (Homogeneous Types)

For data frames where all columns have the same element type, use `List::from_pairs()`:

```rust
struct TimeSeries {
    timestamps: Vec<f64>,
    values: Vec<f64>,
}

impl IntoDataFrame for TimeSeries {
    fn into_data_frame(self) -> List {
        List::from_pairs(vec![
            ("timestamp", self.timestamps),
            ("value", self.values),
        ])
        .set_class_str(&["data.frame"])
        .set_row_names_int(self.timestamps.len())
    }
}

#[miniextendr]
fn time_series() -> TimeSeries {
    TimeSeries {
        timestamps: vec![1.0, 2.0, 3.0],
        values: vec![10.0, 20.0, 30.0],
    }
}
// Automatically converts to data.frame via IntoR
```

### Column-Oriented Data (Heterogeneous Types)

**Important:** For data frames with different column types, use `List::from_raw_pairs()` instead of `from_pairs()`:

```rust
use miniextendr_api::IntoR;

struct MixedData {
    names: Vec<String>,
    ages: Vec<i32>,
    heights: Vec<f64>,
}

impl IntoDataFrame for MixedData {
    fn into_data_frame(self) -> List {
        List::from_raw_pairs(vec![
            ("name", self.names.into_sexp()),
            ("age", self.ages.into_sexp()),
            ("height", self.heights.into_sexp()),
        ])
        .set_class_str(&["data.frame"])
        .set_row_names_int(self.names.len())
    }
}
```

**Why?** `from_pairs()` is generic over a single type `T: IntoR`, so all columns must have the same type. `from_raw_pairs()` accepts pre-converted `SEXP` values, allowing heterogeneous columns.

### Call-Site Control with Wrappers

Force conversion for a specific return without changing the type's default:

```rust
#[miniextendr]
fn as_dataframe() -> ToDataFrame<TimeSeries> {
    ToDataFrame(TimeSeries { /* ... */ })
}

// Or use the extension trait
#[miniextendr]
fn with_extension() -> ToDataFrame<TimeSeries> {
    TimeSeries { /* ... */ }.to_data_frame()
}
```

### Type-Level Default with `PreferDataFrame`

Make a type always convert to data.frame when returned:

```rust
#[derive(PreferDataFrame)]
struct MyData {
    // ... fields ...
}

impl IntoDataFrame for MyData {
    fn into_data_frame(self) -> List {
        // ... implementation ...
    }
}

#[miniextendr]
fn get_data() -> MyData {  // Automatically becomes data.frame in R
    MyData { /* ... */ }
}
```

---

## Comparison: Row vs Column Oriented

### Row-Oriented (Vec of structs)

```rust
vec![
    Measurement { time: 1.0, value: 10.0 },
    Measurement { time: 2.0, value: 20.0 },
]
```

**Pros:**
- Natural Rust data structure
- Easy to work with in Rust code
- Type-safe field access

**Cons:**
- Needs transposition for R
- Memory layout not optimal for R

### Column-Oriented (Struct of Vecs)

```rust
MeasurementDataFrame {
    time: vec![1.0, 2.0],
    value: vec![10.0, 20.0],
}
```

**Pros:**
- Direct R data.frame representation
- No transposition needed
- Memory efficient for R

**Cons:**
- Less ergonomic in Rust
- Easy to create invalid data (mismatched lengths)

---

## Best Practices

### Choosing an Approach

1. **Use `#[derive(DataFrameRow)]`** when:
   - You have row-oriented data in Rust
   - You want type-safe field access
   - You want automatic conversions

2. **Use `DataFrame<T>`** when:
   - You need generic code over many row types
   - You're working with existing IntoList types
   - You want runtime flexibility

3. **Use manual `impl IntoDataFrame`** when:
   - You already have column-oriented data
   - You need custom data.frame attributes
   - You're handling complex validation

### Handling Missing Data

Use `Option<T>` for nullable fields:

```rust
#[derive(IntoList, DataFrameRow)]
struct Record {
    id: i32,
    value: Option<f64>,  // Becomes NA in R when None
}
```

### Validation

Always validate column lengths when manually constructing data frames:

```rust
impl IntoDataFrame for MyData {
    fn into_data_frame(self) -> List {
        assert_eq!(self.col1.len(), self.col2.len(), "Column length mismatch");

        List::from_pairs(vec![
            ("col1", self.col1),
            ("col2", self.col2),
        ])
        .set_class_str(&["data.frame"])
        .set_row_names_int(self.col1.len())
    }
}
```

---

## Implementation Notes

### Row Names

R data frames require row names. miniextendr provides two helpers:

```rust
list.set_row_names_int(n)     // Compact: c(NA, -n) form
list.set_row_names(names_vec)  // Explicit: character vector
```

### Class Attribute

Data frames need the `"data.frame"` class:

```rust
list.set_class_str(&["data.frame"])
```

For subclasses (e.g., tibbles):

```rust
list.set_class_str(&["tbl_df", "tbl", "data.frame"])
```

### Empty Data Frames

```rust
List::from_raw_pairs(Vec::<(&str, SEXP)>::new())
    .set_class_str(&["data.frame"])
    .set_row_names_int(0)
```

---

## Feature Flags

- **Base functionality**: No features required
- **Serde integration**: Requires `serde` feature
  - Enables `impl IntoList for T: Serialize`
  - Enables `AsSerializeRow<T>` wrapper
  - Allows `#[derive(Serialize, DataFrameRow)]`

---

## Examples

See [rpkg/src/rust/dataframe_examples.rs](../rpkg/src/rust/dataframe_examples.rs) for complete working examples.
