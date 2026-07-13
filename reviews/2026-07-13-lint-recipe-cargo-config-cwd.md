# Lint recipe ignored the generated Cargo configuration

## What was attempted

Run the maintainer-wide `just lint` gate after the documentation reconciliation.

## What went wrong

Cargo resolved the R package's framework dependencies from the locked Git
revision instead of the current worktree, then failed because that revision
predated the `fast-default` feature.

## Root cause

The recipe changed directory only to `rpkg` and selected the nested manifest
with `--manifest-path`. Cargo discovers `.cargo/config.toml` from the process
working directory and its ancestors, so it never read the generated
`rpkg/src/rust/.cargo/config.toml` containing the monorepo path overrides.

## Fix

Run Cargo from `rpkg/src/rust`, matching the other R-package recipes, and
restore Cargo's transient source-shaped lockfile update when the recipe exits.
The lint gate now resolves the current worktree crates and exercises the
configuration that package builds actually use without dirtying the tracked
tarball-shaped lockfile.
