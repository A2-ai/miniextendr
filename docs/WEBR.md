# WebR / WASM support

Building miniextendr for [webR](https://docs.r-wasm.org/webr/latest/) — R
compiled to WebAssembly via Emscripten.

**Status: supported, CI-validated.** Three CI tiers run in
`.github/workflows/webr.yml`: tier 1 (`wasm32-unknown-emscripten`
cargo-check, every PR), tier 2 (full `R CMD INSTALL` of `rpkg` inside the
webR container — emcc side-module link), and tier 3 (a webR Node session
that drives `library(miniextendr)` against the wasm install). A local
`just docker-webr-smoke` recipe drives the same path inside the pinned
webR Docker image.

Tracking: umbrella #470. Shipped: tier 1/2/3 CI (#480 / #491 / #492),
cross-package wasm stubs (#493), side-module `RUSTFLAGS`
(`-Zdefault-visibility=hidden`, #494), `link_to_r()` wasm gating (#482), webR
base-image mirror (#496), the redundant `-C relocation-model=pic` flag
dropped (#745), the base-image pin bumped to a tagged webR v0.6.0 / R 4.6.0
release (#755), and dependency guidance (#752 — see "Dependencies and webR"
below). Open follow-ups: #495 (cross-crate trait dispatch), #925 (lint for
eager `importFrom` of compiled deps), #788 (arm64-native dev image — first cut
landed as `Dockerfile.webr-arm64`, pending on-hardware validation; see
"arm64-native dev image" below), #747 (drop mirror creds once the GHCR package
is public).

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
amd64-only — Apple Silicon runs it under Rosetta, slow but works. For a
native-arm64 alternative (no Rosetta), see "arm64-native dev image" below
(`Dockerfile.webr-arm64`, #788 — currently a draft pending on-hardware
validation).

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
   the cdylib pass and regenerate `rpkg/src/rust/wasm_registry.rs`. rpkg's
   committed snapshot is a *real* one (it's committed in lockstep with the
   wrapper / macro surface — see "Generated artifacts" in the root
   `CLAUDE.md`), but this step guarantees it's fresh against the working
   tree: a stale snapshot would register the wrong set of R routines under
   wasm. (The cross-package fixtures under `tests/cross-package/` ship
   deliberately *empty* stubs — content-hash `0000000000000000` — since
   they're never deployed to webR; see #493.)
2. **wasm32 install** — `CC=emcc bash rpkg/configure` followed by
   `R CMD INSTALL --no-test-load --no-staged-install` against
   `/opt/webr/host/R-4.6.0/bin/R` (webR's own host R) with
   `R_MAKEVARS_USER=/opt/webr/packages/webr-vars.mk`. Result lands at
   `/opt/webr/wasm/R-4.6.0/lib/R/library/miniextendr/`.
3. **webR Node session** — imports webR's bundled ESM directly from
   `file:///opt/webr/dist/webr.mjs` (see "The `/opt/webr/dist` import
   gotcha" below), NODEFS-mounts the wasm R lib tree, calls
   `library(miniextendr)`, then runs `testthat::test_local()`. Many tests
   fail under wasm (worker thread / fork / threading assumptions); the
   script reports counts and exits 0 as long as the package itself loads.
   The CI tier-3 equivalent lives in `tests/webr-node-smoke/smoke.mjs`.

First cold run is **1–2 hours** on Apple Silicon (Rosetta amd64 + cargo
wasm32 build). Subsequent runs reuse the docker image and most cargo
artefacts.

## arm64-native dev image (DRAFT — #788)

> **Status: composed but NOT YET VALIDATED on arm64 hardware.** The
> `Dockerfile.webr-arm64` recipe below is written from prebuilt parts and
> resolves the one critical unknown (emcc ABI, see below), but it has not been
> *built or run* on an arm64 box. Treat it as a first cut until the validation
> checklist at the end of this section is green. Until then, the amd64 path
> above (Rosetta) is the supported route.

The amd64 image runs on Apple Silicon only under Rosetta — slow, and the
2026-05-27 attempt at the full datafusion+arrow wasm compile under qemu
exhausted host disk and crashed Docker Desktop. `Dockerfile.webr-arm64` builds
**natively on arm64** by composing prebuilt parts, so there is no emulated
execution and no source build of emcc / flang / R→wasm:

| Piece | Source | Why no source build |
|---|---|---|
| emcc | `emscripten/emsdk:4.0.8-arm64` (linux/arm64/v8) | emsdk ships *prebuilt* emcc per host arch |
| Rust nightly + `wasm32-unknown-emscripten` + `rust-src` | `rustup` `--default-host aarch64-unknown-linux-gnu` | prebuilt arm64 toolchain |
| host R 4.6.0 | `rig add 4.6.0` | prebuilt arm64 R |
| wasm R sysroot (`/opt/webr/{wasm,R/build,tools,packages,dist,src}`) | `COPY --from=` the amd64 mirror | wasm objects + headers + scripts are arch-portable; FS copy only |

Neither flang (Fortran→wasm) nor the R→wasm build is needed: miniextendr is
Rust + C with no Fortran, and the wasm R is already prebuilt — those upstream
parts exist only to build R itself to wasm, which a dev image reuses.

```bash
just docker-webr-arm64-build         # build natively on arm64
just docker-webr-arm64-shell         # interactive shell, repo at /work
just docker-webr-arm64-smoke         # arm64 end-to-end smoke (WEBR_ARM64=1)
```

### Why emcc `4.0.8-arm64` specifically (the ABI match)

The emcc that links `miniextendr.so` must match the emcc that built the
prebuilt wasm R, or the side-module won't load. The mirror's wasm R was built
with Emscripten **4.0.8**. `emscripten/emsdk` publishes *arch-suffixed* tags,
**not** a multi-arch manifest: the bare `:4.0.8` tag is linux/amd64 only, but
`:4.0.8-arm64` is a genuine linux/arm64/v8 build of the **same** 4.0.8 release
(digest `sha256:9d471ceb4bd9e…`, pushed 2025-04-30). Same emcc version on a
different host arch ⇒ same wasm ABI ⇒ no version-skew risk. This is the
best-case answer to #788's open question Q1.

> The #788 issue body assumed host R **4.5.1**; the base image was since bumped
> to webR v0.6.0 / **R 4.6.0** (#755), so the arm64 image pins `rig add 4.6.0`
> to stay header-compatible with the copied wasm R. This is deliberately *not*
> the repo's pinned dev R (`rproject.toml`'s 4.6) — it tracks whatever the
> prebuilt wasm R was built from.

### Validation checklist (needs on-arm64 hardware)

The dev sandbox has no Docker and can't build/run arm64, so the following are
**unverified** and must be checked on an Apple Silicon box before #788 is
closed:

- [ ] **Image builds** — `just docker-webr-arm64-build` completes (donor
      `COPY --from=` resolves, native toolchain installs, sanity-check layer
      passes).
- [ ] **Side-module ABI load** — `just docker-webr-arm64-smoke` Phase 2 links
      `miniextendr.so` with the arm64 emcc and Phase 3's `library(miniextendr)`
      loads it in a webR Node session (proves the 4.0.8 arm64↔amd64-built-R ABI
      really matches, not just by version label).
- [ ] **Sysroot link/load** — the amd64-built wasm sysroot under `/opt/webr`
      links and loads cleanly under the arm64-host emcc end-to-end (no missing
      objects, no header mismatch from the copied tree).
- [ ] **Native-R orchestration on arm64** — Phase 1 (native cdylib wrapper-gen)
      and Phase 2's `R CMD INSTALL` both run through the rig-installed arm64 R
      (`R` on PATH), since the donor's amd64 `/opt/webr/host/R-4.6.0` +
      `/opt/R/current` binaries can't execute on arm64.
- [ ] **Node bundle rebuild** — `make /opt/webr/src/dist/webr.mjs` succeeds
      with the copied `/opt/webr/src` tree and the emsdk image's bundled Node
      22.16.0.
- [ ] **Compile weight / disk** — datafusion+arrow wasm compile is now native
      (no qemu tax) but still heavy; confirm it fits a typical Docker Desktop
      disk budget.

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
| `/opt/R/current/bin/R` | Native (rig-managed 4.6.0). Phase 1 of the smoke script — host cdylib + wrapper-gen. |
| `/opt/webr/host/R-4.6.0/bin/R` | webR's own host R, configured for wasm cross-compilation. Phase 2 — wasm `R CMD INSTALL` with `webr-vars.mk`. |
| `/opt/webr/wasm/R-4.6.0/lib/R/library/` | wasm R library tree where the side-module ends up. NODEFS-mounted into the webR Node session. |

`R_SOURCE=/opt/webr/R/build/R-4.6.0` and `WASM_TOOLS=/opt/webr/tools` must
be exported during the wasm install — `webr-vars.mk` references both.

## Other webR build constraints

- **`linkme` does not support `wasm32-*` targets.** `linkme-impl` emits a
  `unsupported_platform` compile error for any `target_os` outside its
  whitelist. miniextendr leans on `linkme::distributed_slice` for runtime
  registration of `R_CallMethodDef`s, ALTREP class init, and trait dispatch
  tables; on WASM that's replaced by a host-generated `wasm_registry.rs`
  that pre-bakes the same registrations at build time. Cross-crate trait
  dispatch on WASM is the remaining follow-up — tracked in #495.
- **No host execution of WASM during install.** `--no-test-load` and
  `--no-staged-install` are mandatory. Anything that loads the side-module
  on the host (e.g. `dyn.load`-based wrapper-gen) is gated off via
  `IS_WASM_INSTALL`; R wrappers and `wasm_registry.rs` are pre-generated
  by Phase 1's native build and shipped through into Phase 2's source tree.
- **Worker thread is off.** R-on-WASM is single-threaded; the
  `worker-thread` feature must be disabled. Already feature-gated.
- **`RUSTFLAGS` for the side-module** are set by `rpkg/configure.ac`'s
  `is_wasm_install` branch (and mirrored into the minirextendr templates):
  `-Zdefault-visibility=hidden` (#494). This flag is load-bearing, not
  cosmetic: webR links the Rust staticlib into a `-s SIDE_MODULE=1` shared
  object, and without hidden default visibility the staticlib exports ~3000
  mangled stdlib/dep symbols into the side-module's EXPORT table. webR's
  JS-side `dyn.load` then fails — `TypeError: Cannot read properties of
  undefined` on the pinned emcc, or a hard `emcc: error: invalid export name`
  on emcc 4.0.8+. Hiding symbols by default leaves only the `#[no_mangle]
  extern "C"` entry points exported. This is `savvy`'s approach
  (yutannihilation/savvy#372), endorsed by webR's maintainer
  (r-wasm/webr#532). Note `-s SIDE_MODULE=1` is an emcc *link* flag supplied
  by `webr-vars.mk`, not a `RUSTFLAG` — the staticlib is a `cargo build
  --lib` archive cargo never links. (`-C relocation-model=pic` was set here
  too until #745: PR #749 proved the link succeeds without it and tier-3
  confirmed the runtime load is unaffected — wasm32-unknown-emscripten is
  position-independent by default, so the flag was a no-op.)

## Running a webR session in Node (the two-bundle gotcha)

webR's esbuild config emits **two** bundles and only one runs in Node:

| Bundle | Path | Runtime |
|---|---|---|
| **browser** | `/opt/webr/dist/webr.mjs` | browser only — stubs out `fs`/`worker_threads`/`url` via `blankImportPlugin`, *crashes in Node* |
| **Node** | `/opt/webr/src/dist/{webr.mjs,webr.cjs}` | Node — the `.mjs` carries a `__dirname`/`__filename`/`createRequire` banner |

So the Node runner imports the **Node** bundle, with **no** `baseUrl`:

```js
import { WebR } from "file:///opt/webr/src/dist/webr.mjs";
const webR = new WebR({ interactive: false });   // NO file:// baseUrl
```

Two non-obvious constraints:

1. **The image deletes `src/dist` and `src/node_modules`** (its Dockerfile runs
   `make clean` to shrink the published image, see `.webr/Dockerfile`). You must
   **rebuild the Node bundle first**: `cd /opt/webr/src && make
   /opt/webr/src/dist/webr.mjs`. That target chains `npm ci` →
   `webR/config.ts` (sed from `.in`) → `npm run build` (tsc + esbuild) → an
   asset-copy of `R.wasm`/`R.js`/`vfs`/`webr-worker.js` out of
   `/opt/webr/dist` into `/opt/webr/src/dist`, so the bundle resolves all
   runtime assets via its own `__dirname` — hence no `baseUrl` needed (~20s on
   a warm runner).
2. **Do NOT set a `file://` baseUrl.** Node 18+'s `new Worker(string)` rejects
   `file://` URL strings with `ERR_WORKER_PATH`, so the bundle crashes at init
   while building the `webr-worker.js` worker path. (This is the trap the old
   `import { WebR } from "file:///opt/webr/dist/webr.mjs"` + `baseUrl` advice
   walked straight into — that imported the *browser* bundle, which can't run
   in Node at all.)

`tests/webr-node-smoke/smoke.mjs` (CI tier-3) is the worked reference; its
header comment is the source of truth for the bundle layout. The Node process
must `webR.close()` (terminates the worker) and call `process.exit()` in a
top-level `.finally()` — otherwise the worker keeps Node's event loop alive and
the run hangs until the watchdog `timeout` kills it (exit 124, see
`reviews/2026-05-29-tier3-webr-node-smoke-exit-hang.md`).

## Dependencies and webR

The wasm `R CMD INSTALL` finishes with a lazy-load / byte-compile step that
spawns a *host* R whose `.libPaths()` points into the wasm library tree (see
"Two R installations" above). If your package's `NAMESPACE` eagerly imports
a **compiled** package — `importFrom(somePkg, …)` or `import(somePkg)` where
`somePkg` ships a `.so` — that step calls `loadNamespace(somePkg)`, the host
R tries to `dyn.load` the wasm-built `somePkg.so`, and dies:

```
unable to load shared object '.../somePkg.so': invalid ELF header
```

This is the same host-R-loads-a-wasm-object failure that bites webR's own
base packages — handled by installing to an empty temp library, the
`install-to-temp-lib` pattern; see tier 2 / #491 / #744. The difference is
the *trigger*: #491/#744 are about webR's own base-package `.so`s, whereas
this failure is driven by *your package's own declared imports*, so the
framework can't paper over it for you.

### Why `--no-byte-compile` alone doesn't save you

Both the wasm-install scripts and CI pass `--no-byte-compile` (see
`tests/webr-smoke.sh` and `.github/workflows/webr.yml`, mirroring rwasm's
flag set). That suppresses the *byte-compile* half of the cascade, but the
lazy-load step that materialises the namespace still runs `loadNamespace()`
for everything in your `Imports`/`Depends` namespace-load graph. Skipping
byte-compilation does **not** prune your declared imports, so an
`importFrom` of a compiled package still reaches for its `.so`. The only
robust fix is to keep the compiled dependency out of the namespace-load graph
entirely — see the guidance below.

**Guidance:**

- **Pure-R dependencies** (rlang, lifecycle, cli, glue, …) are safe to
  `importFrom` — they have no `.so` for the host R to choke on.
- **Compiled or heavy dependencies you only need at runtime** (Shiny, DBI
  backends, data.table, …) belong in `Suggests`, not `Imports`. Call them
  with `pkg::fn()` behind a `rlang::check_installed()` /
  `requireNamespace()` guard. That keeps them out of the namespace-load
  graph, so the wasm install's lazy-load never reaches for their `.so`.

This mirrors what the astra downstream did (moved its Shiny stack to
`Suggests` + `::`). Documented under #752; a future `minirextendr_doctor()` /
`miniextendr_check_static()` lint that flags eager `importFrom` of
known-compiled packages when a webR target is configured is tracked
separately in #925 (the "is it compiled?" / "is webR the target?" detection
needs design — see that issue).

## CI

`.github/workflows/webr.yml` runs three tiers. **Tier 1** is `cargo check
--target wasm32-unknown-emscripten -p miniextendr-api` on every PR matching
the paths filter (`miniextendr-api/**`, `miniextendr-macros/**`,
`miniextendr-engine/**`, `miniextendr-lint/**`, `rpkg/**`,
`tests/cross-package/**`, `Cargo.{toml,lock}`, `Dockerfile.webr`,
`.github/workflows/webr.yml`). It catches cfg-gating regressions and
macro-emission bugs that fail to compile on wasm32; it does **not** catch
link errors or runtime issues — those are tier 2/3 work.

The tier-1 job sets `R_HOME=$RUNNER_TEMP` because
`miniextendr-api/build.rs::link_to_r()` unconditionally invokes `R RHOME`
and the runner has no R installed. The `rpkg/src/rust` cargo check is
currently dropped from tier 1 because `rpkg/configure` invokes `Rscript`
directly, which the dummy `RUNNER_TEMP` path lacks. Issue #482 tracks
gating `link_to_r()` on `CARGO_CFG_TARGET_ARCH != "wasm32"` so the
dummy-R_HOME workaround can disappear and the rpkg cargo-check can rejoin
tier 1.

**Tier 2 + 3** run as a single `webr-install` job inside the webR container
(`ghcr.io/a2-ai/webr-mirror`, a digest-preserved mirror of
`ghcr.io/r-wasm/webr` — see #496 / `.github/workflows/mirror-webr.yml`).
The job runs the same three phases as the local smoke: Phase 1 (native
install regenerates `wasm_registry.rs`), Phase 2 (emcc wasm install →
`/tmp/wasm-lib/miniextendr`, the empirical validator for the side-module
`RUSTFLAGS`), then **tier 3** — the Node + webR session
(`tests/webr-node-smoke/smoke.mjs`) that NODEFS-mounts the wasm install,
installs the package's Imports from `repo.r-wasm.org`, and drives
`library(miniextendr)`. Tier 2 only proves the side-module *links*; tier 3
is what proves it *loads* in a real webR runtime.

## See also

- Issue #470 — umbrella tracking issue for webR/WASM support.
- Issue #495 — cross-crate trait dispatch; #752 — dependency guidance
  (this section); #925 — follow-up lint for `importFrom` of compiled deps;
  #788 — arm64-native dev image (first cut: `Dockerfile.webr-arm64` + the
  `docker-webr-arm64-*` just recipes + the `WEBR_ARM64=1` smoke path).
- Issues #491 / #744 — the base-package variant of the host-R-loads-a-wasm-
  object failure, solved via the install-to-temp-lib pattern (the dependency
  guidance above is the consumer-package-imports variant of the same failure).
- `tests/webr-node-smoke/smoke.mjs` — the CI tier-3 Node runner (single source
  of truth for the runtime smoke; `tests/webr-smoke.sh` Phase 3 invokes it).
- `tests/webr-smoke.sh` — the local end-to-end smoke runner. Mirrors the green
  `webr.yml` tier-2/3 job step-for-step (Phase 2 → `/tmp/wasm-lib`, Phase 3 →
  `make` the Node bundle, then run `smoke.mjs`). The default (amd64) image runs
  under Rosetta on Apple Silicon; `WEBR_ARM64=1` selects the draft
  `Dockerfile.webr-arm64` native-arm64 path (#788).
- `Dockerfile.webr-arm64` — draft native-arm64 dev image (#788): amd64 sysroot
  donor + `emscripten/emsdk:4.0.8-arm64` + native arm64 Rust/R. See
  "arm64-native dev image" above for the validation checklist.
- `.webr/` — vendored clone of the webR repo for offline reference.
- `.webr/Dockerfile` — upstream Rust toolchain install we inherit.
- `.webr/packages/webr-vars.mk` — the Makevars override webR uses for
  every R package install under WASM.
