# The unified DataFrame interface

miniextendr exposes one owned [`DataFrame`] type and two conversion traits that
mirror the scalar/vector surface (`IntoR` / `TryFromSexp`):

| Trait | Method | Direction | Analogue |
|---|---|---|---|
| `IntoDataFrame` | `rows.into_dataframe()? -> DataFrame` | Rust → R | `IntoR` |
| `FromDataFrame` | `Vec::<Row>::from_dataframe(&df)? -> Vec<Row>` | R → Rust | `TryFromSexp` |

Both verbs live **on the data** (`Vec<Row>` / `&DataFrame`), not on a companion
type. Errors are a single [`DataFrameError`]. There is one NA contract: missing
cells round-trip as `Option<T>` fields.

## The owned `DataFrame` type

`DataFrame` wraps a validated `data.frame` SEXP. It implements `IntoR` (returns
the backing SEXP) and `TryFromSexp` (validates the `data.frame` class), so it
flows through `#[miniextendr]` signatures like any other type:

```rust
use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, IntoList, miniextendr};

#[derive(Clone, IntoList, DataFrameRow)]
pub struct Point { pub x: f64, pub y: f64, pub label: String }

// Rust → R: build a data.frame from a row vector.
#[miniextendr]
fn make_points(df: DataFrame) -> DataFrame {
    let rows: Vec<Point> = Vec::<Point>::from_dataframe(&df).unwrap();   // R → Rust
    rows.into_dataframe().unwrap()                                       // Rust → R
}
```

## Parallel fast paths (`feature = "rayon"`)

Explicit `_par` variants produce the **same** `DataFrame` / `Vec<Row>` as the
sequential verbs — parallelism is an opt-in method, not a hidden threshold:

```rust
let df   = rows.into_dataframe_par()?;          // #777 flattened (column,row-range) fill
let rows = Vec::<Point>::from_dataframe_par(&df)?; // #765 off-main-thread row assembly
```

Dropping `_par` (building without the `rayon` feature) degrades cleanly to the
sequential call — the verb name is stable across feature sets.

## Builder for heterogeneous columns (`feature = "rayon"`)

When you are filling columns directly (not transposing a `Vec<Row>`), use the
builder, which yields a `DataFrame`:

```rust
let df = DataFrame::builder(nrow)
    .column::<f64>("x", |chunk, off| { /* fill */ })
    .column_str("label", |i| Some(format!("p{i}")))
    .build();
```

## How `#[derive(DataFrameRow)]` wires this up

The orphan rule forbids the derive from writing `impl IntoDataFrame for Vec<Row>`
in your crate (`IntoDataFrame` and `Vec` are both foreign there). Instead the
derive implements the `#[doc(hidden)]` local marker `DataFrameRowConvert` on your
local `Row` type, and `miniextendr_api` carries the blanket
`impl<T: DataFrameRowConvert> IntoDataFrame for Vec<T>`. You still call the public
verbs — the indirection is invisible.

`FromDataFrame` is emitted only for **simple scalar-field structs** (the shapes
with an R→Rust reader). Calling `from_dataframe` on a shape without a reader
(expansion / struct-flatten / nested-enum / map columns) returns a clear
`DataFrameError` rather than failing to compile. Extending the reader to those
shapes is tracked in the follow-up issue.

## serde rows

Types that derive `serde::Serialize` / `Deserialize` convert through the
`SerdeRows<T>` newtype (so the serde path never collides with the derive's
concrete `Vec<Row>` conversions):

```rust
let df   = SerdeRows(rows).into_dataframe()?;
let rows = SerdeRows::<Row>::from_dataframe(&df)?.into_inner();
```

## Migration from the legacy surface

| Was | Now |
|---|---|
| `ColumnarDataFrame`, `DataFrameView`, `convert::DataFrame<T>` (three types) | one `DataFrame` |
| `Row::to_dataframe(rows)` + companion `IntoR` | `rows.into_dataframe()?` |
| `Row::try_from_dataframe(sexp)` (bare `String` error) | `Vec::<Row>::from_dataframe(&df)?` (`DataFrameError`) |
| `RDataFrameBuilder::new(n)` | `DataFrame::builder(n)` |
| four conversion error types | one `DataFrameError` |

The legacy companion methods (`to_dataframe`, `from_rows`, `try_from_dataframe`)
remain as the internal engine the trait impls delegate to; the redundant
public *types* are demoted behind the new façade and tracked for removal.
