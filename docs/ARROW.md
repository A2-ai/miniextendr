# Arrow Integration

Zero-copy conversions between R vectors and Apache Arrow arrays.

## Quick Reference

```rust
use miniextendr_api::{miniextendr, SEXP};
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

// Compute on Arrow, return to R (copies on return - new data)
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

### `Cow<'_, str>`: scalar

```rust
#[miniextendr]
pub fn greet(name: Cow<'_, str>) -> String {
    // name is Cow::Borrowed - points directly into R's CHARSXP data
    // No allocation unless you call .to_mut()
    format!("Hello, {}!", name)
}
```

### `Vec<Cow<'_, str>>`: vector, zero-copy per element

```rust
#[miniextendr]
pub fn upper_first(words: Vec<Cow<'_, str>>) -> Vec<String> {
    // Each element is Cow::Borrowed (zero-copy from R's CHARSXP pool)
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
pub fn count_non_na(words: Vec<Option<Cow<'_, str>>>) -> i32 {
    words.iter().filter(|w| w.is_some()).count() as i32
}
```

### `Cow<'_, [T]>`: numeric slices

```rust
#[miniextendr]
pub fn sum_cow(x: Cow<'_, [f64]>) -> f64 {
    // Cow::Borrowed - x points directly into R's REALSXP data
    x.iter().sum()
}

// Round-trip: input is zero-copy (Cow::Borrowed into R's INTSXP), but the
// IntoR direction copies into a fresh R vector. Unlike Arrow buffers, a bare
// &[T] carries no provenance to prove it points at an R vector start, so a
// borrowed sub-slice can't be told apart from a full borrow — the copy is the
// only sound choice (see #880).
#[miniextendr]
pub fn passthrough_cow(x: Cow<'_, [i32]>) -> Cow<'_, [i32]> {
    x  // zero-copy in; copy back out
}
```

### `RCow<'_, T>`: safe zero-copy round-trip

When you want the round-trip back to R to *also* be zero-copy, use `RCow`
instead of `Cow`. `RCow`'s borrowed arm remembers the source SEXP it was read
from, so `IntoR` returns that exact R object: no copy. A plain `Cow<[T]>`
has nowhere to keep the source, so it always copies (#880):

```rust
use miniextendr_api::RCow;

// Zero-copy BOTH directions: returns the original R vector unchanged.
#[miniextendr]
pub fn passthrough_rcow(x: RCow<'_, f64>) -> RCow<'_, f64> {
    x
}

// Mutation triggers copy-on-write, then materializes a fresh R vector.
#[miniextendr]
pub fn doubled_rcow(mut x: RCow<'_, f64>) -> RCow<'_, f64> {
    for v in x.to_mut() {
        *v *= 2.0;
    }
    x
}
```

A borrowed `RCow` can only be produced by reading an R vector (its fields are
private), so — unlike `Cow::Borrowed(&slice[2..5])` — it can never be a
sub-slice. That structural invariant is what makes returning the stored SEXP
sound. Slice it via `Deref` to get a plain `&[T]`; use `to_mut()` /
`into_owned()` when you need to mutate or keep the data past the call.

### `ProtectedStrVec` vs `StrVec`

`ProtectedStrVec` and `StrVec` both wrap an R STRSXP and provide zero-copy
`&str` access to its elements. They differ in GC safety:

| | `StrVec` | `ProtectedStrVec` |
|---|---|---|
| Storage | SEXP and lifetime marker | SEXP, length and owned-root state |
| Copy | `Copy` | `!Copy` |
| GC protection | Borrows a root (call argument, `ProtectScope`, owner) | Preserved until drop |
| Borrow lifetime | The view's `'a` | Borrow of `&self` |
| Iterator | `StrVecIter<'a>` | `ProtectedStrVecIter<'a>` |

`ProtectedStrVec` keeps its source alive across `.Call` boundaries and supports
arbitrary drop order. Its accessors tie strings and views to `&self`:

```rust
let dangling: &str;
{
    let sv = unsafe { ProtectedStrVec::new(sexp) };
    dangling = sv.get_str(0).unwrap();
}
// Using dangling here fails to compile: sv does not live long enough.
```

`StrVec<'a>` is a lightweight view for an already-rooted character vector.
`owner.as_strvec()` binds its lifetime to a `ProtectedStrVec` owner. The raw
`TryFromSexp` conversion is unavailable for `StrVec` and `RCow`: use
`try_from_root(root)` with a `ProtectScope` root. Generated wrappers bind their
view to the live argument slot. `TryFromRoot` also supplies zero-copy `Cow`
inputs, including `Vec<Cow<'_, str>>` and `Vec<Option<Cow<'_, str>>>`.
Raw `TryFromSexp` for those Cow types returns owned data, so it may safely
outlive the source. Main-thread wrappers bind shared `&[T]` and `&str`
parameters to the argument slot the same way; mutable references and
reference containers remain #1427.

**Usage examples:**

```rust
use miniextendr_api::ProtectedStrVec;
use std::collections::HashSet;

#[miniextendr]
pub fn count_unique(strings: ProtectedStrVec) -> i32 {
    // Lifetimes tied to &self - compiler enforces GC safety
    let unique: HashSet<&str> = strings.iter()
        .filter_map(|s| s)  // skip NA
        .collect();
    unique.len() as i32
}

// Can't return &str - ProtectedStrVec is consumed by IntoR, so there's
// nothing to borrow from. Return String or the whole ProtectedStrVec.
#[miniextendr]
pub fn first_non_na(strings: ProtectedStrVec) -> String {
    strings.iter()
        .find_map(|s| s)
        .unwrap_or("")
        .to_owned()
}
```

```rust
use miniextendr_api::StrVec;

#[miniextendr]
pub fn has_empty(strings: StrVec<'_>) -> bool {
    // The wrapper binds the view to the call's argument, so neither it nor
    // a &str read from it can outlive the call.
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
    // Arrow compute - result is a new array (Rust-allocated)
    x.iter()
        .map(|v| v.filter(|&i| i > 0))
        .collect()
}
```

### Arrow → R (automatic SEXP recovery)

When an Arrow array's data buffer came from R (via `sexp_to_arrow_buffer`),
`IntoR` returns the original SEXP instead of copying. The buffer was recorded
in a registry when it was made, so this is a lookup, not a guess (see
[Buffer registry](#buffer-registry-zero-copy-recovery) below). No wrapper types
needed.

```rust
// This is zero-copy BOTH directions:
#[miniextendr]
pub fn identity(x: Float64Array) -> Float64Array {
    x  // R→Arrow (zero-copy) → Arrow→R (registry hit, zero-copy)
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

    // Build new batch - original columns return to R zero-copy,
    // new column copies (it's Rust-allocated)
    let mut fields = df.schema().fields().to_vec();
    fields.push(Arc::new(Field::new("doubled", DataType::Float64, true)));
    let schema = Arc::new(Schema::new(fields));

    let mut columns = df.columns().to_vec();
    columns.push(Arc::new(new_col));

    RecordBatch::try_new(schema, columns).unwrap()
}
```

### `alloc_r_backed_buffer`: Rust→Arrow→R zero-copy

Allocate an Arrow buffer backed by R memory from the start. Write through
the raw SEXP pointer, then wrap in Arrow types. When the array is later
converted to R, the buffer registry finds the original SEXP.

```rust
use miniextendr_api::optionals::arrow_impl::alloc_r_backed_buffer;

#[miniextendr]
pub fn generate_sequence(n: i32) -> SEXP {
    use miniextendr_api::IntoR;
    let n = n as usize;

    // Allocate buffer as R REALSXP - data lives in R's heap
    let (buffer, sexp) = unsafe { alloc_r_backed_buffer::<f64>(n) };

    // Fill through the SEXP's raw pointer (before wrapping in Arrow)
    unsafe {
        let ptr = miniextendr_api::sys::REAL(sexp);
        for i in 0..n {
            *ptr.add(i) = i as f64;
        }
    }

    // Wrap as Arrow array
    let values = arrow_buffer::ScalarBuffer::<f64>::from(buffer);
    let array = Float64Array::new(values, None);

    // IntoR → registry hit → returns the same REALSXP (zero-copy)
    array.into_sexp()
}
```

### `RStringArray`: string round-trip tracking

Arrow's StringArray and R's STRSXP have incompatible layouts (contiguous data+offsets
vs per-element CHARSXPs). Automatic recovery can't work for strings.
`RStringArray` explicitly tracks the source STRSXP.

```rust
use miniextendr_api::optionals::arrow_impl::RStringArray;

#[miniextendr]
pub fn string_passthrough(x: RStringArray) -> RStringArray {
    // x.source is Some(strsxp) - IntoR returns original STRSXP
    x
}

#[miniextendr]
pub fn string_lengths(x: RStringArray) -> Vec<i32> {
    // Deref to StringArray - all Arrow APIs work
    x.iter().map(|opt| opt.map(|s| s.len() as i32).unwrap_or(-1)).collect()
}
```

### ALTREP for Cow string vectors

`Vec<Cow<'static, str>>` supports ALTREP with serialization. Borrowed input tied to
one call must become owned before it can back a persistent ALTREP object;
static Rust strings can remain borrowed:

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
    // saveRDS/readRDS works - serializes to STRSXP, deserializes back to Vec<Cow>
}
```

## How It Works

### Buffer registry (zero-copy recovery)

`sexp_to_arrow_buffer` wraps an R vector's data in an Arrow buffer without
copying. It preserves the vector (`R_PreserveObject`) for as long as the
buffer lives, and, for a standard (non-ALTREP) vector, records the buffer in
a registry: data address and byte length → the SEXP. The buffer's guard
removes the entry when it drops.

`IntoR` for `Float64Array`, `Int32Array` and `UInt8Array` hands back the source
vector only on an exact registry hit, pointer and byte length. Anything else is
copied into a fresh R vector:

- buffers Arrow or DataFusion allocated, such as the `vec![v; 1]` of a
  one-row aggregate column;
- a slice of an R-backed buffer, such as DataFusion's contiguous-run filter
  output, which starts or ends elsewhere (#867);
- a buffer over an ALTREP vector, such as `1:n`. An ALTREP vector's data can
  be another object's memory (the vector R's wrapper class wraps, or the Rust
  buffer behind an Arrow ALTREP vector), so two live vectors can share one
  data address. A standard vector's data lies inside its own allocation, which
  is what lets the registry key name exactly one SEXP;
- an R-backed buffer under a null mask whose null slots do not already hold
  R's NA (`0` for raw). Arrow kernels can keep the values buffer and swap only
  the mask: arrow-select's `nullif` does, so a `NULLIF` result is still an
  exact registry hit. Handing back the source vector would lose those nulls.
  An array read from R builds its nulls from R's NAs, so it passes this check
  and stays zero-copy.

Earlier versions guessed instead. They read the bytes in front of the data
pointer as an R vector header and accepted them if the type, ALTREP bit and
length matched, behind a check that the buffer was unsliced with capacity
exactly its length. That check assumed buffers Arrow allocates itself are
rounded up to 64 bytes, which a `Vec`-backed buffer is not. So a one-row
aggregate's column went through the header read on every conversion, and
heap bytes that happened to look like a header made the data.frame point into
freed Rust memory. The registry reads no memory the buffer does not own.

### String conversion (`charsxp_to_str`)

`charsxp_to_str()` reads `R_CHAR` + `LENGTH` without copying for UTF-8 and
ASCII strings and for native strings in a UTF-8 session, then validates the
bytes in O(n) time. Latin-1 strings, and native non-ASCII strings in a session
whose locale isn't UTF-8, are translated with `Rf_translateCharUTF8`, and
`bytes` strings are rejected. `charsxp_to_cow()`
returns `Cow::Borrowed` for the fast path and `Cow::Owned` when translation is
required.

## Type Decision Tree

```text
Need strings from R?
├── Scalar → Cow<'_, str>                     (zero-copy for UTF-8/ASCII)
├── Vector, need ownership → Vec<String>      (copies, lossy NA→"")
├── Vector, read-only → Vec<Cow<'_, str>>     (zero-copy per UTF-8/ASCII element)
├── Vector, NA-aware → Vec<Option<Cow<'_, str>>>
├── View with GC safety → ProtectedStrVec
└── Lightweight view → StrVec<'_>             (Copy, borrows the call's argument)

Need numerics from R?
├── As Rust slice → &[f64] / &[i32]           (zero-copy, borrows the call's argument)
├── Copy-on-write → Cow<'_, [f64]>            (zero-copy, copies on .to_mut())
├── As Arrow array → Float64Array             (zero-copy both directions)
└── Owned copy → Vec<f64>                     (copies)

Need data frames?
├── As Arrow → RecordBatch                    (primitive cols zero-copy both ways)
└── As Arrow (string cols too) → use RStringArray per column
```

The Cow and slice parameters above borrow through `TryFromRoot`; their
lifetimes end with the call. A raw `TryFromSexp` conversion returns owned Cow
data instead, and a raw slice conversion still requires the source to remain
reachable (#1427).
