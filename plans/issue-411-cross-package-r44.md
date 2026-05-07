# Issue #411 — declare R (>= 4.4) for cross-package DESCRIPTIONs

## Fix
Add `Depends: R (>= 4.4)` to:
- `tests/cross-package/consumer.pkg/DESCRIPTION`
- `tests/cross-package/producer.pkg/DESCRIPTION`

Insert between `Version:` and `Authors@R:`, matching the placement used in
`rpkg/DESCRIPTION` (set by PR #410).

## Verify
- `grep '%||%' tests/cross-package/{consumer,producer}.pkg/R/*.R` confirms the
  operator is still in use.
- `just cross-check` clean (no new NOTEs/WARNINGs).

## Out of scope
- `minirextendr/inst/templates/` — no DESCRIPTION files yet.
