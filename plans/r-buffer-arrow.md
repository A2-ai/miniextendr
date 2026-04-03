# R-Buffer-Allocated Arrow Objects

## Problem (solved)

Arrow→R always copied, even when the Arrow buffer IS R memory.

## Solution: Automatic SEXP pointer recovery

### How it works

R stores vector data at `sexp + sizeof(SEXPREC_ALIGN)`. All vector types (REALSXP, INTSXP, RAWSXP, STRSXP, VECSXP) use `VECTOR_SEXPREC` — same header size.

At package init, we measure the offset. Then in IntoR:

```
data_ptr - SEXPREC_DATA_OFFSET = candidate SEXP
verify: type tag ∧ length ∧ DATAPTR_RO round-trip
```

If verification passes, return the original SEXP. No copy.

### What's zero-copy now

| Direction | Type | Zero-copy? |
|-----------|------|-----------|
| R→Arrow | Float64Array, Int32Array, UInt8Array | Yes (was already) |
| Arrow→R | Float64Array, Int32Array, UInt8Array | **Yes (new)** — automatic recovery |
| Arrow→R | RecordBatch | **Yes (new)** — per-column via array IntoR |
| Arrow→R | StringArray | No (incompatible layouts) |
| Arrow→R | BooleanArray | No (i32 vs bit-packed) |

### Opt-in wrapper types (for strings)

`RStringArray` wraps `StringArray` + source STRSXP. Since string layouts are incompatible, pointer recovery can't work — the wrapper carries provenance explicitly.

`RPrimitive<T>` and `RSourced` trait exist but are largely superseded by automatic recovery for primitives. Useful as explicit documentation of intent.

## Done

- `r_memory` module: `init_sexprec_data_offset()`, `try_recover_r_sexp()`, `sexprec_data_offset()`
- `Float64Array`, `Int32Array`, `UInt8Array` IntoR with automatic recovery
- `RecordBatch` IntoR gets zero-copy per-column automatically
- `RSourced` trait, `RPrimitive<T>`, `RStringArray` (opt-in wrappers)
- `RRecordBatch` removed (redundant — automatic recovery covers it)

## Known limitations

- **First read on non-R pointers**: `try_recover_r_sexp` reads 4 bytes from `data_ptr - offset`, which is arbitrary heap memory for Rust-allocated buffers. Safe in practice (mapped memory), technically UB. Triple verification prevents false positives.
- **Sliced buffers**: `array.slice()` shifts the data pointer. Recovery fails, falls through to copy.
- **ALTREP vectors**: data isn't at fixed offset. Recovery fails (caught by DATAPTR_RO round-trip), copies.

## Also done

- `alloc_r_backed_buffer<T>(len)`: allocate Arrow buffers as R SEXPs for Rust→Arrow→R zero-copy
- `Cow<'_, [T]>` IntoR: automatic SEXP recovery for borrowed slices from R
