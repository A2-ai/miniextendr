# WebR CI workflow

Goal: a CI job that exercises the wasm32 build path end-to-end, gated on
paths so docs-only PRs don't fire it. **Optional.** Steps 2–7 of
`webr-support.md` are merged; everything compiles for
`wasm32-unknown-emscripten`. This plan is what closes the loop on
"reviewer can know wasm32 still works without running it locally."

Status: **not started**. The user explicitly deferred CI ("not sure what
you want to do in CI") during the implementation sprint. Pick this up
when the wasm32 path needs ongoing protection.

## What success looks like

A `webr.yml` workflow that runs on PR + push to main when wasm-relevant
files change, and reports a single `wasm32 check` status. Three
escalating tiers of work — pick one to start.

## Tier 1 — `cargo check` only (smallest, fastest)

Catches: cfg-gating regressions, macro emission bugs that fail to
compile on wasm32, anything that breaks the linkme replacement.
Doesn't catch: link errors, runtime errors.

Cost: ~3 min per PR (cargo check is fast). No emcc / R needed.

```yaml
# .github/workflows/webr.yml
name: webR / wasm32
on:
  pull_request:
    paths:
      - 'miniextendr-api/**'
      - 'miniextendr-macros/**'
      - 'rpkg/**'
      - 'tests/cross-package/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'Dockerfile.webr'
      - '.github/workflows/webr.yml'
  push:
    branches: [main]
    paths: # …same…
  workflow_dispatch:

jobs:
  cargo-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-emscripten
      - uses: Swatinem/rust-cache@v2
      - name: cargo check (miniextendr-api)
        run: cargo check -p miniextendr-api --target wasm32-unknown-emscripten
      - name: cargo check (rpkg)
        run: |
          bash ./rpkg/configure
          cd rpkg/src/rust
          cargo check --target wasm32-unknown-emscripten
```

`cargo check` for `wasm32-unknown-emscripten` works on **stable** Rust
without `-Z build-std` because we only need to typecheck — no link.
Rust ships a precompiled `std` for that target on stable. (Linking
against webR's pinned Emscripten is what needs nightly + `build-std`,
and that's tier 2.)

`./rpkg/configure` is needed so `.cargo/config.toml` gets the
`[patch."git+url"]` overrides that point at workspace siblings. Without
it, rpkg fetches `miniextendr-api` from `main` (via
`Cargo.toml`'s `git = "..."` dep) — works on PRs to main, but breaks if
the PR also touches `miniextendr-api` because the live source isn't
reflected.

## Tier 2 — Full `R CMD INSTALL` via `Dockerfile.webr`

Catches everything tier 1 catches, plus: link errors, missing
`wasm_registry.rs`, Makevars issues, the actual rwasm/emcc dance.
Doesn't catch: runtime behaviour (loading in webR / running R code).

Cost: ~10–15 min per PR (image pull + cdylib build + emcc link +
R CMD INSTALL into the WASM lib tree).

```yaml
jobs:
  rcmd-install:
    runs-on: ubuntu-latest
    container: ghcr.io/r-wasm/webr@sha256:0b9e6e41134275d186633cc0b2564f0d03c66b36c0412fdbae51c7572cdbdd40
    steps:
      - uses: actions/checkout@v4
      # The base image has nightly + `wasm32-unknown-emscripten` + `rust-src`
      # already. We just need `just` + `autoconf` to drive our recipes.
      - name: Layer just + autoconf
        run: |
          apt-get update && apt-get install -y --no-install-recommends autoconf
          curl -fsSL "https://github.com/casey/just/releases/download/1.36.0/just-1.36.0-x86_64-unknown-linux-musl.tar.gz" \
            | tar -xz -C /usr/local/bin just
      - name: Configure rpkg
        run: |
          # webR-vars.mk-style overrides: CC=emcc, R_MAKEVARS_USER=webr-vars.mk
          export R_MAKEVARS_USER=/opt/webr/packages/webr-vars.mk
          bash ./rpkg/configure
      - name: R CMD INSTALL --no-test-load
        run: |
          R_LIBS=/tmp/wasm-lib /opt/R/current/bin/R CMD INSTALL \
            --no-docs --no-test-load --no-staged-install \
            --library=/tmp/wasm-lib \
            rpkg
```

**Open question:** does the base webR image have a wasm32 R built such
that `R CMD INSTALL` produces a wasm32 side-module? Verify locally with
`just docker-webr-shell` before committing the workflow.

**`R_MAKEVARS_USER=webr-vars.mk` is required.** Without it, the install
uses the host's CC (gcc) and produces a native shared object — not a
wasm side-module. The webR image ships
`/opt/webr/packages/webr-vars.mk` with `CC=emcc`, `LDFLAGS=-s
SIDE_MODULE=1`, etc.

**Ties into step 8.** `rpkg/configure.ac` doesn't yet detect `CC=emcc`
and adjust cargo / Makevars. Tier 2 needs step 8 landed first;
otherwise the cargo step still tries to link a native cdylib.

## Tier 3 — Smoke test in webR Node session

Catches everything plus: the package actually loads in webR and a
`#[miniextendr]` function returns sensible output.

Cost: tier 2 + ~5 min for Node + webR startup + test execution.

```yaml
- name: Smoke test
  run: |
    /opt/webr/host/bin/Rscript -e '
      .libPaths(c("/tmp/wasm-lib", .libPaths()))
      library(miniextendr)
      result <- some_exported_fn(...)
      stopifnot(...)
    '
```

The `webR Node` invocation is what `webr` gem / `r-wasm/webr` runs in
its own CI. Crib from `.webr/src/tests-webr/` (the local clone of the
webR repo we vendored as offline reference). The exact incantation has
to be worked out — webR itself runs tests in a JS host, not via
`Rscript` directly. Likely looks like:

```bash
node -e "const { WebR } = require('@r-wasm/webr');
         const webR = new WebR();
         await webR.init();
         await webR.installPackage('/tmp/wasm-lib/miniextendr');
         const result = await webR.evalR('miniextendr::some_exported_fn()');
         console.log(await result.toJs());"
```

Defer until tier 2 is solid.

## What's *not* in scope for this plan

- macOS / Windows wasm32 builds. webR's image is x86_64 Linux only;
  cross-platform CI is upstream's problem, not ours. amd64 emulation
  on arm64 macOS via Rosetta works for *local* dev (per `Dockerfile.webr`)
  but is too slow for CI.
- Caching the cdylib `target/` dir under `-Z build-std`. tier 2 only —
  can be added later via `actions/cache@v4` keyed on `Cargo.lock`.
  Saves 3–5 min per CI run after first warm-up.
- Mirroring `ghcr.io/r-wasm/webr` to `ghcr.io/a2-ai/...`. Pull-rate-limit
  hedge. File as follow-up only if rate-limited in practice.

## Recommended starting point

**Tier 1.** Two cargo-check steps + paths filter. ~3 min, catches the
common regressions, no Docker / emcc / R complications. Tier 2 + 3 land
once step 8 is in (so `configure.ac` cooperates with `CC=emcc`) and
once we've manually walked through the workflow inside
`just docker-webr-shell` to know the exact command shape.

## Implementation steps

1. Open a PR with just the tier-1 workflow file. Verify it reports
   `wasm32 check / cargo check` as a success status on a no-op PR.
2. Add `paths:` filter; verify a docs-only PR doesn't fire it.
3. (Later — after step 8 lands) extend with tier 2: container =
   pinned webR digest, `R CMD INSTALL --no-test-load`. Verify locally
   inside `just docker-webr-shell` first.
4. (Later still) tier 3 smoke test. May want `r-wasm/webr` on npm or
   the JS shim from `.webr/src/tests-webr/`.

## Risks

- **`Dockerfile.webr` digest divergence.** When webR upstream rolls,
  we bump the digest in `Dockerfile.webr` deliberately. CI's container
  digest must match. Either reference the same `WEBR_BASE` arg or
  hardcode the digest twice and accept the duplication. Lean toward
  hardcoding twice — `Dockerfile.webr` is local-dev, CI is reviewable
  in PR; mismatches caught visually.
- **Tier 2 R CMD INSTALL surface.** The webR upstream install path uses
  `pak` / `rwasm::build_pkg`, not bare `R CMD INSTALL`. Verify whether
  the bare invocation works against `webr-vars.mk`. If not, fall back
  to driving `Rscript -e 'rwasm::build_pkg(...)'` from the workflow.
