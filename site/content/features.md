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

| Derive | Purpose |
|--------|---------|
| `#[derive(ExternalPtr)]` | Wrap struct as R external pointer |
| `#[derive(DataFrameRow)]` | Convert struct to/from data frame row |
| `#[derive(Vctrs)]` | Generate vctrs-compatible class |
| `#[derive(RFactor)]` | Map enum variants to R factor levels |
| `#[derive(PreferExternalPtr)]` | Prefer ExternalPtr over data conversion |
| `#[derive(PreferDataFrame)]` | Prefer DataFrame over ExternalPtr |

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
