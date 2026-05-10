# WebR / WASM support for miniextendr

Goal: build miniextendr-api (and the rpkg `miniextendr` R package) for the
`wasm32-unknown-emscripten` target, so it can be installed inside webR
(R compiled to WASM via Emscripten).

## Companion artefacts

This plan is the index. The detailed work is split across these files:

**Landed (steps 1–7 + Dockerfile, in #470's stack):**

- **`plans/wasm-registry-codegen.md`** — the linkme replacement. How we
  snapshot the runtime data (`MX_CALL_DEFS`, `MX_ALTREP_REGISTRATIONS`,
  `MX_TRAIT_DISPATCH`) on host and reconstruct it via a generated
  `wasm_registry.rs` on WASM. Owns the macro-emission changes,
  symbol-name stabilisation, cdylib writer, and the `mod` include in
  `miniextendr_init!`. Steps 2–5 + 7 dispatched there.
- **`plans/webr-dockerfile.md`** — local-dev build environment.
  `Dockerfile.webr` inheriting `ghcr.io/r-wasm/webr` digest-pinned,
  `just docker-webr-*` recipes. Landed in #476.
- **`docs/WEBR.md`** — user-facing summary of the toolchain
  requirements (target triple, why nightly, why
  `-Z build-std=std,panic_abort`). Long-lived contributor docs.

**Open follow-ups (still to do):**

- **`plans/webr-ci.md`** — three escalating tiers of CI (cargo check
  only / Docker `R CMD INSTALL` / webR Node smoke test). Tier 1 is
  the recommended starting point — small, fast, catches the common
  cfg-gating regressions. Step 6 of this doc.
- **`plans/webr-configure-and-build-rs.md`** — `rpkg/configure.ac`
  detection of `CC=emcc` (sets `CARGO_BUILD_TARGET=wasm32-unknown-emscripten`,
  nightly `+`, `-Z build-std=std,panic_abort`, side-module RUSTFLAGS,
  refuses build if `wasm_registry.rs` missing) + `rpkg/build.rs`
  validation of the snapshot's generator-version. Step 8 of this doc.
- **`plans/webr-cross-package-stubs.md`** — empty `wasm_registry.rs`
  stubs in `tests/cross-package/{consumer,producer}.pkg/src/rust/`.
  Low priority; only matters if cross-package crates ever land in
  CI's wasm32 path.

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

## Strategy: cfg-gate linkme + host-time snapshot codegen

The full design lives in **`plans/wasm-registry-codegen.md`**. One-paragraph
summary:

The host build already does a complete `cdylib` + `dyn.load` pass to write
`R/<pkg>-wrappers.R`. We extend that pass to also serialise the contents of
`MX_CALL_DEFS` / `MX_ALTREP_REGISTRATIONS` / `MX_TRAIT_DISPATCH` (the three
runtime-critical slices) into a `wasm_registry.json` snapshot, including
the symbol name of each referenced wrapper / register fn / vtable. A
`build.rs` in `miniextendr-api` reads that JSON under
`cfg(target_arch = "wasm32")` and emits a `wasm_registry.rs` to `OUT_DIR`
that re-creates the slice contents using `extern { … }` declarations
referencing the user crate's `#[no_mangle]` exports. The macros gate every
`#[distributed_slice(...)]` attribute behind `cfg_attr(not(target_arch =
"wasm32"), …)`. WASM builds compile linkme-free; the linker resolves the
externs against the same wrapper / register fns that exist today.

Symbol-name stability, vtable encoding, schema versioning,
cross-crate trait dispatch, and the alternatives we didn't pick (inventory,
ctor) — all in `plans/wasm-registry-codegen.md`.

## Steps

✅ = landed in #470's stack; ⏳ = open follow-up.

1. ✅ **Stabilise symbol names** — closed by #392, #393, #394 (altrep
   register fn `#[no_mangle]`, trait vtable `#[no_mangle]`, generic
   vtable name collision via FNV hash suffix).
2. ✅ **`cfg_attr`-gate `#[distributed_slice]` emissions** — #471.
3. ✅ **Symbol fields on entry types** — #472. `AltrepRegistration.symbol`
   + `TraitDispatchEntry.vtable_symbol`. (`MX_CALL_DEFS` already has
   `name == symbol`, no enrichment needed.)
4. ✅ **Host-time `wasm_registry.rs` writer** — #473. Pure formatter +
   C-callable entry point `miniextendr_write_wasm_registry`.
5. ✅ **Cfg-branch registry slices** — #474. `OnceLock`-backed runtime
   table on wasm32 + `install_wasm_runtime_slices(...)` public fn.
   Departure from earlier plan: snapshot lives in user-crate, not
   miniextendr-api (see commit message for rationale).
6. ⏳ **CI workflow** — see `plans/webr-ci.md`. Three escalating tiers;
   tier 1 (cargo check only) is recommended starting point.
7. ✅ **`miniextendr_init!` + `Makevars.in` wiring** — #475. Macro
   emits cfg-gated `mod __miniextendr_wasm_registry;` + install call;
   Makevars co-generates `R/<pkg>-wrappers.R` + `src/rust/wasm_registry.rs`
   in the same cdylib pass.
8. ⏳ **`rpkg/configure.ac` `CC=emcc` detection + `rpkg/build.rs`
   snapshot validation** — see `plans/webr-configure-and-build-rs.md`.

## Other open follow-ups

- ⏳ **Cross-package wasm32 stubs** — `plans/webr-cross-package-stubs.md`.
  Low priority; only the rpkg path is exercised in practice.

## Stack-ordering gotcha (from the merge cascade)

Step 7's `Makevars` calls `miniextendr_write_wasm_registry` via
`getNativeSymbolInfo`. For that symbol to appear in the cdylib's
dynamic symbol table, the `as *const ()` reference in
`miniextendr_register_routines` (added in step 5) must be present.
Step 5 must merge before step 7's CI will pass — the linker won't
pull a Rust dep-crate `#[no_mangle]` fn into a cdylib unless something
in the final crate's call graph references it. Recorded for future
reorderings of this plan's steps.

## Non-goals for this branch

- Threads / mirai / worker-thread under WASM. Emscripten pthreads exist but
  R-on-WASM is single-threaded and rwasm builds with no-std-pthreads. The
  `worker-thread` feature must be off on WASM (already feature-gated; verify).
- Vendoring story under webR (rwasm has its own vendor handling — see step 1).
- Performance optimisation (`-O3`, LTO) — get a working build first.
