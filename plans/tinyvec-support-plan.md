# Plan: tinyvec Support (TinyVec/ArrayVec Conversions)

Goal: Add an optional `tinyvec` feature that lets users pass `TinyVec`/`ArrayVec` values across the R boundary with the same ergonomics and NA behavior as existing `Vec<T>` conversions.

## 1) Types to cover

Primary targets:
- `TinyVec<[T; N]>` for small, growable vectors (inline + heap fallback).
- `ArrayVec<[T; N]>` for fixed-capacity inline vectors.

Element coverage (mirror existing `Vec<T>` support):
- Native R types: `i32`, `f64`, `u8`, `RLogical`, `Rboolean`.
- Coerced numeric types: `i8`, `i16`, `u16`, `f32`, `i64`, `u64`, `isize`, `usize`.
- Strings: `String`, `&str`.
- NA-aware vectors: `Option<T>` for logical/numeric/string.

Out of scope (first pass):
- `SliceVec<'a, [T; N]>` from R (borrows cannot outlive the SEXP).
- Custom element types beyond what `Vec<T>` already supports.

## 2) Rust surface syntax (examples)

```rust
use tinyvec::{ArrayVec, TinyVec};
use miniextendr_api::prelude::*;

#[miniextendr]
fn sum_small(x: TinyVec<[i32; 8]>) -> i32 {
    x.into_iter().sum()
}

#[miniextendr]
fn top3(x: ArrayVec<[f64; 3]>) -> ArrayVec<[f64; 3]> {
    x
}

#[miniextendr]
fn maybe_names(x: TinyVec<[Option<String>; 4]>) -> i32 {
    x.into_iter().filter(|v| v.is_some()).count() as i32
}
```

## 3) Conversion strategy

### 3.1 `TryFromSexp` (R -> tinyvec)
- `TinyVec<[T; N]>`: convert using existing `Vec<T>` conversions, then `TinyVec::from_iter` (or `from_vec` if available).
- `ArrayVec<[T; N]>`: same conversion path, but check `len <= N` and return a clear error if capacity would overflow.
- `Option<TinyVec<...>>`: `NULL -> None`, otherwise `Some`.

### 3.2 `IntoR` (tinyvec -> R)
- For `T: RNativeType`, use `as_slice()` and the existing slice-to-R path to avoid an intermediate `Vec`.
- For non-native `T` (e.g. `String`, `Option<T>`), collect into `Vec<T>` and reuse existing `Vec<T>` conversions.
- `Option<TinyVec<...>>`: `None -> NULL`, `Some` -> vector.

### 3.3 NA handling
- Match current semantics: `bool` errors on NA, `Option<bool>` preserves NA, same for numeric and string `Option`.
- Include index and capacity details in error messages for easier debugging.

## 4) Implementation plan (files)

1) Add feature flag + dependency:
   - `miniextendr-api/Cargo.toml`: new `tinyvec` feature and optional dependency.
2) New optional module:
   - `miniextendr-api/src/optionals/tinyvec_impl.rs`
   - Re-export `TinyVec` and `ArrayVec` plus any type aliases (e.g. `RTinyVec<T, N>`).
3) Wire module + re-exports:
   - `miniextendr-api/src/optionals.rs` (feature table + `mod tinyvec_impl`).
   - `miniextendr-api/src/lib.rs` (feature table + `pub use` behind `cfg`).
4) R package feature passthrough:
   - `rpkg/src/rust/Cargo.toml.in` (add `tinyvec` feature).
   - `rpkg/configure.ac` (include `tinyvec` in default feature list, if desired).

## 5) Tests

Feature-gated tests in `miniextendr-api/tests/from_r.rs` and `miniextendr-api/tests/into_r.rs`:
- `TinyVec<[i32; 4]>`, `TinyVec<[f64; 4]>`, `TinyVec<[bool; 4]>`.
- `TinyVec<[Option<bool>; 4]>`, `TinyVec<[Option<String>; 4]>` (NA handling).
- `ArrayVec<[i32; 3]>` success + overflow error (`len > 3`).
- Round-trip test: R -> TinyVec -> R preserves values.

## 6) Docs

- Update feature tables in:
  - `miniextendr-api/src/lib.rs`
  - `miniextendr-api/README.md`
  - `miniextendr-api/src/optionals.rs`
- Add module docs with examples in `tinyvec_impl.rs`.
