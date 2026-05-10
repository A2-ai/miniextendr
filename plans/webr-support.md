# WebR / WASM support for miniextendr

Goal: build miniextendr-api (and the rpkg `miniextendr` R package) for the
`wasm32-unknown-emscripten` target, so it can be installed inside webR
(R compiled to WASM via Emscripten).

## Companion artefacts

This plan is the index. The detailed work is split across three files:

- **`plans/wasm-registry-codegen.md`** — the linkme replacement. How we
  snapshot the runtime data (`MX_CALL_DEFS`, `MX_ALTREP_REGISTRATIONS`,
  `MX_TRAIT_DISPATCH`) on host and reconstruct it via a generated
  `wasm_registry.rs` on WASM. Owns the macro-emission changes,
  symbol-name stabilisation, cdylib JSON writer, and `build.rs` codegen.
  Steps 3-5 below dispatch there.
- **`plans/webr-dockerfile.md`** — the build environment.
  `Dockerfile.webr` (inheriting `ghcr.io/r-wasm/webr` digest-pinned),
  `just docker-webr-*` recipes, CI sequencing (cargo check → R CMD
  INSTALL → `rwasm::build_pkg` → webR Node smoke test). Step 6 below
  dispatches there.
- **`docs/WEBR.md`** — user-facing summary of the toolchain
  requirements (target triple, why nightly, why
  `-Z build-std=std,panic_abort`). Anything that belongs in long-lived
  contributor docs goes there, not here.

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

## Next concrete steps

1. **Read `r-wasm/rwasm`** — confirm exact cargo invocation, vendoring
   handling, and Makevars overrides it expects. Update this plan with
   findings before writing any code.
2. **Audit proc-macro emission** — list every `#[distributed_slice(...)]`
   site in `miniextendr-macros`, confirm each emits a corresponding extern
   "C" symbol (or uniquely-named pub static) usable by name from
   `wasm_registry.rs`.
3. **Replace linkme on WASM** — full design in
   `plans/wasm-registry-codegen.md`. Three sub-steps: stabilise symbol
   names (altrep + vtable `#[no_mangle]`), `cfg_attr`-gate every
   `#[distributed_slice]` attribute the macros emit, extend the cdylib
   write step to serialise a `wasm_registry.json` snapshot.
4. **`miniextendr-api` build.rs**: read `wasm_registry.json` under
   `cfg(target_arch = "wasm32")`, emit `wasm_registry.rs` to `OUT_DIR`,
   `include!` it from `registry.rs`. (Detail in
   `plans/wasm-registry-codegen.md`.)
5. **Verify**: `cargo check --target wasm32-unknown-emscripten -Z
   build-std=std,panic_abort` on `miniextendr-api` and `rpkg/src/rust`.
   Linker should resolve all wrapper / altrep / vtable extern names
   from the user crate's `#[no_mangle]` exports.
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
