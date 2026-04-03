# Zero-Copy Inherent Limits

Things that can't be fixed without breaking changes or upstream work. Each section describes the limit, why it exists, and what a fix would look like.

## 1. Provenance UB on SEXP type-tag read

`try_recover_r_sexp` reads 4 bytes from `candidate.0 as *const u32` where the pointer was derived via `wrapping_byte_sub` from an Arrow buffer pointer. Under Rust's strict provenance model, the derived pointer has provenance over the Arrow allocation, not the SEXP header before it. The read is UB in Miri; harmless on real hardware.

**Fix**: A one-line C helper that takes a raw pointer, casts to SEXP, and calls `TYPEOF()`. C has no provenance model — the cast is legal. Requires either:
- A `cc` build dependency on miniextendr-api (currently has none)
- An inline assembly escape hatch (nightly-only)
- Waiting for Rust's strict provenance RFC to stabilize `ptr::with_exposed_provenance`

**Workaround if needed now**: Add a `c_helpers.c` to miniextendr-api with `int mx_try_typeof(void* ptr) { return TYPEOF((SEXP)ptr); }` and call it from Rust via `extern "C"`.

## 2. TryFromSexp has no lifetime parameter

`TryFromSexp` returns `Self` with no input lifetime. Borrowed types must use `'static`:

```rust
impl TryFromSexp for &'static str { ... }
impl TryFromSexp for Cow<'static, [T]> { ... }
```

The `'static` is a lie — the data is only valid during the `.Call` invocation. `ProtectedStrVec` works around this for strings (ties borrows to `&self`), but there's no general solution.

**Fix**: GAT-based trait with a lifetime parameter:

```rust
trait TryFromSexp<'a> {
    type Output;
    fn try_from_sexp(sexp: SEXP) -> Result<Self::Output, Error>;
}

impl<'a> TryFromSexp<'a> for &'a str { ... }
impl<'a, T: RNativeType> TryFromSexp<'a> for &'a [T] { ... }
impl<'a, T: RNativeType> TryFromSexp<'a> for Cow<'a, [T]> { ... }
```

This is a breaking change to the trait signature. The proc-macro would need to thread the lifetime through generated wrappers. Every downstream `TryFromSexp` impl would need updating.

**Cost**: Major. Touches every conversion, every macro expansion, every user-facing type. Worth doing if/when there's a 0.2 release with other breaking changes.

## 3. String layout incompatibility (STRSXP vs Arrow StringArray)

R's STRSXP: array of CHARSXP pointers, each pointing to an interned string with its own length. Strings are not contiguous in memory.

Arrow's StringArray: one contiguous `u8` data buffer + an `i32` offsets buffer. Strings are packed end-to-end.

Zero-copy between these layouts is impossible — the data must be rearranged. `RStringArray` tracks the source STRSXP for round-trip optimization (return original if unmodified), but the R→Arrow direction always copies.

**Fix options**:
- **ALTREP STRSXP backed by Arrow StringArray**: R sees an STRSXP, but `Elt(i)` constructs a CHARSXP on demand from Arrow's contiguous buffer. The Arrow data stays in Arrow format; R materializes only the elements it accesses. Already partially possible with the existing ALTREP string infrastructure.
- **Arrow StringView over STRSXP**: Arrow 55+ has `StringViewArray` (inline small strings + pointer to large). Could store CHARSXP pointers as the "large string" pointers. Requires Arrow to accept non-contiguous backing buffers, which it doesn't currently support.
- **Neither is truly zero-copy** — one direction always pays. The question is which direction pays and when (eager vs lazy).

## 4. Arrow buffer slicing breaks recovery

`array.slice(offset, len)` shifts the data pointer by `offset * sizeof(T)`. The shifted pointer no longer points to `SEXP + sizeof(SEXPREC_ALIGN)`, so recovery fails and IntoR copies.

**Fix**: Track the original SEXP + offset in a side-channel. Options:
- **Wrapper type** (like `RPrimitive`) that carries `(SEXP, offset)` — but this doesn't compose with Arrow's `ArrayRef` ecosystem.
- **Arrow metadata**: Store the SEXP address in the array's `Buffer` metadata. Arrow doesn't expose custom metadata on buffers (only on schemas/fields).
- **Registration map**: `HashMap<*const u8, (SEXP, usize)>` mapping any data pointer to its source SEXP + byte offset within the R vector. Recovery computes `data_ptr - offset_in_sexp` to get the R vector start, then verifies. Adds global state.
- **Accept the limitation**: Sliced arrays are typically intermediate results in compute pipelines. The final result is usually a new array (Rust-allocated), which would copy anyway. The round-trip optimization matters most for pass-through and filter operations, not slicing.

**Recommendation**: Accept the limitation. Document that `slice()` breaks zero-copy IntoR. If a specific use case needs sliced zero-copy, the wrapper approach (`RPrimitive` with offset tracking) is available as opt-in.
