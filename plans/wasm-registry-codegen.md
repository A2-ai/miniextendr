# WASM registry codegen — eliminating linkme on `wasm32-*`

Companion to `plans/webr-support.md` (which sketches this at a high level).
This plan goes deep on the *mechanism*: how we replace linkme's link-time
gather with a host-time snapshot that gets compiled in on WASM.

The user-facing problem from the parent plan in one sentence:
**linkme refuses to compile on `wasm32-*` (it `compile_error!`s in
`linkme-impl-0.3.36/src/declaration.rs:48-51`), but our runtime needs the
contents of three of its slices — `MX_CALL_DEFS`, `MX_ALTREP_REGISTRATIONS`,
`MX_TRAIT_DISPATCH`.** We need a different gather path on WASM that
produces equivalent data without linkme.

## The core idea

linkme gathers entries at link time by giving each `#[distributed_slice]`
element a unique linker section, then exposing the section's start/end
symbols as a slice. On platforms where this works it's beautiful — every
crate in the dep graph contributes to the same global slice with no
coordination. On WASM it's not implemented and won't be soon.

What linkme is doing, conceptually, is **build-time aggregation**. We can
do the same aggregation **at host build time, in our own tooling**, by:

1. Doing a complete native build of the user's crate (we already do this
   for wrapper generation — the cdylib + `dyn.load` step).
2. While we have the cdylib loaded, iterate every linkme slice and
   write a Rust source file (`wasm_registry.rs`) directly. The file
   declares each referenced symbol via `extern { fn …; static …; }` and
   constructs the runtime slices as ordinary `&[T]` constants pointing
   at those externs.
3. On WASM, compile the user's crate with linkme **gone** — both the
   `#[distributed_slice]` declarations in `miniextendr-api/src/registry.rs`
   and the per-element `#[distributed_slice(...)]` attributes the macros
   emit. `wasm_registry.rs` is included via `#[path = "wasm_registry.rs"]
   mod wasm_registry;` and provides the slice contents instead. The WASM
   linker resolves the externs against the user crate's `#[no_mangle]`
   exports — same wrappers and register fns that exist today.

The end state is symmetric with how we already handle R wrappers:
generated on host once, committed (or regenerated each build with a
diff-check), consumed at install time without re-running the host step.

## What linkme currently aggregates

From `miniextendr-api/src/registry.rs`:

| Slice | Element type | Runtime use? | Symbol that needs a name on WASM |
|---|---|---|---|
| `MX_CALL_DEFS` | `R_CallMethodDef { name, fun, numArgs }` | yes — `R_registerRoutines` | `fun` — fn pointer to a `#[no_mangle]` C wrapper. Today's name == today's R-visible name (Rust ident matches `name` field). |
| `MX_ALTREP_REGISTRATIONS` | `fn()` | yes — called eagerly at `R_init_*` | the function. Today's name: `__mx_altrep_reg_<lowercase_ident>` (`miniextendr-macros/src/altrep.rs:65-66`). Currently emitted without `#[no_mangle]` — needs fixing. |
| `MX_TRAIT_DISPATCH` | `TraitDispatchEntry { concrete_tag, trait_tag, vtable }` | yes — `universal_query` | `vtable` — `*const c_void` pointing at a `pub static` named `__VTABLE_<TRAIT>_FOR_<TYPE>` (`miniextendr-macros/src/miniextendr_impl_trait.rs:790`). Need to verify it's `#[no_mangle]`. |
| `MX_R_WRAPPERS` | `RWrapperEntry { priority, content, source_file }` | no — host wrapper-gen only | n/a (string data, written to `.R` file once on host) |
| `MX_MATCH_ARG_CHOICES` | `MatchArgChoicesEntry { placeholder, choices_str, preferred_default }` | no | n/a (write-time substitution) |
| `MX_MATCH_ARG_PARAM_DOCS` | `MatchArgParamDocEntry { placeholder, several_ok, choices_str }` | no | n/a |
| `MX_CLASS_NAMES` | `ClassNameEntry { rust_type, r_class_name, class_system }` | no | n/a |

So only the **first three** need to round-trip through `wasm_registry.rs`.
The other four are host-only — we can leave them as empty `&[T]` constants
on WASM (or skip them entirely with `cfg`) and nothing breaks at runtime.

## Stable name story (prerequisite work)

For codegen to reference the right symbols, the macros have to emit
**deterministic, externally-addressable** names today. State of play, from
inspecting `miniextendr-macros/src`:

### ✅ C wrappers — already stable
`c_wrapper_builder.rs:408,436,535,574` emit `#[unsafe(no_mangle)] pub
extern "C-unwind" fn <name>(...)`. Name == R-visible name. Already
ABI-stable for `wasm_registry.rs` to declare via `extern { fn <name>(…)
-> SEXP; }`.

### ⚠️ ALTREP register fns — need `#[no_mangle]`
`altrep.rs:67-74` emits:

```rust
fn __mx_altrep_reg_<lowercase_ident>() {
    <#ident as RegisterAltrep>::get_or_init_class();
}
```

Plain `fn`, no `pub`, no `#[no_mangle]`. The linkme entry takes its
address, and on host that's fine because the linker can intra-crate
resolve it. On WASM we need `wasm_registry.rs` (in `miniextendr-api`) to
reference it from a *different* compilation unit, so the function must
be `#[unsafe(no_mangle)] pub extern "C" fn __mx_altrep_reg_<…>()`.
Cheap change, no behaviour delta on host either.

### ⚠️ Trait vtables — need to confirm `#[no_mangle]`
`miniextendr_impl_trait.rs:790` chooses `__VTABLE_{TRAIT}_FOR_{TYPE}`
(`format_ident!`). Need to confirm the static is `pub` + `#[no_mangle]`
+ has a stable repr (`#[repr(C)]`) so `extern { static …: u8; }` (or
typed alias) works. If not, add the attributes.

### Naming collisions
`__mx_altrep_reg_<lowercase>` collides if two ALTREP types in the same
crate differ only in case (`MyType` vs `Mytype`). Vanishingly unlikely
in practice but a real bug today, independent of WASM. Fix opportunistically:
include the source file's hash or the full case-preserved ident with
`_` separators.

`__VTABLE_<TRAIT>_FOR_<TYPE>` collides if a user implements the same
trait twice on the same type with different `label = "..."` — yes, we
support that (CLAUDE.md `MXL009`). The macro must already disambiguate;
verify the disambiguator is part of the static name and not just the
linkme entry payload.

These are prerequisites whether or not we ever build for WASM. Filing
as #-issue when the implementation lands; mention in the PR.

## The snapshot format

The cdylib emits **Rust source directly** — one file, no intermediate.
The earlier draft of this plan routed through a JSON manifest that a
`build.rs` would convert to Rust; that's overengineered. The compiler
is the tool that needs to consume names and turn them into linker
references, so we may as well hand it Rust source and skip the
intermediate format. (We can't skip the codegen step entirely with
`include_bytes!` of a binary blob — the linker needs `extern { fn name; }`
declarations to resolve symbols statically; bytes-at-runtime would mean
`dlsym`-style lookups which WASM side modules don't cleanly support.)

The cdylib already string-formats arbitrary R source for `wrappers.R`;
formatting Rust source for `wasm_registry.rs` is the same kind of work,
not noticeably riskier.

The generated `wasm_registry.rs` (sketch — committed to the user crate
as `<pkg>/src/rust/<crate>/src/wasm_registry.rs`):

```rust
// AUTO-GENERATED — DO NOT EDIT.
// Produced on host by `just wasm-prepare` (the cdylib write step).
// Compiled on wasm32-* targets in place of the linkme distributed_slices.

use crate::ffi::{R_CallMethodDef, SEXP};
use crate::registry::TraitDispatchEntry;
use crate::abi::mx_tag;
use core::ffi::c_void;

unsafe extern "C-unwind" {
    fn my_fn(call: SEXP, a: SEXP, b: SEXP) -> SEXP;
    fn __mx_altrep_reg_mytype();
    static __VTABLE_FOO_FOR_BAR: u8;
}

pub static MX_CALL_DEFS_WASM: &[R_CallMethodDef] = &[
    R_CallMethodDef {
        name: c"my_fn".as_ptr(),
        fun: Some(unsafe {
            core::mem::transmute::<unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP, _>(my_fn)
        }),
        numArgs: 2,
    },
];

pub static MX_ALTREP_REGISTRATIONS_WASM: &[fn()] = &[__mx_altrep_reg_mytype];

pub static MX_TRAIT_DISPATCH_WASM: &[TraitDispatchEntry] = &[
    TraitDispatchEntry {
        concrete_tag: mx_tag::from_u64(0x…),
        trait_tag: mx_tag::from_u64(0x…),
        vtable: &__VTABLE_FOO_FOR_BAR as *const u8 as *const c_void,
    },
];

// First line of the file is a generator-version + content hash header
// (commented). When Rust signatures change, the hash changes; CI fails
// `git diff --exit-code` until `just wasm-prepare` is rerun.
```

`registry.rs` then does:

```rust
#[cfg(not(target_arch = "wasm32"))]
mod linkme_registry {
    use linkme::distributed_slice;
    #[distributed_slice] pub static MX_CALL_DEFS: [R_CallMethodDef];
    // …etc
}

#[cfg(target_arch = "wasm32")]
#[path = "wasm_registry.rs"]
mod wasm_registry;

#[cfg(not(target_arch = "wasm32"))]
pub use linkme_registry::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_registry::*;
```

The `miniextendr_register_routines` body that iterates the slices doesn't
change — it still reads `MX_CALL_DEFS.iter()`, just from a `&[T]` instead
of a `linkme::DistributedSlice<[T]>`.

## Macro changes

The proc-macro emission changes minimally. For each slice the macros
target today:

```rust
#[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS)]
#[linkme(crate = ::miniextendr_api::linkme)]
static MY_ENTRY: R_CallMethodDef = R_CallMethodDef { … };
```

becomes:

```rust
#[cfg_attr(not(target_arch = "wasm32"),
    ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS),
    linkme(crate = ::miniextendr_api::linkme))]
static MY_ENTRY: R_CallMethodDef = R_CallMethodDef { … };
```

(Both `cfg_attr` items in one tuple — `cfg_attr` accepts that form.)

On WASM the static is still emitted (it's harmless dead data, but `cargo
check` passes); on non-WASM the linkme attributes attach. The static
*itself* never changes shape, which means the macros stay simple — no
"emit different code for WASM" branch.

The proc-macro will need a touch more: every entry that references a
function or static by name should *also* carry that name as a string,
so the cdylib write step has it without reverse-symbol-lookup. We add
a parallel slice `MX_CALL_DEF_SYMBOLS: [&'static str]` (indexed
in lockstep with `MX_CALL_DEFS`) populated by the same macro emission.
Same trick for `MX_ALTREP_REGISTRATION_SYMBOLS` and
`MX_TRAIT_DISPATCH_SYMBOLS`. The FFI-layer types (`R_CallMethodDef`)
stay clean.

## Build orchestration

Three contexts:

### Native build (status quo)
`Makevars.in` builds the staticlib then the cdylib then runs Rscript to
write `R/<pkg>-wrappers.R`. Add: the same Rscript invocation also writes
`src/rust/<crate>/src/wasm_registry.rs` next to it, derived from the same
in-memory linkme slices. Output is committed alongside the R wrappers.

### Native build with `MINIEXTENDR_WASM_PREPARE=1`
Same as native, but also compiles `miniextendr-api` with `--target
wasm32-unknown-emscripten -Z build-std=std,panic_abort` to verify the
generated `wasm_registry.rs` actually compiles end-to-end. Wraps under
a new `just wasm-prepare` recipe.

### WASM install (under webR)
Skips the cdylib + Rscript pipeline entirely (we can't `dyn.load` a WASM
side-module from host R). Compiles the committed `wasm_registry.rs`
straight through `cargo`. If the file is missing or its content hash
header doesn't match the rest of the source, a small build.rs check
panics with: "Run `just wasm-prepare` on the host first." (The build.rs
is a tiny existence/hash check, not a code generator — that runs on
host only.)

## CI / verification

A new GitHub Actions job:
1. `just wasm-prepare` (regenerates both `wrappers.R` and
   `wasm_registry.rs`).
2. `git diff --exit-code` on those paths.
3. `cargo check --target wasm32-unknown-emscripten -Z
   build-std=std,panic_abort` on `miniextendr-api` and `rpkg/src/rust`.

Step 2 enforces "regenerate when you change Rust signatures." Step 3
catches the case where a user adds a `#[miniextendr]` item that doesn't
make it into the snapshot (the new wrapper symbol won't be referenced
by `wasm_registry.rs`, so the runtime registration ends up missing the
entry — which is the same kind of bug a stale `wrappers.R` produces
today, caught the same way).

## Why not <X>?

### `inventory` crate
`inventory` does runtime registration via `ctor`-style init functions
running on module load. Maybe works on `wasm32-unknown-emscripten`
(emscripten supports `.init_array`), but I haven't verified — and the
verification cost is high (need to actually run a webR session and
check that constructors fired). Also: `inventory` uses a `Mutex` /
`OnceCell` global registry, costing a dynamic allocation per registered
item. Linkme's `&[T]` model is zero-cost at runtime. Snapshot codegen
matches that.

### `ctor` crate directly
Same uncertainty about emscripten + same runtime allocation cost.

### Always-runtime registration (manual)
Could replace linkme everywhere with explicit `pkg_init() { registry::
register_my_fn(); registry::register_my_altrep(); … }` calls. This is
what you'd do without a proc-macro, and it's terrible — every new
`#[miniextendr]` item requires touching a central list, which is
exactly what linkme exists to avoid. Skip.

### Snapshot the `target/` artefacts
Could ship a prebuilt `wasm32-unknown-emscripten` `.a` of
`miniextendr-api` and reuse it instead of rebuilding. That's downstream
of this work — once the snapshot codegen exists and the WASM build
works, we *could* cache the `.a` for CI speed. Not a substitute for
this plan.

## Risks and unknowns

1. **`#[no_mangle]` on monomorphised generics.** Trait vtables for
   generic types might be mangled per-instance. If
   `__VTABLE_FOO_FOR_BAR<T>` exists per-T, codegen needs one entry per
   monomorphisation. Audit before implementing.
2. **`extern "C-unwind"` on emscripten.** The cdylib wrappers are
   `extern "C-unwind"` because Rust panics need to propagate back to R.
   On `wasm32-unknown-emscripten` with `panic = "abort"`, unwinding is
   a no-op. The signature should still match — `extern "C-unwind"` and
   `extern "C"` are ABI-compatible when nothing actually unwinds. But
   the linker may still require exact-match decls. Test.
3. **Per-crate snapshots, not per-build snapshots.** Each user crate
   that uses miniextendr needs its *own* `wasm_registry.rs`, generated
   from its *own* cdylib write step. The tooling has to discover the
   right path (`CARGO_MANIFEST_DIR` for the user crate, not for
   `miniextendr-api`). Confirm `MINIEXTENDR_CDYLIB_WRAPPERS=1` already
   passes a path that's user-crate-relative.
4. **Schema evolution.** When we add new slice types in the future,
   the generator emits a "generator version" comment in the file
   header. The build.rs check refuses to compile a `wasm_registry.rs`
   whose generator version is older than the consuming
   `miniextendr-api`.
5. **Cross-crate trait dispatch.** Trait impls in producer crates
   register vtables for trait IDs declared in consumer crates. The
   snapshot in producer-crate captures those entries — but when the
   consumer crate is built for WASM, *its* `wasm_registry.rs` doesn't
   know about producer-crate's vtables. The consumer's
   `MX_TRAIT_DISPATCH` slice ends up empty for its own use. Solution:
   each crate's snapshot covers entries *its* code defines, and at
   final link time the multiple `wasm_registry.rs` files (one per
   crate) all compile in. Sounds workable, but I want to draw out the
   crate-graph case before implementing — file as the first task in
   the implementation PR.

## Implementation order (when we get to it)

1. **Stabilise symbol names** (small PR, no WASM dependency):
   - `altrep.rs`: `pub extern "C" fn` + `#[unsafe(no_mangle)]` for
     `__mx_altrep_reg_*`.
   - `miniextendr_impl_trait.rs`: confirm/fix `pub static` +
     `#[unsafe(no_mangle)]` + `#[repr(C)]` on `__VTABLE_*` items.
   - Audit collision risks (case-fold collisions in altrep names,
     label-disambiguator in vtable names).
2. **Macro emission**: add `cfg_attr(not(target_arch = "wasm32"), …)`
   wrappers around every `#[distributed_slice(...)]` attribute.
   Native builds unchanged; WASM builds have linkme-free macro output.
   Verify with `cargo check --target=wasm32-unknown-emscripten -Z
   build-std=std,panic_abort` that crates compile (won't link without
   the next step).
3. **Add `MX_*_SYMBOLS` slices**: parallel `&'static str` slices in
   lockstep with the runtime-critical ones, populated by macros so
   the cdylib write step has the symbol names without reverse lookup.
4. **Cdylib snapshot writer**: extend `miniextendr_write_wrappers` to
   format `<crate>/src/wasm_registry.rs` directly (Rust source, not
   JSON) using the runtime slices + the new symbol slices. Output
   includes a generator-version + content-hash comment header.
5. **`registry.rs` cfg branching**: `#[path = "wasm_registry.rs"] mod
   wasm_registry;` under `cfg(target_arch = "wasm32")`, drop linkme
   `distributed_slice` declarations under that cfg. Re-export to a
   uniform API. A tiny `build.rs` checks the file exists and the
   generator version matches `miniextendr-api`'s; panics with a
   "Run `just wasm-prepare`" hint otherwise.
6. **`Makevars.in`** (rpkg): branch `CARGO_BUILD_TARGET=wasm32-*` →
   skip cdylib + Rscript wrapper-gen (assert the snapshot exists
   instead). Pass `--target wasm32-unknown-emscripten -Z
   build-std=std,panic_abort` to cargo. RUSTFLAGS for relocation /
   side-module as needed.
7. **`just wasm-prepare`** recipe: regen + diff-check both `wrappers.R`
   and `wasm_registry.rs`.
8. **CI job**: in the Dockerfile.webr from `webr-dockerfile.md`, run
   `just wasm-prepare` then `cargo check` on the WASM target.

Steps 1-2 are independent of WASM — they're cleanups that improve the
codebase whether or not the WASM port lands. Land those first as a
small de-risking PR.
