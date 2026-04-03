# R-Native Arrow Strings: Zero-Copy String Vectors

## Problem

`Vec<String>` from R does N heap allocations — one `String` per element. Each element goes through `Rf_translateCharUTF8` → `CStr::from_ptr` (O(n) strlen) → `.to_owned()` (memcpy). For a 1M-element character vector, that's 1M allocations + 1M strlen scans + 1M copies.

Meanwhile, R's STRSXP already has all the data: each CHARSXP is a pointer to an interned, GC-managed string with a known `LENGTH`. We already exploit this for `&'static str` and `Vec<&'static str>` (zero-copy via `charsxp_to_str`). But `Vec<String>` forces ownership.

## What "R-native Arrow" means

Arrow's string layout: one contiguous data buffer + an offsets array. No per-string allocation. The idea is to do the analogous thing for R's STRSXP: instead of extracting individual Strings, present the STRSXP as a structured view that borrows directly from R's CHARSXP pool.

## Done: Scalar `Cow<str>` fix

`Cow<'static, str>` now returns `Cow::Borrowed` — delegates to the `&'static str` impl, zero-copy via `charsxp_to_str`. Same fix applied to `Cow<'static, [T]>` for numeric slices.

## Remaining: Vector-level zero-copy

### 1. `Vec<Cow<'static, str>>` impl

The simplest next step. Each element is `Cow::Borrowed(&'static str)` from `charsxp_to_str`. No per-element allocation. Users who need to mutate individual strings get copy-on-write via `.to_mut()`.

```rust
impl TryFromSexp for Vec<Cow<'static, str>> {
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // ... type check, len ...
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i) };
            if charsxp == unsafe { R_NaString } {
                result.push(Cow::Borrowed(""));
            } else {
                result.push(Cow::Borrowed(unsafe { charsxp_to_str(charsxp) }));
            }
        }
        Ok(result)
    }
}
```

Cost: one `Vec` allocation (for the Cow pointers), zero string copies. Each `Cow::Borrowed` is 2 words (ptr + len).

Also: `Vec<Option<Cow<'static, str>>>` for NA-aware variant.

### 2. `RStringVec` — Arrow-style string view type

A struct that holds the STRSXP SEXP and provides indexed access without materializing Strings:

```rust
/// Zero-copy view over an R STRSXP. No per-element allocation.
pub struct RStringVec {
    sexp: SEXP,  // the STRSXP, must be GC-protected
    len: usize,
}

impl RStringVec {
    pub fn len(&self) -> usize { self.len }
    
    /// Get element as &str (zero-copy from CHARSXP).
    pub fn get(&self, i: usize) -> Option<&str> {
        if i >= self.len { return None; }
        let charsxp = unsafe { STRING_ELT(self.sexp, i as R_xlen_t) };
        if charsxp == unsafe { R_NaString } {
            return None;  // NA → None
        }
        Some(unsafe { charsxp_to_str(charsxp) })
    }
    
    /// Iterate without allocation.
    pub fn iter(&self) -> impl Iterator<Item = Option<&str>> { ... }
    
    /// Materialize into Vec<String> only when needed.
    pub fn to_vec_string(&self) -> Vec<String> { ... }
}
```

This is the "Arrow-style" type: the backing data lives in R, and you access it through a view. The SEXP must be GC-protected (via `OwnedProtect` or by being an argument to a `.Call`).

Key design decisions:
- `get()` returns `Option<&str>` — None for NA (not lossy like `Vec<String>`)
- Implements `Index`, `IntoIterator`, etc.
- `TryFromSexp` impl makes it usable in `#[miniextendr]` function signatures
- No `IntoR` needed (it already IS an R object — just return the SEXP)

### 3. Encoding edge case

`charsxp_to_str` assumes UTF-8 (validated by `miniextendr_assert_utf8_locale()` at init). This is safe for CE_UTF8 and CE_NATIVE (in UTF-8 locale) strings. But R can have CE_LATIN1 or CE_BYTES strings even in a UTF-8 session.

For the zero-copy path, non-UTF-8 CHARSXPs would need to either:
- **Reject** (return an error) — strictest, safest
- **Fall back to `Rf_translateCharUTF8`** — zero-copy for UTF-8, one copy for non-UTF-8 (this is what the current `String` path does)

The fallback approach is better: check `IS_UTF8(charsxp) || IS_ASCII(charsxp)`, and only call `Rf_translateCharUTF8` + copy for the rare non-UTF-8 case. This makes `Cow` the perfect type — `Borrowed` for UTF-8, `Owned` for translated strings.

### 4. String ALTREP integration

`RStringVec` could back a string ALTREP — R sees a STRSXP, but the `Elt` method delegates to `RStringVec::get()`. This enables lazy computation of string elements (e.g., from a Rust iterator or database cursor) while presenting as a normal R character vector.

This is already partially supported via the existing ALTREP string infrastructure (`AltrepString` trait), but `RStringVec` would be the zero-copy *input* path (R → Rust), while ALTREP is the lazy *output* path (Rust → R).

## Priority order

1. `Vec<Cow<'static, str>>` + `Vec<Option<Cow<'static, str>>>` — immediate win, small diff
2. `RStringVec` — the real zero-copy view type
3. Encoding-aware fallback in `RStringVec` — handles edge cases
4. ALTREP integration — connects input and output paths

## What this does NOT cover

- Contiguous string buffers (Arrow's actual layout with offsets into one `u8` buffer) — R's STRSXP stores strings as individual CHARSXPs, not contiguously. There's no way to get Arrow's layout without copying. But we avoid per-*element* allocation, which is the main cost.
- Mutable string vectors — R strings are immutable (CHARSXPs are interned). Mutation requires `SET_STRING_ELT` with a new CHARSXP. `RStringVec` is read-only by design.
