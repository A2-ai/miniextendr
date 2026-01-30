# Data Frame Conversion in miniextendr

miniextendr provides comprehensive support for converting between Rust types and R data frames, with three complementary approaches offering different trade-offs between ergonomics and flexibility.

## Overview

| Approach | Best For | Code Generation | Flexibility |
|----------|----------|-----------------|-------------|
| `#[derive(DataFrameRow)]` | Type-safe, ergonomic APIs | ✅ Generates DataFrame type | ⭐⭐⭐ Easy |
| `DataFrameRows<T>` | Generic, reusable code | ❌ No codegen | ⭐⭐ Moderate |
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

### Attributes (TODO)

Future attributes for customization:

```rust
#[derive(DataFrameRow)]
#[dataframe(name = "Measurements")]  // Custom DataFrame name
#[dataframe(collection = "Box<[T]>")]  // Use Box<[T]> instead of Vec<T>
struct Measurement { /* ... */ }
```

---

## Approach 2: DataFrameRows<T>

Generic type for transposing row-oriented data. Works with any `T: IntoList`.

### With IntoList Types

```rust
#[derive(IntoList)]
struct Point {
    x: f64,
    y: f64,
}

#[miniextendr]
fn points() -> DataFrameRows<Point> {
    DataFrameRows::from_rows(vec![
        Point { x: 1.0, y: 2.0 },
        Point { x: 3.0, y: 4.0 },
    ])
}
```

### With Serialize Types

When the `serde` feature is enabled, use `AsSerializeRow` wrapper:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Event {
    timestamp: f64,
    message: String,
}

#[miniextendr]
fn events() -> DataFrameRows<AsSerializeRow<Event>> {
    DataFrameRows::from_rows(vec![
        AsSerializeRow(Event { timestamp: 1.0, message: "start".into() }),
        AsSerializeRow(Event { timestamp: 2.0, message: "end".into() }),
    ])
}
```

### Methods

```rust
impl<T: IntoList> DataFrameRows<T> {
    pub fn new() -> Self;
    pub fn from_rows(rows: Vec<T>) -> Self;
    pub fn push(&mut self, row: T);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

// Also implements FromIterator
let df: DataFrameRows<Point> = points.into_iter().collect();
```

---

## Approach 3: Manual Implementation

For full control or complex scenarios, implement `IntoDataFrame` manually.

### Column-Oriented Data

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

2. **Use `DataFrameRows<T>`** when:
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
