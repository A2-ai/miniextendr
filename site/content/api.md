+++
title = "API Reference"
weight = 30
description = "Rustdoc API reference for every miniextendr crate"
template = "api.html"
+++

The full `cargo doc` output for each workspace crate -- private items, hidden items, and type layouts are included (`--document-private-items`, `--document-hidden-items`, `--show-type-layout`). Links to crates.io dependencies resolve to docs.rs via `-Z rustdoc-map`.

## Core traits

| Trait | Crate | Purpose |
|-------|-------|---------|
| `TryFromSexp` | `miniextendr_api` | Convert an R `SEXP` into a Rust type (fallible) |
| `IntoR` | `miniextendr_api` | Convert a Rust type into an R `SEXP` |
| `IntoRAs` | `miniextendr_api` | Convert with explicit storage-type coercion |
| `Coerce<T>` | `miniextendr_api` | Explicit coercion between Rust representation types |
| `TypedExternal` | `miniextendr_api` | Marker for types wrapped in `ExternalPtr` (carry R-visible name + type id) |
| `IntoExternalPtr` | `miniextendr_api` | Convert a Rust value into an `ExternalPtr` SEXP |
| `AltrepLen` | `miniextendr_api` | Required for all ALTREP classes: report length |
| `AltIntegerData` / `AltRealData` / ... | `miniextendr_api` | Per-type ALTREP data-access traits |
| `AltrepExtract` | `miniextendr_api` | Abstraction over materialization (blanket impl for `TypedExternal`) |

## Procedural macros

| Macro | Kind | Purpose |
|-------|------|---------|
| `#[miniextendr]` | attribute | Export a `pub fn`, `impl` block, or `trait` to R; automatic registration via `linkme::distributed_slice` |
| `miniextendr_init!()` | function-like | Generate `R_init_<pkg>`; calls `package_init()` for panic hooks, runtime init, ALTREP setup, and routine registration |
| `#[r_ffi_checked]` | attribute | On `unsafe extern` blocks: generate checked variants (main-thread assert) and `*_unchecked` variants |
| `typed_list!(...)` | function-like | Build a validated dots spec for `#[miniextendr(dots = typed_list!(...))]` |
| `list!(...)` | function-like | Construct an R list from Rust values inline |

See [Features](../features/) for the full derive macro table (20 derives across five groups) and `#[miniextendr]` attribute options.

## Registration model

Items annotated with `#[miniextendr]` self-register at link time via `linkme::distributed_slice`. There is no `miniextendr_module!` macro -- it was removed in 2026-03-08. `miniextendr_init!()` is the only required call in `lib.rs`.

## ExternalPtr storage

`ExternalPtr<T>` stores `Box<Box<dyn Any>>`: a thin pointer in `R_ExternalPtrAddr` pointing to a fat pointer on the heap (carries the `Any` vtable). Type recovery uses `Any::downcast`, not R tag symbols. `cached_ptr` fields must have mutable provenance.
