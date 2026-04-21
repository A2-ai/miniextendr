+++
title = "Features"
weight = 10
description = "Cargo features, derive macros, and optional subsystems"
+++

## Cargo Features

miniextendr uses cargo features to gate optional functionality:

| Feature | Description |
|---------|-------------|
| `serde_r` | Direct Rust-R serialization via serde |
| `serde_full` | Both JSON and native R serialization |
| `worker-thread` | Dedicated worker thread for Rust code |
| `default-worker` | Implies `worker-thread`, makes it the default |
| `nonapi` | Access to non-API R functions (`DATAPTR`, etc.) |
| `rayon` | Parallel computation via rayon |
| `materialization-tracking` | ALTREP materialization diagnostics |

## Derive Macros

miniextendr ships roughly twenty derives grouped by what they produce.

### Wrapper types

| Derive | Purpose |
|--------|---------|
| `#[derive(ExternalPtr)]` | Wrap struct as an `EXTPTRSXP`; implements `TypedExternal` and `IntoExternalPtr` |
| `#[derive(RNativeType)]` | Newtype wrapper around a native R scalar (`i32`, `f64`, `bool`, `String`, etc.) |

### ALTREP

Typed derives generate the full ALTREP class from field attributes. The manual `Altrep` derive lets you implement the per-method traits yourself.

| Derive | Purpose |
|--------|---------|
| `#[derive(AltrepInteger)]` | Integer ALTREP class from `#[altrep(len, elt, class, …)]` fields |
| `#[derive(AltrepReal)]` | Real (double) ALTREP class |
| `#[derive(AltrepLogical)]` | Logical ALTREP class |
| `#[derive(AltrepRaw)]` | Raw (byte) ALTREP class |
| `#[derive(AltrepString)]` | Character ALTREP class (`Vec<Option<String>>` preserves `NA_character_`) |
| `#[derive(AltrepComplex)]` | Complex ALTREP class |
| `#[derive(AltrepList)]` | List ALTREP class |
| `#[derive(Altrep)]` | Manual pattern — registers the class; you implement `AltrepLen` and `Alt*Data` |

### List / data-frame round-tripping

| Derive | Purpose |
|--------|---------|
| `#[derive(IntoList)]` | Convert struct → named R list |
| `#[derive(TryFromList)]` | Convert named R list → struct |
| `#[derive(DataFrameRow)]` | Treat struct as a data-frame row; generates a companion DataFrame type |
| `#[derive(Vctrs)]` | vctrs-compatible S3 vector class (`Vctr`, `Rcrd`, `ListOf` kinds) |

### Enums ↔ R

| Derive | Purpose |
|--------|---------|
| `#[derive(RFactor)]` | Map enum variants to R factor levels |
| `#[derive(MatchArg)]` | Map enum variants to R character values via `match.arg` |

### Conversion preference

Control which `IntoR` / `TryFromSexp` path a type takes when multiple are possible.

| Derive | Purpose |
|--------|---------|
| `#[derive(PreferExternalPtr)]` | Prefer `ExternalPtr` wrapping |
| `#[derive(PreferDataFrame)]` | Prefer data-frame representation |
| `#[derive(PreferList)]` | Prefer named-list representation |
| `#[derive(PreferRNativeType)]` | Prefer native R scalar representation |

## Attribute Options

The `#[miniextendr]` attribute supports many options:

```rust
// Function-level
#[miniextendr]                       // Basic export
#[miniextendr(strict)]               // Reject lossy conversions
#[miniextendr(internal)]             // @keywords internal
#[miniextendr(noexport)]             // Suppress @export
#[miniextendr(unwrap_in_r)]          // Return Result errors as R values
#[miniextendr(default = "value")]    // Default parameter value

// Impl-level (class systems)
#[miniextendr]                       // Env style (default)
#[miniextendr(r6)]                   // R6 class
#[miniextendr(s3)]                   // S3 methods
#[miniextendr(s4)]                   // S4 class
#[miniextendr(s7)]                   // S7 class
#[miniextendr(label = "name")]       // Label for multiple impl blocks
```

## Variadic Arguments (Dots)

R's `...` becomes `&Dots` in Rust:

```rust
use miniextendr_api::dots::Dots;

#[miniextendr]
pub fn count_args(_dots: &Dots, ...) -> i32 {
    _dots.len() as i32
}
```

Validate dot structure with `typed_list!`:

```rust
#[miniextendr(dots = typed_list!(x: i32, y: f64))]
pub fn structured_dots(_dots: &Dots, ...) -> f64 {
    dots_typed.x as f64 + dots_typed.y
}
```

## Factors (Enums)

Map Rust enums to R factors:

```rust
#[derive(miniextendr_api::RFactor)]
pub enum Color { Red, Green, Blue }

#[miniextendr]
pub fn describe_color(color: Color) -> &'static str {
    match color {
        Color::Red => "warm",
        Color::Green | Color::Blue => "cool",
    }
}
```
