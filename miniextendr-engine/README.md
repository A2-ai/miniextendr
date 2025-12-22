# miniextendr-engine

Standalone R embedding engine for Rust binaries/tests.

This crate centralizes:
- Linking to `libR` (via `build.rs`).
- R initialization (`Rf_initialize_R`) and mainloop setup.
- A minimal runtime handle for processing events and interrupts.

> Note: This crate uses **non-API** R internals (`Rembedded.h`,
> `Rinterface.h`). It is **not** intended for use inside R packages.

## Usage

```rust
// SAFETY: Must be called once, from the main thread.
let engine = unsafe {
    miniextendr_engine::REngine::build()
        .with_args(&["R", "--quiet", "--vanilla"])
        .init()
        .expect("Failed to initialize R")
};

// ... use R APIs from the main thread ...

std::mem::forget(engine); // optional: intentionally leak the handle
```

### Initialization details

- `R_HOME` is ensured (via `R RHOME`) if missing.
- `Rf_initialize_R` is used (not `Rf_initEmbeddedR`) to avoid double
  `setup_Rmainloop()` calls.
- `setup_Rmainloop()` is called exactly once after initialization.

See `ENGINE.md` in the repo root for rationale and history.

### Runtime sentinel

The engine exposes a lightweight sentinel:

```rust
if miniextendr_engine::r_initialized_sentinel() {
    // R has been initialized in this process.
}
```

This checks the C stack markers set by `Rf_initialize_R`.

## Requirements

- R installed and `R` available on PATH.
- System linker able to link against `libR` (handled by `build.rs`).

## Crate status

This crate is intentionally small and low‑level; most user‑facing functionality
lives in `miniextendr-api` and `miniextendr-macros`.
