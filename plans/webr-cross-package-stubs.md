# Cross-package `wasm_registry.rs` stubs

Status: **not started**. Low priority — `tests/cross-package/{consumer,producer}.pkg`
are native-only trait-ABI test crates, never deployed to webR.

## What's needed

`miniextendr_init!()` macro now emits a wasm32-cfg-gated
`#[path = "wasm_registry.rs"] mod __miniextendr_wasm_registry;` (landed
with #475). Any crate that calls `miniextendr_init!()` therefore needs
a `wasm_registry.rs` next to its `lib.rs` for the wasm32 build to
succeed.

The cross-package crates each invoke `miniextendr_init!()`:

```
$ grep -rn "miniextendr_init" tests/cross-package
tests/cross-package/consumer.pkg/src/rust/lib.rs:11:miniextendr_api::miniextendr_init!();
tests/cross-package/producer.pkg/src/rust/lib.rs:15:miniextendr_api::miniextendr_init!();
```

So:
- `tests/cross-package/consumer.pkg/src/rust/wasm_registry.rs` — empty stub
- `tests/cross-package/producer.pkg/src/rust/wasm_registry.rs` — empty stub

Both modeled on `rpkg/src/rust/wasm_registry.rs`'s stub format:

```rust
// AUTO-GENERATED — DO NOT EDIT.
// generator-version: 1
// content-hash:      0000000000000000
//
// THIS IS A STUB. Cross-package test crates aren't deployed to webR;
// the file exists only so `cargo check --target
// wasm32-unknown-emscripten` resolves the wasm32-gated `mod` decl
// emitted by `miniextendr_init!()`.

use ::miniextendr_api::ffi::R_CallMethodDef;
use ::miniextendr_api::registry::{AltrepRegistration, TraitDispatchEntry};

pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[];
pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &[];
pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[];
```

## Why low priority

- The stubs don't get loaded — they exist purely to satisfy the
  `mod` declaration. If you never `cargo check --target
  wasm32-unknown-emscripten` on consumer.pkg / producer.pkg, you
  never need them.
- Native build (the only one that matters for these crates) is unaffected
  — the `mod` is `#[cfg(target_arch = "wasm32")]`-gated, so on native
  the file isn't required.

## When to do this

- If a future PR adds wasm32 to CI and runs cargo check on
  cross-package (unlikely — they're trait-ABI tests, not deployment
  targets).
- If trait dispatch ends up needing a cross-crate `wasm_registry.rs`
  merge protocol (see `wasm-registry-codegen.md` Risks #5). At that
  point the cross-package stubs become real test fixtures, not stubs.

Until then: skip.

## Implementation

Trivial: two new files, copy-paste of rpkg's stub. ~50 lines total.
File as a follow-up issue if needed; otherwise don't bother.
