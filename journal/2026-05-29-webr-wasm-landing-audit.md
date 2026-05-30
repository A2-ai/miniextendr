# webR / wasm support — landing audit

**Date:** 2026-05-29
**Branch:** `review/webr-support` (worktree off `origin/main` @ `2ea76d4b`)
**Scope:** retrospective on how webR/wasm support landed — what works, what was
over-built, what was self-inflicted churn, and the loose ends still open.

## TL;DR

The feature **works and is CI-validated** (tier-1/2/3 all green as of the #750
merge). The architecture is sound and proportionate to the genuinely hard
problem (linkme doesn't compile on wasm32). The "too much / stupid things"
worry is mostly **not** about over-engineering the core — it's about a handful
of **self-inflicted detours that were merged and then reverted/questioned in the
same two sprints**, one **speculative sub-feature that no CI exercises**, and
**stale tracking/docs** that now misrepresent the design. None are load-bearing
bugs; all are cleanup.

## State of webR/wasm support

- **Target:** `wasm32-unknown-emscripten`, nightly + `-Z build-std=std,panic_abort`.
- **Core mechanism:** linkme `#[distributed_slice]` (used for `MX_CALL_DEFS`,
  `MX_ALTREP_REGISTRATIONS`, `MX_TRAIT_DISPATCH`) `compile_error!`s on wasm32.
  Replacement: a host-generated `rpkg/src/rust/wasm_registry.rs` snapshot
  (~12.1K lines) baked at build time. `registry.rs` `cfg`-branches: native uses
  the linkme slices; wasm reads a `OnceLock` populated at `R_init` from the
  snapshot. The five *host-only* slices (wrapper-gen etc.) are
  `cfg(not(wasm32))`-gated, as is the 471-line `wasm_registry_writer`.
- **Build wiring:** `configure.ac` detects `CC=emcc`, sets
  `IS_WASM_INSTALL`/`CARGO_BUILD_TARGET`/nightly/build-std and side-module
  `RUSTFLAGS` (`-Zdefault-visibility=hidden` [load-bearing], `-C
  relocation-model=pic` [probably redundant — #745]). `Makevars.in` skips the
  cdylib + wrapper-gen pass on wasm. `build.rs` validates the snapshot's
  `generator-version` header against `wasm_registry_writer::GENERATOR_VERSION`.
- **Worker thread:** stays *feature-enabled* on wasm (so the native-generated
  snapshot doesn't skew), but every spawn path is gated
  `not(target_family = "wasm")` → runs inline (#758).
- **CI (3 tiers, `.github/workflows/webr.yml`):** tier-1 `cargo check -p
  miniextendr-api` (every matching PR); tier-2 full `R CMD INSTALL` via emcc
  side-module link inside the webR container; tier-3 Node+webR session that
  `library(miniextendr)`s the wasm install. Container pulled from a
  self-hosted mirror (`ghcr.io/a2-ai/webr-mirror`, weekly refresh).
- **Footprint:** ~1.8K hand-written lines (workflows, Dockerfile, `docs/WEBR.md`,
  two smoke runners, the writer) + ~12.1K generated + scattered cfg-gates in 15
  Rust source files across the api/macros crates.
- **Timeline:** Sprint A 2026-05-06→11 (#397, #471–#497: gating + codegen +
  tier-1 + docker + docs). Sprint B 2026-05-27→29 (#742–#758: make it actually
  *install and load* — tier-2/3, RUSTFLAGS, force_link, worker-inline).

## What was done *well* / proportionately

- The linkme-snapshot design is the right call and is cleanly cfg-gated; the
  generator-version + content-hash guard is exactly the kind of fail-loud check
  this needs.
- `-Zdefault-visibility=hidden` (#751/#756) is a real fix with real evidence
  (~3000 mangled symbols otherwise leak into the side-module export table and
  break webR's `dyn.load`), and it matches savvy/webR-maintainer precedent.
- `docs/WEBR.md` (285 lines) is genuinely good — it explains *why* nightly,
  *why* hidden visibility, the two-R-install model, the `/opt/webr/dist` import
  gotcha. This is the strongest artifact of the whole effort.
- Three-tier CI matches the savvy/webR norm; tiers are well-justified (link vs
  load are different failure classes).

## Where we did too much / detours that landed then got undone

1. **Makevars GNU-make detour (#754 reverted #475/#481/#744).** Earlier commits
   wrote `ifeq`/`else ifeq`/order-only (`| $(CARGO_AR)`) into `Makevars.in` —
   GNU-make-only constructs in a file that must be portable POSIX make. #754
   ripped them back out. Net: the wasm branch went in via a non-portable
   mechanism and had to be rewritten. Self-inflicted.

2. **PIC flag added on a hypothesis, now suspected redundant (#745).** #744/#751
   inject `-C relocation-model=pic` on the theory emcc rejects non-PIC objects
   for `-s SIDE_MODULE=1`. tier-2 links fine without proof it's needed; wasm32
   is PIC-ish by default. #745 tracks dropping it. Flag-first, validate-later.

3. **force_link static→fn dance (#756).** `-Zdefault-visibility=hidden` (#751)
   immediately hid `miniextendr_force_link`, the symbol stub.c relies on, so
   #756 had to re-export it. #751 and #756 are two halves of one change merged
   as two PRs three days apart — #751 shipped a known-incomplete state.

4. **Plan-doc churn.** #471 added 876 lines across `plans/webr-support.md`,
   `plans/wasm-registry-codegen.md`, `plans/webr-dockerfile.md`; #497 + #512
   deleted all of them ~1–9 days later (plans/ retired wholesale). Normal-ish,
   but it means the design rationale briefly lived in three places and now lives
   only in `docs/WEBR.md` + closed issues.

## Speculative / unvalidated work

5. **Cross-package wasm stubs (#493/#742) are inert and unchecked.**
   `tests/cross-package/{producer,consumer}.pkg/src/rust/wasm_registry.rs` are
   empty placeholders (`content-hash: 0000000000000000`). `tests/cross-package/**`
   is in webr.yml's `paths:` trigger, **but the only wasm job that compiles
   anything is `cargo check -p miniextendr-api`** — cross-package is never built
   for wasm in CI. The feature these stubs would serve (cross-crate trait
   dispatch under wasm, #495) is **known-broken/open**. So we shipped wasm
   scaffolding for a fixture that is never deployed to webR and never
   wasm-checked, in support of a capability that doesn't work yet. Smallest,
   most defensible candidate for "did too much."

6. **Mirror infra built partly to protect a bad pin (#496/#746/#748, #755).**
   The mirror (102-line weekly workflow + cutover + private-package creds) is
   defensible for CI reliability. But its headline rationale — "upstream's
   `:main` digest is an orphan and could be GC'd" — exists because we pinned an
   *untagged* `:main` snapshot in the first place. #755 now wants to bump to a
   *tagged release* digest, which would have avoided the orphan problem. Mirror
   = fine; the orphan-pin that motivated it = avoidable.

## Coupling worth naming (not a mistake, but a cost)

7. **Snapshot-from-native forces `worker-thread` on for wasm (#758).** Because
   `wasm_registry.rs` is generated by a *native* build with rpkg's default
   features (which include `worker-thread`, Cargo.toml:70), dropping the feature
   on wasm would leave the snapshot with ~31 dangling entries. So the feature
   must stay enabled and every spawn path is `not(target_family="wasm")`-gated
   to run inline. This contradicts #470's stated non-goal ("worker-thread under
   WASM … already feature-gated; verify, don't reimplement") — the snapshot
   design quietly forced a *third* worker code path. The #758 fix is clean and
   well-documented, but the need for it is a hidden tax of generating the
   snapshot from a differently-configured build.

## Stale tracking / docs (cheap fixes, currently misleading)

8. **#470 umbrella links three dead files.** Its design pointers
   (`plans/webr-support.md`, `plans/wasm-registry-codegen.md`,
   `plans/webr-dockerfile.md`) are all GONE (retired #512). The canonical entry
   point for the whole effort 404s on its own design. Its step checkboxes also
   still show only Step 1 ticked though Steps 2–8 all shipped.

9. **`tests/webr-smoke.sh` Phase 3 is rotted.** `docs/WEBR.md` itself admits the
   local smoke's Phase 3 "still uses the old `npm install file:///opt/webr/src`
   approach, which is broken in the current image." So the *local* end-to-end
   runner (441 lines, "resurrected" once in #743) is half-dead while CI tier-3
   (`smoke.mjs`) is the real validator. Two smoke runners, one rotting.

## Recommendations (flat priority)

- **P1 — Fix #470 umbrella links + checkboxes,** or close it (Steps 1–8 done;
  only #495 is genuinely incomplete). It's the front door and it's misleading.
- **P1 — Resolve #745** (drop or keep PIC) with an actual no-PIC tier-2 run, and
  delete the flag if it links. Stop carrying an unvalidated RUSTFLAG.
- **P2 — Decide cross-package-on-wasm (#493/#495):** either add a tier-1
  `cargo check --target wasm32 -p {producer,consumer}` so the stubs aren't
  dead, or pull the stubs + drop `tests/cross-package/**` from webr.yml's paths
  until #495 is real. Right now they're cargo-cult scaffolding.
- **P2 — Either fix `tests/webr-smoke.sh` Phase 3 to the `/opt/webr/dist` import
  or retire it** in favour of `smoke.mjs`. Don't keep a known-broken runner.
- **P3 — #755 (tagged pin) + #747 (drop mirror creds once public):** finish the
  mirror hygiene so the "orphan digest" framing goes away.
- **P3 — Note the worker-thread-stays-on-for-wasm coupling in
  `miniextendr-api/CLAUDE.md`** so the next person doesn't "helpfully" drop the
  feature on wasm and silently skew the registry.

## Bottom line

We did *not* over-build the core — the snapshot codegen and 3-tier CI are
proportionate. What accumulated is **detour debt** (Makevars portability, PIC
flag, force_link split across PRs), **one speculative inert sub-feature**
(cross-package wasm stubs / #495), and **stale front-door docs** (#470 links,
the rotting local smoke). All cleanup, no firefighting. The single most
"did-too-much" item is the cross-package wasm scaffolding for a capability that
doesn't work and a deployment that never happens.
