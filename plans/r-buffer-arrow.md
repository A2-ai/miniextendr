# R-Buffer-Allocated Arrow Objects

## Problem

Arrow→R always copies, even when the Arrow buffer IS R memory.

`Float64Array.into_sexp()` allocates a new REALSXP and `copy_from_slice`s. But if that Float64Array was created from R via `sexp_to_arrow_buffer` (which uses `Buffer::from_custom_allocation(RPreservedSexp)`), the data is already in R's heap.

## Design: Trait + Generic Newtype

### The `RSourced` trait

```rust
/// Trait for Arrow types that may be backed by R memory.
pub trait RSourced {
    /// The original R SEXP if this value is zero-copy from R.
    fn r_source(&self) -> Option<SEXP>;

    /// Whether nulls came from R sentinels (safe to return SEXP as-is).
    /// False means Arrow operations added/changed nulls.
    fn nulls_from_sentinels(&self) -> bool;
}
```

### Generic newtype: `RPrimitive<T>`

One type covers all primitive Arrow arrays (f64, i32, u8, etc.):

```rust
pub struct RPrimitive<T: ArrowPrimitiveType> {
    array: PrimitiveArray<T>,
    source: Option<SEXP>,
    sentinel_nulls: bool,
}

impl<T: ArrowPrimitiveType> RSourced for RPrimitive<T> {
    fn r_source(&self) -> Option<SEXP> { self.source }
    fn nulls_from_sentinels(&self) -> bool { self.sentinel_nulls }
}

impl<T: ArrowPrimitiveType> Deref for RPrimitive<T> {
    type Target = PrimitiveArray<T>;
    fn deref(&self) -> &PrimitiveArray<T> { &self.array }
}

impl<T: ArrowPrimitiveType> AsRef<PrimitiveArray<T>> for RPrimitive<T> { ... }
impl<T: ArrowPrimitiveType> AsRef<dyn Array> for RPrimitive<T> { ... }
```

### How conversion works

**TryFromSexp** (R→Arrow, zero-copy):
```rust
impl TryFromSexp for RPrimitive<Float64Type> {
    fn try_from_sexp(sexp: SEXP) -> Result<Self, _> {
        let array = Float64Array::try_from_sexp(sexp)?;  // existing zero-copy path
        Ok(RPrimitive {
            array,
            source: Some(sexp),
            sentinel_nulls: true,  // nulls came from R's NA sentinels
        })
    }
}
```

**IntoR** (Arrow→R, zero-copy when R-backed):
```rust
impl IntoR for RPrimitive<Float64Type> {
    fn into_sexp(self) -> SEXP {
        if self.sentinel_nulls {
            if let Some(sexp) = self.source {
                return sexp;  // zero-copy: return original R vector
            }
        }
        // Fallback: copy (current behavior)
        self.array.into_sexp()
    }
}
```

### String wrapper: `RStringArray`

R's STRSXP and Arrow's StringArray have incompatible layouts (per-element CHARSXPs vs contiguous data+offsets). Zero-copy is impossible for the data, but round-trip tracking avoids rebuilding:

```rust
pub struct RStringArray {
    array: StringArray,
    source: Option<SEXP>,  // original STRSXP
}

impl RSourced for RStringArray { ... }
impl Deref for RStringArray { type Target = StringArray; ... }
```

IntoR: if source is Some and no mutations occurred, return the original STRSXP.

### RecordBatch wrapper: `RRecordBatch`

```rust
pub struct RRecordBatch {
    batch: RecordBatch,
    column_sources: Vec<Option<SEXP>>,  // per-column R vectors
    sentinel_nulls: Vec<bool>,
}

impl Deref for RRecordBatch { type Target = RecordBatch; ... }
```

Data frame round-trips: each column tracks its R source independently.

### Serialization

No special handling needed. Serialization goes through the inner Arrow type (which handles it). On deserialize, `source` is `None` — falls through to copy path. Clean.

### Arrow compute on R-backed arrays

When Arrow compute kernels operate on `RPrimitive<T>`, they access the inner array via Deref. The result is a plain Arrow array (no R provenance). Wrapping the result:

```rust
impl<T: ArrowPrimitiveType> RPrimitive<T> {
    /// Wrap a computed Arrow array (no R source).
    pub fn from_arrow(array: PrimitiveArray<T>) -> Self {
        RPrimitive { array, source: None, sentinel_nulls: false }
    }

    /// Wrap with known R source.
    pub fn from_r(array: PrimitiveArray<T>, sexp: SEXP) -> Self {
        RPrimitive { array, source: Some(sexp), sentinel_nulls: true }
    }
}
```

### Usage in #[miniextendr] functions

```rust
#[miniextendr]
pub fn double_values(x: RPrimitive<Float64Type>) -> RPrimitive<Float64Type> {
    // x derefs to &Float64Array, so Arrow APIs work transparently
    let result: Float64Array = x.iter()
        .map(|v| v.map(|f| f * 2.0))
        .collect();
    RPrimitive::from_arrow(result)  // no R source → will copy on IntoR
}

#[miniextendr]
pub fn passthrough(x: RPrimitive<Float64Type>) -> RPrimitive<Float64Type> {
    x  // R source preserved → zero-copy round-trip!
}
```

## Implementation order

1. `RSourced` trait + `RPrimitive<T>` with Deref/AsRef
2. `TryFromSexp` for `RPrimitive<Float64Type>`, `RPrimitive<Int32Type>`, `RPrimitive<UInt8Type>`
3. `IntoR` with zero-copy fast path
4. `RStringArray` wrapper
5. `RRecordBatch` wrapper
6. Proc-macro support (ensure #[miniextendr] handles RPrimitive<T> in function signatures)
