# WebR / WASM support for miniextendr

Goal: build miniextendr-api (and the rpkg `miniextendr` R package) for the
`wasm32-unknown-emscripten` target, so it can be installed inside webR
(R compiled to WASM via Emscripten).

## Companion artefacts

This plan covers the source-side port (linkme gating, codegen of a
`wasm_registry.rs`, `Makevars.in` / `configure.ac` branching). Two related
documents own the rest:

- **`docs/WEBR.md`** — user-facing summary of the toolchain requirements
  (target triple, why nightly, why `-Z build-std=std,panic_abort`).
  Anything that belongs in long-lived contributor docs goes there, not
  here.
- **`plans/webr-dockerfile.md`** — the build environment. Plans
  `Dockerfile.webr` (inheriting `ghcr.io/r-wasm/webr` digest-pinned),
  the `just docker-webr-*` recipes, and the CI sequencing
  (cargo check → R CMD INSTALL → `rwasm::build_pkg` → webR Node smoke
  test). Step 6 below dispatches there.

When information here goes stale, prefer to delete it and link out
rather than duplicate.

## How webR builds R packages (investigation summary)

Reference: `.webr/` — a clone of <https://github.com/r-wasm/webr> in this repo.

### Toolchain
- Build host: Linux x86_64 with the official `ghcr.io/r-wasm/flang-wasm:main`
  base image (provides Emscripten SDK + LLVM flang for Fortran).
- Rust nightly with `rustup target add wasm32-unknown-emscripten` and
  `--component rust-src` (see `.webr/Dockerfile:46-53`). The image installs a
  fake `rustc`/`cargo` Debian package (version 99.0) so apt-installed R
  packages don't pull the distro toolchain over the rustup one.
  Nightly is **mandatory**, not stylistic — `rust-src` enables
  `cargo -Z build-std=std,panic_abort`, which we need to (a) rebuild
  `std` against webR's pinned Emscripten ABI rather than rustup's
  snapshot, and (b) get `panic = "abort"` applied to `std` itself so the
  panic strategy is consistent across the call graph. Full rationale in
  `docs/WEBR.md`.
- Native R (`rig add 4.5.1`) is installed at `/opt/R/current/bin/R`. webR then
  builds a *separate* R-for-WASM in `host/` and `wasm/` subtrees. Packages are
  installed against the WASM R via the host R binary acting as the loader.
- Two R packages drive package builds: `pak` (dep resolution) and
  `r-wasm/rwasm` (the actual `cargo`/`emcc` orchestration).

### Per-package build flow
The webR `webr` R package itself is the canonical example
(`.webr/packages/webr/`, built by `.webr/packages/Makefile`):

```make
WASM_TOOLS="$(TOOLS)" \
R_SOURCE="$(R_SOURCE)" \
R_MAKEVARS_USER="$(WEBR_ROOT)/packages/webr-vars.mk" \
$(R_HOST_EXE) CMD INSTALL --library="$(R_WASM_LIB)" $(notdir $@) \
  --no-docs --no-test-load --no-staged-install
```

The interesting bit is `R_MAKEVARS_USER=webr-vars.mk`, which redefines:
- `CC=emcc`, `CXX=em++`, `FC=emfc`, `AR=emar`
- `CFLAGS = -std=gnu11 -DNDEBUG <WASM_OPT> -fvisibility=default …`
- `LDFLAGS = -s SIDE_MODULE=1 -s WASM_BIGINT -s ASSERTIONS=1 …` (relocatable
  WASM side modules — that's how R's `dyn.load()` finds package symbols)
- Strips `LIBR`, `LIBINTL`, `STRIP_*` to no-ops.

So an R package's `src/Makevars` is honoured; the package author just has to
make sure `src/Makevars` cooperates with `CC=emcc`/`LDFLAGS=-s SIDE_MODULE=1`.

For Rust packages, `rwasm` adds a Cargo step that (presumably — confirm by
reading `r-wasm/rwasm` next) runs `cargo +nightly build
--target=wasm32-unknown-emscripten -Z build-std=std,panic_abort` and links the
resulting `.a` into the side-module via `emcc`.

### Concrete requirements for miniextendr to build under webR
1. **Cargo target**: `wasm32-unknown-emscripten` (not `wasm32-unknown-unknown`
   — we need libc / pthreads stubs from emscripten).
2. **Toolchain**: nightly with `-Z build-std` (the prebuilt sysroot for
   `wasm32-unknown-emscripten` is missing items rwasm needs).
3. **Makevars must be flexible**: `CC` will already be `emcc` from
   `webr-vars.mk`. Our `Makevars.in` invokes `cargo` directly, so we need a
   path where `CARGO_BUILD_TARGET` is set to `wasm32-unknown-emscripten` and
   the resulting staticlib is linked by `emcc` — not by R's normal linker.
4. **No cdylib build at install time**: R wrapper generation happens via
   `dyn.load()` of the cdylib. That doesn't work in WASM-on-host (the cdylib
   is WASM, the host R is native). Wrappers must be generated on the host
   *before* the WASM build, then carried into the WASM install as
   pre-generated R source.
5. **No host execution during install**: rwasm runs install with
   `--no-test-load`. Anything that requires loading the side-module on the
   host (e.g. `Rscript -e "dyn.load(...)"`) must be skipped.
6. **WASM-friendly link flags**: Cargo for `wasm32-unknown-emscripten` already
   emits `*.a` archives, but `RUSTFLAGS` needs `-C relocation-model=pic` and
   `-C link-args=-s SIDE_MODULE=1` to play with emscripten's side-module
   linking — verify experimentally.

Open question: exact rwasm invocation and how it handles vendoring + offline.
Action: read `r-wasm/rwasm` repo contents in a follow-up turn.

## linkme on WASM

Hard fact: `linkme` does **not** support `wasm32-unknown-emscripten` (or
`wasm32-unknown-unknown`, or `wasm32-wasip1/p2`). Verified at
`rpkg/vendor/linkme-impl-0.3.36/src/declaration.rs:48-51`:

```rust
let msg = "distributed_slice is not implemented for this platform";
let error = Error::new_spanned(&input, msg);
let unsupported_platform = error.to_compile_error();
```

The supported `target_os` list is uefi/windows/linux/macos/ios/tvos/android/
fuchsia/illumos/freebsd/openbsd/psp/none — emscripten is none of these and
falls through to the `unsupported_platform` arm. So any crate that mentions
`#[distributed_slice]` (declaration *or* element registration) fails to
compile on `wasm32-unknown-emscripten`.

### Where miniextendr touches linkme

Two surfaces:

**Slice declarations (in `miniextendr-api/src/registry.rs`):**
| Slice | Used by | Runtime-critical? |
|---|---|---|
| `MX_CALL_DEFS` | `miniextendr_register_routines` → `R_registerRoutines` | **YES** — without this, `.Call("foo", ...)` from R has nothing to dispatch to |
| `MX_ALTREP_REGISTRATIONS` | `miniextendr_register_routines` (calls each `fn()`) | **YES** — ALTREP class registration is required at package init for any user ALTREP type to round-trip via readRDS |
| `MX_TRAIT_DISPATCH` | `universal_query` (runtime trait dispatch) | **YES** — needed for trait-on-typed-ptr lookups |
| `MX_R_WRAPPERS` | `write_r_wrappers_to_file` (host wrapper-gen only) | NO at runtime |
| `MX_MATCH_ARG_CHOICES` | `write_r_wrappers_to_file` (placeholder substitution) | NO at runtime |
| `MX_MATCH_ARG_PARAM_DOCS` | `write_r_wrappers_to_file` (placeholder substitution) | NO at runtime |
| `MX_CLASS_NAMES` | `write_r_wrappers_to_file` (placeholder substitution) | NO at runtime |

**Slice element emission (in `miniextendr-macros`):** every `#[miniextendr]`,
`#[miniextendr] impl`, `#[miniextendr] impl Trait for Type`, `#[derive(Vctrs)]`,
`#[derive(AltrepInteger/...)]`, and the `MX_MATCH_ARG_*` substatic helpers
emit `#[::miniextendr_api::linkme::distributed_slice(...)]` attributes. So
*every* user crate that uses miniextendr inherits the linkme requirement
transitively — gating only `miniextendr-api` is not enough.

The user's framing — "linkme is just for wrapper generation" — is partially
correct (3 of the 7 slices are write-time only) but **`MX_CALL_DEFS`,
`MX_ALTREP_REGISTRATIONS`, and `MX_TRAIT_DISPATCH` are runtime-critical**.
We need a different runtime registration mechanism on WASM, not just "skip
wrapper-gen".

## Strategy: feature-gate linkme + host-time codegen of a `wasm_registry.rs`

### High-level plan
1. Introduce a `wasm` cfg / feature in `miniextendr-api`. When active:
   - Don't import `linkme`.
   - Don't declare any `#[distributed_slice]` statics.
   - Replace each slice with a manually-populated `&[T]` constant that is
     `include!`'d from a generated file `wasm_registry.rs`.
2. Make the proc-macros `cfg`-aware: when emitting a slice element, gate the
   `#[distributed_slice]` attribute on `not(target_arch = "wasm32")` (or on
   the absence of a `wasm` cfg flag), and additionally emit a stable extern
   symbol or `pub fn`/`pub static` that the host wrapper-gen step can collect.
3. Extend the existing `miniextendr_write_wrappers` cdylib entry point so it
   *also* writes the `wasm_registry.rs` file when `MINIEXTENDR_CDYLIB_WRAPPERS`
   is set. The file content is built from the same in-memory linkme slices,
   serialised as static-array Rust source.
4. In `Makevars.in` (rpkg), when `CARGO_BUILD_TARGET=wasm32-unknown-emscripten`:
   - Skip the cdylib build and `dyn.load`-based wrapper-gen (host can't
     dyn.load a WASM side-module). Wrappers + `wasm_registry.rs` must already
     exist on disk, generated by a prior **host** build.
   - Pass `--features miniextendr-api/wasm` (or equivalent cfg) to cargo.
5. Add a `just wasm-prepare` recipe: do a host build with
   `MINIEXTENDR_CDYLIB_WRAPPERS=1`, capture
   `R/<pkg>-wrappers.R` and `src/rust/<pkg>/src/wasm_registry.rs`, commit
   them. Add a CI check that diffs them on every PR.

### What `wasm_registry.rs` looks like (sketch)

```rust
// AUTO-GENERATED — do not edit. Produced on host by `just wasm-prepare`.
// Used in place of linkme `#[distributed_slice]` on wasm32 targets.

use crate::ffi::R_CallMethodDef;
use crate::registry::{TraitDispatchEntry, /* ... */};

extern "C" {
    fn __miniextendr_call_my_fn(call: SEXP, args: SEXP) -> SEXP;
    fn __miniextendr_register_altrep_MyType();
    // ... one extern per registered item, one per crate
}

pub static MX_CALL_DEFS: &[R_CallMethodDef] = &[
    R_CallMethodDef {
        name: c"my_fn".as_ptr(),
        fun: Some(unsafe { core::mem::transmute(__miniextendr_call_my_fn as *const ()) }),
        numArgs: 2,
    },
    // ...
];

pub static MX_ALTREP_REGISTRATIONS: &[fn()] = &[
    __miniextendr_register_altrep_MyType,
    // ...
];

pub static MX_TRAIT_DISPATCH: &[TraitDispatchEntry] = &[ /* ... */ ];
```

The runtime slices on WASM are just `&[T]` instead of
`distributed_slice<T>`, but the iterator API
(`MX_CALL_DEFS.iter()`) is identical — so `miniextendr_register_routines`
needs no changes once the slice type is unified.

The proc macros must, in addition to the existing `#[distributed_slice]`
emission, ensure that the wrapper functions and altrep registration functions
have **stable, predictable extern names** so the generated file's `extern "C"
{}` block can refer to them. The current `__miniextendr_call_*` names appear
to follow this convention already (verify).

### Open questions / unknowns

- Does the proc-macro currently emit deterministic, stable C-ABI names for
  every wrapper function and altrep `register_*` function? If not, that's a
  prerequisite for the codegen approach above.
- What about *user* crates that consume miniextendr? They also use linkme
  via the macros — so user crates need the same `wasm` cfg gating, and they
  need their own `wasm_registry.rs` generated on host. The simplest answer:
  the cdylib's existing `miniextendr_write_wrappers` already iterates the
  full slice (which includes user-crate entries because the cdylib links
  the whole user staticlib). So the same dyn.load-on-host step that produces
  `<pkg>-wrappers.R` can produce `wasm_registry.rs` next to it.
- Trait dispatch entry: `vtable: *const c_void` is a function-pointer-shaped
  static reference. Need to encode it as `&FOO_VTABLE as *const _ as *const
  c_void` in the generated file, with `extern { static FOO_VTABLE: …; }`
  declarations. Each vtable type is monomorphised — possibly need to expose
  vtable-type aliases via a stable path.
- `MatchArgChoicesEntry` etc. are write-time only — they don't need to be
  in `wasm_registry.rs` at all. WASM can leave them empty.
- `RWrapperEntry` has the same property — empty on WASM.
- Whether `wasm32-unknown-emscripten` actually allows static initialisers
  with function pointers in `static` items: yes, Cargo does support this on
  emscripten (no `pthread_once` needed; the linker emits a regular .data
  segment).

## Next concrete steps

1. **Read `r-wasm/rwasm`** — confirm exact cargo invocation, vendoring
   handling, and Makevars overrides it expects. Update this plan with
   findings before writing any code.
2. **Audit proc-macro emission** — list every `#[distributed_slice(...)]`
   site in `miniextendr-macros`, confirm each emits a corresponding extern
   "C" symbol (or uniquely-named pub static) usable by name from
   `wasm_registry.rs`.
3. **Prototype**: in this branch, gate `linkme` import +
   `#[distributed_slice]` declarations in `registry.rs` behind a
   `not(target_arch = "wasm32")` (cfg, not feature — feature can leak across
   crates and we want the gate to be automatic). Stub the slices with empty
   `&[T]` literals on `wasm32`. Verify `cargo check
   --target=wasm32-unknown-emscripten -Z build-std` builds the api crate
   alone. (Won't link, but compile-error == green.)
4. **Gate proc-macro emission**: macros must emit
   `#[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::
   distributed_slice(...))]` (and possibly omit the linkme-crate-rename
   attribute on wasm). Re-run step 3 with the rpkg user crate.
5. **Codegen `wasm_registry.rs`** from the existing `miniextendr_write_wrappers`
   path. Plumb a second output file alongside `R/<pkg>-wrappers.R`.
6. **Docker + CI**: implement the build environment per
   `plans/webr-dockerfile.md` — single-stage `Dockerfile.webr` inheriting
   `ghcr.io/r-wasm/webr` (digest-pinned), layering `just`, `autoconf`,
   `clang`/`libclang-dev`, `pkg-config`. Add `just docker-webr-build` /
   `just docker-webr-shell` recipes. The CI job exercises this image
   through cargo check → R CMD INSTALL → `rwasm::build_pkg` → webR Node
   smoke test, gated on a `paths:` filter so docs-only PRs don't fire it.
7. **rpkg `Makevars.in`**: branch on `CARGO_BUILD_TARGET=wasm32-*` to skip
   cdylib + Rscript wrapper-gen and pass the right `RUSTFLAGS`.
8. **`configure.ac`**: detect `CC=emcc` (or an explicit `--with-wasm`
   override) and set `CARGO_BUILD_TARGET=wasm32-unknown-emscripten` +
   `IS_WASM_INSTALL=true`. Refuse to build if `wasm_registry.rs` /
   wrappers are missing (with a pointer to `just wasm-prepare`).

## Non-goals for this branch

- Threads / mirai / worker-thread under WASM. Emscripten pthreads exist but
  R-on-WASM is single-threaded and rwasm builds with no-std-pthreads. The
  `worker-thread` feature must be off on WASM (already feature-gated; verify).
- Vendoring story under webR (rwasm has its own vendor handling — see step 1).
- Performance optimisation (`-O3`, LTO) — get a working build first.
