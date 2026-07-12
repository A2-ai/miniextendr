# Plan: audit 2026-07-12 — standalone scaffold first-run contract (git-init, latch guard, e2e CI)

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `fix/audit-scaffold-first-run-contract`.

Covers 2026-07-12 audit worklist items 5, 6, 7. Sibling of
`plans/2026-07-13-audit-quickstart-canonical-path.md` (docs-only);
independent PRs, coordinate only on the Quick Start's "git step" wording.

## Verified state

- **The standalone scaffold never git-inits.**
  `minirextendr/R/create.R:20-62` (`create_miniextendr_package`) =
  `usethis::create_package()` + `use_miniextendr(template_type = "rpkg")`.
  No git anywhere. The **monorepo** path DOES init:
  `create.R:157-159` (`if (!fs::file_exists(usethis::proj_path(".git")))
  usethis::use_git()`) inside `create_miniextendr_monorepo`
  (`create.R:81+`). `use_miniextendr` only *warns* about a missing git
  workspace (`create.R:324-337`), and the git-hooks step degrades to a tip
  (`create.R:163-168`). This is exactly what the audit observed: fresh
  standalone package, no `.git`, hooks skipped.
- **Consequence (deterministic, reproduced by the audit):** with no `.git`
  ancestor, the first `devtools::document()`/`R CMD build`-style run fires
  configure's auto-vendor self-repair → `inst/vendor.tar.xz` created →
  tarball mode → wrapper generation skipped → `NAMESPACE` with `useDynLib`
  and zero exports. The generated template already documents the trap
  (`minirextendr/inst/templates/rpkg/lib.rs:5-27`).
- **The hang (audit worklist 7) is UNCONFIRMED as to mechanism**: the
  audit's `devtools::document()` probe completed the install, then the
  parent sat in `processx_poll` after the child exited; a freshly spawned
  detached `sccache` daemon was the only same-time survivor
  (inherited-output-pipe EOF is a strong inference, not a proven repro).

## Work items (flat order)

1. **git-init in the standalone scaffold.** Port the monorepo block
   (`create.R:157-159`) into `create_miniextendr_package` (after
   `usethis::proj_set(path)`, before `use_miniextendr(...)` so the
   in-repo warning at `create.R:324-337` and the hooks step both see the
   repo). Guard on git availability (`nzchar(Sys.which("git"))` — same
   probe as `create.R:330`) and degrade to a `cli::cli_alert_warning`
   naming the auto-vendor consequence when git is absent. Verify
   `usethis::use_git()` is non-interactive-safe here (it prompts about
   committing in interactive sessions; the scaffold tests run
   non-interactively today via the monorepo path, so precedent exists —
   confirm with a fresh non-interactive `callr` run).
2. **Fail-early latch guard for fresh source trees** (worklist 6 — the
   audit offers "support fully" or "fail early"; recommended and planned
   here: fail early). In `rpkg/configure.ac`'s auto-vendor branch, before
   producing the tarball: if `R/<pkg>-wrappers.R` is absent (fresh source
   tree that has never generated wrappers — an unpacked distribution
   tarball always ships it), abort with guidance to run
   `minirextendr::miniextendr_build()` instead of silently sealing a
   package whose namespace will be empty. This cannot break CRAN/end-user
   tarball installs (they have wrappers on disk) nor monorepo/git dev
   (auto-vendor is already skipped when a `.git` ancestor exists).
   Remember the build rules: edit `configure.ac`, re-run `autoconf`,
   commit the regenerated `configure`; `configure.ac` must not call
   `minirextendr::*` and must not mutate sources.
3. **Port item 2 to the scaffold templates.** `rpkg/` is master;
   `minirextendr/inst/templates/rpkg/configure.ac` gets the same guard,
   then `just templates-approve` locks the delta and `just
   templates-check` verifies. (Template `lib.rs:5-27` prose may then
   soften "produce an empty namespace" to "abort with guidance" — keep the
   warning, update the described failure mode.)
4. **Turn the exact Quick Start into a CI-run e2e test** (worklist 5). A
   testthat test in `minirextendr/tests/testthat/` (the suite already does
   real scaffold/build/install round-trips) that runs the documented
   commands via `callr` from (a) a fresh non-git temp dir and (b) a
   git-rooted temp dir: `create_miniextendr_package()` →
   `miniextendr_build()` → assert: no `inst/vendor.tar.xz` left, generated
   `R/<pkg>-wrappers.R` and `NAMESPACE` agree (exports non-empty), the two
   scaffold example functions are callable, and the process exits (wrap in
   a timeout). Mark it appropriately for runtime cost (the suite already
   tolerates ~8 min; if this doubles a hot path, gate the second variant
   behind the existing slow-test convention used by neighbouring tests —
   read `minirextendr/tests/testthat/` helpers first).
5. **Bound investigation of the processx/sccache hang** (worklist 7).
   Two attempts max (repo rule: don't thrash): (a) stopped/fresh sccache
   daemon + one scaffold build through `processx`/`callr`, watching for a
   surviving `processx_poll` parent; (b) same with `RUSTC_WRAPPER=""` to
   prove/disprove the sccache-inherited-pipe theory. If reproduced: file a
   GitHub issue with the repro (implementer files it — repo policy;
   reference this plan) and, if the fix is a one-liner in our launch path
   (e.g. closing/redirecting fds when spawning cargo), take it here.
   If not reproduced: write `reviews/2026-07-XX-sccache-processx-hang.md`
   recording both attempts, and drop a caveat note in the issue-less
   summary of the PR body.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1     # must print 4.6.x
just worktree-sync                            # FIRST
cd rpkg && autoconf && cd -                   # after configure.ac edits
just configure                                # dev-mode sanity on rpkg
just minirextendr-test 2>&1 > /tmp/audit-scaffold-test.log   # Read it
grep -E '\[ FAIL [0-9]+' /tmp/audit-scaffold-test.log
just templates-approve && just templates-check   # after the template port
just test-bootstrap-vendor                    # latch regression suite still green
```

## Must NOT touch

- The auto-vendor triggers for genuine tarball installs (CLAUDE.md "The
  install-mode latch" — trigger 3 must keep working; the guard keys on
  wrappers-file absence, not on disabling auto-vendor).
- `NOT_CRAN` semantics; no new env-var knobs (`FORCE_VENDOR`-style flags
  are explicitly rejected in CLAUDE.md).
- `rpkg/R/miniextendr-wrappers.R` / `wasm_registry.rs` (generated).
- `docs/GETTING_STARTED.md` (sibling plan owns it).

## Done criteria

- Fresh `create_miniextendr_package()` output is a git repo with hooks
  installed (git present) or a loud warning (git absent).
- A never-built, non-git source tree hitting configure's auto-vendor
  branch aborts with `miniextendr_build()` guidance instead of sealing an
  empty-namespace tarball; unpacked-tarball installs are unaffected
  (`just test-bootstrap-vendor` green).
- The documented Quick Start runs end-to-end in CI from non-git and git
  dirs with callable exports and no leftover latch.
- Hang investigation resolved per item 5 (issue+fix, or reviews/ entry).
- `just minirextendr-test`, `just templates-check` green.

## Escalation rule

If reality diverges from this plan — `use_git()` misbehaves
non-interactively, the configure guard can't distinguish fresh-source from
unpacked-tarball reliably, or the e2e test exceeds a tolerable runtime —
**stop, commit nothing further, and report back. Do not improvise.**
