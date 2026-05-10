+++
title = "WebR / WASM support"
weight = 50
description = "Building miniextendr for webR — R compiled to WebAssembly via Emscripten."
+++

Building miniextendr for [webR](https://docs.r-wasm.org/webr/latest/) — R
compiled to WebAssembly via Emscripten.

**Status: supported for local dev.** `wasm32-unknown-emscripten` cargo-check
runs on every PR (`.github/workflows/webr.yml`); a local
`just docker-webr-smoke` recipe drives the full install path inside the
pinned webR Docker image and runs the testthat suite under wasm. Open
follow-ups (CI tier 2/3, `link_to_r()` gating, cross-package stubs) are
tracked in `plans/webr-support.md`.

## Target

`wasm32-unknown-emscripten` — *not* `wasm32-unknown-unknown`. We need
Emscripten's libc/pthread shims because R itself relies on them; webR's
build of R links against `emcc`'s sysroot.

## Why Rust nightly is mandatory

webR's reference Dockerfile installs nightly Rust with `--component rust-src`
and `--target wasm32-unknown-emscripten`. The `rust-src` component is the
giveaway: it enables `cargo -Z build-std`, which is nightly-only. Two reasons
we genuinely need it (not just "the upstream uses it, so we copy"):

### 1. Emscripten ABI must match the active `emcc`

`rustup target add wasm32-unknown-emscripten` ships a precompiled `std`,
but that `std` was built against whatever Emscripten was current when the
toolchain snapshot was cut. The Emscripten ABI shifts between releases —
libc shim layout, exception model (Wasm exceptions vs JS exceptions),
filesystem layer, syscall numbering. webR pins its own Emscripten via the
`ghcr.io/r-wasm/flang-wasm` base image, and that version routinely diverges
from what rustup's snapshot saw.

Mismatched `std` against active `emcc` produces either link-time symbol
errors or, worse, runtime UB in the libc layer. The only robust fix is to
rebuild `std` from source against the live Emscripten toolchain — which is
exactly what `-Z build-std=std,panic_abort` does. Stable `cargo` cannot do
this.

### 2. `panic = "abort"` for `std` itself

R-on-WASM doesn't unwind the way native targets do — Emscripten's exception
support is its own world, and `rwasm` builds with `panic = "abort"` to dodge
the entire issue. `Cargo.toml`'s `[profile.*] panic = "abort"` only affects
*your* crate; the precompiled `std` shipped by rustup is built with
`panic = "unwind"` and stable Cargo can't relink it. `-Z build-std=std,
panic_abort` rebuilds `std` itself with the matching panic strategy, so the
panic-abort cfg is consistent across the whole call graph.

### Knock-on consequence

Anything we ship that targets webR is implicitly nightly. We don't need
to bend over backwards to keep the WASM code path stable-clean — feature
gates that require nightly cargo are fine on this path, as long as the
native (non-WASM) build remains stable-buildable.

## Building locally

Everything lives inside the `Dockerfile.webr` image (inherits
`ghcr.io/r-wasm/webr` digest-pinned, layers `just`/`autoconf`/`cargo-limit`).
amd64-only — Apple Silicon runs it under Rosetta, slow but works.

```bash
just docker-webr-build         # one-time image build (~5–10 min cold)
just docker-webr-shell         # interactive shell, repo bind-mounted at /work
just docker-webr-test          # cargo check miniextendr-api on wasm32 (fast)
just docker-webr-smoke         # full smoke: build wasm side-module + load in
                               # webR Node session + run testthat suite
```

`just docker-webr-smoke` (`tests/webr-smoke.sh`) drives three phases inside
the container, then prints a testthat pass/fail/skip summary:

1. **Native `R CMD INSTALL` of `rpkg`** against `/opt/R/current/bin/R` to run
   the cdylib pass and regenerate `rpkg/src/rust/wasm_registry.rs` — the
   committed version is a stub (zero-length slices, content-hash
   `0000000000000000`). Without this step the wasm build technically
   succeeds but the package registers zero R routines.
2. **wasm32 install** — `CC=emcc bash rpkg/configure` followed by
   `R CMD INSTALL --no-test-load --no-staged-install` against
   `/opt/webr/host/R-4.5.1/bin/R` (webR's own host R) with
   `R_MAKEVARS_USER=/opt/webr/packages/webr-vars.mk`. Result lands at
   `/opt/webr/wasm/R-4.5.1/lib/R/library/miniextendr/`.
3. **webR Node session** — `@r-wasm/webr` linked from `/opt/webr/src`,
   NODEFS-mounts the wasm R lib tree, calls `library(miniextendr)`, then
   runs `testthat::test_local()`. Many tests fail under wasm (worker
   thread / fork / threading assumptions); the script reports counts and
   exits 0 as long as the package itself loads.

First cold run is **1–2 hours** on Apple Silicon (Rosetta amd64 + cargo
wasm32 build). Subsequent runs reuse the docker image and most cargo
artefacts.

## How `CC=emcc` cooperates with our build

webR's per-package install passes `R_MAKEVARS_USER=webr-vars.mk` which
overrides `CC=emcc`, `CXX=em++`, `LDFLAGS=-s SIDE_MODULE=1 -s WASM_BIGINT
-s ASSERTIONS=1 …`, and zeroes out `LIBR`/`LIBINTL`/`STRIP_*`.
`rpkg/configure.ac` detects this — when `CC` matches `emcc|em++` it sets:

- `IS_WASM_INSTALL=true` (substituted into `Makevars`)
- `CARGO_BUILD_TARGET=wasm32-unknown-emscripten`
- `RUST_TOOLCHAIN=+nightly` (only if not already pinned)
- `CARGO_BUILD_STD_FLAG=-Z build-std=std,panic_abort`

…and refuses to proceed if `src/rust/wasm_registry.rs` is absent (the wasm
install path can't run the host cdylib pass that would otherwise regenerate
it). `rpkg/src/rust/build.rs` enforces a related invariant: when building
for wasm32 it parses the `// generator-version: N` header out of
`wasm_registry.rs` and panics if it doesn't match the constant mirrored
from `miniextendr-api/src/wasm_registry_writer.rs::GENERATOR_VERSION`.
Bump both together when the generated-file shape changes.

`Makevars.in` splits the `$(WRAPPERS_R)` rule across `IS_WASM_INSTALL` so
the cdylib prerequisite is only declared on the native branch — wasm32
installs neither build nor `dyn.load` the cdylib (host R can't load a wasm
side module).

## Two R installations inside the container

webR's image carries two distinct R trees and you have to reach for the
right one:

| Path | Use |
|---|---|
| `/opt/R/current/bin/R` | Native (rig-managed 4.5.1). Phase 1 of the smoke script — host cdylib + wrapper-gen. |
| `/opt/webr/host/R-4.5.1/bin/R` | webR's own host R, configured for wasm cross-compilation. Phase 2 — wasm `R CMD INSTALL` with `webr-vars.mk`. |
| `/opt/webr/wasm/R-4.5.1/lib/R/library/` | wasm R library tree where the side-module ends up. NODEFS-mounted into the webR Node session. |

`R_SOURCE=/opt/webr/R/build/R-4.5.1` and `WASM_TOOLS=/opt/webr/tools` must
be exported during the wasm install — `webr-vars.mk` references both.

## Other webR build constraints

- **`linkme` does not support `wasm32-*` targets.** `linkme-impl` emits a
  `unsupported_platform` compile error for any `target_os` outside its
  whitelist. miniextendr leans on `linkme::distributed_slice` for runtime
  registration of `R_CallMethodDef`s, ALTREP class init, and trait dispatch
  tables; on WASM that's replaced by a host-generated `wasm_registry.rs` —
  see `plans/wasm-registry-codegen.md` for the design.
- **No host execution of WASM during install.** `--no-test-load` and
  `--no-staged-install` are mandatory. Anything that loads the side-module
  on the host (e.g. `dyn.load`-based wrapper-gen) is gated off via
  `IS_WASM_INSTALL`; R wrappers and `wasm_registry.rs` are pre-generated
  by Phase 1's native build and shipped through into Phase 2's source tree.
- **Worker thread is off.** R-on-WASM is single-threaded; the
  `worker-thread` feature must be disabled. Already feature-gated.
- **`RUSTFLAGS` for the side-module link** are not yet locked in — there's
  a `# TODO(#470)` in `rpkg/src/Makevars.in` flagging
  `-C relocation-model=pic -C link-args=-s SIDE_MODULE=1` as the proposed
  set. The smoke script is the empirical validator; if the wasm side-module
  fails to link, that's the next thing to verify against `rwasm`'s flags.

## CI

`.github/workflows/webr.yml` runs `cargo check --target
wasm32-unknown-emscripten -p miniextendr-api` on every PR matching the
paths filter (`miniextendr-api/**`, `miniextendr-macros/**`,
`miniextendr-engine/**`, `miniextendr-lint/**`, `rpkg/**`,
`tests/cross-package/**`, `Cargo.{toml,lock}`, `Dockerfile.webr`,
`.github/workflows/webr.yml`). It catches cfg-gating regressions and
macro-emission bugs that fail to compile on wasm32; it does **not** catch
link errors or runtime issues — those are tier 2/3 work.

The job sets `R_HOME=$RUNNER_TEMP` because
`miniextendr-api/build.rs::link_to_r()` unconditionally invokes `R RHOME`
and the runner has no R installed. The `rpkg/src/rust` cargo check is
currently dropped from tier 1 because `rpkg/configure` invokes `Rscript`
directly, which the dummy `RUNNER_TEMP` path lacks. Issue #482 tracks
gating `link_to_r()` on `CARGO_CFG_TARGET_ARCH != "wasm32"` so the
dummy-R_HOME workaround can disappear and the rpkg cargo-check can rejoin
tier 1.

## See also

- `plans/webr-support.md` — index plan; flags landed (✅) vs still open.
- `plans/wasm-registry-codegen.md` — design for the linkme replacement on
  WASM.
- `plans/webr-dockerfile.md` — `Dockerfile.webr` design.
- `plans/webr-configure-and-build-rs.md` — `configure.ac` + `build.rs`
  rationale (now landed in #481).
- `plans/webr-ci.md` — three-tier CI plan; tier 1 landed in #480, tier 2/3
  open.
- `plans/webr-cross-package-stubs.md` — `tests/cross-package/*` stub plan;
  low priority, only relevant if cross-package crates join wasm32 CI.
- `tests/webr-smoke.sh` — the local end-to-end smoke runner.
- `.webr/` — vendored clone of the webR repo for offline reference.
- `.webr/Dockerfile` — upstream Rust toolchain install we inherit.
- `.webr/packages/webr-vars.mk` — the Makevars override webR uses for
  every R package install under WASM.
