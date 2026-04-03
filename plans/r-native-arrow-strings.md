# R-Native Arrow Strings: Zero-Copy String Vectors

## Problem

`Vec<String>` from R does N heap allocations — one `String` per element. Each element goes through `Rf_translateCharUTF8` → `CStr::from_ptr` (O(n) strlen) → `.to_owned()` (memcpy). For a 1M-element character vector, that's 1M allocations + 1M strlen scans + 1M copies.

Meanwhile, R's STRSXP already has all the data: each CHARSXP is a pointer to an interned, GC-managed string with a known `LENGTH`. We already exploit this for `&'static str` and `Vec<&'static str>` (zero-copy via `charsxp_to_str`). But `Vec<String>` forces ownership.

## What "R-native Arrow" means

Arrow's string layout: one contiguous data buffer + an offsets array. No per-string allocation. The idea is to do the analogous thing for R's STRSXP: instead of extracting individual Strings, present the STRSXP as a structured view that borrows directly from R's CHARSXP pool.

## Done

### Scalar Cow (commit 1)

`Cow<'static, str>` now returns `Cow::Borrowed` — delegates to the `&'static str` impl, zero-copy via `charsxp_to_str`. Same fix applied to `Cow<'static, [T]>` for numeric slices.

### Vec<Cow> + encoding-aware helper + StrVec iterators (commit 2)

- **`charsxp_to_cow()`** — encoding-safe helper: tries `from_utf8` on `R_CHAR` data (O(1) zero-copy), falls back to `Rf_translateCharUTF8` + copy only for non-UTF-8 CHARSXPs.
- **`Vec<Cow<'static, str>>`**, **`Vec<Option<Cow<'static, str>>>`**, **`Box<[Cow<'static, str>]>`** — zero per-element allocation for UTF-8 strings.
- **`StrVec::get_cow()`** — encoding-safe element access returning `Option<Cow<str>>`.
- **`StrVec::iter()`** → `StrVecIter` — zero-copy iteration yielding `Option<&str>`.
- **`StrVec::iter_cow()`** → `StrVecCowIter` — encoding-safe iteration yielding `Option<Cow<str>>`.
- **`IntoIterator for StrVec`** — `for elem in strvec { ... }` works.

## Remaining

### String ALTREP integration

`StrVec` could back a string ALTREP — R sees a STRSXP, but the `Elt` method delegates to Rust-computed strings. This enables lazy computation of string elements (e.g., from a Rust iterator or database cursor) while presenting as a normal R character vector.

This is already partially supported via the existing ALTREP string infrastructure (`AltrepString` trait). The zero-copy *input* path (R → Rust) is now done; ALTREP is the lazy *output* path (Rust → R).

## What this does NOT cover

- Contiguous string buffers (Arrow's actual layout with offsets into one `u8` buffer) — R's STRSXP stores strings as individual CHARSXPs, not contiguously. There's no way to get Arrow's layout without copying. But we avoid per-*element* allocation, which is the main cost.
- Mutable string vectors — R strings are immutable (CHARSXPs are interned). Mutation requires `SET_STRING_ELT` with a new CHARSXP. `StrVec` is read-only by design.
