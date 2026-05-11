# miniextendr-bench

Benchmark crate. **Separate workspace member** — its own `Cargo.toml` and `target/`. See root `CLAUDE.md` for project rules.

## Layout
- `benches/` — criterion harness inputs.
- `src/` — fixtures + helpers used by the benches.
- `BENCH_RESULTS_<date>.md` — snapshotted results. Date the file; don't overwrite history.
- `README.md` — how to run / interpret.

## Notes
- Run via `just bench-*` recipes — direct `cargo bench --workspace` won't pick this up cleanly because of patch-table differences.
- New numbers go in a new dated `BENCH_RESULTS_*.md`; reference the commit SHA in the file. Old result files stay for trend comparison.
