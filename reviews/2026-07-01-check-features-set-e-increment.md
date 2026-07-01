# `just check-features` died at combo 1 — `((passed++))` under `set -e`

## What was attempted

Audit A3 (2026-07-01) flagged `just check-features` as broken because combo
`"growth-debug,materialization-tracking"` references a feature deleted in
3ea3525b. While repairing the combo list, the first full local run of the
recipe failed **after the first combo succeeded**, before ever reaching the
dead combo.

## What went wrong

```
--- Checking: serde,ndarray ---
    Finished `dev` profile ... in 6.70s
error: Recipe `check-features` failed with exit code 1
```

No cargo error — the combo compiled fine, then the recipe aborted.

## Root cause

The recipe runs under `set -euo pipefail` and counted successes with
`((passed++))`. Bash arithmetic commands return exit status 1 when the
expression evaluates to 0 — and post-increment evaluates to the *old* value.
So with `passed=0`, the first `((passed++))` returns status 1 and `set -e`
kills the script. The recipe has been incapable of passing since the counter
was introduced, independent of the dead-feature combo the audit blamed.
(The audit's inference was reasonable — cargo does hard-error on unknown
features — but the recipe never got that far.)

## Fix

`passed=$((passed + 1))` — arithmetic *expansion* in an assignment always
exits 0. Landed with the feature-matrix-hygiene PR alongside the combo-list
repair; the recipe now completes 20/20 combos and runs weekly in CI
(scheduled-only `check-features` job) so it can't silently rot again.

## Lesson

`((var++))`, `((var--))`, and any `((expr))` that can evaluate to zero are
`set -e` landmines. In strict-mode scripts use `var=$((var + 1))`, or
`((var++)) || true` if the command form is unavoidable.
