# R-Native Arrow Strings: Zero-Copy String Vectors

## Problem

`Vec<String>` from R does N heap allocations — one `String` per element. Each element goes through `Rf_translateCharUTF8` → `CStr::from_ptr` (O(n) strlen) → `.to_owned()` (memcpy). For a 1M-element character vector, that's 1M allocations + 1M strlen scans + 1M copies.

Meanwhile, R's STRSXP already has all the data: each CHARSXP is a pointer to an interned, GC-managed string with a known `LENGTH`. We already exploit this for `&'static str` and `Vec<&'static str>` (zero-copy via `charsxp_to_str`). But `Vec<String>` forces ownership.

## What "R-native Arrow" means

Arrow's string layout: one contiguous data buffer + an offsets array. No per-string allocation. The idea is to do the analogous thing for R's STRSXP: instead of extracting individual Strings, present the STRSXP as a structured view that borrows directly from R's CHARSXP pool.

## Done

### Commit 1: Scalar Cow fix

`Cow<'static, str>` and `Cow<'static, [T]>` now return `Cow::Borrowed` — zero-copy.

### Commit 2: Vec<Cow> + encoding-safe helper + StrVec iterators

- `charsxp_to_cow()` — tries `from_utf8` on `R_CHAR` (O(1) zero-copy), falls back to `Rf_translateCharUTF8` for non-UTF-8.
- `Vec<Cow<'static, str>>`, `Vec<Option<Cow<'static, str>>>`, `Box<[Cow<'static, str>]>` TryFromSexp.
- `StrVec::get_cow()`, `iter()`, `iter_cow()`, `IntoIterator`.

### Commit 3: ProtectedStrVec + IntoR for Cow vectors

- `ProtectedStrVec` — owns `OwnedProtect`, ties all borrowed data to `&self` lifetime (not `'static`). Prevents use-after-GC at compile time.
- `IntoR` for `Vec<Cow<str>>`, `Box<[Cow<str>]>`, `Vec<Option<Cow<str>>>` — complete round-trip.
- `ProtectedStrVec` has `TryFromSexp`, `IntoR`, `iter()`, `iter_cow()`, `Debug`.

### Commit 4: ALTREP with seamless serialization

- `Vec<Cow<'static, str>>` and `Vec<Option<Cow<'static, str>>>` get full ALTREP support (dataptr + serialize).
- Serialize: `Rf_mkCharLenCE` hits R's CHARSXP cache for borrowed strings — no string data copy.
- Unserialize: `TryFromSexp` uses `charsxp_to_cow` → `Cow::Borrowed` for UTF-8 — zero-copy back.

## Design decisions

- **`'static` on `TryFromSexp` types**: `TryFromSexp` returns `Self` with no lifetime parameter, so borrowed types must use `'static`. This is already the case for `&'static str`, `&'static [T]`, etc. `ProtectedStrVec` is the opt-in safe alternative with proper lifetimes.
- **Encoding fallback in `charsxp_to_cow`**: Uses `from_utf8` first (O(1)), falls back to `Rf_translateCharUTF8` + `Cow::Owned` for non-UTF-8. Works seamlessly — caller doesn't need to think about encodings.
- **NA semantics**: Non-Option variants map NA to `""` (matches `Vec<String>` behavior). Option variants preserve NA as `None`.
