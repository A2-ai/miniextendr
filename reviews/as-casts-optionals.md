# Review: `as` casts remaining in optionals/

## What happened

Core modules (into_r, from_r, serde/*, dataframe, list, factor, allocator) had
lossy `as` casts replaced with `From`/`TryFrom` per CLAUDE.md policy. But the
optionals/ directory still has ~50+ `as isize`, `as i32`, `as f64` casts on
R data paths.

## Files affected

- `toml_impl.rs` — ~20 casts (`len as isize`, `i as isize`, `i64 as i32`, `i64 as f64`)
- `time_impl.rs` — ~10 casts
- `serde_impl.rs` — ~5 remaining after partial fix
- `num_bigint_impl.rs` — some `as i32` on arithmetic
- Others have fewer

## Pattern

Most are `usize as isize` for `Rf_allocVector` length and `SET_*_ELT` index args.
The fix is mechanical: `let n: R_xlen_t = len.try_into().expect("...")`.

## Risk

On 64-bit (all current R platforms), `usize as isize` is safe for realistic sizes.
The risk is theoretical (usize > isize::MAX would wrap to negative). But CLAUDE.md
says to use TryFrom, and the core modules are now consistent.

## Fix

Apply the same pattern as the agent did for core files. Mechanical replacement,
no design decisions needed.
