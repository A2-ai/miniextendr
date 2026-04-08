+++
title = "Prefer* Derives Guide"
weight = 53
+++

How to control which R representation a Rust type uses when returned from
`#[miniextendr]` functions.

## The Problem

A Rust struct might implement multiple conversion traits: `IntoList`, `IntoExternalPtr`,
`IntoDataFrame`. When you return it from a function, which conversion does `IntoR` use?

```rust
#[derive(IntoList, ExternalPtr)]
pub struct Point { x: f64, y: f64 }

#[miniextendr]
pub fn get_point() -> Point {
    Point { x: 1.0, y: 2.0 }
    // Does this return a list or an ExternalPtr?
}
```

The answer depends on which `IntoR` implementation exists. Prefer* derives let you
choose explicitly.

## The Four Markers

| Derive | Marker Trait | Routes `IntoR` Through | R Result |
|--------|-------------|----------------------|----------|
| `PreferList` | `PrefersList` | `IntoList` | Named list |
| `PreferDataFrame` | `PrefersDataFrame` | `IntoDataFrame` | data.frame |
| `PreferExternalPtr` | `PrefersExternalPtr` | `ExternalPtr::new()` | External pointer |
| `PreferRNativeType` | `PrefersRNativeType` | `AsRNative` | Native R vector |

Each derive does two things:
1. Implements the empty marker trait
2. Implements `IntoR` routing through the corresponding conversion

## Usage

### PreferList

Return struct as an R named list:

```rust
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

Requires: `IntoList` (the `PrefersList` trait has `IntoList` as a supertrait).

### PreferDataFrame

Return struct as an R data.frame (the struct must implement `IntoDataFrame`):

```rust
#[derive(IntoList, DataFrameRow, PreferDataFrame)]
pub struct Measurement {
    pub time: f64,
    pub value: f64,
}
```

Note: In practice, `#[miniextendr(dataframe)]` or `#[derive(DataFrameRow)]` handles
this automatically via the companion type pattern. `PreferDataFrame` is for advanced
cases where you have a custom `IntoDataFrame` impl.

### PreferExternalPtr

Return struct as an opaque R external pointer:

```rust
#[derive(ExternalPtr, PreferExternalPtr)]
pub struct Engine {
    state: Vec<f64>,
}

#[miniextendr]
pub fn create_engine() -> Engine {
    Engine { state: vec![0.0; 100] }
}
// R: <externalptr> (opaque, accessed via methods)
```

### PreferRNativeType

Return struct as a native R scalar (for newtype wrappers):

```rust
#[derive(RNativeType, PreferRNativeType)]
pub struct Meters(f64);

#[miniextendr]
pub fn distance() -> Meters {
    Meters(42.0)
}
// R: 42.0 (numeric scalar)
```

## Function-Level Override: `prefer = "..."`

Instead of deriving a marker on the type, you can override per-function:

```rust
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

The function-level `prefer` is a one-off override. The type-level derive sets the
permanent default.

## Via `#[miniextendr]` on Structs

The `#[miniextendr(prefer = "...")]` attribute on structs combines ExternalPtr
generation with a preference marker:

```rust
#[miniextendr(prefer = "list")]
pub struct Config { pub name: String, pub value: f64 }
// Generates: IntoList + TryFromList + PreferList

#[miniextendr(prefer = "native")]
pub struct Wrapper { pub data: Vec<i32> }
// Generates: ExternalPtr + PreferRNativeType
```

## Conflicts

You cannot derive two Prefer* markers on the same type — they would both try to
implement `IntoR`, causing a conflict:

```rust
// ERROR: conflicting IntoR implementations
#[derive(PreferList, PreferExternalPtr)]
pub struct Bad { x: f64 }
```

Similarly, `#[derive(ExternalPtr)]` generates its own `IntoR` impl, so you cannot
combine it with `PreferList`:

```rust
// ERROR: ExternalPtr already implements IntoR
#[derive(ExternalPtr, PreferList)]
pub struct AlsoBad { x: f64 }
```

Use the function-level `prefer = "..."` override when you need different
representations for the same type in different contexts.

## See Also

- [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md) — `prefer` attribute reference
- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) — IntoR trait and conversion system
- [DATAFRAME.md](DATAFRAME.md) — DataFrame conversion patterns
