# miniextendr-api

Core runtime crate for Rust ↔ R interop.

This crate provides:
- FFI bindings to R’s C API.
- Safe(ish) conversions between Rust and R types.
- The worker‑thread pattern for panic isolation and Drop safety.
- ALTREP traits and helpers.
- Re-exports of `miniextendr-macros` for ergonomics.

Most users should depend on this crate directly.

## Quick start

```rust
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Register exports in your package/module:

```rust
use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod mypkg;
    fn add;
}
```

## Features

- `nonapi` – enable non‑API R symbols (tracked in `NONAPI.md`).
- `rayon` – parallel helpers for R vectors.
- `connections` – experimental R connection framework (unstable API).

## Threading model

1) **Default worker‑thread pattern**
   - `#[miniextendr]` typically runs Rust code on a worker thread.
   - R API calls are marshalled back to the main thread.
   - Protects against R longjmp skipping Rust destructors.

2) **Opt‑in non‑main‑thread R calls** (unsafe)
   - Enabled via feature `nonapi`.
   - Disables R stack checking; still requires serialized access.
   - See `THREADS.md` and `NONAPI.md`.

## Publishing to CRAN

`miniextendr-api` is **CRAN‑compatible** when used correctly:

- Do **not** enable `nonapi` unless you are prepared for CRAN checks to flag
  non‑API symbol usage.
- Ensure all Rust dependencies are vendored in your R package tarball.
- Commit generated wrappers (`R/miniextendr_wrappers.R`) before release.
- Run `R CMD check` on the release tarball.

For embedding R in standalone binaries or integration tests, use
`miniextendr-engine` instead of embedding inside an R package.

## Maintainer

- Keep FFI bindings aligned with current R headers.
- Update conversion behavior tests when R semantics change.
- Ensure roxygen/doc extraction remains in sync with macro behavior.
- Track any non‑API symbols in `NONAPI.md` and gate them behind `nonapi`.
- Run integration tests against current R versions.

## Related docs

- `docs.md` – overview and API notes.
- `altrep.md` – ALTREP design and examples.
- `THREADS.md` – threading model.
- `NONAPI.md` – non‑API symbol tracking.
- `COERCE.md` – coercion strategy.
- `RAYON.md` – rayon integration.
