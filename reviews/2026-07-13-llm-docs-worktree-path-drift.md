# LLM docs inventory: worktree path drift

## What was attempted

Regenerate the tracked LLM-ready rustdoc corpus in a fresh worktree.

## What went wrong

The cargo-revendor impl inventory changed its `Source:` line from the previous
author's absolute worktree path to the current worktree path. No cargo-revendor
API had changed.

## Root cause

The generator passed an absolute JSON filename only for the standalone
`cargo-revendor` workspace. The inventory renderer faithfully embedded that
argument, while root-workspace inventories received stable relative paths.

## Fix

Pass `cargo-revendor/target/doc/cargo_revendor.json` relative to the repository
root, matching every other corpus input. Regeneration is now independent of the
checkout/worktree location and no longer records local filesystem paths.
