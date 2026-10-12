# Prefer* Derives Guide

How to control which R representation a Rust type uses when returned from
`#[miniextendr]` functions.

## The Problem

A Rust struct might implement several conversion traits: `IntoList`, `TypedExternal`
(from `#[derive(ExternalPtr)]`), `DataFrameRow`. When you return it from a function,
which conversion does `IntoR` use?

```rust
#[derive(IntoList, ExternalPtr)]
pub struct Point { x: f64, y: f64 }

#[miniextendr]
pub fn get_point() -> Point {
    Point { x: 1.0, y: 2.0 }
    // Does this return a list or an ExternalPtr?
}
```

A type has at most one `IntoR` impl, and the answer is whichever derive wrote it.
`#[derive(ExternalPtr)]` writes one, so `get_point()` returns an external pointer.
`IntoList` writes none: with `#[derive(IntoList)]` alone, returning `Point` does
not compile. Prefer* derives write the `IntoR` impl for the representation you
choose.

## The Prefer Derives

| Derive | Routes `IntoR` Through | Needs | R Result |
|--------|----------------------|-------|----------|
| `PreferList` | `IntoList::into_list` | `IntoList` | Named list |
| `PreferExternalPtr` | `ExternalPtr::new()` | a hand-written `TypedExternal` | External pointer |
| `PreferRNativeType` | `AsRNative` | `RNativeType` (so `Copy`) | Native R vector |
| `PreferVctrs` (`vctrs` feature) | `IntoVctrs::into_vctrs` | `#[derive(Vctrs)]` | vctrs object |

Each derive writes the type's `IntoR` impl and nothing else. There are no marker
traits to bound on: a type's representation default is just the `IntoR` impl
the derive wrote.

## Usage

### PreferList

Return struct as an R named list:

<!-- docs-examples-check -->
```rust
use miniextendr_api::{IntoList, PreferList, TryFromList, miniextendr};

#[derive(IntoList, TryFromList, PreferList)]
pub struct Config {
    pub name: String,
    pub value: f64,
    pub enabled: bool,
}

#[miniextendr]
pub fn default_config() -> Config {
    Config { name: "default".into(), value: 1.0, enabled: true }
}
// R: list(name = "default", value = 1.0, enabled = TRUE)
```

Requires: `IntoList` (the generated `IntoR` calls `into_list`). Add
`TryFromList` to reconstruct `Config` from an R `List`.

### Returning data frames

`#[derive(DataFrameRow)]` generates a `{Row}DataFrame` companion that already
converts to a data.frame. Return that companion, a `BuiltDataFrame`, or wrap
rows in `AsDataFrame`:

<!-- docs-examples-check -->
```rust
use miniextendr_api::{AsDataFrame, BuiltDataFrame, DataFrameRow, IntoDataFrame, miniextendr};

#[derive(DataFrameRow)]
pub struct Measurement {
    pub time: f64,
    pub value: f64,
}

#[miniextendr]
pub fn measurements() -> BuiltDataFrame {
    vec![Measurement { time: 0.0, value: 1.5 }].into_dataframe().unwrap()
}

#[miniextendr]
pub fn measurements_wrapped() -> AsDataFrame<Vec<Measurement>> {
    AsDataFrame(vec![Measurement { time: 0.0, value: 1.5 }])
}
```

See [DATAFRAME.md](DATAFRAME.md) for the row-type derive.

### PreferExternalPtr

`#[derive(ExternalPtr)]` already writes an `IntoR` that returns an external
pointer, so it needs no `PreferExternalPtr` (adding one is a conflicting-impl
error). `PreferExternalPtr` is for a type whose `TypedExternal` impl you write
by hand:

<!-- docs-examples-check -->
```rust
use miniextendr_api::externalptr::TypedExternal;
use miniextendr_api::{PreferExternalPtr, miniextendr};

#[derive(PreferExternalPtr)]
pub struct Engine {
    state: Vec<f64>,
}

impl TypedExternal for Engine {
    const TYPE_NAME: &'static str = "Engine";
    const TYPE_NAME_CSTR: &'static [u8] = b"Engine\0";
    const TYPE_ID_CSTR: &'static [u8] = concat!(module_path!(), "::Engine\0").as_bytes();
}

#[miniextendr]
pub fn create_engine() -> Engine {
    Engine { state: vec![0.0; 100] }
}
// R: <externalptr> (opaque, accessed via methods)
```

### PreferRNativeType

Return struct as a native R scalar (for newtype wrappers; `RNativeType`
requires `Copy`):

<!-- docs-examples-check -->
```rust
use miniextendr_api::{PreferRNativeType, RNativeType, miniextendr};

#[repr(transparent)]
#[derive(Clone, Copy, RNativeType, PreferRNativeType)]
pub struct Meters(f64);

#[miniextendr]
pub fn distance() -> Meters {
    Meters(42.0)
}
// R: 42.0 (numeric scalar)
```

## Function-Level Override: `prefer = "..."`

Instead of deriving a Prefer* on the type, you can override per-function:

<!-- docs-examples-check -->
```rust
use miniextendr_api::{ExternalPtr, IntoList, miniextendr};

#[derive(IntoList, ExternalPtr)]
pub struct Point { x: f64, y: f64 }

#[miniextendr(prefer = "list")]
pub fn point_as_list() -> Point {
    Point { x: 1.0, y: 2.0 }
}
// R: list(x = 1.0, y = 2.0)

#[miniextendr(prefer = "externalptr")]
pub fn point_as_ptr() -> Point {
    Point { x: 1.0, y: 2.0 }
}
// R: <externalptr>
```

| Value | Wraps In | Routes Through |
|-------|----------|----------------|
| `"list"` | `AsList<T>` | `IntoList` |
| `"externalptr"` | `AsExternalPtr<T>` | `ExternalPtr::new()` |
| `"vector"` / `"native"` | `AsRNative<T>` | Native R vector |
| `"auto"` | (none) | Default `IntoR` impl |

There is no function-level `"dataframe"`: return `AsDataFrame<Vec<Row>>` or a
`BuiltDataFrame` instead.

The function-level `prefer` is a one-off override. The type-level derive sets the
permanent default.

## Via `#[miniextendr]` on Structs

On a struct, `prefer = "..."` selects what `#[miniextendr]` generates:

| Attribute | Generates |
|-----------|-----------|
| `prefer = "list"` | Same as `#[miniextendr(list)]`: `IntoList` + `TryFromList` + `PreferList` |
| `prefer = "dataframe"` | Same as `#[miniextendr(dataframe)]`: `DataFrameRow` |
| `prefer = "externalptr"` | Same as the default: `ExternalPtr` |
| `prefer = "native"` | `ExternalPtr` without its `IntoR`, plus `PreferRNativeType` |

`prefer = "native"` still needs `RNativeType`, so it only fits a
`#[repr(transparent)]` newtype over a native type:

<!-- docs-examples-check -->
```rust
use miniextendr_api::{RNativeType, miniextendr};

#[miniextendr(prefer = "list")]
pub struct Config { pub name: String, pub value: f64 }
// Generates: IntoList + TryFromList + PreferList

#[miniextendr(prefer = "native")]
#[repr(transparent)]
#[derive(Clone, Copy, RNativeType)]
pub struct Meters(f64);
// Generates: ExternalPtr (no IntoR) + PreferRNativeType
```

## Conflicts

You cannot derive two Prefer* derives on the same type. They would both try to
implement `IntoR`, causing a conflict:

```rust
// ERROR: conflicting IntoR implementations
#[derive(PreferList, PreferExternalPtr)]
pub struct Bad { x: f64 }
```

Alongside the conflicting-impl error, rustc reports a duplicate definition named
`__miniextendr_conflicting_Prefer_derives__keep_ONE_or_use_call_site_As_wrappers`,
which names the fix.

Similarly, `#[derive(ExternalPtr)]` generates its own `IntoR` impl, so you cannot
combine it with `PreferList`:

```rust
// ERROR: ExternalPtr already implements IntoR
#[derive(ExternalPtr, PreferList)]
pub struct AlsoBad { x: f64 }
```

Use the function-level `prefer = "..."` override, or a call-site wrapper
(`AsList`, `AsExternalPtr`, `AsDataFrame`), when you need different
representations for the same type in different contexts.

## See Also

- [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md): `prefer` attribute reference
- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md): IntoR trait and conversion system
- [DATAFRAME.md](DATAFRAME.md): DataFrame conversion patterns
