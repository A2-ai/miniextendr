---
name: miniextendr-serde
description: Use when the user asks about serializing Rust types to R without writing TryFromSexp or IntoR by hand, how serde Serialize/Deserialize maps to R lists and vectors, what RSerializeNative and RDeserializeNative do, how AsSerialize works as a return type, when to reach for serde vs hand-rolled conversions, or how the native serde path differs from serde_json.
---

# miniextendr Serde Integration

The `serde` feature in `miniextendr-api` provides a direct R serialization
path: Rust types that implement `serde::Serialize` can be converted to R
objects, and types that implement `serde::Deserialize` can be reconstructed
from R objects — without going through an intermediate JSON string.

## When to use this skill

- "How do I convert a Rust struct to an R list without writing IntoR manually?"
- "What is RSerializeNative?"
- "What is the AsSerialize wrapper type?"
- "How does serde NA handling work?"
- "When should I use serde vs hand-rolling TryFromSexp / IntoR?"
- "What is the difference between the native serde feature and serde_json?"
- "How do I serialize a type from an external crate that has no Serialize impl?"

## Key concepts

### Feature gate

The serde integration is behind the `serde` feature:

```toml
[dependencies]
miniextendr-api = { version = "...", features = ["serde"] }
```

The optional `serde_json` feature adds a separate JSON-string path
(`json_string` submodule). The two features are independent.

### Type mappings

Serialization (Rust to R) and deserialization (R to Rust) use the following
mappings. The full mapping tables are in `miniextendr-api/src/serde.rs`.

Rust scalar types produce R scalars of the corresponding type. `Vec<primitive>`
uses smart dispatch: `Vec<i32>` becomes an integer vector, `Vec<f64>` a numeric
vector, `Vec<bool>` a logical vector, `Vec<String>` a character vector. A
`Vec<struct>` becomes a list of lists.

Structs serialize as named R lists. `HashMap<String, T>` serializes as a named
list. Unit enum variants serialize as a character scalar. Data enum variants
serialize as a tagged list `list(tag = value)`. `Option<T>::None` serializes
as NA (for primitive types) or as NULL.

### RSerializeNative and RDeserializeNative

`RSerializeNative` and `RDeserializeNative` are adapter traits in
`miniextendr-api/src/serde/traits.rs`. They have blanket implementations for
all `serde::Serialize` and `serde::Deserialize` types respectively:

- `T: Serialize` automatically gets `to_r(&self) -> Result<SEXP, String>`.
- `T: Deserialize` automatically gets `from_r(sexp: SEXP) -> Result<T, String>`.

When you write `#[miniextendr] impl RSerializeNative for MyType {}`, the
generated R wrapper gets a `to_r()` method that calls `to_r(&self)` on the
Rust side. Similarly `#[miniextendr] impl RDeserializeNative for MyType {}`
adds a `from_r(data)` class method.

The convenience functions `to_r<T: Serialize>` and `from_r<T: Deserialize>`
are also exported from `miniextendr-api/src/serde/traits.rs` for calling
directly inside Rust code when you do not want the R class method form.

### AsSerialize wrapper

`AsSerialize<T>` (in `miniextendr-api/src/serde/traits.rs`) is a newtype
wrapper that implements `IntoR` for any `T: Serialize`. This lets you return
a serializable type from a `#[miniextendr]` function directly, without
implementing `IntoR` manually or going through a class method:

```rust
#[miniextendr]
fn make_point(x: f64, y: f64) -> AsSerialize<Point> {
    AsSerialize(Point { x, y })
}
// Returns list(x = 1.0, y = 2.0) in R
```

### NA roundtrip

`Option<T>` reads both `NULL` and a length-1 `NA` as `None`, and `None`
serializes as `NULL` (the serializer never sees the absent inner type). So
`None` survives the Rust-R-Rust roundtrip, but the general mapping is not
exact: `Some(NA_real_)` reads back as `None`, a one-element `Vec` as its
element inside an untagged enum, an integer above 2^53 as a double (#1417),
`i32::MIN` as `NA` (#1419). An `ExternalPtr` snapshot uses the exact mode
in `serde/exact.rs` instead (#2256; `docs/EXTERNALPTR.md`, "The snapshot
encoding").

### R objects in a serde value: `RObject`

`miniextendr_api::serde::RObject` holds an R object (a data frame, a
formula, another saved object) in a serde type, rooted by a `Preserved`.
`to_r` writes the object itself, `from_r` gives the same object back, and an
`ExternalPtr` snapshot saves it into the `.rds` and restores it, also inside
untagged / internally tagged enums and `#[serde(flatten)]` (the snapshot
decoder lends serde's buffer a token). `from_r` refuses it inside those
buffered contexts; the columnar builders put it in a list column; every
other serializer (`serde_json`, `RValue::from_serde`) refuses it. A
type holding one is `!Send`: take it as `&T` / `&mut T`, not
`ExternalPtr<T>`. See `docs/SERDE_R.md`, "R objects inside a serde value".

### Remote derive for external types

When you need to serialize a type from an external crate that has no
`Serialize`/`Deserialize` impl, use serde's remote derive pattern:
define a shadow type mirroring the external type's structure, annotate it
with `#[serde(remote = "ExternalType")]`, and reference it via
`#[serde(with = "ShadowType")]` on fields. A worked example for
`std::time::Duration` is in `miniextendr-api/src/serde.rs`.

### Native serde vs serde_json

| Aspect | Native serde (`serde` feature) | serde_json (`serde_json` feature) |
|--------|-------------------------------|-----------------------------------|
| Intermediate | None — direct SEXP | JSON string |
| Type preservation | Yes (i32 stays integer) | No (all numbers → f64) |
| NA handling | Full Option<T> support | Limited |
| Performance | Direct conversion | Extra parse/stringify pass |
| Interop | R-native only | JSON string compatible with other tools |

Use native serde when round-tripping Rust structs to R and back. Use serde_json
when you need a JSON string for file I/O or interoperability with non-R systems.

## How it works

The core serialization path is `RSerializer::to_sexp(value)` in
`miniextendr-api/src/serde/ser.rs`. It implements serde's `Serializer` trait
and drives the mapping from serde's data model to R SEXPs.

The deserialization path is `RDeserializer::from_sexp_to(sexp)` in
`miniextendr-api/src/serde/de.rs`. It implements serde's `Deserializer` trait
and reads R SEXPs through serde's visitor protocol.

The `columnar` submodule (`miniextendr-api/src/serde/columnar.rs`) provides
`vec_to_dataframe`, `vec_to_dataframe_split` and `SerdeRowBuilder` for
converting `T: Serialize` rows to an R data frame in columnar layout; they
return the rooted `BuiltDataFrame` (there is no separate columnar frame type).
`SerdeRows(rows).into_dataframe()` is the same path through the
`IntoDataFrame` verb.

## Decision trees

### Serde-derive or hand-rolled conversions?

Use serde (`#[derive(Serialize, Deserialize)]`) when:
- The type has many fields and you want all of them serializable to R lists
  without writing each TryFromSexp / IntoR impl by hand.
- The type is already `Serialize`/`Deserialize` for other purposes (e.g.,
  writing JSON config files).
- You need to accept the type as input from R as a named list.

Use hand-rolled `TryFromSexp` / `IntoR` when:
- The type needs a custom R representation that does not map naturally to a
  named list (e.g., a packed binary format, an ALTREP vector, or a raw SEXP).
- Performance is critical and the serde visitor overhead is measurable.
- The type participates in strict-mode or coerce-mode conversion logic that
  the serde path does not expose.

### Which serde entry point to use?

- Returning a struct from a `#[miniextendr]` function without a class method:
  use `AsSerialize<T>` as the return type.
- Exposing `to_r()` / `from_r()` as class methods on an ExternalPtr type:
  use `#[miniextendr] impl RSerializeNative for MyType {}` and
  `#[miniextendr] impl RDeserializeNative for MyType {}`.
- Calling serde conversion inside Rust code (no R class method):
  use the free functions `to_r(&value)` and `from_r::<T>(sexp)`.
- Converting `Vec<T>` to a columnar R data frame:
  use `vec_to_dataframe(&items)` from the `columnar` submodule.

## Key files

- `miniextendr-api/src/serde.rs` — module doc with full type-mapping tables,
  smart Vec dispatch explanation, NA roundtrip, remote derive examples, and the
  native-vs-json comparison. Re-exports: `RSerializer`, `RDeserializer`,
  `RSerdeError`, `RSerializeNative`, `RDeserializeNative`, `to_r`, `from_r`,
  `AsSerialize`, `SerdeRows`, `vec_to_dataframe` and the other columnar helpers.
- `miniextendr-api/src/serde/traits.rs` — `RSerializeNative`,
  `RDeserializeNative`, `to_r`, `from_r`, and `AsSerialize<T>`.
- `miniextendr-api/src/serde/ser.rs` — `RSerializer` (serde Serializer impl).
- `miniextendr-api/src/serde/de.rs` — `RDeserializer` (serde Deserializer impl).
- `miniextendr-api/src/serde/exact.rs` (+ `exact/ser.rs`, `exact/de.rs`) — the
  exact snapshot encoding an `ExternalPtr` save uses; module docs hold its
  encoding and `deserialize_any` tables.
- `miniextendr-api/src/serde/robject.rs` — `RObject` and its hand-over to the
  miniextendr serializers / deserializers (newtype marker, thread-locals,
  buffered-token registry).
- `miniextendr-api/src/serde/columnar.rs` — `vec_to_dataframe`,
  `vec_to_dataframe_split`, `SerdeRowBuilder`.

## Common pitfalls

- **All-None columns land as LGLSXP**: in the serde columnar path (`vec_to_dataframe`, `SerdeRows`),
  columns where every row's `Option<T>` was `None` are stored as a logical NA
  column (`LGLSXP`), not `list(NULL, NULL, ...)`. R coerces logical NA to the
  surrounding type on first use (`c(NA, 1L)` → integer vector). Mixed
  Some/None columns are unaffected.

- **serde_json feature is separate from serde**: enabling `features = ["serde"]`
  does not enable the JSON-string submodule. The JSON path requires `features =
  ["serde_json"]`. They can both be enabled simultaneously.

- **AsSerialize panics on serialization failure**: the `IntoR` impl for
  `AsSerialize<T>` calls `to_r(&self.0).unwrap_or_else(|e| r_stop(...))`.
  Serialization failures become R errors (via `r_stop`), not Rust panics.
  If you need explicit error handling, use `to_r` directly.

- **Remote derive requires a shadow type for each external type**: serde's
  remote derive cannot be applied directly to a type you do not own. You must
  define a shadow struct/enum that mirrors the layout. See the `Duration`
  example in `miniextendr-api/src/serde.rs` for the full pattern.

## Related skills

- `miniextendr-conversions` — hand-rolled `TryFromSexp` / `IntoR`: the
  alternative to serde for custom R representations.
- `miniextendr-externalptr` — how Rust structs are stored as R objects;
  `RSerializeNative` and `RDeserializeNative` are typically implemented on
  ExternalPtr-backed types.
- `miniextendr-macros` — `#[miniextendr] impl RSerializeNative for T {}` and
  `#[miniextendr] impl RDeserializeNative for T {}` are processed like any
  other impl block.
