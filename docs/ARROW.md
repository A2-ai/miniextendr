# Arrow Integration

Zero-copy conversions between R vectors and Apache Arrow arrays.

## Quick Reference

```rust
use miniextendr_api::{miniextendr, ffi::SEXP};
use miniextendr_api::optionals::arrow_impl::*;

// R numeric → Arrow Float64Array → back to R: zero-copy both directions
#[miniextendr]
pub fn passthrough_numeric(x: Float64Array) -> Float64Array {
    x
}

// R integer → Arrow Int32Array → back to R: zero-copy both directions
#[miniextendr]
pub fn passthrough_integer(x: Int32Array) -> Int32Array {
    x
}

// Compute on Arrow, return to R (copies on return — new data)
#[miniextendr]
pub fn doubled(x: Float64Array) -> Float64Array {
    x.iter().map(|v| v.map(|f| f * 2.0)).collect()
}

// RecordBatch round-trip: primitive columns zero-copy per-column
#[miniextendr]
pub fn passthrough_df(df: RecordBatch) -> RecordBatch {
    df
}
```

## Zero-Copy String Vectors

R stores strings as STRSXP (array of CHARSXP pointers). Each CHARSXP is interned,
GC-managed, and has a known `LENGTH`. Instead of copying into `String`, borrow directly.

### `Cow<'static, str>` — scalar

```rust
#[miniextendr]
pub fn greet(name: Cow<'static, str>) -> String {
    // name is Cow::Borrowed — points directly into R's CHARSXP data
    // No allocation unless you call .to_mut()
    format!("Hello, {}!", name)
}
```

### `Vec<Cow<'static, str>>` — vector, zero-copy per element

```rust
#[miniextendr]
pub fn upper_first(words: Vec<Cow<'static, str>>) -> Vec<String> {
    // Each element is Cow::Borrowed (zero-copy from R's CHARSXP pool)
    // Only UTF-8 strings borrow; non-UTF-8 gets translated (rare, auto)
    words.iter().map(|w| {
        let mut s = w.to_string();
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        s
    }).collect()
}

// NA-aware variant: None for NA_character_
#[miniextendr]
pub fn count_non_na(words: Vec<Option<Cow<'static, str>>>) -> i32 {
    words.iter().filter(|w| w.is_some()).count() as i32
}
```

### `Cow<'static, [T]>` — numeric slices

```rust
#[miniextendr]
pub fn sum_cow(x: Cow<'static, [f64]>) -> f64 {
    // Cow::Borrowed — x points directly into R's REALSXP data
    x.iter().sum()
}

// Round-trip: if x was borrowed from R, IntoR returns the original SEXP
#[miniextendr]
pub fn passthrough_cow(x: Cow<'static, [i32]>) -> Cow<'static, [i32]> {
    x  // zero-copy: SEXP pointer recovery finds the original R vector
}
```

### `ProtectedStrVec` — GC-safe string view with proper lifetimes

```rust
use miniextendr_api::ProtectedStrVec;
use std::collections::HashSet;

#[miniextendr]
pub fn count_unique(strings: ProtectedStrVec) -> i32 {
    // Lifetimes tied to &self (not 'static) — compile-time GC safety
    let unique: HashSet<&str> = strings.iter()
        .filter_map(|s| s)  // skip NA
        .collect();
    unique.len() as i32
}

#[miniextendr]
pub fn first_non_na(strings: ProtectedStrVec) -> &str {
    // iter_cow() handles non-UTF-8 CHARSXPs gracefully
    strings.iter_cow()
        .find_map(|s| s)
        .map(|cow| cow.as_ref())
        .unwrap_or("")
}
```

### `StrVec` — lightweight STRSXP wrapper (Copy, no protection)

```rust
use miniextendr_api::StrVec;

#[miniextendr]
pub fn has_empty(strings: StrVec) -> bool {
    // StrVec is Copy — just a SEXP wrapper. Caller must ensure GC protection.
    // Safe in .Call context (R protects arguments).
    strings.iter().any(|opt| opt == Some(""))
}
```

## Arrow Arrays

### R → Arrow (already zero-copy for primitives)

```rust
use miniextendr_api::optionals::arrow_impl::*;

#[miniextendr]
pub fn arrow_mean(x: Float64Array) -> f64 {
    // x.values() points directly into R's REALSXP data (zero-copy)
    // NA values are tracked in Arrow's null bitmap, not in the data
    let sum: f64 = x.iter().flatten().sum();
    let count = x.len() - x.null_count();
    sum / count as f64
}

#[miniextendr]
pub fn arrow_filter_positive(x: Int32Array) -> Int32Array {
    // Arrow compute — result is a new array (Rust-allocated)
    x.iter()
        .map(|v| v.filter(|&i| i > 0))
        .collect()
}
```

### Arrow → R (automatic SEXP recovery)

When an Arrow array's data buffer came from R (via `sexp_to_arrow_buffer`),
`IntoR` automatically recovers the original SEXP using pointer arithmetic.
No wrapper types needed.

```rust
// This is zero-copy BOTH directions:
#[miniextendr]
pub fn identity(x: Float64Array) -> Float64Array {
    x  // R→Arrow (zero-copy) → Arrow→R (pointer recovery, zero-copy)
}

// This copies on return (new data, not from R):
#[miniextendr]
pub fn squares(x: Float64Array) -> Float64Array {
    x.iter().map(|v| v.map(|f| f * f)).collect()
}
```

### RecordBatch (data.frame)

```rust
use arrow_array::cast::AsArray;

#[miniextendr]
pub fn df_add_column(df: RecordBatch) -> RecordBatch {
    let col0: &Float64Array = df.column(0).as_primitive();

    // Compute new column
    let new_col: Float64Array = col0.iter()
        .map(|v| v.map(|f| f * 2.0))
        .collect();

    // Build new batch — original columns return to R zero-copy,
    // new column copies (it's Rust-allocated)
    let mut fields = df.schema().fields().to_vec();
    fields.push(Arc::new(Field::new("doubled", DataType::Float64, true)));
    let schema = Arc::new(Schema::new(fields));

    let mut columns = df.columns().to_vec();
    columns.push(Arc::new(new_col));

    RecordBatch::try_new(schema, columns).unwrap()
}
```

### `alloc_r_backed_buffer` — Rust→Arrow→R zero-copy

Allocate an Arrow buffer backed by R memory from the start. When the array
is later converted to R, pointer recovery finds the original SEXP.

```rust
use miniextendr_api::optionals::arrow_impl::alloc_r_backed_buffer;

#[miniextendr]
pub fn generate_sequence(n: i32) -> Float64Array {
    let n = n as usize;
    // Allocate buffer as R REALSXP
    let (buffer, _sexp) = unsafe { alloc_r_backed_buffer::<f64>(n) };
    let mut values = arrow_buffer::ScalarBuffer::<f64>::from(buffer);

    // Fill via Arrow APIs
    // ... (would need unsafe mutable access to the buffer)

    Float64Array::new(values, None)
    // IntoR → pointer recovery → returns the REALSXP (zero-copy)
}
```

### `RStringArray` — string round-trip tracking

Arrow's StringArray and R's STRSXP have incompatible layouts (contiguous data+offsets
vs per-element CHARSXPs). Automatic pointer recovery can't work for strings.
`RStringArray` explicitly tracks the source STRSXP.

```rust
use miniextendr_api::optionals::arrow_impl::RStringArray;

#[miniextendr]
pub fn string_passthrough(x: RStringArray) -> RStringArray {
    // x.source is Some(strsxp) — IntoR returns original STRSXP
    x
}

#[miniextendr]
pub fn string_lengths(x: RStringArray) -> Vec<i32> {
    // Deref to StringArray — all Arrow APIs work
    x.iter().map(|opt| opt.map(|s| s.len() as i32).unwrap_or(-1)).collect()
}
```

### ALTREP for Cow string vectors

`Vec<Cow<'static, str>>` supports ALTREP with seamless serialization:

```rust
use miniextendr_api::IntoRAltrep;
use std::borrow::Cow;

#[miniextendr]
pub fn lazy_strings(prefix: &str, n: i32) -> SEXP {
    let strings: Vec<Cow<'static, str>> = (0..n)
        .map(|i| Cow::Owned(format!("{}_{}", prefix, i)))
        .collect();
    strings.into_sexp_altrep()
    // R sees a character vector; elements computed on demand via ALTREP Elt
    // saveRDS/readRDS works — serializes to STRSXP, deserializes back to Vec<Cow>
}
```

## How It Works

### SEXP Pointer Recovery (`r_memory` module)

R stores vector data at a fixed offset from the SEXP header:

```
[VECTOR_SEXPREC header (48 bytes on 64-bit)] [data...]
 ^                                            ^
 SEXP                                         DATAPTR_RO(sexp)
```

At package init, we measure this offset. Then:

```
candidate_sexp = data_ptr - offset
verify: TYPEOF(candidate) == expected AND LENGTH(candidate) == expected AND DATAPTR_RO(candidate) == data_ptr
```

This works because:
- All R vector types use `VECTOR_SEXPREC` (same header size)
- The offset is constant within an R session
- ALTREP vectors fail the DATAPTR_RO check (data isn't at fixed offset)
- Rust-allocated buffers fail the type/length checks

### Encoding-safe string conversion (`charsxp_to_cow`)

`charsxp_to_cow()` tries `from_utf8` on R's `R_CHAR` data (O(1) for validation).
If the CHARSXP is valid UTF-8 (the common case in UTF-8 locales), returns
`Cow::Borrowed` — zero-copy. For non-UTF-8 (CE_LATIN1, CE_BYTES), falls back to
`Rf_translateCharUTF8` which copies. The caller doesn't need to think about encodings.

## Type Decision Tree

```
Need strings from R?
├── Scalar → Cow<'static, str>          (zero-copy, encoding-safe)
├── Vector, need ownership → Vec<String> (copies, lossy NA→"")
├── Vector, read-only → Vec<Cow<'static, str>>  (zero-copy per element)
├── Vector, NA-aware → Vec<Option<Cow<'static, str>>>
├── View with GC safety → ProtectedStrVec
└── Lightweight view → StrVec           (Copy, caller manages GC)

Need numerics from R?
├── As Rust slice → &[f64] / &[i32]    (zero-copy, 'static lifetime)
├── Copy-on-write → Cow<'static, [f64]> (zero-copy, copies on .to_mut())
├── As Arrow array → Float64Array       (zero-copy both directions)
└── Owned copy → Vec<f64>              (copies)

Need data frames?
├── As Arrow → RecordBatch             (primitive cols zero-copy both ways)
└── As Arrow (string cols too) → use RStringArray per column
```
