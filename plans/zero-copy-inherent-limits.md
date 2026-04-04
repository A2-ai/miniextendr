# Zero-Copy Inherent Limits

Reviewed 2026-04-04. Most items here turned out to be non-issues on investigation.

## 1. Provenance UB on SEXP type-tag read — NOT WORTH FIXING

`try_recover_r_sexp` reads 4 bytes through a pointer derived via `wrapping_byte_sub`. Under strict provenance this is UB. Investigated adding a C helper (`cc` build dep + `c_helpers.c`) — works but adds complexity for a theoretical concern.

**Verdict**: Not worth fixing. `wrapping_byte_sub` makes the arithmetic defined. The read is from mapped heap memory. LLVM doesn't exploit provenance UB on reads. Miri flags it; real hardware doesn't care. Every allocator, GC, and FFI system in Rust does the same thing. Triple verification (type + length + DATAPTR_RO) prevents false positives.

If Rust stabilizes `ptr::with_exposed_provenance`, revisit then — it would be a one-line fix with no new dependencies.

## 2. TryFromSexp has no lifetime parameter — BREAKING CHANGE, DEFER

The `'static` lifetime on borrowed types (`&str`, `Cow<[T]>`) is a lie. `ProtectedStrVec` works around it for strings. A proper fix needs GAT-based `TryFromSexp<'a>` — major breaking change touching every conversion, every macro expansion, every user type.

**Verdict**: 0.2 material. Not actionable now.

## 3. String layout incompatibility — ALREADY SOLVED

Investigated ALTREP STRSXP backed by Arrow StringArray. Turns out this already exists: `StringArray` has `impl_altstring_from_data!` + `RegisterAltrep` + `AltStringData::elt`. Users can call `string_array.into_sexp_altrep()` today — R gets a lazy STRSXP that constructs CHARSXPs on demand from Arrow's contiguous buffer.

**Verdict**: Paper tiger. Already implemented.

## 4. Arrow buffer slicing breaks recovery — ACCEPT

`array.slice()` shifts the data pointer. Recovery fails, falls through to copy. This is correct behavior — sliced arrays are intermediate results in compute pipelines. The final result is usually a new array that would copy anyway.

**Verdict**: Accepted limitation. Documented in ARROW.md.
