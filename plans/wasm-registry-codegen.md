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
   serialise each entry as plain data plus the *name* of the symbol it
   points at.
3. Emit a Rust source file (`wasm_registry.rs`) that re-creates the slice
   contents from `extern { fn …; static …; }` declarations. On WASM the
   linker resolves the names against the user's crate's `#[no_mangle]`
   exports.
4. On WASM, compile the user's crate with linkme **gone** — both the
   `#[distributed_slice]` declarations in `miniextendr-api/src/registry.rs`
   and the per-element `#[distributed_slice(...)]` attributes the macros
   emit. `include!("wasm_registry.rs")` provides the slice contents
   instead.

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

Two places we could emit:

### Option A — emit Rust source directly from the cdylib

The cdylib's `miniextendr_write_wrappers` extends to write
`wasm_registry.rs` next to `wrappers.R`. Pros: one step, one file format
the rest of the build already understands. Cons: cdylib has to know
about Rust syntax (string-formatting `extern "C-unwind" fn <name>(…)`
declarations), and any escape bug is a nightmare to debug because the
output doesn't compile and the cdylib is opaque.

### Option B — emit JSON manifest, build.rs converts to source

Cdylib emits `wasm_registry.json` containing structured data (entries
+ symbol names + arities). A `build.rs` in `miniextendr-api` reads it
when `target_arch = "wasm32"` and writes `OUT_DIR/wasm_registry.rs`,
which `miniextendr-api/src/registry.rs` `include!`s under the cfg.
Pros: cdylib emits trivial data, build.rs uses `quote!` which is
already a dev-dependency surface we accept. Test coverage is easy
(JSON round-trip). Cons: two file formats, two build steps.

**Recommendation: Option B.** Separation of concerns wins here —
`miniextendr_write_wrappers` already pushes the limit of "what should
live in the runtime crate." Letting it emit data and pushing codegen
into a build-time tool is cleaner. Bonus: the JSON is human-readable
in PR diffs, which makes "did I forget to regenerate?" reviewable
without compiling anything.

The JSON shape:

```json
{
  "schema_version": 1,
  "generator_version": "miniextendr-api 0.x.y",
  "manifest_hash": "sha256:…",
  "call_defs": [
    { "name": "my_fn", "symbol": "my_fn", "num_args": 2 }
  ],
  "altrep_registrations": [
    { "symbol": "__mx_altrep_reg_mytype" }
  ],
  "trait_dispatch": [
    {
      "concrete_tag": "0x…",          // mx_tag is u64, hex-encoded
      "trait_tag": "0x…",
      "vtable_symbol": "__VTABLE_FOO_FOR_BAR"
    }
  ]
}
```

`manifest_hash` is computed over the sorted entries, used by build.rs to
short-circuit regeneration when nothing has changed.

The generated `wasm_registry.rs` (sketch):

```rust
// AUTO-GENERATED by build.rs from wasm_registry.json. Do not edit.

use crate::ffi::{R_CallMethodDef, SEXP};
use crate::registry::TraitDispatchEntry;
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
mod wasm_registry {
    include!(concat!(env!("OUT_DIR"), "/wasm_registry.rs"));
    pub use {MX_CALL_DEFS_WASM as MX_CALL_DEFS, /* …rename to match */};
}

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
function or static by name should *also* set a stable name on the entry
(e.g. an extra `__symbol: &'static str` field). The cdylib write step
reads this field to emit the JSON. Without it, the cdylib only has the
fn pointer at runtime and would have to reverse-symbol-lookup
(awkward, fragile).

We can add this without bumping the public schema by keeping `__symbol`
on the entry types themselves — they're documented as private-ish.
`R_CallMethodDef` is from R proper, so we can't extend it; instead we
add a parallel slice `MX_CALL_DEF_SYMBOLS: [&'static str]` indexed
in lockstep, populated by the same macro emission. Less elegant but
keeps the FFI-layer types clean.

## Build orchestration

Three contexts:

### Native build (status quo)
`Makevars.in` builds the staticlib then the cdylib then runs Rscript to
write `R/<pkg>-wrappers.R`. Add: same Rscript also writes
`src/rust/<pkg>/wasm_registry.json` (path is derived from
`CARGO_MANIFEST_DIR`). Output is committed alongside the wrappers.

### Native build with `MINIEXTENDR_WASM_PREPARE=1`
Same as native, but also compiles `miniextendr-api` with `--target
wasm32-unknown-emscripten -Z build-std=std,panic_abort` to verify the
generated `wasm_registry.rs` actually compiles end-to-end. Wraps under
a new `just wasm-prepare` recipe.

### WASM install (under webR)
Skips the cdylib + Rscript pipeline entirely (we can't `dyn.load` a WASM
side-module from host R). Reads the committed `wasm_registry.json`
through `miniextendr-api`'s build.rs, generates `wasm_registry.rs`,
compiles. If `wasm_registry.json` is missing or has a stale hash,
build.rs `panic!`s with: "Run `just wasm-prepare` on the host first."

## CI / verification

A new GitHub Actions job:
1. `just wasm-prepare` (regenerates both `wrappers.R` and
   `wasm_registry.json`).
2. `git diff --exit-code` on those paths.
3. `cargo check --target wasm32-unknown-emscripten -Z
   build-std=std,panic_abort` on `miniextendr-api` and `rpkg/src/rust`.

Step 2 enforces "regenerate when you change Rust signatures." Step 3
catches the case where a user adds a `#[miniextendr]` item that doesn't
make it into the snapshot (hash mismatch → build.rs panic in step 3).

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
   that uses miniextendr needs its *own* `wasm_registry.json`, generated
   from its *own* cdylib write step. The tooling has to discover the
   right path (`CARGO_MANIFEST_DIR` for the user crate, not for
   `miniextendr-api`). Confirm `MINIEXTENDR_CDYLIB_WRAPPERS=1` already
   passes a path that's user-crate-relative.
4. **Schema evolution.** When we add new slice types in the future,
   `schema_version` must bump and old snapshots fail closed. Build.rs
   should refuse to compile against a snapshot whose `schema_version`
   doesn't match the running `miniextendr-api`.
5. **Cross-crate trait dispatch.** Trait impls in producer crates
   register vtables for trait IDs declared in consumer crates. The
   snapshot in producer-crate captures those entries — but when the
   consumer crate is built for WASM, *its* `wasm_registry.json` doesn't
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
3. **Cdylib snapshot writer**: extend `miniextendr_write_wrappers` to
   serialise `MX_CALL_DEFS`/`MX_ALTREP_REGISTRATIONS`/`MX_TRAIT_DISPATCH`
   plus the parallel symbol-name slices into JSON. Add the parallel
   `MX_*_SYMBOLS` slices and macro emission for them.
4. **`miniextendr-api` build.rs**: read `wasm_registry.json`, emit
   `wasm_registry.rs` to `OUT_DIR`. Hash-based short-circuit. Schema
   version check.
5. **`registry.rs` cfg branching**: `include!` the generated file under
   `cfg(target_arch = "wasm32")`, drop linkme `distributed_slice`
   declarations under that cfg. Re-export to a uniform API.
6. **`Makevars.in`** (rpkg): branch `CARGO_BUILD_TARGET=wasm32-*` →
   skip cdylib + Rscript wrapper-gen (assert the snapshot exists
   instead). Pass `--target wasm32-unknown-emscripten -Z
   build-std=std,panic_abort` to cargo. RUSTFLAGS for relocation /
   side-module as needed.
7. **`just wasm-prepare`** recipe: clean snapshot + regen + diff-check.
8. **CI job**: in the Dockerfile.webr from `webr-dockerfile.md`, run
   `just wasm-prepare` then `cargo check` on the WASM target.

Steps 1-2 are independent of WASM — they're cleanups that improve the
codebase whether or not the WASM port lands. Land those first as a
small de-risking PR.
