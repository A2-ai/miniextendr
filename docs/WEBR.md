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
dropped (#745), and the base-image pin bumped to a tagged webR v0.6.0 / R 4.6.0
release (#755). Open follow-ups: #495 (cross-crate trait dispatch), #752
(dependency guidance — see "Dependencies and webR" below), #788 (arm64-native
dev image), #747 (drop mirror creds once the GHCR package is public).

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
base packages (handled by installing to an empty temp library; see tier 2 /
#491), but here it's triggered by *your package's own declared imports*, so
the framework can't paper over it for you.

**Guidance:**

- **Pure-R dependencies** (rlang, lifecycle, cli, glue, …) are safe to
  `importFrom` — they have no `.so` for the host R to choke on.
- **Compiled or heavy dependencies you only need at runtime** (Shiny, DBI
  backends, data.table, …) belong in `Suggests`, not `Imports`. Call them
  with `pkg::fn()` behind a `rlang::check_installed()` /
  `requireNamespace()` guard. That keeps them out of the namespace-load
  graph, so the wasm install's lazy-load never reaches for their `.so`.

This mirrors what the astra downstream did (moved its Shiny stack to
`Suggests` + `::`). Tracked in #752; a future `minirextendr_doctor()` /
`miniextendr_check_static()` lint may flag eager `importFrom` of
known-compiled packages.

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
- Issue #495 — cross-crate trait dispatch; #752 — dependency guidance;
  #788 — arm64-native dev image.
- `tests/webr-node-smoke/smoke.mjs` — the CI tier-3 Node runner (single source
  of truth for the runtime smoke; `tests/webr-smoke.sh` Phase 3 invokes it).
- `tests/webr-smoke.sh` — the local end-to-end smoke runner. Mirrors the green
  `webr.yml` tier-2/3 job step-for-step (Phase 2 → `/tmp/wasm-lib`, Phase 3 →
  `make` the Node bundle, then run `smoke.mjs`). The base image is amd64-only,
  so it can't be exercised on an arm64 dev box today — tracked in #788.
- `.webr/` — vendored clone of the webR repo for offline reference.
- `.webr/Dockerfile` — upstream Rust toolchain install we inherit.
- `.webr/packages/webr-vars.mk` — the Makevars override webR uses for
  every R package install under WASM.
