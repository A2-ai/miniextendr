# miniextendr-macros-core

Shared naming helpers for the miniextendr proc-macro crate.

## What this crate provides

- **Naming helpers** -- `call_method_def_ident_for()` and
  `r_wrapper_const_ident_for()` which produce the deterministic identifiers
  used by the attribute macro for naming generated registration statics.

## Usage

This crate is an internal implementation detail:

```toml
[dependencies]
miniextendr-macros-core = { workspace = true }
```

```rust
// In miniextendr-macros (proc-macro crate)
pub(crate) use miniextendr_macros_core::{call_method_def_ident_for, r_wrapper_const_ident_for};
```
