# miniextendr-bench

Benchmarks for `miniextendr` conversions and interop behavior.

This crate depends on `miniextendr-engine` to embed R, so it is intended for
local development and performance investigations (not publishing).

## Run benchmarks

From the repo root:

```sh
just bench --bench translate
# or:
cargo bench --manifest-path=miniextendr-bench/Cargo.toml --bench translate
```

## Notes

- Requires R installed and available on PATH.
- Uses `divan` as the benchmark harness.
- See `miniextendr-bench/benches/` for the full target list (including `trait_abi`), and `miniextendr-bench/src/bench_plan/` for a high-level plan.

## What is measured

Some selected targets:

- `translate`: string extraction costs
- `trait_abi`: mx_erased trait dispatch (vtable query + method calls)

Many others cover conversions, FFI calls, ExternalPtr, ALTREP, worker routing, and more.

## Publishing to CRAN

This crate is **not** part of any R package build and should never be shipped
in a CRAN tarball. It embeds R and is purely for developer benchmarking.

## Maintainer

- Keep benchmarks aligned with current conversion paths in `miniextendr-api`.
- Update any fixture sizes or data if performance goals change.
- Re-run benchmarks after any substantial FFI or conversion changes.
- Ensure `miniextendr-engine` remains the only embedding dependency.
