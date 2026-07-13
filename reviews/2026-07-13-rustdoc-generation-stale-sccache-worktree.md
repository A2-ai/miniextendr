# Rustdoc generation hit a stale sccache worktree

## What was attempted

Regenerate `rust-llm-docs/generated/` and inspect the reverse dependency tree
for a warning emitted by `proc-macro-error2`.

## What went wrong

Both the generator's first compilation attempt and a plain `cargo tree` query
failed before Rust ran. sccache tried to launch `rustc` with its working
directory set to the already-removed
`.claude/worktrees/webr-wasm-review` worktree.

## Root cause

The long-lived sccache process retained a deleted worktree as its compiler
working directory. Even metadata-only Cargo commands invoke `rustc -vV`, so
the stale directory broke commands that do not otherwise compile the project.

## Fix

Run the affected command with `RUSTC_WRAPPER=` to bypass the stale sccache
process. The generator then completed. After refreshing from current `main`,
`cargo tree -i proc-macro-error2 --all-features` confirmed that the package is
no longer present: #1341 disabled `tabled`'s derive defaults and removed the
entire `tabled_derive` → `proc-macro-error2` chain.
