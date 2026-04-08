+++
title = "Raw Conversions"
weight = 35
+++

Convert Rust POD (Plain Old Data) types to and from R raw vectors using `bytemuck`.

**Source**: `miniextendr-api/src/raw_conversions.rs`
**Feature gate**: `raw_conversions`

```toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["raw_conversions"] }
```

## Why Raw Conversions?

R raw vectors (`RAWSXP`) are untyped byte buffers. This module provides a safe bridge between Rust structs and R raw vectors for scenarios where you need to:

- Store binary data in R objects (e.g., geometry, protocol buffers, audio samples)
- Pass structured data between R and Rust without serialization overhead
- Persist Rust structs in R's `.rds` files

The conversion is zero-copy when alignment permits, falling back to a copy when it does not.

## Wrapper Types

| Type | Format | Type Tag | Use Case |
|------|--------|----------|----------|
| `Raw<T>` | Headerless | No | Fast single value, no validation |
| `RawSlice<T>` | Headerless | No | Fast sequence, no validation |
| `RawTagged<T>` | Header + data | Yes | Single value with safety checks |
| `RawSliceTagged<T>` | Header + data | Yes | Sequence with safety checks |

**Headerless** types store raw bytes directly. Fast, but decoding is the caller's responsibility.

**Tagged** types prepend a 16-byte `RawHeader` (magic `"MXRB"`, version, element size, count) and set an `mx_raw_type` R attribute with the Rust type name. Decoding validates the header, element size, and type name.

## Quick Start

```rust
use bytemuck::{Pod, Zeroable};
use miniextendr_api::raw_conversions::{Raw, RawSlice};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[miniextendr]
fn encode_vec3(x: f64, y: f64, z: f64) -> Raw<Vec3> {
    Raw(Vec3 { x: x as f32, y: y as f32, z: z as f32 })
}

#[miniextendr]
fn decode_vec3(raw: Raw<Vec3>) -> Vec<f64> {
    vec![raw.0.x as f64, raw.0.y as f64, raw.0.z as f64]
}
```

From R:

```r
bytes <- encode_vec3(1.0, 2.0, 3.0)  # Returns a raw vector (12 bytes)
decode_vec3(bytes)                     # Returns c(1.0, 2.0, 3.0)
```

## POD Requirements

All types used with raw conversions must implement `bytemuck::Pod` and `bytemuck::Zeroable`. These traits guarantee the type is safe to interpret as raw bytes:

- `#[repr(C)]` layout (deterministic field order)
- No padding bytes that could leak uninitialized memory
- No pointers, references, or `bool` fields
- All fields must themselves be `Pod`

```rust
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
```

The `Pod` and `Zeroable` derives are re-exported from `miniextendr_api::raw_conversions` for convenience.

## Headerless Format (Raw / RawSlice)

### Raw\<T\> -- Single Value

Stores `size_of::<T>()` bytes in a raw vector. No metadata.

```rust
// Encode
#[miniextendr]
fn pack_point(x: i32, y: i32) -> Raw<Point> {
    Raw(Point { x, y })
}

// Decode
#[miniextendr]
fn unpack_point(raw: Raw<Point>) -> Vec<i32> {
    let p = raw.into_inner();
    vec![p.x, p.y]
}
```

### RawSlice\<T\> -- Sequence

Stores `n * size_of::<T>()` bytes contiguously. The element count is inferred from the raw vector length.

```rust
#[miniextendr]
fn pack_points(xs: Vec<i32>, ys: Vec<i32>) -> RawSlice<Point> {
    let points: Vec<Point> = xs.iter().zip(ys.iter())
        .map(|(&x, &y)| Point { x, y })
        .collect();
    RawSlice(points)
}

#[miniextendr]
fn unpack_xs(raw: RawSlice<Point>) -> Vec<i32> {
    raw.inner().iter().map(|p| p.x).collect()
}
```

## Tagged Format (RawTagged / RawSliceTagged)

Tagged types add a 16-byte header and an R attribute for safer decoding.

### Header Layout

```text
Offset  Size  Field
0       4     magic: "MXRB"
4       4     version: 1 (u32)
8       4     elem_size (u32)
12      4     elem_count (u32)
```

### R Attribute

Tagged types set an `mx_raw_type` attribute on the raw vector containing the Rust type name (e.g., `"my_crate::Point"`). On decode, the attribute is checked first -- if the type name does not match, decoding fails before reading any bytes.

### Usage

```rust
#[miniextendr]
fn pack_tagged(x: i32, y: i32) -> RawTagged<Point> {
    RawTagged(Point { x, y })
}

#[miniextendr]
fn unpack_tagged(raw: RawTagged<Point>) -> Vec<i32> {
    let p = raw.into_inner();
    vec![p.x, p.y]
}
```

Passing the wrong type produces a clear error:

```r
pack_tagged(1L, 2L) |> unpack_other_type()
#> Error: type mismatch: expected other_crate::OtherType, got my_crate::Point
```

## Safety Model

- **Alignment**: Always checked. Misaligned data is copied to an aligned buffer (no UB).
- **Length**: Always checked. Mismatched byte counts return `SexpError::InvalidValue`.
- **Type tag** (tagged only): Checked via R attribute before reading bytes.
- **Endianness**: Not handled. Bytes are stored in native layout. Data is **not portable** across architectures.
- **No interior pointers**: `Pod` guarantees the type contains no pointers, so the bytes are self-contained.

## Error Types

`RawError` covers all failure modes:

| Variant | Cause |
|---------|-------|
| `LengthMismatch` | Raw vector byte count does not match `size_of::<T>()` (or not a multiple for slices) |
| `AlignmentMismatch` | Internal -- handled by copy fallback, not exposed to users |
| `InvalidHeader` | Bad magic bytes or unsupported version in tagged format |
| `TypeMismatch` | `mx_raw_type` attribute does not match expected Rust type name |

## Standalone Helper Functions

For working with raw bytes outside the `#[miniextendr]` function boundary:

```rust
use miniextendr_api::raw_conversions::{raw_to_bytes, raw_from_bytes,
                                        raw_slice_to_bytes, raw_slice_from_bytes};

let bytes = raw_to_bytes(&Point { x: 1, y: 2 });
let point: Point = raw_from_bytes(&bytes).unwrap();

let bytes = raw_slice_to_bytes(&[Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]);
let points: Vec<Point> = raw_slice_from_bytes(&bytes).unwrap();
```

## Choosing a Format

| Criterion | Headerless | Tagged |
|-----------|-----------|--------|
| Overhead | None | 16 bytes + attribute |
| Type safety | None (caller must know the type) | Validated on decode |
| Persistence | Fragile across code changes | Catches size/type drift |
| Speed | Fastest | Negligible overhead |

Use **headerless** for ephemeral data within a single session where the type is always known. Use **tagged** for data that may be saved to `.rds` files or passed between sessions/packages.
