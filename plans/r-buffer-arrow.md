# R-Buffer-Allocated Arrow Objects

## Problem

Arrow→R always copies, even when the Arrow buffer IS R memory.

Currently: `Float64Array.into_sexp()` allocates a new REALSXP and `copy_from_slice`s into it. But if that Float64Array was created from an R REALSXP via `sexp_to_arrow_buffer` (which uses `Buffer::from_custom_allocation(RPreservedSexp)`), the data is already in R's heap — we're copying from R to R.

## Goal

Round-trip R→Arrow→R should be zero-copy for types where the memory layouts are compatible (primitives: f64, i32, u8). The Arrow array is a view over R's SEXP data, and converting back to R returns the original SEXP.

## Architecture

### Buffer provenance tracking

When `sexp_to_arrow_buffer` creates an Arrow Buffer from an R SEXP, register the mapping in a global registry:

```rust
/// Maps Arrow buffer data pointers back to their source R SEXPs.
static R_BUFFER_REGISTRY: Mutex<HashMap<*const u8, SEXP>> = ...;

unsafe fn sexp_to_arrow_buffer<T: RNativeType>(sexp: SEXP) -> Buffer {
    // ... existing code: R_PreserveObject, from_custom_allocation ...
    let ptr = ffi::DATAPTR_RO(sexp) as *const u8;
    R_BUFFER_REGISTRY.lock().insert(ptr, sexp);
    // ...
}
```

On `RPreservedSexp::drop`, remove the entry:
```rust
impl Drop for RPreservedSexp {
    fn drop(&mut self) {
        let ptr = unsafe { ffi::DATAPTR_RO(self.0) } as *const u8;
        R_BUFFER_REGISTRY.lock().remove(&ptr);
        unsafe { ffi::R_ReleaseObject_unchecked(self.0) }
    }
}
```

### Zero-copy Arrow→R for R-backed buffers

In `IntoR for Float64Array` (and Int32Array, UInt8Array):

```rust
impl IntoR for Float64Array {
    fn into_sexp(self) -> SEXP {
        // Check if this array's values buffer came from R
        if self.null_count() == 0 {
            let ptr = self.values().as_ptr() as *const u8;
            if let Some(sexp) = R_BUFFER_REGISTRY.lock().get(&ptr) {
                // Zero-copy: return the original R SEXP
                return *sexp;
            }
        }
        // Fallback: allocate new R vector and copy (current behavior)
        // ...
    }
}
```

The `null_count() == 0` check is required because R uses sentinel NAs (NA_integer_, NA_real_) while Arrow uses a separate null bitmap. If the Arrow code added nulls that weren't NA sentinels, we can't return the original SEXP.

### R-allocated Arrow buffers (Rust→Arrow→R)

For Arrow arrays created in Rust that will go to R: allocate the buffer as an R vector from the start.

```rust
/// Allocate an Arrow Buffer backed by a new R vector.
/// When this array is later converted to R, the SEXP is returned directly.
pub fn alloc_r_backed_buffer<T: RNativeType>(len: usize) -> (Buffer, SEXP) {
    unsafe {
        let (sexp, _) = alloc_r_vector::<T>(len);
        let buffer = sexp_to_arrow_buffer::<T>(sexp);
        (buffer, sexp)
    }
}
```

Usage:
```rust
fn compute_in_rust(n: usize) -> Float64Array {
    let (buffer, _sexp) = alloc_r_backed_buffer::<f64>(n);
    let mut slice: &mut [f64] = unsafe { /* mutable view of buffer */ };
    for i in 0..n {
        slice[i] = (i as f64).sqrt();
    }
    Float64Array::new(ScalarBuffer::from(buffer), None)
}
// When this array is returned to R, IntoR detects the R-backed buffer → zero-copy
```

### Null bitmap considerations

Arrow's null bitmap is separate from the data buffer. When converting Arrow→R:
- **No nulls**: Return original SEXP directly (data unchanged)
- **Has nulls**: Must write NA sentinels into the data, which means we need a mutable copy. For R-backed buffers, we could mutate in-place if we have exclusive ownership (Arc refcount == 1). Otherwise, copy.

The Arrow `try_new` methods accept a `NullBuffer` parameter. For R-backed arrays, the null bitmap should be constructed from R's NA sentinels (already done in `build_i32_null_buffer` / `build_f64_null_buffer`). The data buffer retains the NA sentinels, so Arrow→R can ignore the bitmap if the source was R.

Track this with a flag: did nulls come from R sentinels or from Arrow operations?

```rust
struct RBackedMeta {
    sexp: SEXP,
    nulls_from_sentinels: bool,  // true = R NAs, safe to return SEXP as-is
}
```

### Strings

R's STRSXP (array of CHARSXP pointers) and Arrow's StringArray (contiguous data + offsets) are fundamentally different layouts. Zero-copy is impossible in either direction.

However, for the round-trip case (R→StringArray→R): if we track the source STRSXP, we can return it on the way back. The Arrow StringArray is a read-only view; if no mutations occurred, the original STRSXP is still valid.

For Rust→Arrow→R strings: no shortcut. Must build STRSXP element by element (current behavior).

### What `try_new` enables

Arrow's `PrimitiveArray::try_new(values: ScalarBuffer<T>, nulls: Option<NullBuffer>)` accepts pre-built buffers. This is the key API: we construct `ScalarBuffer` from `Buffer::from_custom_allocation` (R memory), pass it to `try_new`, and get an Arrow array backed by R.

Similarly `GenericByteArray::try_new` for strings, `StructArray::try_new` for data frames, etc. The pattern is always: build Buffers from R memory, pass to `try_new`.

## Implementation order

1. **Buffer provenance registry** — global `HashMap<*const u8, SEXP>`, register in `sexp_to_arrow_buffer`, deregister in `RPreservedSexp::drop`
2. **Zero-copy `IntoR` for R-backed primitives** — check registry in Float64Array, Int32Array, UInt8Array `into_sexp()`
3. **`alloc_r_backed_buffer`** — allocate Arrow buffers as R SEXPs for the Rust→Arrow→R path
4. **Null bitmap handling** — track whether nulls came from R sentinels; write NA sentinels back when needed
5. **String round-trip** — track source STRSXP for StringArray, return it on IntoR if unmodified
6. **RecordBatch** — apply column-level zero-copy to data frame round-trips

## What this does NOT cover

- Arrow compute kernels operating on R-backed buffers — these create new Arrow-allocated buffers, breaking the R provenance. The result would go through the copy path on IntoR.
- ALTREP output for Arrow arrays — already implemented separately.
- Arrow string layouts (contiguous data + offsets) in R — would require a custom R representation, not standard STRSXP.
