# `Missing<T>` was never callable with an absent argument

Date: 2026-07-03. Context: plan 06's R-side test backfill (audit A8/B6 —
"`Missing<T>` has NO R-side tests") added `test-missing.R` covering the four
`missing_test_*` exports. Every truly-missing call errored.

## What was attempted

`missing_test_f64()` (argument genuinely absent) — per `Missing<T>`'s
documented semantics this should map to `Missing::Absent`.

## What went wrong

```
Error in `missing_test_f64()`: argument "x" is missing, with no default
```

for all four fixtures. Present-argument calls worked, so nothing looked wrong
anywhere else — the *entire absent path*, the feature's whole point, was dead
in every generated wrapper (standalone fns and all six class systems).

## Root cause

The wrapper codegen emitted a prelude statement:

```r
if (missing(x)) x <- quote(expr=)
.Call(C_f, .call = match.call(), x)
```

Rebinding `x` to the empty symbol makes the binding hold `R_MissingArg` — and
R raises "argument is missing, with no default" on **symbol lookup** of any
binding holding that sentinel. So the `.Call`'s evaluation of `x` threw before
Rust ever saw the argument. No statement-form workaround exists
(`delayedAssign`, renamed bindings — all end in a lookup); the sentinel must
be *produced at the argument position*, where it passes as a value:

```r
.Call(C_f, .call = match.call(), if (missing(x)) quote(expr=) else x)
```

## Fix

`RArgumentBuilder::build_call_args_vec` (r_wrapper_builder.rs) now emits the
inline conditional for `Missing<T>` params; `build_missing_prelude` /
`collect_missing_params` and the prelude plumbing in `lib.rs`,
`r_class_formatter.rs`, and all six class generators are deleted. Both
codegen paths route through the same args builder, so one emission site fixes
standalone fns and methods alike.

## Lessons

- An untested export can hide a **100%-broken** feature, not just edge-case
  drift — the wrapper looked plausible and the C side was correct; only an
  actual absent-argument call could catch it.
- R's missing-argument sentinel is value-passable but not binding-passable.
  Any codegen that needs to forward missingness must do it in expression
  position at the call site.
