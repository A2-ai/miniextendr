# LLM docs regeneration: redundant rustdoc link

## What was attempted

Regenerate the tracked `rust-llm-docs/generated/` corpus with `just llm-docs`
after reconciling source rustdoc and the manuals.

## What went wrong

Rustdoc runs with warnings denied and rejected an explicit target on the
`effective_threads` intra-doc link in `optionals/parallel.rs` as redundant.

## Root cause

The transferred documentation draft changed a locally resolvable
``[`effective_threads`]`` link into an explicit path-qualified target. The
newer rustdoc lint correctly treats that equivalent target as unnecessary.

## Fix

Restored the implicit intra-doc link and reran the corpus generator. Source
rustdoc remains the authority; generated corpus files are never hand-edited.
