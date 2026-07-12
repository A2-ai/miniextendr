# Triage summary: 2026-07-12 project audit → plan branches

Date: 2026-07-13. Re-verified against main @ 656e5cdd (which IS the
audited revision — the audit's SHA matches current main; #1311/#1316/#1317
are already contained in it, 656e5cdd being #1317's merge commit). Every
finding below was independently re-verified before disposition; plan-file
anchors were checked against source, not copied from the audit.

Per-session decision: findings became **plan files on sibling branches**
(one plan-only commit each, branched from origin/main), not GitHub issues.

## Disposition table

| # | Audit finding / worklist item | Disposition | Branch / reason |
|---|---|---|---|
| 1 | CLI product decision: demote vs delegate | **Maintainer decision** | Flagged in the CLI plan's "Deferred" section; pre-existing on-disk `plans/2026-07-01-cli-adopt-or-demote.md` already frames it (recommend: demote until it delegates) |
| 2 | `init package` broken (path collision + pre-dispatch discover) + black-box tests | **Plan** | `fix/audit-cli-init-path-collision` → `plans/2026-07-13-audit-cli-init-path-collision.md` (verified: main.rs:48-51, project.rs:78-79, cli.rs:11-14 vs :117-127, init.rs:36-40) |
| 3 | Delete duplicated CLI scaffold templates | **Maintainer decision** | Depends on item 1; audit itself says don't repair the duplicate independently |
| 4 | Quick Start contradicts scaffold | **Plan** | `docs/audit-quickstart-canonical-path` → `plans/2026-07-13-audit-quickstart-canonical-path.md` (all six GETTING_STARTED.md defect ranges re-verified, + create.R:513 contradiction) |
| 5 | Quick Start as CI e2e test | **Plan** (folded) | into `fix/audit-scaffold-first-run-contract` |
| 6 | First-run `devtools::document()` contract | **Plan** | `fix/audit-scaffold-first-run-contract` → `plans/2026-07-13-audit-scaffold-first-run-contract.md` (new root-cause finding: standalone `create_miniextendr_package` never git-inits — create.R:20-62; only the monorepo path does, create.R:157-159) |
| 7 | sccache/processx hang investigation | **Plan** (folded, bounded) | into `fix/audit-scaffold-first-run-contract` item 5 (2 repro attempts max, then reviews/ entry) |
| 8 | webR Tier 3 testthat `unreachable` | **Plan — confirmed LIVE, not stale** | `ci/audit-webr-tier3-testthat-unreachable` → `plans/2026-07-13-audit-webr-tier3-testthat-unreachable.md`. #1311's fix WORKS (cross-package legs pass in the failing run); the leg was added by #1299 on 2026-07-11 and has never been green. No open issue covers it |
| 9 | `just test` fails on floating stable | **Plan — diagnosis corrected** | `ci/audit-trybuild-rust-src-determinism` → `plans/2026-07-13-audit-trybuild-rust-src-determinism.md`. Mechanism is the documented rust-src COMPONENT skew (nailed during #1239), not a Rust-1.97 channel regression; CI authoritative. Residue (local determinism) is real and planned |
| 10 | 11 broken doc links + no `zola check` gate | **Plan** | `docs/audit-doc-link-anchors` → `plans/2026-07-13-audit-doc-link-anchors.md` (all 7 internal + 4 external re-verified; root cause: GitHub-vs-Zola slugifier divergence + `internal_level = "warn"` in site/config.toml:12; `#preserve-list` target genuinely missing) |
| 11 | status/doctor semantics contradiction | **Plan** (folded) | into `fix/audit-cli-init-path-collision` item 6 (status.rs:100-104,:151,:224,:333 vs workflow.rs:222) |
| 12 | `--json` dishonest | **Plan** (folded) | into `fix/audit-cli-init-path-collision` item 5 (commands.rs:24-41) |
| 13 | CLI README stale (`workflow configure --cran`) | **Plan** (folded) | into `fix/audit-cli-init-path-collision` item 4 (README.md:48; flag absent from parser) |
| 14 | Standard-path warnings (impl-tag nudge ×2, proc-macro-error2) | **Plan** | `fix/audit-standard-path-warnings` → `plans/2026-07-13-audit-standard-path-warnings.md`. Verified distinct from #1292 (clippy) and #1261 (closed by d10462ce). Chain: api `tabled 0.20` → `tabled_derive 0.11` → `proc-macro-error2 2.0.1` |
| 15 | Generated-artifact contract drift | **Plan** | `docs/audit-generated-artifact-contract` → `plans/2026-07-13-audit-generated-artifact-contract.md` (README.md:80-84, rpkg/README.md:39-50, AGENTS.md:130 vs :171-174, CLAUDE.md:269-270 all re-verified) |
| 16 | Minimal showcase package | **Skipped — maintainer decision** | Product-scoping call (new package, naming, docs placement); nothing to plan until decided |
| 17 | fast/standard/exhaustive test lanes | **Skipped — maintainer decision** | P3 ergonomics with wide design latitude; needs the maintainer's preferred lane semantics first |
| 18 | Test output summaries | **Skipped** | Rides on item 17's design |
| 19 | Release-gate clarity (webR/Windows/docs blocking?) | **Skipped — maintainer decision** | The webR plan (item 8) forces the first concrete gate-vs-informational call; generalize from there |
| 20 | Deferred full release-path audit | **Skipped — process item** | Explicitly sequenced by the audit after the front-door fixes; schedule, don't plan |

## Corrections to the audit made during triage

- **Item 9**: "Rust 1.97 changed trybuild output on the floating stable
  channel" is the wrong mechanism. Committed `.stderr` baselines contain
  no stdlib span text; CI's minimal-profile stable passes at the same
  version. The divergence is the local **rust-src component** (documented
  false-positive mode in this repo; CI authoritative). The plan targets
  the surviving residue: deterministic local `just test` + docs caveat.
- **Item 8 framing**: not a runtime regression — the failing leg was
  born red when #1299 introduced it (2026-07-11); build/install/load and
  the #1273/#1311 cross-package isolation checks all pass in the same run.
- **Item 6 root cause sharpened**: the audit's "no Git repository, so hook
  installation was skipped" is a scaffold asymmetry — the monorepo
  creator git-inits (create.R:157-159), the standalone creator does not
  (create.R:20-62). That one missing block is what arms the auto-vendor
  latch on the documented first run.

## Branch inventory (all siblings off origin/main, plan-only commits)

- `fix/audit-cli-init-path-collision`
- `docs/audit-quickstart-canonical-path`
- `fix/audit-scaffold-first-run-contract`
- `ci/audit-webr-tier3-testthat-unreachable`
- `ci/audit-trybuild-rust-src-determinism`
- `docs/audit-doc-link-anchors`
- `docs/audit-generated-artifact-contract`
- `fix/audit-standard-path-warnings`
- `plans/audit-2026-07-12-triage` (this file)

Suggested execution order (dependency-light, impact-first): CLI init →
quickstart docs → scaffold contract → webR tier 3 → doc anchors →
warnings → trybuild determinism → artifact contract. The two docs plans
and the two ci plans have no file overlap with anything else; the
quickstart docs plan and the scaffold plan touch different files but
should coordinate the "git step" wording (noted in both).
