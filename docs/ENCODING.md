# Encoding and Locale

This document covers how miniextendr converts R strings in any locale, and its
encoding probing utilities.

**Source:** `miniextendr-api/src/encoding.rs`, `miniextendr-api/src/from_r.rs`

## Strings in any locale

A miniextendr package loads in any locale, including `LANG=C`, `LANG=POSIX`
and an unset locale (minimal containers, batch jobs with a stripped
environment). Each conversion from an R string to Rust reads that string's own
encoding mark:

| R string | Conversion to Rust |
|----------|--------------------|
| ASCII | borrowed (`R_CHAR` + `LENGTH`, zero-copy) |
| marked UTF-8 | borrowed; the bytes are validated, and malformed UTF-8 is an error |
| native (unmarked), in a UTF-8 session | borrowed and validated, as for UTF-8 |
| native non-ASCII, in a non-UTF-8 session | translated from the session's encoding with `Rf_translateCharUTF8` |
| marked Latin-1 | translated with `Rf_translateCharUTF8` |
| marked `bytes` | an error: arbitrary bytes are not text |

Whether a native string needs translation is R's decision:
`Rf_translateCharUTF8` hands back the string's own bytes when none is needed,
and it reads the session's current locale, so a `Sys.setlocale()` mid-session
is followed. A translated string is a copy (`String`, a `Vec<Cow<str>>`
element), or borrowed from R's buffer for the current `.Call` (`&str`, a
scalar `Cow<str>` parameter). Borrowed views that can
outlive the call (`StrVec` / `ProtectedStrVec` accessors) refuse a string that
needs translation; read it as `String` or `Cow<str>`.

Strings miniextendr gives R are always marked UTF-8, so they are correct in
any locale.

### Refused strings

An argument conversion refuses a `bytes` string, or one whose bytes are not
valid UTF-8, through its `Err`, so the R error is the argument error a wrong
type gets: the crate's `conversion_error_class`, `kind = "conversion"`, and
`e$param`. A vector conversion lists every refused element in one message:

```r
x <- c("a", "µg", NA, "ñ"); Encoding(x) <- "bytes"
echo(x)  # #[miniextendr] fn echo(x: Vec<String>) -> Vec<String>
#> Error in echo(x): 'x' must be character: marked as bytes, not text (elements 2, 4)
```

The scalar form reads `'x' must be a single string: marked as bytes, not
text`; a string that is not valid UTF-8 reads `not valid UTF-8`. The serde
deserializers refuse it with an `RSerdeError`; `DataFrame::group_by` with a
`DataFrameError` naming every refused key, and `group_by_metadata` /
`group_declaration` every refused grouping-variable name.

Reads that cannot return an error, such as `SexpExt::string_elt_str`,
`SexpExt::r_char_str`, the `StrVec` accessors, and `DataFrame::names()`, read
with the same encoding rules and **panic** on a string that is not text,
naming the element. Name lookups (`List` / `NamedList` / `TypedList` by
name, `DataFrame::column_raw` / `contains_column`) translate a name as a
conversion does and skip a name that is not text, which no Rust `&str` can
match. Names that only go into messages (a class attribute, a connection's
description) are read lossily, with `U+FFFD` for bytes that are not UTF-8.

A `match_arg` choice marked as bytes is compared by its bytes, as R's
`match.arg()` compares it: it matches a choice only when its bytes are that
choice's UTF-8. `AsNumeric` reads it as non-numeric, as `as.numeric()` gives
`NA` for it.

### The translation warning

In a session whose locale isn't UTF-8, the first native non-ASCII string a
package translates raises one warning, of class
`miniextendr_native_translation`, when the call returns:

```text
Warning message:
translated a string from the session's native encoding to UTF-8: the R
session's locale isn't UTF-8, so characters that encoding can't represent
became <xx> escapes. Start R in a UTF-8 locale (for example LANG=C.UTF-8) to
avoid translation. Shown once per session.
```

In a C or POSIX locale the session's encoding is ASCII, so a byte such as
`0xE9` comes out as `<e9>` (R's own escape for an unrepresentable byte). ASCII
and UTF-8-marked data never warn. Muffle it with
`withCallingHandlers(..., miniextendr_native_translation = function(w) invokeRestart("muffleWarning"))`.

R >= 4.2.0 uses UTF-8 by default on mainstream platforms; set `LANG=C.UTF-8`
(or another UTF-8 locale) where the environment leaves it unset.

## Encoding Info (Non-API, Embedding Only)

The `miniextendr_encoding_init()` function snapshots R's internal encoding
state into a static `REncodingInfo` struct. This is **only available when
embedding R** (via `miniextendr-engine`), not in R packages.

### Why Not in R Packages

`miniextendr_encoding_init()` reads non-API symbols from R's `Defn.h`
(`utf8locale`, `mbcslocale`, `known_to_be_latin1`). These symbols are not a
supported package API, so this snapshot is reserved for embedding builds.

### REncodingInfo

When the `nonapi` feature is enabled and `miniextendr_encoding_init()` has run:

```rust
use miniextendr_api::encoding;

if let Some(info) = encoding::encoding_info() {
    println!("UTF-8 locale: {:?}", info.utf8_locale);
    println!("multibyte locale: {:?}", info.mbcs_locale);
    println!("unknown strings are Latin-1: {:?}", info.known_to_be_latin1);
}
```

| Field | Type | Description |
|-------|------|-------------|
| `utf8_locale` | `Option<bool>` | Whether R considers the locale UTF-8 |
| `mbcs_locale` | `Option<bool>` | Whether R considers the locale multibyte |
| `known_to_be_latin1` | `Option<bool>` | Whether R treats unknown strings as Latin-1 |

All fields require the `nonapi` feature. Without it, `REncodingInfo` is an
empty struct and `encoding_info()` returns `Some(&REncodingInfo {})` after init.

### Debug Output

Set `MINIEXTENDR_ENCODING_DEBUG=1` to print the encoding snapshot at init time:

```bash
MINIEXTENDR_ENCODING_DEBUG=1 R -e 'library(miniextendr)'
# [miniextendr] encoding init: REncodingInfo { utf8_locale: Some(true), ... }
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

miniextendr honors each CHARSXP's encoding tag and lets R decide, per
native string, whether the session's locale needs a translation. The common
path (ASCII, UTF-8, native in a UTF-8 session) stays zero-copy.
