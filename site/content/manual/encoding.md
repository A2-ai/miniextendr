+++
title = "Encoding and Locale"
weight = 39
description = "This document covers miniextendr's UTF-8 locale requirement and encoding probing utilities."
+++

This document covers miniextendr's UTF-8 locale requirement and encoding
probing utilities.

**Source:** `miniextendr-api/src/encoding.rs`

## UTF-8 Locale Assertion

miniextendr requires a UTF-8 locale. The `miniextendr_assert_utf8_locale()`
function is called during package initialization (`R_init_*`) and terminates
with an R error if the session is not UTF-8.

This is necessary because miniextendr's internal `charsxp_to_str` assumes all
CHARSXP bytes are valid UTF-8. R >= 4.2.0 uses UTF-8 by default on all
platforms, so this only fails on very old or misconfigured R installations.

### How It Works

1. Calls `l10n_info()` (public R API) during `R_init_*`
2. Reads the `"UTF-8"` element from the result
3. If `FALSE`, raises an R error:
   `"miniextendr requires a UTF-8 locale (R >= 4.2.0 uses UTF-8 by default)"`

### Initialization Integration

The assertion is called automatically by `package_init()` (via `miniextendr_init!`):

```rust
// lib.rs - the macro handles UTF-8 assertion automatically
miniextendr_api::miniextendr_init!(mypkg);
```

No user action is required - `miniextendr_init!` includes the UTF-8 locale
check as part of the standard initialization sequence.

## Encoding Info (Non-API, Embedding Only)

The `miniextendr_encoding_init()` function snapshots R's internal encoding
state into a static `REncodingInfo` struct. This is **only available when
embedding R** (via `miniextendr-engine`), not in R packages.

### Why Not in R Packages

`miniextendr_encoding_init()` reads non-API symbols from R's `Defn.h`
(`utf8locale`, `latin1locale`, `known_to_be_utf8`, `R_nativeEncoding`). These
symbols are not exported from R's shared library (`libR.so` / `R.dll`), so they
are unavailable to packages loaded via `.Call`.

### REncodingInfo

When the `nonapi` feature is enabled and `miniextendr_encoding_init()` has run:

```rust
use miniextendr_api::encoding;

if let Some(info) = encoding::encoding_info() {
    println!("native encoding: {:?}", info.native_encoding);
    println!("UTF-8 locale: {:?}", info.utf8_locale);
    println!("Latin-1 locale: {:?}", info.latin1_locale);
    println!("known_to_be_utf8: {:?}", info.known_to_be_utf8);
}
```

| Field | Type | Description |
|-------|------|-------------|
| `native_encoding` | `Option<String>` | R's native encoding name |
| `utf8_locale` | `Option<bool>` | Whether R considers the locale UTF-8 |
| `latin1_locale` | `Option<bool>` | Whether R considers the locale Latin-1 |
| `known_to_be_utf8` | `Option<bool>` | R's stricter "known to be UTF-8" flag |

All fields require the `nonapi` feature. Without it, `REncodingInfo` is an
empty struct and `encoding_info()` returns `Some(&REncodingInfo {})` after init.

### Debug Output

Set `MINIEXTENDR_ENCODING_DEBUG=1` to print the encoding snapshot at init time:

```bash
MINIEXTENDR_ENCODING_DEBUG=1 R -e 'library(miniextendr)'
# [miniextendr] encoding init: REncodingInfo { native_encoding: Some("UTF-8"), ... }
```

This is only useful when embedding R or on platforms where the non-API symbols
happen to be exported.

## R's Encoding Model

For background, R has two layers of encoding:

1. **Per-CHARSXP tags** -- each R string carries an encoding mark (UTF-8,
   Latin-1, bytes, or native). Functions like `Rf_mkCharCE` and
   `Rf_translateCharUTF8` work with these tags.

2. **Global locale state** -- R tracks whether the session locale is UTF-8 or
   Latin-1. This affects how "native" strings are interpreted.

miniextendr sidesteps most of this complexity by requiring UTF-8 up front. All
Rust strings (`&str`, `String`) are UTF-8 by definition, so the assertion
ensures R's native encoding matches Rust's.
