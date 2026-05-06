# Issue #384 — declare R (>= 4.4) for `%||%` in generated wrappers

## Fix
Add `Depends: R (>= 4.4)` to `rpkg/DESCRIPTION`. Convention: place it just after the `Type:` field or between `Title:` and `Authors@R:`, matching how other CRAN packages format it.

## Audit
Verify generated wrappers still use `%||%` (grep `rpkg/R/miniextendr-wrappers.R`). If `%||%` was removed in some recent refactor, this declaration becomes unnecessary — note it in the PR body.

## Out of scope
Other downstream packages (`tests/cross-package/*/DESCRIPTION`, `minirextendr/inst/templates/*/DESCRIPTION`) may also need `Depends: R (>= 4.4)`. Check whether they regenerate wrappers using `%||%`; if so, file a follow-up issue (don't expand this PR's scope).

## Acceptance
- `rpkg/DESCRIPTION` contains `Depends: R (>= 4.4)`.
- `just r-cmd-build` and `just r-cmd-check` clean (or noted in PR body if not run).
