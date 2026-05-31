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

`FromDataFrame` is emitted for every **struct** row shape: simple scalar fields,
column expansion (`[T; N]`, `Vec<T>` + `width`, `Vec<T>`/`Box<[T]>` + `expand`),
struct-flatten (nested `DataFrameRow` fields, including several levels of nesting),
and opaque list-columns (un-annotated `Vec<scalar>` / `Box<[scalar]>` fields stored
as VECSXP list-columns — each row's element is deserialized via `Vec<elem>:
TryFromSexp` and `.into()`-converted to the field container type). Each reader is the
exact inverse of its writer — it regroups the suffixed expansion columns, reads each
`<field>_`-prefixed sub-frame back through the nested type's own reader, and
deserializes opaque list-column elements per row.

A few struct shapes still have no reader and return a clear `DataFrameError` from
`from_dataframe` (rather than failing to compile): borrowed fields (`&[T]` /
`&str` — owned R data can't produce a borrow), `#[dataframe(skip)]` fields (the
column was never written), `#[dataframe(as_list)]` and opaque non-scalar collection
columns (`HashMap`, `HashSet`, `Vec<Option<T>>` list-columns), and
`#[dataframe(tag)]` structs. **Enum** row shapes have no reader yet; that gap is
tracked in the follow-up issue.

Ragged `width`/`expand` columns round-trip losslessly because the writer only
ever pads *trailing* slots with `NA`: the reader flattens the present values back
into the `Vec`, and re-writing pads the same trailing slots again.

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
| `DataFrameView`, `convert::DataFrame<T>` (duplicate `DataFrame` types) | one `DataFrame` |
| `ToDataFrame<Companion>` return wrapper + `value.to_data_frame()` | `rows.into_dataframe()?` |
| `convert::SerializeDataFrame<T>` / `AsSerializeRow<T>` | `serde::SerdeRows(rows).into_dataframe()?` |
| `Row::try_from_dataframe(sexp)` (bare `String` error) | `Vec::<Row>::from_dataframe(&df)?` (`DataFrameError`) |
| `RDataFrameBuilder::new(n)` | `DataFrame::builder(n)` |
| four conversion error types | one `DataFrameError` |

The redundant public types above have been **removed** (#781) — there is no
backwards-compat shim. The legacy companion methods (`to_dataframe`, `from_rows`,
`try_from_dataframe`) remain as the internal engine the trait impls delegate to.
The serde columnar assembler has been aligned with the façade (#783): the internal
columnar newtype is now folded into `DataFrame` (all serde column helpers return
`DataFrame` directly), and the streaming serde-row builder is now `SerdeRowBuilder`
(paired with `SerdeRows`). Two serde-side builders remain distinct from
`DataFrame::builder`: `SerdeRowBuilder<T>` for incremental serde rows assembled into
one `DataFrame`, and `NamedDataFrameListBuilder` for a named list of frames.
