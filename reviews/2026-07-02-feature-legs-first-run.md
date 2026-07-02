# First run of the denylisted-feature legs: five latent defects in one afternoon

## What was attempted

Plan `plans/2026-07-01-denylist-feature-runtime-ci.md` (audit A5/A10): build rpkg
with the features `tools/detect-features.R` denylists (`worker-default`,
`strict-default`, `coerce-default`, `r6-default`, `s7-default`, `nonapi`,
`indicatif`, `growth-debug`, `refcount-fast-hash`) and actually run testthat
against each combination — something no CI job had ever done.

## What went wrong

Every single leg failed on first contact, each for a different latent reason:

1. **`strict-default` didn't compile.** The strict return path in
   `c_wrapper_builder.rs::sexp_conversion_expr` stripped the `Option` wrapper
   from the declared return type before the strict-helper lookup, so
   `Option<i64>`-returning fns emitted `checked_into_sexp_i64(<Option<i64>>)`
   → E0308. The `OptionIntoR` arm binds the *whole* Option; only the unwrap
   arms bind the inner value. Fix: `result_holds_unwrapped` parameter.

2. **`s7-default` didn't install.** Bare impls with methods named `var`, `get`,
   `col`, `row`, `diag`, `reshape` flip to S7, and the generated
   `S7::method(<existing fn>, Class) <-` fails at load when the name resolves
   to a plain (non-generic) base function. `sum`/`mean`/`min`/`max` are group
   generics and register fine — only plain closures break. Filed #1114;
   affected fixture impls pinned `#[miniextendr(env)]`.

3. **`r6-default` didn't install.** r6 trait-impl wrapper names are not
   class-qualified (`r6_trait_<Trait>_<method>`), so two impls of one trait
   collide in the generated wrappers — caught by the registry.rs duplicate
   guard (whose message only mentions S7). Filed #1115; bare multi-impl trait
   fixtures pinned env.

4. **`worker-default` didn't compile.** Parameters/returns that are `!Send`
   (`AltrepSexp`, the R-backed `RDVector`/`RDMatrix`/`RndVec`/`RndMat` views,
   `ProtectedStrVec`, `Box<dyn Fn>` streaming closures) can't move into
   `run_on_worker`. The macro's stay-on-main-thread rule only recognized the
   literal `SEXP` type. Fix: `is_main_thread_bound_input` list for framework
   types; `no_worker` pins for user-side `!Send` fixtures.

5. **`nonapi` didn't even `dyn.load`.** `sys::nonapi_encoding` declared
   `known_to_be_utf8` / `latin1locale` / `R_nativeEncoding` — all `extern0`
   (= `attribute_hidden`) in R's `Defn.h`. A data relocation against a hidden
   symbol aborts `dyn.load` of the whole package **even if the code path is
   never called** (`miniextendr_encoding_init` is `#[no_mangle]`, so it can't
   be dead-stripped). Fix: declare only exported flags (`utf8locale`,
   `mbcslocale`, `known_to_be_latin1`).

Bonus semantic find: under `coerce`, parameters convert *from the R-native
scalar type*, which **narrows** accepted SEXP types (a coerce'd `bool` takes
`1L` but rejects `TRUE`) — the R-side `stopifnot(is.logical(x))` precondition
then contradicted the C side entirely, making coerce'd bool params unusable.
Precondition now emits an integer gate for coerce'd bool (#616 class); the
semantics question is #1112.

## Root cause (the meta one)

"Compiles under clippy_all" was doing all the safety work for these features,
and clippy never compiles *rpkg* with them, never generates their wrappers,
never loads the result into R. Each failure lived exactly one stage past where
CI stopped looking: codegen (1), R load (2, 3, 5), cross-crate trait bounds (4).

## Fix

The `feature-legs` job in ci.yml (weekly + workflow_dispatch) now builds,
installs, and tests four leg bundles. Divergence assertions live in
`rpkg/tests/testthat/test-feature-defaults.R` against the fixtures in
`rpkg/src/rust/feature_default_fixtures.rs`.

## Lessons

- A feature that changes codegen has *four* failure surfaces: macro expansion,
  crate compile, wrapper generation/load, and runtime semantics. A compile
  gate covers two at best.
- On macOS and Linux both, an `unsafe extern` **data** declaration is a
  load-time liability, not a call-time one. Function declarations bind lazily;
  statics do not. Never declare a `Defn.h` global without checking `extern0`
  vs `extern` (or `nm -gU libR`).
- `#[no_mangle] pub extern` functions are never dead-stripped, so "nobody
  calls it" does not protect a package from what such a function references.
