# PR-C: Vendor consistency — bootstrap.R lock regen + `miniextendr_vendor()` reconcile

**Context**: `analysis/build-system-investigation-2026-05-11.md` §7.5, §7.6, §12.2, §12.3.

## Problems

### C1. `bootstrap.R` does not regenerate Cargo.lock

`just vendor` (`justfile:394-428`) does three steps:
1. Move `.cargo/config.toml` aside (so the dev patch override isn't active).
2. Delete `Cargo.lock` and `cargo generate-lockfile` (resolves miniextendr-{api,lint,macros}
   against bare git URLs → "tarball-shape" lock).
3. Restore `.cargo/config.toml` and run `cargo-revendor`.

`bootstrap.R` (`rpkg/bootstrap.R:26-51`) does only step 3. If a developer
has a source-shape (dirty) lock on disk and runs `devtools::build()` /
`rcmdcheck()` / `r-lib/actions/check-r-package`, bootstrap.R packages
that dirty lock into the tarball. Subsequent offline install fails with
"requires a lock file" error.

### C2. `miniextendr_vendor()` diverges from `just vendor`

The R-side `miniextendr_vendor()` (`minirextendr/R/workflow.R:155-`, with helpers in
`minirextendr/R/vendor.R`) does two post-processing steps that are
inconsistent with `just vendor` after PR #408:

1. **`minirextendr/R/vendor.R:137-142`** — overwrites `.cargo-checksum.json`
   to `{"files":{}}` for every vendored crate. Wrong: `cargo-revendor`
   already computes valid checksums; clearing them defeats cargo's
   verification.
2. **`minirextendr/R/workflow.R:169-173`** — strips `checksum = "..."`
   lines from `Cargo.lock`. Wrong: post-#408 the lock retains checksums
   that match the vendored crates.

Two vendor-producing paths (`just vendor` and `miniextendr_vendor()`)
producing different vendor trees is a "works for maintainer, fails for
scaffolded user" trap.

## Files to change

### C1

- `rpkg/bootstrap.R` — perform the same lock-regeneration dance as
  `just vendor` *before* running `cargo-revendor`. Add a shell helper
  (inline in the file) that:
  - Saves `rpkg/src/rust/.cargo/config.toml` if present (`mv`).
  - Deletes `rpkg/src/rust/Cargo.lock`.
  - Runs `cargo generate-lockfile --manifest-path rpkg/src/rust/Cargo.toml`.
  - Restores the config.
  - Then proceeds with the existing cargo-revendor invocation.
- `minirextendr/inst/templates/rpkg/bootstrap.R` — mirror the change
  (this is the template for standalone-rpkg scaffolds; `monorepo/rpkg/bootstrap.R`
  is the lean variant that doesn't auto-vendor at all — leave it alone).

A cleaner alternative: factor the regenerate-lock dance into a shell
helper (e.g., `tools/regenerate-lock.sh`) and have both `just vendor`
and `bootstrap.R` invoke it. Prefer this if not too disruptive.

### C2

- `minirextendr/R/vendor.R:137-142` — delete the `{"files":{}}` overwrite
  block entirely. Cargo-revendor's recompute_checksums (PR #408) already
  writes valid checksums into `.cargo-checksum.json`. Post-trim, we trust
  cargo-revendor's output.
- `minirextendr/R/workflow.R:169-173` — delete the checksum-strip block.
  The lock retains checksums by design post-#408.
- Consider whether `miniextendr_vendor()` should be replaced with a
  thin shell-out to `cargo-revendor` (preferred per investigation §12.3),
  or whether the current implementation minus the broken post-processing
  is fine. **Decision**: keep the R implementation if it has reasons to
  exist (e.g., works when cargo-revendor isn't on PATH); just remove the
  two broken post-processing steps. Document the policy at the top of
  `vendor.R`.

## Tests / verification

### C1

1. Set up a dirty source-shape lock:
   ```bash
   cd /Users/elea/Documents/GitHub/miniextendr/rpkg/src/rust
   # Deliberately put a path+file:/// entry in the lock (simulate dev drift)
   # Easiest: bash ./configure to ensure patch is active, then cargo build
   # to dirty the lock with path+ sources for miniextendr-* crates.
   ```
2. From the source-tree-with-dirty-lock state, run `Rscript -e 'devtools::build("/Users/elea/Documents/GitHub/miniextendr/rpkg")'`.
3. Extract the resulting tarball and inspect `rpkg/src/rust/Cargo.lock`.
4. Assert no `source = "path+..."` lines for miniextendr-{api,lint,macros}.
   (Confirms bootstrap.R regenerated the lock before vendoring.)
5. Try `R CMD INSTALL <tarball>` in an offline test (e.g.,
   `HTTP_PROXY=http://0.0.0.0:1` to force offline). Assert success.

### C2

1. Clean checkout. Run `just vendor`. Snapshot `rpkg/vendor/` (e.g.,
   `find rpkg/vendor -type f -name 'Cargo.toml' -o -name '.cargo-checksum.json' | sort | xargs sha256sum > /tmp/just-vendor.sha256`).
2. Reset (`rm -rf rpkg/vendor rpkg/inst/vendor.tar.xz`).
3. Run `Rscript -e 'minirextendr::miniextendr_vendor("rpkg")'`. Snapshot same paths.
4. Assert sha256 manifests are byte-identical OR document remaining
   intentional divergence (e.g., compression timestamp).
5. Assert `rpkg/vendor/<crate>/.cargo-checksum.json` does NOT contain
   `"files":{}` (with empty map) — it should have populated file hashes.

## Risk

PR-C is the highest-risk of the four. Changes to the vendor / lock
plumbing affect every install path that hits bootstrap.R. Must be
verified against the full test plan §T7 cells before merge.

Suggested mitigation: split into two commits (C1 first, C2 second) and
run `just r-cmd-check` between them. Both commits must keep the
"vendor.tar.xz inside tarball produces working offline install" property.

## Not in scope

- A formal cargo-revendor shell-out replacement for `miniextendr_vendor()`.
  That's a larger refactor; file as a follow-up.
- Test plan §T7 full harness (separate work).

## PR title

`fix(vendor): bootstrap.R lock regeneration + miniextendr_vendor() consistency`

## PR body

Reference §7.5, §7.6 of investigation. State the symptom: scaffolded-user
vendor trees diverge from maintainer-built ones, and dirty-lock dev
builds ship broken tarballs. Link to the two source-of-truth lines
(`vendor.R:137-142`, `workflow.R:169-173`) being deleted.

## Branch

`fix/vendor-consistency`
