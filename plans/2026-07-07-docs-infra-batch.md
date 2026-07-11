# Docs / build-infra batch — #1083, #1079, #1120, #1032, #1085, #76 (+#1252 note)

Six items touching docs, release mechanics, and scaffolding, plus the #1252
residual-risk paragraph (folded into #1120's docs unit). All premises
re-verified 2026-07-11 against main @ 17f634d8 (original pass: 88c493fd):
docs/RELEASING.md live + no docs-internal/; bare git deps at
`rpkg/src/rust/Cargo.toml:84,90`; unwired Ops example at
`num_bigint_impl.rs:244-245`; monorepo configure.ac 0 × SOURCE_IS_GIT;
create.R anchors `:125/:273/:276/:463`, zero gitattributes anywhere;
top-level site pages present. #1052 (sequencing prerequisite for #1079) is
now CLOSED — #1079 unblocked.

Independent single PRs; branches:
- DI-1 #1083 → `docs/1083-docs-internal-split`
- DI-2 #1079 → `build/1079-release-pinning`
- DI-3 #1120 (+#1252 note) → `docs/1120-ops-trait-examples`
- DI-4 #1032 → `build/1032-monorepo-tarball-guard`
- DI-5 #1085 → `feat/1085-scaffold-gitattributes`
- DI-6 #76 → `docs/76-site-dedup`

Shared commands (each unit; worktree):

```bash
rig default 4.6 && R --version | head -1     # only for units running R recipes
just worktree-sync                            # FIRST
# DI-4/DI-5: just minirextendr-test 2>&1 > /tmp/di-minir.log  (grep '\[ FAIL ')
#            + just templates-approve && just templates-check where templates change
# DI-1/DI-6: just site-docs && just site-build   (verify only — commit NOTHING under site/content/manual or site/public)
# DI-2: bash scripts/bump-version.sh --help or dry-run form first; never run a real bump
# DI-3: compile-verify doc examples via the probe procedure in
#       plans/2026-07-11-issues-1266-1267-1268-altrep-docs-trio.md (probe file
#       under rpkg/src/rust, cargo check from THAT dir, delete before commit)
```

Escalation rule (all units): if reality diverges from this plan — an anchor
doesn't match, a premise no longer holds — **stop, commit nothing further,
and report back. Do not improvise.**

Additional item folded into DI-3 (#1252, disposition PARK+document): add one
residual-risk paragraph to the existing "Aliasing foot-gun (#1104)" callout
in `docs/TYPE_CONVERSIONS.md:635-642`: the wrapper-level guard compares
top-level parameter SEXPs only; a `Vec<&mut [T]>` list ELEMENT aliasing a
direct slice parameter (`f(list(v), v)`) is not detected (the `Vec`
conversion guards intra-list duplicates separately); `Vec<&mut [T]>` params
are rare and the cross-site registry needed to close it is deliberately not
built — reference issue #1252. Do NOT close #1252 (it stays open as the
tracking issue; reference it in the PR body).

---

## #1083 — split RELEASING.md; stop internal leaking to the public site

**STATE CHANGED since the issue was filed.** PR #1082 is **MERGED**, not draft —
`docs/RELEASING.md` exists on main with maintainer content (verified: "Mode A/B",
"cargo-revendor", the #1079 reproducibility gap). Since `scripts/docs-to-site.sh`
globs all `docs/*.md` and `pages.yml` deploys on push, **this maintainer content
is already live on the public manual.** So this is now remediation, not prevention.

Steps:
1. `mkdir docs-internal/` (naturally site-excluded — converter globs `docs/*.md`,
   pages path filter is `docs/**`; confirm `docs-internal/**` is not caught by
   re-reading both).
2. Move the maintainer-facing bulk of `docs/RELEASING.md` → `docs-internal/RELEASING.md`
   (release process, build internals, latch/revendor mechanics, #1079 gap).
3. Keep public install instructions (`remotes::install_github`, vendored-tarball
   install, `cargo install --git --tag` for the CLI) in `docs/` — either a slim
   public `docs/RELEASING.md` or folded into `docs/GETTING_STARTED.md`.
4. Document the `docs/` (public) vs `docs-internal/` (maintainer) convention in
   CLAUDE.md + AGENTS.md orientation (mirror the rule to both, per the AGENTS.md
   note).
5. Optional CI assertion: grep `docs/*.md` for an internal marker
   (`<!-- internal -->` or similar) and fail. Cheap; add it so the leak can't recur.
6. Verify the site build no longer contains the moved content: `just site-docs`
   then grep the generated `site/content/manual/` (gitignored — don't commit it).

Done: `just site-build` output has no maintainer release internals; convention
documented; `Closes #1083`.

## #1079 — release reproducibility: pin framework git deps + refresh stale lock

`rpkg/src/rust/Cargo.toml:84,90` declare `miniextendr-api`/`-lint` (and `-macros`)
with bare `git = "https://github.com/A2-ai/miniextendr"` — no tag/rev. Committed
`Cargo.lock` is stale (issue says 27 commits behind).

- Primary (option 1): extend `scripts/bump-version.sh` to stamp
  `tag = "vX.Y.Z"` into the three framework deps at release time, making the
  released tree self-describing. Verify bump-version.sh's current shape and
  where it already edits versions before adding the stamp step.
- Secondary (option 2): refresh the committed lock as part of the release
  ritual (it re-stamps via `cargo revendor --stamp-lock` — see `just vendor`).
- Document (option 3, honest fallback) that only the built vendored tarball is
  bit-reproducible; point install users there.
- **Coordinate with #1052** (same file, different concern — dev-drift guard vs
  release pinning). The #1052 fix restores the lock's committed shape after dev
  builds; this fix decides what that committed shape should be at a release.
  Sequence #1052 first (it's the smaller, already-planned change) so the
  restore target is stable before adding release stamping.

Done: a release tag's committed tree names its framework version;
`Closes #1079` (leaving #1052 as the dev-side companion).

## #1120 — Ops-trait doc examples advertise unwired trait-ABI registration

Confirmed: the adapter Ops traits (`RBigIntOps` at `num_bigint_impl.rs:256`,
plus `RUuidOps`/`RRegexOps`/`RDuration`/`ROrderedFloatOps`/`RDecimalOps`/
`RIndexMapOps`/`RBuf`/`RBufMut`/`RParallelIterator`/`RParallelExtend`, serde
`RSerialize`/`RDeserialize`/`RSerializeNative`/`RDeserializeNative`) carry doc
examples showing `#[miniextendr] impl <Trait> for MyType {}` — but the trait
declarations don't carry `#[miniextendr]`, so no vtable is generated and the
examples don't compile as written (only `RRngOps`/`RDistributionOps` got wired,
in #1119).

- Chosen direction (option b — docs match reality, cheapest and honest): rewrite
  each trait's doc example to what compiles today: bring the trait into scope and
  call methods on the concrete type, or expose selected methods via an inherent
  `#[miniextendr] impl`. Make the examples doc-tests (` ``` ` not ` ```ignore `)
  where feasible so they can't rot again — but many need R runtime, so gate with
  `no_run` / `ignore` honestly and back them with a compiling fixture instead.
- Do NOT wire the traits ABI-style here (option a) — that's per-trait design work
  (exhaustive-vtable rule, `skip`/`r_name` decisions) and each is its own PR when
  a user actually wants `obj$Trait$method()`. Note this split in the PR.
- Grep every `impl <OpsTrait> for` in the doc comments:
  `grep -rn "impl R.*Ops for\|impl RSerialize\|impl RBuf" miniextendr-api/src/`.

Done: every Ops-trait doc example compiles as written or is backed by a compiling
fixture; PR body records the deferred wiring option per-trait.

## #1032 — extend leaked-tarball configure guard to monorepo template

Confirmed: `minirextendr/inst/templates/rpkg/configure.ac` has `SOURCE_IS_GIT` (6
hits); `minirextendr/inst/templates/monorepo/rpkg/configure.ac` has **0** — no
`.git` walk, so the #1029 guard couldn't be ported.

- Add the `SOURCE_IS_GIT` `.git`-ancestor walk to the monorepo template (mirror
  the standalone `rpkg/` template's block).
- Add the leaked-tarball guard block (scaffold-appropriate wording, no `just`).
  bootstrap.R already sets `MINIEXTENDR_BOOTSTRAP=1` there, so the bypass exists.
- m4 gotchas (from #1029): no `#` in `AC_MSG_ERROR` body; spell out "issue NNNN"
  not "#NNNN"; escape `[`/`]` as `@<:@`/`@:>@`.
- `just templates-approve` → verify `just templates-check`. Confirm the generated
  monorepo `configure` regenerates cleanly (`autoconf` in the template dir if the
  flow requires it).

Done: monorepo template errors loudly on a leaked tarball in a git tree, same as
the standalone template; `templates-check` clean; `Closes #1032`.

## #1085 — scaffold .gitattributes marking generated artifacts -merge

**DRIFT**: gitignore emission now has TWO paths after #1151/#1194 —
`create.R:125` `use_template("gitignore", ...)` (the plain-package path) and
`create.R:273,276` `mx_ignore_patterns(...)` writers (the rpkg/monorepo path),
plus `use_miniextendr_gitignore()` at `create.R:463`. The .gitattributes writer
must hook every place .gitignore is written.

- Decide: template file vs generated-from-`mx_ignore_patterns`-style function.
  Given #1194 just unified ignore patterns into a code function, mirror that —
  add an `mx_gitattributes_patterns()` (or fold into the same helper) rather than
  a static template, so it stays DRY. Content:
  ```
  NAMESPACE          -merge
  R/*-wrappers.R     -merge
  configure          -merge
  man/*.Rd           -merge
  ```
  (safe to list `R/*-wrappers.R` even when untracked — attr just never applies).
  `-merge` beats `merge=ours` (no per-clone `git config` needed).
- Emit `.gitattributes` beside every `.gitignore` write (all three sites above;
  monorepo/rpkg gets the rpkg-subdir path).
- `upgrade_miniextendr_package()` (`minirextendr/R/upgrade.R:14`) must add/refresh
  `.gitattributes` for already-scaffolded packages.
- Tests: `minirextendr/tests/testthat/` — assert a fresh scaffold (plain + rpkg +
  monorepo) writes `.gitattributes` with the four entries; assert upgrade adds it.
- `just templates-check` if any template file is involved; `just minirextendr-test`.

Done: fresh scaffolds + upgraded packages carry `.gitattributes`; proven pattern
from A2-ai/dvs2#255; `Closes #1085`.

## #76 — site: deduplicate top-level content pages vs manual/

Confirmed: hand-written `site/content/*.md` (getting-started, architecture,
api, class-systems, error-handling, externalptr, features, altrep, ...) overlap
the auto-generated `site/content/manual/` (from `docs/`, gitignored since #593).

- Chosen direction: delete the hand-written top-level duplicates, point the Zola
  nav at the `manual/` versions — single source of truth = `docs/`. (The alt
  "curated summaries" option means maintaining two lengths forever; reject it.)
- Map each `site/content/*.md` to its `manual/` equivalent before deleting; some
  top-level pages (`_index.md`, genuinely site-only landing/nav) stay.
- Update `config.toml` / nav / any `[[menu]]` entries and internal links pointing
  at the deleted slugs.
- Verify: `just site-build` succeeds, no broken internal links (Zola errors on
  broken `@/` links — that's the check), nav resolves to manual pages.
- See `site/CLAUDE.md` for the pipeline; the deleted files are tracked (they're
  hand-written, not the gitignored generated `manual/`), so this is a real diff.

Done: one source of truth per topic; site builds; nav points at manual;
`Closes #76`.
