// AUTO-GENERATED — DO NOT EDIT.
//
// generator-version: 1
// content-hash:      0000000000000000
//
// THIS IS A STUB. The cross-package test crates (consumer.pkg / producer.pkg)
// are native-only trait-ABI fixtures and are never deployed to webR. This file
// exists solely so `cargo check --target wasm32-unknown-emscripten` can resolve
// the wasm32-gated `mod __miniextendr_wasm_registry;` that `miniextendr_init!()`
// emits (see miniextendr-macros/src/lib.rs). The three slices are intentionally
// empty — a real snapshot is only produced for deployable crates by
// `miniextendr_write_wasm_registry` during the host cdylib pass.
//
// See #493. Cross-crate trait dispatch under wasm_registry (#495) is the
// follow-up that would make these slices non-empty.

use ::miniextendr_api::registry::{AltrepRegistration, TraitDispatchEntry};
use ::miniextendr_api::sys::R_CallMethodDef;

pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[];
pub static MX_ALTREP_REGISTRATIONS_WASM: &[AltrepRegistration] = &[];
pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[];
