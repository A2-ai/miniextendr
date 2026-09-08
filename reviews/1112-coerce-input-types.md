# #1112: coercion narrowed valid input types

Attempted: enable `coerce` or `coerce-default` on existing numeric and boolean
parameters without changing valid R callers.

Failure: numeric conversion forced one native storage type; logical booleans
were rejected. The generated R gates also narrowed integer vectors and even
`Option<bool>`, which has no corresponding coercion mapping.

Root cause: macro-generated extraction bypassed the runtime's multi-source
numeric converters, and R checks separately inferred the coercion behavior.

Fix: reuse the numeric converters, extend boolean conversion with integer 0/1
while preserving logicals, and select R checks from the same coercion mapping.
Keep strict precedence and indexed, batched vector diagnostics. Regression
coverage exercises scalar/vector source types, empty/invalid inputs, worker and
per-parameter paths, optional booleans, and strict/coerce interactions in default
and coerce-default builds.

Validation: the default R build passes 231 assertions and coerce-default passes
234, both without failures or warnings. The full Rust test suite, all-manifest
check/clippy, three CI Clippy feature configurations, formatting, template sync,
and API corpus regeneration/check also pass.

Switching back from the feature build exposed a separate cached-archive relink
bug, tracked in #1498 after a duplicate search. Moving only the generated shared
library to Trash forces a local relink; the build-system fix belongs to #1498.
