# miniextendr-bench

Benchmarks for `miniextendr` conversions and interop behavior.

This crate depends on `miniextendr-engine` to embed R, so it is intended for
local development and performance investigations (not publishing).

## Run benchmarks

From the repo root:

```sh
cd miniextendr-bench
cargo bench --bench translate
```

## Notes

- Requires R installed and available on PATH.
- Uses `divan` as the benchmark harness.

## What is measured

The `translate` benchmark focuses on string extraction costs:

- `R_CHAR(charsxp)` fast path (UTF‑8/ASCII)
- `Rf_translateCharUTF8(charsxp)` translation path
- End‑to‑end conversions (`CHARSXP → CStr → String`)
- `TryFromSexp<String>` for STRSXP inputs

It includes both UTF‑8 and Latin‑1 fixtures to highlight the cost of always
translating vs taking an encoding‑aware fast path.

## Publishing to CRAN

This crate is **not** part of any R package build and should never be shipped
in a CRAN tarball. It embeds R and is purely for developer benchmarking.

## Maintainer

- Keep benchmarks aligned with current conversion paths in `miniextendr-api`.
- Update any fixture sizes or data if performance goals change.
- Re-run benchmarks after any substantial FFI or conversion changes.
- Ensure `miniextendr-engine` remains the only embedding dependency.
