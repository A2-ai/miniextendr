# Lockfile + configure + minirextendr unification

> **Status (2026-05-07): DONE.** All nine items shipped between PR #400 and
> #408 (see Related at bottom for the full mapping). Item 8
> (`[[patch.unused]]` rule) was added to `lock-shape-check.R`, the
> `just lock-shape-check` recipe, and both pre-commit hooks in the
> close-out PR; the same PR removed the obsolete `checksum = ` block from
> `rpkg/tools/lock-shape-check.R` (and the minirextendr template + scaffold
> hook) that #408 forgot to update. This file is kept as historical
> reference.

## Goal

The committed `src/rust/Cargo.lock` in any miniextendr-based R package (rpkg
in this repo, scaffolded packages from minirextendr) should be **the same
shape across every mode** — dev-mode, CRAN-mode, install-via-`R CMD INSTALL`,
install-via-`devtools::install()`, install-via-`R CMD build` + tarball,
CI builds. Any drift is a bug. Any path that silently dirties the lock without
restoring it is a bug.

> **Reality check:** a single byte-for-byte lock that works for both source
> resolution (with `[patch."git+url"]` rewriting framework crates to local
> paths) and offline tarball install (which needs `git+url#<sha>` for source
> replacement) is **unreachable** within cargo's design — cargo records
> `path+file:///<absolute-path>` for patched deps in the lock, which isn't
> portable across machines, and the lock is the projection of resolution
> rather than declaration. So this plan accepts that the canonical committed
> shape is *tarball-shape* and aims to:
>
> - **Make the canonical shape self-healing**: every recipe that mutates
>   the lock restores tarball-shape on exit.
> - **Make drift detection loud and early**: configure / install fails fast,
>   not at offline install time.
> - **Make recovery trivial**: a one-call lock-only repair, available in
>   scaffolded packages.
> - **Eliminate the trap categories** that aren't fundamental to cargo:
>   tarball leaks, `.cargo/config.toml` deletion, cargo-revendor patch
>   blindness (#338), missing pre-commit hooks in scaffolded packages.

## Background — what's tarball-shape and why?

See [`docs/CARGO_LOCK_SHAPE.md`](../docs/CARGO_LOCK_SHAPE.md) (PR #401). Summary:

1. **No `checksum = "..."` lines.** Vendored crates ship with empty
   `.cargo-checksum.json`; cargo refuses to verify registry checksums against
   them at offline install time.
2. **No `path+...` source for `miniextendr-{api,lint,macros}`.** Cargo's
   source-replacement matches lockfile entries against the vendored copy by
   the source identifier; only `git+https://github.com/A2-ai/miniextendr#<sha>`
   matches the vendored layout.

Dev iteration silently violates both:

- `cargo build` adds `checksum =` lines back for crates.io deps.
- `[patch."git+url"]` in `.cargo/config.toml` rewrites framework crates to
  `path+file:///` entries.

Today's recovery: `just vendor` (heavy — also rebuilds `vendor/` and
`inst/vendor.tar.xz`), `just update` (lock-only, this repo's `chore`
recipe), `miniextendr_vendor()` (heavy, scaffolded packages).

## Trap inventory — why the lock keeps drifting

These are the actual recurring failure modes hit during this session and
prior sessions, in priority order:

1. **`cargo-revendor` ignores `[patch."git+url"]` in `.cargo/config.toml`** (#338).
   Vendored copy of `miniextendr-{api,lint,macros}` always reflects the
   committed git SHA, never the local workspace. `vendor-sync-check` fails
   by construction on any PR that edits a workspace crate (#337). Workaround
   today: explicit `--source-root .` flag in `just vendor`. Scaffolded
   packages without a `--source-root` setup hit the unworked path.

2. **`inst/vendor.tar.xz` leaks across recipes**. Once the file exists,
   configure flips into tarball mode and stays there for every subsequent
   install — including dev iteration, which then doesn't see workspace edits
   via `[patch."git+url"]`. Hit during this session: a stale tarball from a
   prior `R CMD build` made `just rcmdinstall` silently install in tarball
   mode against the OLD pinned commit, and macro changes never propagated.
   Existing mitigation: `trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT` in the
   build/check recipes, but it's incomplete — fail mid-recipe, kill -9, etc.
   leave the tarball behind.

3. **`.cargo/config.toml` deletion**. `R CMD INSTALL` runs in a temp dir;
   when configure runs there it writes `.cargo/config.toml` inside the temp
   tree, but the source tree's `.cargo/config.toml` ends up missing
   afterward (mechanism: `R CMD INSTALL --no-clean-on-error` cleanup, build
   hooks, or worktree detection edge cases). Re-running configure
   regenerates it, but only if you run a `just` recipe — bare `cargo build`
   doesn't trigger configure. End user symptom: builds against `git+url#`
   sources instead of local siblings, monorepo edits invisible.

4. **`devtools::document()` re-introduces `checksum =` lines silently**.
   It re-compiles the cdylib in source mode, which dirties the lock. The
   pre-commit hook catches the eventual commit attempt, but only after
   the user has staged. No early warning.

5. **`[[patch.unused]]` noise**. After `cargo build` records that the
   `[patch.crates-io]` config is set in `.cargo/config.toml` for a crate
   the lock doesn't depend on, it appends `[[patch.unused]]` entries. Not
   a tarball-shape violation per `lock-shape-check`, but spurious diff in
   every commit-staging context.

6. **No pre-commit hook in scaffolded packages**. `minirextendr` doesn't
   ship `.githooks/pre-commit` with `lock-shape-check` equivalent.
   Scaffolded users have no automated tripwire for the same drift.

7. **No drift detection at configure time**. Install proceeds against a
   broken lock until cargo's offline source replacement chokes deep into
   the build, with an error message that doesn't point at the cause.

## Plan items, flat priority order

### 1. cargo-revendor: read `[patch."git+url"]` from `.cargo/config.toml` (#338)

Today, `cargo-revendor` follows `Cargo.lock` source URLs verbatim. When the
lock has `git+https://github.com/A2-ai/miniextendr#<sha>` for
`miniextendr-api`, vendor/ gets the GitHub-hosted snapshot regardless of
`.cargo/config.toml`'s `[patch."git+url"] miniextendr-api = { path = "..." }`.

**Change:** before vendoring a git-source crate, parse `.cargo/config.toml`
(merging `--config` CLI args), look up `[patch."git+<url>"]` for
`<crate-name>`, and if a `path = "..."` entry exists, copy from that path
instead of the registry/git fetch.

**Acceptance:**
- `just vendor` from a worktree with edits to `miniextendr-api/src/foo.rs`
  produces `rpkg/vendor/miniextendr-api-0.1.0/src/foo.rs` with those edits,
  *without* `--source-root` being passed.
- `just vendor-sync-check` passes after `just vendor` regardless of which
  workspace crate was touched.
- Integration test in `cargo-revendor/tests/integration.rs` covers the
  patched-git-source case.
- `--source-root` becomes redundant for the framework-crate case (kept for
  back-compat / cross-monorepo scenarios). Document the redundancy.

**Files:**
- `cargo-revendor/src/main.rs` — patch-table parsing
- `cargo-revendor/src/vendor.rs` — vendor copy redirection
- `cargo-revendor/tests/integration.rs` — new patch-from-config case
- `justfile` `vendor:` recipe — drop `--source-root .` once auto-detection
  works (or keep as belt-and-suspenders).

Closes #338. Unblocks the symptom-level #337.

### 2. cargo-revendor: recompute valid `.cargo-checksum.json` post-trim

Today `cargo-revendor` clears `.cargo-checksum.json` to empty after
CRAN-trim (strip-toml-sections / strip-tests / etc), and the lock has to be
post-stripped to remove `checksum =` lines or cargo's offline install fails.

**Change:** after CRAN-trim writes a vendored crate, recompute the SHA-256
of every remaining file (excluding `.cargo-checksum.json` itself) and write
the JSON in cargo's format. Cargo's offline source replacement will verify
the lockfile's `checksum =` lines against these recomputed values.

**Net effect:** the committed `Cargo.lock` can keep `checksum =` lines for
crates.io deps. One of the two divergences between dev-shape and
tarball-shape collapses. `just update` and `lock-shape-check` both stop
caring about checksum lines.

**Caveats:**
- This only collapses the *checksum* divergence. The `path+` vs
  `git+url#<sha>` divergence for framework crates remains (cargo limitation,
  see plan TL;DR).
- Need to verify cargo actually verifies vendored checksums against the
  lockfile when source-replacement is active. (Believed yes; needs
  empirical confirmation in an integration test.)

**Acceptance:**
- `cargo-revendor` writes valid `.cargo-checksum.json` for every crate in
  `vendor/` after trim.
- An offline install in tarball mode succeeds with a lock that retains
  registry `checksum =` lines.
- `lock-shape-check` either drops the `checksum =` rule or the rule gets
  inverted (require checksums for crates.io deps).
- Integration test: build, modify a file in vendored crate manually, expect
  cargo to error with a checksum mismatch.

**Files:**
- `cargo-revendor/src/vendor.rs` — `clear_checksums` → `rewrite_checksums`
- `cargo-revendor/src/checksum.rs` — new module if needed
- `justfile` `vendor:` recipe — drop the post-vendor `sed '/^checksum = /d'`
- `justfile` `lock-shape-check:` — re-evaluate the rule
- `.githooks/pre-commit` — drop or invert the checksum block

### 3. Configure-time drift detection

`configure` already knows whether it's in source or tarball mode. It can
spot-check the committed `Cargo.lock` and fail (or warn) loudly when the
shape doesn't match what the install path expects, instead of letting the
build chug along until cargo's offline mode chokes on a checksum mismatch.

**Change:** in `rpkg/configure.ac` (and the scaffolded-package equivalent
in `minirextendr/inst/templates/rpkg/configure.ac`), add a
`AC_CONFIG_COMMANDS` block that runs `tools/lock-shape-check.R` (a
hand-rolled script — `configure.ac` rule says no `minirextendr::*` calls).
The script walks `src/rust/Cargo.lock` and:

- In **source mode** (no `inst/vendor.tar.xz`): no shape requirement; the
  lock will be silently rewritten by cargo. Skip.
- In **tarball mode**: assert tarball-shape (no `path+...` for framework
  crates after item 2 lands; no other rules). On failure, print the
  concrete error and exit non-zero with a recovery hint.

**Acceptance:**
- Unpacking a tarball with an in-source-shape lock fails at configure with
  a clear message, before any cargo invocation.
- Source installs are unaffected.

**Files:**
- `rpkg/tools/lock-shape-check.R` (new, plain R + grep)
- `rpkg/configure.ac` — `AC_CONFIG_COMMANDS([lock-shape-check], ...)`
- `minirextendr/inst/templates/rpkg/configure.ac` — same
- `minirextendr/inst/templates/rpkg/tools/lock-shape-check.R` — same
- `just templates-approve` to record the new template files

### 4. Trap-cleanup: `inst/vendor.tar.xz` removal on every recipe that touches it

Today `r-cmd-build` and `r-cmd-check` use `trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT`,
but `devtools-build` doesn't, and the trap doesn't fire under SIGKILL or
abnormal mid-shell exit. This session hit a leaked tarball from an unknown
source.

**Change:**
- Audit every recipe that *creates* `inst/vendor.tar.xz`: `vendor`,
  `r-cmd-build`, `r-cmd-check`, `devtools-build`, `devtools-check`, and
  the minirextendr equivalents. Each must have a working trap-cleanup
  joined into one logical line (the just-recipe-line-per-bash gotcha is in
  CLAUDE.md memory).
- Add `just clean-vendor-leak` recipe that explicitly removes the tarball
  + warns. Hook it into a `just doctor` (next item).
- minirextendr equivalent: `miniextendr_clean_vendor_leak()`, called from
  `miniextendr_doctor()`.

**Acceptance:**
- After any normal recipe completion, `inst/vendor.tar.xz` is absent.
- After any abnormal recipe exit, `just doctor` flags the leak.

### 5. `miniextendr_repair_lock()` — fast lock-only repair (minirextendr)

minirextendr today only has `miniextendr_vendor()`, which is heavy
(~minutes — runs `cargo-revendor`, builds `vendor/`, compresses tarball).
For the common case of "I just ran `devtools::install()` and now my lock
has checksums + path+ entries", users need a sub-second repair.

**Change:** mirror this repo's `just update` rpkg variant as a public
exported function `miniextendr_repair_lock(path = ".")`.

```r
miniextendr_repair_lock <- function(path = ".") {
  with_project(path)
  cargo_cfg <- usethis::proj_path("src", "rust", ".cargo", "config.toml")
  cargo_cfg_bak <- paste0(cargo_cfg, ".tmp_repair")
  if (fs::file_exists(cargo_cfg)) fs::file_move(cargo_cfg, cargo_cfg_bak)
  withr::defer(if (fs::file_exists(cargo_cfg_bak)) fs::file_move(cargo_cfg_bak, cargo_cfg))

  rust_dir <- usethis::proj_path("src", "rust")
  withr::with_dir(rust_dir, system2("cargo", c("update")))
  # After item 2 lands, the strip step goes away.
  lockfile <- usethis::proj_path("src", "rust", "Cargo.lock")
  lock_content <- readLines(lockfile, warn = FALSE)
  lock_content <- lock_content[!grepl("^checksum = ", lock_content)]
  writeLines(lock_content, lockfile)
}
```

**Acceptance:**
- Documented in `getting-started.Rmd` as the go-to recovery for "I see
  diff in `Cargo.lock` after dev iteration".
- Added to `miniextendr_doctor()` output: when the lock is in source-shape,
  recommend the call.
- Dual-export: also expose as `miniextendr` exposed function (rpkg) for
  monorepo users.

**Files:**
- `minirextendr/R/repair-lock.R` (new)
- `minirextendr/vignettes/getting-started.Rmd` — section update
- `minirextendr/R/diagnose.R` — `miniextendr_doctor()` lock check

### 6. Pre-commit hook in scaffolded packages

`miniextendr/.githooks/pre-commit` blocks commits that introduce
`checksum =` lines or `path+` sources into `rpkg/src/rust/Cargo.lock`
(and the related staged-without-NAMESPACE-update rule). Scaffolded packages
have no equivalent.

**Change:** add `minirextendr::use_miniextendr_lock_hook()` that drops a
hook into `.git/hooks/pre-commit` (or sets `core.hooksPath` to a tracked
`.githooks/` dir, matching this repo's pattern). The hook runs a check
that's lock-shape-aware after item 2 lands (only checks for `path+`).

**Acceptance:**
- Scaffolded user runs `use_miniextendr_lock_hook()` once per checkout.
- Subsequent commit attempts that would commit a source-shape lock fail
  with a recovery hint (`miniextendr_repair_lock()`).
- Documented in `getting-started.Rmd`.

**Files:**
- `minirextendr/R/use-hooks.R` (new) or extend `minirextendr/R/git-hooks.R`
- `minirextendr/inst/templates/githooks/pre-commit`
- `minirextendr/vignettes/getting-started.Rmd`

### 7. Audit `R CMD INSTALL` config.toml fate

Diagnose what's actually deleting `rpkg/src/rust/.cargo/config.toml` after
some operations. Hypotheses:

- `R CMD INSTALL` clean-up step (probably not — it operates on temp dir).
- `pkgbuild::compile_dll()` cleaning a cargo target dir that includes the
  config (definitely not — config isn't in target/).
- `--no-clean-on-error` semantics (worth ruling out).
- Some `tools/*.R` script side-effect (worth ruling out).

**Change:** instrument `bash ./configure` with a `trap` that captures the
config.toml state at exit. Run a session of:
`just configure; ls .cargo/`, then various install invocations, observing
when the file disappears.

If the cause is benign (e.g., the config IS preserved, but my session was
seeing a different worktree state), document and close. If it's a real
delete, add a self-healing check: configure detects missing config.toml in
source mode and re-runs the writer.

**Acceptance:**
- Root cause documented in a `reviews/` file.
- If real: a follow-up self-heal commit.
- If illusory: a note in CLAUDE.md so the next session doesn't chase it.

### 8. `[[patch.unused]]` noise eradication

When `[patch.crates-io]` in `.cargo/config.toml` references crates the
current lock doesn't depend on, cargo appends `[[patch.unused]]` entries
to the lock. Not a tarball-shape violation, but spurious commit-time diff.

**Change:** investigate where the `[patch.crates-io]` extras come from. If
configure is writing patches for *all* miniextendr crates regardless of
which the current rpkg manifest depends on, narrow the patch to only what
the manifest needs. Probably `tools/configure-cargo.R` or
`configure.ac`'s `cargo-config` AC_CONFIG_COMMANDS block.

**Acceptance:**
- After `just configure && cargo build`, the lock has no `[[patch.unused]]`
  block.
- `lock-shape-check` adds a rule to flag any `[[patch.unused]]` entry.

### 9. Documentation: cross-mode cheat-sheet

Once items 1–8 land, the user-facing flow shrinks. Update:

- `docs/CARGO_LOCK_SHAPE.md` (PR #401): drop the checksum-strip step from
  the recovery snippet (after item 2), document the new
  `miniextendr_repair_lock()`, point at the configure-time drift check.
- `docs/CRAN_COMPATIBILITY.md`: update "What `just vendor` actually does"
  to reflect the new cargo-revendor patch-aware behavior.
- `minirextendr/vignettes/getting-started.Rmd`: replace the manual
  "delete `inst/vendor.tar.xz` after R CMD build" with a pointer to the
  recipe trap-cleanup that does it automatically (item 4).

## Out of scope (file as separate issues)

- **Single byte-for-byte lock for all modes**: cargo limitation, see
  TL;DR. Document the limitation in the new CARGO_LOCK_SHAPE doc.
- **Rewriting Cargo.lock at install time**: violates the configure.ac rule
  ("never mutate sources during ./configure"). The drift detection
  (item 3) is the in-bounds version.
- **Eliminating `[patch."git+url"]` entirely**: would require workspace
  refactoring (e.g., embedding miniextendr-{api,lint,macros} as path deps
  inside `rpkg/src/rust/`), which conflicts with monorepo dev ergonomics.

## Acceptance for the whole plan

After all items land:

- `just configure && devtools::install("rpkg") && devtools::document("rpkg") && git diff src/rust/Cargo.lock` produces **zero diff**.
  - Achieved via item 2 (no checksum drift) + the existing patch override
    (path+ drift is reset by re-running `just update` or `miniextendr_repair_lock()`,
    surfaced via item 7).
- `R CMD build rpkg` then `R CMD INSTALL miniextendr_X.Y.Z.tar.gz` succeeds
  even on a fresh checkout where someone forgot to run `just vendor` —
  configure (item 3) flags the bad lock with a clear error before cargo
  starts building.
- A new scaffolded package created with `create_miniextendr_package("foo")`
  has a pre-commit hook installed (item 6), `miniextendr_doctor()` flags
  drift (item 5), and `miniextendr_repair_lock()` is one call away.
- `vendor-sync-check` passes on every PR that edits a workspace crate
  (item 1).

## Order of execution (during the dedicated session)

1. **Item 1 first** — unblocks #337/#338, lets `vendor-sync-check` work
   on cross-cutting PRs. Foundation for the rest.
2. **Item 2 next** — collapses one of the two lock-shape divergences.
   Largest reduction in idiosyncrasy.
3. **Item 3 + 7** — drift detection + audit. Catches anything still
   broken before it hurts.
4. **Item 4 + 8** — eliminate the trap categories.
5. **Item 5 + 6** — minirextendr parity (repair fn + hook).
6. **Item 9** — docs sweep last, when everything else is stable.

Tests, integration tests, and CI-gating updates run alongside each item;
no separate "test phase". Per project rule.

## Related

- #337 — `vendor-sync-check` chicken-and-egg, closes when item 1 lands.
- #338 — `cargo-revendor` patch reading, item 1.
- PR #400 — `just update` recipe + lock refresh (this session, merged).
- PR #401 — `docs/CARGO_LOCK_SHAPE.md` (this session, open).
- `docs/CRAN_COMPATIBILITY.md` — current install-mode reference.
- CLAUDE.md memory: `feedback_no_branch_scoping` — fix issues as
  encountered, no "out of scope for this branch". This plan is the
  inversion: the issues are now scoped *into* a dedicated session because
  they're cross-cutting.
