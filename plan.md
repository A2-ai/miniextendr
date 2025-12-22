# miniextendr Review Plan

This file is a living checklist + scratchpad for keeping the repo coherent.
It intentionally avoids hard-coded line numbers (they drift quickly).

## Scope

- `miniextendr-api/` - runtime crate (FFI, worker pattern, ALTREP, conversions)
- `miniextendr-macros/` - proc-macros (`#[miniextendr]`, `miniextendr_module!`, `#[r_ffi_checked]`)
- `rpkg/` - example R package + build tooling (autoconf + Cargo)

## Quick orientation

- Main crate index: `miniextendr-api/src/lib.rs`
- High-level docs: `docs.md`, `COERCE.md`, `altrep.md`, `THREADS.md`, `NONAPI.md`
- Example package glue: `rpkg/src/entrypoint.c`, `rpkg/src/rust/lib.rs`, `rpkg/R/miniextendr_wrappers.R`

## Hotspots (largest Rust files)

Re-generate at any time:

```sh
find miniextendr-api/src -name '*.rs' -print0 | xargs -0 wc -l | sort -nr
find miniextendr-macros/src -name '*.rs' -print0 | xargs -0 wc -l | sort -nr
```

As of 2025-12-14:

| Area | File | LOC |
|------|------|-----|
| api | `miniextendr-api/src/altrep_data.rs` | 2,372 |
| api | `miniextendr-api/src/altrep_impl.rs` | 1,411 |
| api | `miniextendr-api/src/externalptr.rs` | 1,309 |
| api | `miniextendr-api/src/coerce.rs` | 723 |
| api | `miniextendr-api/src/ffi.rs` | 632 |
| macros | `miniextendr-macros/src/lib.rs` | 1,949 |

## Current checklist

- [ ] Docs match current APIs and `justfile` recipes (avoid drift).
- [ ] ALTREP fallbacks use the correct sentinel (`NULL` vs `R_NilValue`) for the specific R callsite.
- [ ] Threading story is consistent:
  - worker-thread pattern (R APIs on main thread via `with_r_thread`)
  - `thread` module (`nonapi` + stack-check disabling) is explicitly opt-in and documented
- [ ] `rpkg/` build pipeline stays reproducible (vendoring + wrapper generation).

## Known TODOs (code)

There are TODOs in `miniextendr-macros/src/lib.rs` (refactors + error messages + dots ergonomics).
Keep these as TODOs if they reflect deliberate roadmap; otherwise convert to issues or delete.
