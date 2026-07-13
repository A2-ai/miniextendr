# Stale issue cache led to a duplicate GitHub issue

## What was attempted

Track a dependency future-incompatibility warning discovered while reconciling
the manuals and generated Rust documentation.

## What went wrong

Issue #1347 duplicated the existing canonical tracker, #1329.

## Root cause

The local `ISSUES/` cache had not been refreshed since issue #1215, and the
open-issue index was not read before filing. The live dependency investigation
was correct, but it started from the source tree instead of the repository's
existing-work ledger.

## Fix

Closed the duplicate #1347, verified #1341 had resolved the dependency chain,
and closed canonical issue #1329 with that evidence. Added `just
issues-refresh`, which refreshes both indexes and every open issue body while
trashing stale body files, plus a root instruction requiring an end-to-end
index read and relevant body search before any future `gh issue create`.
