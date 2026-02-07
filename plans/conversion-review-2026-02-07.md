# Conversion System Review (2026-02-07)

## Scope Reviewed

- `miniextendr-api/src/from_r.rs`
- `miniextendr-api/src/into_r.rs`
- `miniextendr-api/src/coerce.rs`
- `miniextendr-api/src/as_coerce.rs`
- `miniextendr-api/src/into_r_as.rs`
- `miniextendr-api/src/convert.rs`
- `miniextendr-api/src/raw_conversions.rs`
- `miniextendr-api/src/altrep.rs`
- `miniextendr-api/src/altrep_bridge.rs`
- `miniextendr-api/src/altrep_impl.rs`
- `miniextendr-api/src/altrep_traits.rs`
- Optional conversion adapters (spot checks): `url_impl.rs`, `toml_impl.rs`, `serde_impl.rs`, `uuid_impl.rs`, plus pattern scan across `src/optionals/*`

## Findings (Ordered by Severity)

### [P0] `Altrep<T>` conversion passes unprotected `data1` into `R_new_altrep`

- `miniextendr-api/src/into_r.rs:1795`
- `miniextendr-api/src/into_r.rs:1798`
- `miniextendr-api/src/into_r.rs:1799`

`IntoR for Altrep<T>` builds `data1` via `ExternalPtr::new(self.0)` and calls `R_new_altrep` without protecting `data1`. If `R_new_altrep` allocates and GC runs, `data1` can be collected before class construction completes.

Contrast: macro unserialize path already protects `data1` correctly:

- `miniextendr-api/src/altrep_impl.rs:159`
- `miniextendr-api/src/altrep_impl.rs:160`
- `miniextendr-api/src/altrep_impl.rs:161`

Recommended fix: `PROTECT(data1)` across `R_new_altrep` (and `UNPROTECT` after), matching the existing unserialize pattern.

### [P0] Safe string conversion path can invoke UB on non-UTF8 CHARSXP

- `miniextendr-api/src/from_r.rs:60`
- `miniextendr-api/src/from_r.rs:66`
- `miniextendr-api/src/from_r.rs:72`
- `miniextendr-api/src/from_r.rs:78`

`charsxp_to_str` and `charsxp_to_str_unchecked` call `from_utf8_unchecked` on raw CHARSXP bytes. R can hold non-UTF8 encodings (latin1/native/bytes), so this is not guaranteed valid UTF-8. Because these helpers are used in safe `TryFromSexp` impls, invalid input can trigger UB.

Representative call sites:

- `miniextendr-api/src/from_r.rs:1979`
- `miniextendr-api/src/from_r.rs:2057`
- `miniextendr-api/src/from_r.rs:3102`
- `miniextendr-api/src/optionals/url_impl.rs:83`
- `miniextendr-api/src/optionals/toml_impl.rs:150`
- `miniextendr-api/src/optionals/serde_impl.rs:438`

Recommended fix: prefer `Rf_translateCharUTF8` + checked `CStr::to_str()` (or a clear lossy policy), and reserve unchecked decoding for explicitly `unsafe` APIs.

### [P0] Mutable reference conversions are unsound under aliasing and lifetime extension

- `miniextendr-api/src/from_r.rs:1416`
- `miniextendr-api/src/from_r.rs:1438`
- `miniextendr-api/src/from_r.rs:1830`
- `miniextendr-api/src/from_r.rs:1847`

`TryFromSexp` provides safe impls returning `&'static mut T` and `&mut [T]` from SEXP memory with no cross-argument alias checks. Passing the same R object to two mutable parameters can produce aliased `&mut` references (UB). The API also enables extending borrows beyond `.Call` lifetime constraints.

Related lifetime model acknowledges `'static` is a convenience lie:

- `miniextendr-api/src/ffi.rs:226`
- `miniextendr-api/src/ffi.rs:229`
- `miniextendr-api/src/ffi.rs:311`

Recommended fix: move mutable borrowed conversions behind explicit `unsafe` API boundaries or enforce runtime alias checks at wrapper generation for multi-arg mutable borrows.

### [P1] Float-to-`isize`/`usize` range checks accept rounded out-of-range maxima

- `miniextendr-api/src/coerce.rs:915`
- `miniextendr-api/src/coerce.rs:936`

Current upper checks use `>` against `isize::MAX as f64` / `usize::MAX as f64`. On 64-bit targets these casts round up (`2^63`, `2^64`), so out-of-range values at those rounded boundaries pass checks and then saturating float-to-int casts produce max values.

Recommended fix: make upper checks `>=` for these two conversions and add boundary tests around `isize::MAX as f64` and `usize::MAX as f64`.

### [P1] ALTREP trampolines do not contain panic boundaries

- `miniextendr-api/src/altrep_bridge.rs:26`
- `miniextendr-api/src/altrep_bridge.rs:33`
- `miniextendr-api/src/altrep_bridge.rs:91`
- `miniextendr-api/src/altrep_bridge.rs:102`

Trampolines call trait methods directly inside `extern "C-unwind"` with no `catch_unwind` or conversion to R errors. A panic in user ALTREP code can unwind through R/C frames and terminate the process.

Recommended fix: wrap each trampoline body with `std::panic::catch_unwind` and convert panics to `r_stop`/error return conventions.

### [P1] Tagged raw conversion writes type metadata but never validates it on decode

- Type tag is written:
  - `miniextendr-api/src/raw_conversions.rs:414`
  - `miniextendr-api/src/raw_conversions.rs:446`
- Decode path ignores tag and validates only header shape:
  - `miniextendr-api/src/raw_conversions.rs:485`
  - `miniextendr-api/src/raw_conversions.rs:518`
- `RawError::TypeMismatch` exists but is unused:
  - `miniextendr-api/src/raw_conversions.rs:82`

Impact: different POD types with same size/layout can decode successfully as the wrong type.

Recommended fix: read `mx_raw_type` attribute during `TryFromSexp` for `RawTagged<T>` / `RawSliceTagged<T>` and return `TypeMismatch` when it does not match `type_name::<T>()`.

### [P2] `DataFrame` conversion panics for unnamed rows

- `miniextendr-api/src/convert.rs:445`
- `miniextendr-api/src/convert.rs:447`

`IntoDataFrame for DataFrame<T>` panics when first row has no names. This is conversion-layer input validation and should ideally produce structured conversion errors, not panic.

Recommended fix: convert to error-returning path (`Result<List, ...>`) or map to explicit R error without panic.

## Validation Performed

- Static code audit across core conversion, coercion, ALTREP, raw, and optional adapter modules.
- Quick behavior check for float boundary rounding in Rust casts (standalone rustc snippet).
- Ran one focused unit test command:
  - `cargo test -p miniextendr-api coerce::tests::test_f64_to_u64 -- --exact` (pass)

## Notes

- No code changes were made in this review; this file is audit output only.
