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
- Benchmark intent and methodology are described in `ENGINE.md`.
