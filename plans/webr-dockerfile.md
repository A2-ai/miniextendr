# Dockerfile for miniextendr WebR builds

Goal: a `Dockerfile.webr` (lives at repo root) that gives us — and CI — a
reproducible environment for building & smoke-testing miniextendr against
webR's WASM toolchain. Inherits the upstream webR image so we don't
re-stage Emscripten, flang, Rust nightly, or R-for-WASM ourselves.

Companion to `plans/webr-support.md` (the actual port work) and
`docs/WEBR.md` (the user-facing rationale). This file is just the build
environment.

## Base image choice

Two upstream images we could inherit from:

| Image | Provides | Why pick it |
|---|---|---|
| `ghcr.io/r-wasm/flang-wasm:main` | Emscripten SDK + LLVM flang only. No Rust, no R. | Smaller. Costs us re-installing nightly Rust + rust-src + rig + rwasm. |
| `ghcr.io/r-wasm/webr:main` | Above + Rust nightly with `wasm32-unknown-emscripten` target + `rust-src` + native R via `rig` + `pak` + `rwasm` + a built `host/`+`wasm/` R tree under `/opt/webr`. | Everything pre-baked. Bigger pull but zero re-stage cost. |

**Recommendation: `ghcr.io/r-wasm/webr:main`.** The webR image already
runs the exact `rustup-init` incantation in `.webr/Dockerfile:38-53`
(nightly + `wasm32-unknown-emscripten` target + `rust-src` component +
the fake `rustc`/`cargo` Debian packages so apt doesn't fight rustup).
Reproducing that on top of `flang-wasm` doubles the maintenance surface
for no win — when webR bumps Emscripten or Rust nightly, we'd have to
chase. Inheriting the squashed image means we move with them.

Pin to a digest, not `:main`. webR's `:main` rolls without notice and
breaks builds. CI bumps the digest as a deliberate PR. (Pattern: same
as `actions/checkout@<sha>` style pinning.)

```dockerfile
ARG WEBR_BASE=ghcr.io/r-wasm/webr@sha256:<digest-here>
FROM ${WEBR_BASE}
```

## What the base already gives us (verified from `.webr/Dockerfile`)

- `PATH` extended with `/opt/emsdk:/opt/emsdk/upstream/emscripten:/usr/local/cargo/bin`
- `EMSDK=/opt/emsdk`, `EMFC=/opt/flang/host/bin/flang`
- `RUSTUP_HOME=/usr/local/rustup`, `CARGO_HOME=/usr/local/cargo`
- Rust nightly with `wasm32-unknown-emscripten` target + `rust-src` component
- Native R at `/opt/R/current/bin/R` via `rig add 4.5.1`
- WASM R tree at `/opt/webr/host/...` and `/opt/webr/wasm/...`
- `R_LIBS_USER=/opt/R/current/lib/R/site-library` — `pak` + `rwasm` already in `.Library`
- `WEBR_ROOT=/opt/webr`
- The fake `rustc_99.0_all.deb` + `cargo_99.0_all.deb` already installed,
  so any apt-driven R package install (e.g. via pak) won't fight our toolchain.

## What we layer on top

Minimum viable set for CI:

| Tool | Why |
|---|---|
| `just` | All our build entry points are just recipes. Without it, contributors can't reproduce CI locally just by `docker run`-ing the image and following `CLAUDE.md`. Install via the prebuilt binary release (`curl …/just-x86_64-unknown-linux-musl.tar.gz`) — apt-installable but old. |
| `autoconf` | `rpkg/configure` is generated from `configure.ac`. Source installs need to regenerate it (or rely on the committed `configure` — verify whether we ship it). Cheap to install via apt. |
| `clang` + `libclang-dev` | `bindgen` runtime dependency for `LinkingTo:` C-API shims (`docs/NATIVE_R_PACKAGES.md`). Needed for any rpkg test that pulls in `cli`/`vctrs`/etc. |
| `pkg-config` | Cargo crates with native deps look it up. The webR image *already* has the `pkg-config` shim at `.webr/tools/shims/pkg-config` for emscripten — verify it's on PATH; if so, skip. |
| `git` | Already in base (rig+rwasm install pulled it). Verify. |

**Optional** but worth layering for dev images (skip in CI to keep the
image small):

- `cargo-limit` (the `cargo lcheck`/`lclippy`/`ltest`/`lbuild` aliases
  per `CLAUDE.md`).
- `sccache` — would help CI iteration but the WASM target's per-invocation
  hashing breaks it the same way it breaks native builds (see CLAUDE.md's
  sccache + incremental note). Defer.
- `tree`, `ripgrep`, `fd-find` — quality-of-life for interactive sessions.

## Pinning the toolchain

The webR base image bumps Rust nightly whenever upstream rebuilds. That's
fine for "track webR" but bad for reproducibility within a PR. Two ways
to pin:

1. **Override the toolchain.** Add `rust-toolchain.toml` in the repo root
   (or under `rpkg/src/rust/`) that names a specific nightly date.
   `cargo` reads it automatically; `rustup` will install the named version
   on first invocation. Cost: extra ~150 MB layer on first build inside
   the container. Benefit: pinning is in-repo, visible in PR diffs.
2. **Use the base image's nightly verbatim.** Pin the base image digest
   tightly. Cost: nightly version isn't visible in our diffs — only the
   digest changes when we rebase on webR. Benefit: zero extra disk.

Recommendation: **(1) `rust-toolchain.toml`** so the pin is in our repo
where reviewers see it. The base image digest pins everything else
(emscripten, flang, R-wasm).

## Stage structure

Single stage. We aren't rebuilding anything heavy — just adding a few
apt packages and a couple of binaries. Multi-stage doesn't pay off here.

```dockerfile
ARG WEBR_BASE=ghcr.io/r-wasm/webr@sha256:<digest>
FROM ${WEBR_BASE}

# OS deps. apt-get update twice can race with parallel layers in CI; keep one block.
RUN apt-get update && apt-get install -y --no-install-recommends \
        autoconf \
        clang \
        libclang-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# just (apt version is too old for our justfiles).
ARG JUST_VERSION=1.36.0
RUN curl -fsSL "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz -C /usr/local/bin just

# Sanity checks — fail the build immediately if the inherited toolchain
# regressed instead of debugging a weird later failure.
RUN rustup target list --installed | grep -q wasm32-unknown-emscripten \
    && rustup component list --installed | grep -q rust-src \
    && cargo --version \
    && emcc --version \
    && /opt/R/current/bin/R --version \
    && Rscript -e 'stopifnot(requireNamespace("rwasm", quietly=TRUE))'

WORKDIR /work
```

`/work` is a plain mount point; CI binds the repo there. We do **not**
`COPY` the source into the image — keeping the image source-free makes
it a pure environment artifact and lets devs `docker run -v $PWD:/work`
without rebuilding for every code change.

## What CI does inside the container

In rough order of cost (cheapest first, so failures surface fast):

1. **`cargo check --target wasm32-unknown-emscripten -Z build-std=std,panic_abort`** on `miniextendr-api`
   alone. Fast feedback that our cfg-gating compiles.
2. Same on `miniextendr-macros`, then on `rpkg/src/rust`.
3. **`R CMD INSTALL`** of rpkg into the WASM library tree using
   webR's Makevars override. Needs `just wasm-prepare` to have already
   produced `R/<pkg>-wrappers.R` and `wasm_registry.rs` — that prep step
   itself runs on a *native* image (or earlier in the same job using
   the webR image's host R). Open: where exactly to run prep — see
   `plans/webr-support.md` step 5.
4. **`rwasm::build_pkg(...)`** as the official webR-blessed install path.
   This is what users will hit; once (3) works we should also exercise
   the rwasm wrapper since it sets flags we don't.
5. **Smoke test in webR.** Load the installed package in a webR Node
   session, call one `#[miniextendr]` function, assert the return value.
   `.webr/` already has a Node-driven test harness in `src/tests-webr/`
   we can crib from.

Steps 3–5 are increasingly load-bearing and increasingly brittle. Land
1+2 first, then iterate.

## Two-image story

We probably want **two** images downstream of this plan, sharing
the same Dockerfile via build args:

| Image | Audience | Difference |
|---|---|---|
| `miniextendr-webr-ci` | CI | Single stage as above. No source baked in. |
| `miniextendr-webr-dev` | Local dev | Same plus `cargo-limit`, `ripgrep`, `fd-find`, `lldb`, possibly a `mise`/`asdf` shim so contributors can use the container as their interactive shell. |

Build the dev variant via `--target dev` if we adopt multi-stage; or
just keep one Dockerfile with an `ARG IMAGE_FLAVOR=ci` toggling extra
`RUN` blocks. Single-Dockerfile keeps maintenance trivial.

## Caching

GitHub Actions cache for the layers above the base image is small —
only the apt packages + just binary + sanity checks. Worth setting up
`actions/cache@v4` keyed on `Dockerfile.webr` hash so PRs that don't
touch the Dockerfile reuse the layer set, but the savings are bounded
because the base image pull dominates wall time. Use BuildKit cache
mounts for `/var/cache/apt` and `/usr/local/cargo/registry` if the build
ever sprouts a `cargo fetch` step.

The expensive cache target is **the cargo `target/` directory under
`-Z build-std`**, which rebuilds `std` on first hit. Mount it as a
named volume in CI (`actions/cache` keyed on `Cargo.lock`) and a host
bind in dev. Estimated savings: 3-5 minutes per CI run after first
warm-up.

## Open questions / decisions deferred to implementation

- **Multi-arch?** webR's `:main` is x86_64-only. ARM64 macOS dev would
  go through QEMU emulation (slow). Not blocking — flag for later.
- **Where does `just wasm-prepare` run?** If it runs on the WASM image,
  the host R inside the image has to load the *native-Linux* cdylib,
  not WASM. `/opt/R/current/bin/R` is native, so this works — but the
  rpkg build into native targets needs space inside the same image.
  Verify before committing to "one image does everything."
- **Whether to bake `ghcr.io/r-wasm/webr` into our org's registry.**
  Pulling from `ghcr.io/r-wasm/webr` on every CI job costs egress and
  is subject to upstream rate limits. Mirror to `ghcr.io/a2-ai/...`
  on a schedule. Not blocking, but file as follow-up.

## Concrete next steps

1. Land `Dockerfile.webr` at repo root with the body above.
2. Add `.dockerignore` covering `target/`, `.claude/`, `.webr/`,
   `node_modules/`, `*.log`, `inst/vendor.tar.xz` to keep the build
   context small (the dockerfile doesn't COPY source, but
   `docker build .` still tars the context — `.webr/` alone is hundreds
   of MB).
3. Add a `just docker-webr-build` recipe wrapping the
   `docker build -f Dockerfile.webr -t miniextendr-webr-ci:dev .`
   incantation, plus `just docker-webr-shell` to run an interactive
   container with the repo bind-mounted at `/work`.
4. Add a CI job (`.github/workflows/webr.yml`) that runs steps 1+2 of
   the "What CI does" list above. Gate it on `paths:` matching the
   Rust crates and the Dockerfile so it doesn't fire on docs-only PRs.
5. Once `plans/webr-support.md` step 5 (codegen of `wasm_registry.rs`)
   lands, extend the CI job to step 3 (R CMD INSTALL) and step 4
   (`rwasm::build_pkg`). Step 5 (smoke test in webR Node) is its own
   PR.
