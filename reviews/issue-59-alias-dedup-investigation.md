# Issue #59 — R CMD check WARNING investigation

## What was investigated
Ran `rcmdcheck::rcmdcheck(..., args = c("--no-manual", "--as-cran", "--no-install"))` on
the worktree's `rpkg/` directory (source mode, after `rv sync`) to get the current
R CMD check output without rebuilding Rust.

## Phase 1 finding: warnings have changed since issue was filed

The issue description mentions:
> Rd files with duplicated alias 'get_value': CounterTraitS7.Rd, S7TraitCounter.Rd
> Rd files with duplicated alias 's4_get_value': CounterTraitS4.Rd, S4TraitCounter.Rd
> Rd files with duplicated alias 'value': S7Celsius.Rd, S7Fahrenheit.Rd

**None of these duplicated-alias warnings fire on the current codebase.** The S7/S4
codegen at `s7_class.rs:619` and `s4_class.rs:138` already uses class-qualified
`@name {ClassName}-{generic_name}` to avoid bare `\alias{generic_name}` in
class-specific `.Rd` files. The S3 `@export get_value` duplication was addressed
by `rpkg/R/generics.R` providing a single canonical home page for shared generics.

## Actual current R CMD check warnings (2 WARNINGs, 2 NOTEs)

```
❯ checking for code/documentation mismatches ... WARNING
  Functions or methods with usage in Rd file 'S7Fahrenheit.Rd' but not in code:
    'convert'

❯ checking Rd \usage sections ... WARNING
  Undocumented arguments in Rd file 'S3Raiser.Rd'
    'id'

  Objects in \usage without \alias in Rd file 'S7Fahrenheit.Rd':
    'convert'
```

## Root cause analysis

### Warning 1: S7Fahrenheit.Rd — `convert` usage/alias mismatch

**Source**: `miniextendr-macros/src/miniextendr_impl/s7_class.rs`, convert_from/convert_to
code path (around line 917+). When a method has `convert_from` or `convert_to`, the
codegen emits:

```r
#' @name convert-S7Celsius-to-S7Fahrenheit
#' @rdname S7Fahrenheit
S7::method(convert, list(S7Celsius, S7Fahrenheit)) <- function(from, to) { ... }
```

The `@rdname S7Fahrenheit` causes roxygen2 to merge this into `S7Fahrenheit.Rd`. Roxygen2
then generates a `\usage` entry for `convert(from, to)` from the function definition.
But the `\name` is `convert-S7Celsius-to-S7Fahrenheit` — not `convert` — so there is no
`\alias{convert}` in `S7Fahrenheit.Rd`. R CMD check sees `convert` in `\usage{}` but no
matching `\alias{}`.

**Fix path A (codegen)**: Add `@aliases convert` to the convert method documentation block in
the codegen so roxygen2 produces `\alias{convert}` in the merged `.Rd` file.

**Fix path B (generics.R)**: Add `#' @name convert NULL` to `rpkg/R/generics.R` to provide a
standalone `convert.Rd` with `\alias{convert}`.

Path A avoids the warning precisely and doesn't require a stub in generics.R.
Path B is simpler but may cause its own cross-reference issues since `convert` is an
S7 package function. The S7 package presumably ships its own `convert.Rd`. 

The cleanest fix: emit `@aliases convert` from the codegen whenever a `convert_from` or
`convert_to` block is generated. This adds `\alias{convert}` to the class `.Rd`, satisfying
R CMD check without creating a competing standalone page.

### Warning 2: S3Raiser.Rd — undocumented `id` parameter

**Source**: `rpkg/src/rust/condition_class_system_tests.rs:98-101`

```rust
#[miniextendr(s3)]
impl S3Raiser {
    pub fn new(id: i32) -> Self {
        S3Raiser { id }
    }
```

The `id` parameter on `new` has no doc comment, so the codegen emits no `@param id …` in
the wrapper. R CMD check sees `new_s3raiser(id)` in `\usage{}` but no `\item{id}{…}` in
`\arguments{}`.

**Fix**: Add a doc comment to the `id` parameter in `condition_class_system_tests.rs`.
```rust
    /// # Arguments
    /// * `id` - Numeric identifier for this raiser instance.
    pub fn new(id: i32) -> Self {
```
Or equivalently, add a `@param id …` doc annotation on the Rust side.

## Design for `convert` alias fix

In `s7_class.rs`, the `convert_from`/`convert_to` code generates the convert method
block. After the documentation block is pushed, we need to add `@aliases convert` so
that roxygen2 includes `\alias{convert}` in the merged `.Rd` page.

File: `miniextendr-macros/src/miniextendr_impl/s7_class.rs`

Location: Phase 4 (convert methods, around line 905+). The doc block for convert methods
currently emits `@name convert-{FromType}-to-{ToType}` and `@rdname {class_name}`.
Adding `@aliases convert` after those lines will resolve the warning.

## No SharedGeneric priority needed

Since the original duplicated-alias warnings no longer fire, the `RWrapperPriority::SharedGeneric`
addition described in the original plan is not needed. The two remaining warnings are
independent doc/annotation gaps, not dedup problems.
