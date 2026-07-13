# rust-llm-docs refresh exposed stale cache state and hidden rustdoc warnings

## What was attempted

Regenerate the committed LLM-ready Rust API corpus, expand the generator to
the maintained `full` feature set, and enforce warning-free rustdoc output.

## What went wrong

The first build could not start because the long-running `sccache` process had
retained a deleted worktree as its working directory. After bypassing that
environmental state, the full-feature warning gate exposed broken/private
intra-doc links and malformed generic type names in source documentation that
the old partial feature list never compiled. The repository-wide `doc-check`
then failed because its R-package command still patched crates.io even though
that manifest now depends on the canonical git URL. Finally, the corpus docs
incorrectly said the benchmark crate had no library target and omitted it.

## Root cause

The generator had no `just` entry point or drift check, used a manually curated
feature list that had fallen behind `full`, and did not deny rustdoc warnings.
The renderer also silently stringified unknown type shapes, which leaked Python
dictionary representations for format-57 `function_pointer` values. The doc
recipe and corpus crate list had drifted independently from the current
workspace layout.

## Fix

Added `just llm-docs` and `just llm-docs-check`, switched generation to the
maintained `full` aggregate with rustdoc warnings denied, repaired the source
docs, removed an unused `tabled` derive dependency path that emitted a future
incompatibility warning, and made item/type schema additions fail fast. The
R-package doc recipes now use the canonical git-source patches, and the corpus
covers every root-workspace crate (including the benchmark library) plus
`cargo-revendor`. The regenerated corpus is now produced by the same checked
path.
