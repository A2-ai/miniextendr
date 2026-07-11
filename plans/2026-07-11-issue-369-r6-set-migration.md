# Plan: #369 — R6 codegen migrates to `$set()` form (roxygen2 8.0.0 per-method docs)

Date: 2026-07-11. Anchors verified against main @ 6de43e9b.
Branch: `refactor/369-r6-set-form`.

roxygen2 8.0.0 documents R6 methods added via
`Class$set("public", "name", function(...) {...})` from a roxygen block
directly preceding the `$set` call. Migrate the R6 generator from one big
inline `R6::R6Class(public = list(...))` to a minimal core + per-method
`$set` blocks, each carrying its own roxygen.

## Verified state

- Generator: `miniextendr-macros/src/miniextendr_impl/r6_class.rs` (623
  lines). `R6::R6Class(` emission `:126`/`:130`; `public = list(` `:139`;
  `initialize` handling `:150-260` (explicit `new()` or minimal
  `initialize(.ptr = NULL)` `:253`); public instance methods follow;
  `active = list(` `:431`.
- **In-repo `$set` precedent** (copy these shapes):
  - Sidecar-field active bindings: `externalptr_derive.rs:783-827` —
    `cls$set("active", "<field>", function(value) {...}, overwrite = TRUE)`
    inside a `.rdata_active_bindings_<Type>(cls)` helper, with the
    R CMD check appeasement line `self <- private <- NULL` (`:798-799`)
    because `$set`-form closures reference `self`/`private` that the
    static checker can't see bound.
  - `miniextendr_impl.rs:733`, `:810` already describe impl-level active
    bindings as "$set after class creation".
- **Out of scope, verified unaffected**: trait R6 methods live in a
  `Type$Trait$method` namespace env, not the class body
  (`miniextendr_impl_trait/r_wrappers.rs:879-930`) — do not touch.
- No minirextendr test greps the `R6Class(` source shape (checked
  `minirextendr/tests/testthat`, templates carry no `R6Class`).
- Class-level `@param` suppression: methods currently omit params covered
  by class-level tags because roxygen2 8.0.0 inherits class-level `@param`
  into inline methods (`r6_class.rs:184-206` region, `class_param_names`
  via `roxygen::extract_param_names`).

## Target emission shape

```r
ClassName <- R6::R6Class("ClassName",
  public = list(
    initialize = function(...) { ... }   # stays inline — R6Class core
  ),
  private = list(.ptr = NULL)            # unchanged
)

#' @description <method doc>
#' @param x <param doc>
ClassName$set("public", "method_name", function(x) {
  .val <- .Call(C_ClassName__method_name, self, x)
  <condition check>
  .val
})

#' @description <binding doc>
ClassName$set("active", "binding_name", function(value) { ... })
```

- `initialize` STAYS inline (both the explicit-`new()` and minimal-`.ptr`
  variants) — R6Class needs a well-formed core, and `$new` doc stays on the
  class page as today.
- Each public method and each impl-level active binding becomes its own
  `$set` call with its own roxygen block directly above (roxygen2 8.0.0
  documents these). NO `overwrite = TRUE` for class-own methods (the
  default FALSE catches accidental name collisions loudly; `overwrite =
  TRUE` stays only in the sidecar helper, which intentionally layers on).
- Each `$set` closure that references `self`/`private` needs the checker
  appeasement — but unlike the sidecar helper (closures inside a function),
  top-level `$set` calls can't do `self <- private <- NULL` without
  polluting the namespace. Use the sidecar helper pattern: wrap ALL of a
  class's `$set` calls in one generated
  `.mx_r6_methods_<ClassName> <- function(cls) { self <- private <- NULL; cls$set(...); ... }`
  invoked immediately after definition… **NO — that would hide the roxygen
  blocks from roxygen2** (blocks must precede top-level `$set` calls).
  Instead: verify first whether top-level `ClassName$set("public", ...,
  function(x) { ... self ... })` even triggers the `checking R code for
  possible problems` no-visible-binding NOTE — R's codetools treats
  function bodies lazily and `self`/`private` in R6 methods are the
  documented roxygen2-8 pattern, and the current INLINE emission has the
  identical closures without appeasement. Expectation: no NOTE (the inline
  form never needed one). If a NOTE appears in `just r-cmd-check`, the
  pre-approved fix is a single `utils::globalVariables(c("self",
  "private"))` emitted once into the wrappers preamble
  (`registry.rs:1169-1220` region), NOT per-call hacks. Record which case
  held in the PR body.

## Class-level @param inheritance (pre-specified branches)

roxygen2 8.0.0 inherits class-level `@param` into inline methods; whether it
does so for `$set`-form methods must be verified, not assumed:

- Implement the migration keeping the current suppression, regen docs, and
  diff `rpkg/man/*.Rd` for R6 classes with class-level params (grep
  `class_param_names` usage; the R6 fixtures in `rpkg/src/rust/` include
  such classes — find one whose Rd currently shows inherited params).
- **Branch A**: inherited params still render on method docs → keep
  suppression, done.
- **Branch B**: they vanish from the Rd → disable the suppression for the
  `$set` path (emit the class-covered `@param`s into each method's block —
  the auto-filler machinery already knows the tags; reuse
  `find_param_tag`/`param_documented` from `roxygen.rs`).
- Either way the resulting Rd must contain NO `(no documentation
  available)` regressions and `just r-cmd-check` must stay at its baseline
  WARNING set (zero if #1261 closed by then).

## Work items (flat order)

1. Restructure `generate_r6_r_wrapper` (`r6_class.rs`): core R6Class with
   initialize + private; then per-method `$set` blocks via the existing
   `MethodDocBuilder`/`DotCallBuilder` (`r_class_formatter.rs`) — the doc
   builder's output moves from inline-indented tags to top-level `#'`
   blocks; check `RoxygenBuilder` supports the `@description`-led block the
   sidecar/namespace emissions already use.
2. Migrate the `active = list(` block (`:431`) to `$set("active", ...)`
   blocks — this also retires the closure-in-loop `local()` capture pattern
   for active bindings if the generator currently emits it (grep `local(`
   in r6_class.rs; simplify only what the new shape makes unnecessary).
3. `Class$set` calls must sort AFTER the class definition in wrappers.R —
   they are part of the same wrapper fragment string, so ordering is
   inherent (verify the fragment stays one `MX_R_WRAPPERS` entry;
   `RWrapperPriority::Class` unchanged).
4. Doc-comment the generator's module header (`r6_class.rs:3`,`:67-68`)
   for the new shape.
5. Snapshots: rebaseline every R6 snapshot (`snapshot_r6_*`,
   `snapshot_r6_active_bindings`); review each — the diff must be pure
   restructuring (same `.Call` bodies, same conditions, blocks relocated).
6. Perf gate (issue's "verify install-time cost" + measure-before-commit
   rule): time `library(miniextendr)` (fresh R session, 10 reps, median)
   on main vs branch after identical `just rcmdinstall`. `$set` runs once
   per method at class-definition eval; expectation: noise. **If median
   load regresses > 5%, stop and report** with the numbers.
7. Regen loop; commit regenerated `NAMESPACE`/`man/*.Rd` (method docs move
   to their own topics or stay merged per rdname — review the man/ diff;
   aliases must not be lost). `just cross-document` for tracked
   cross-package wrappers if producer/consumer have R6 classes (grep
   `miniextendr(r6` in `tests/cross-package/*/src`).
8. Suites: devtools (R6 fixture tests are the regression surface),
   cross, minirextendr, templates-check, three clippy legs, fmt.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document
# (no new exports — restructuring; single install OK)
cargo test -p miniextendr-macros 2>&1 > /tmp/369-macros.log
just test 2>&1 > /tmp/369-rust.log
just devtools-test 2>&1 > /tmp/369-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/369-devtools.log   # devtools::test always exits 0
just cross-test 2>&1 > /tmp/369-cross.log
just minirextendr-test 2>&1 > /tmp/369-minir.log
just r-cmd-check 2>&1 > /tmp/369-rcmdcheck.log   # Read; WARNING baseline unchanged
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

trybuild `.stderr`: not expected to change; the 5 pre-existing
`derive_dataframe_enum_*` mismatches (#1239) stay untouched; any OTHER
change → stop and report.

## Must NOT touch

- Trait R6 namespace emission (`r_wrappers.rs:879+`), the sidecar
  active-binding helper (`externalptr_derive.rs:783-827`), other class
  systems, `r_class_formatter.rs` behavior consumed by the other 5
  generators (additive changes only).
- The `initialize`/`.ptr` contract and `new_<class>()` free-function
  wrappers.
- Generated files (`wrappers.R`, `wasm_registry.rs`) — regen only.

## Done criteria

- R6 wrappers emit core + per-method `$set` blocks with per-method roxygen;
  Rd output has no lost aliases/params and no filler regressions; the
  @param-inheritance branch is recorded; load-time delta measured and
  within gate; suites + snapshots + three clippy legs green; `Fixes #369`.

## Escalation rule

If reality diverges from this plan — roxygen2 does not document the
`$set` form as the blog promises (Rd diff loses method sections), the
no-visible-binding NOTE appears and `globalVariables` doesn't clear it,
the perf gate trips — **stop, commit nothing further, and report back.
Do not improvise.**
