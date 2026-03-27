# Plan: Fix class-system-matrix test failures (13 remaining)

## Root Causes

### 1. Missing `$.ClassName` dispatch for Env inherent impls (7 failures)

Types with `#[miniextendr]` (Env) inherent impl + different-class-system trait impl
don't get `$.ClassName` generated. The `$` dispatch method is only generated when
the inherent impl codegen runs for env-class — but the class-system-matrix types
split inherent and trait impls across different class systems.

**Affected**: CounterTraitS3, CounterTraitS4, CounterTraitR6 (all have env inherent
but non-env trait impl)

**Fix**: Ensure env-class inherent impl codegen always generates `$.ClassName`
and `[[.ClassName` dispatch methods, regardless of what class system the trait
impl uses. The inherent impl class system determines instance dispatch.

**Files**: `miniextendr-macros/src/miniextendr_impl/env_class.rs` (generates `$` method)

### 2. S4 virtual class error (2 failures)

`CounterTraitS4(10L)` fails with "trying to generate object from virtual class".
The S4 class `CounterTraitS4` is registered via `setClass()` but methods::new()
fails — the class definition may not be visible in the test environment.

**Fix**: Ensure `setClass()` runs during package load, not lazily. Check if
`methods::setClass` is being called in the right namespace.

### 3. S7 `$` on instances (3 failures)

S7 0.2.1 intercepts `$` on S7 instances too, not just class objects.
`counter$get_value()` is blocked even though `get_value` is an S7 method.

S7 instances should use S7 generic dispatch: `get_value(counter)` not
`counter$get_value()`. But the inherent impl is Env-class which expects `$`.

**Fix**: When inherent impl is env-class but type is also an S7 class, the S7
`$` interception takes priority. Either:
- (a) Don't use S7 `new_class()` for types with env inherent impls
- (b) Generate a custom `$.S7_object` method that falls through to env dispatch
- (c) Change tests to use generic dispatch for S7 types

### 4. R6 missing initialize (1 failure)

`R6TraitCounter$new_r6trait(10L)` calls `R6TraitCounter$new(.ptr = .val)` internally,
but R6Class definition doesn't have an `initialize` method.

**Fix**: Ensure R6 factory methods that call `$new(.ptr=...)` have a matching
`initialize` in the R6Class definition.

## Priority

Medium — these are class-system-matrix edge cases (env inherent + different trait
impl). The common case (same class system for inherent + trait) works fine.
The 3392 passing tests confirm all standard patterns work.
