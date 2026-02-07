# S7 Fallback Remediation Plan

## Scope
Fix the three known fallback issues in S7 wrapper generation:
1. `class_any` fallback currently crashes on ordinary objects due to hardcoded `x@.ptr`.
2. `s7(fallback)` is ignored in the generic-override branch.
3. Tests only assert string presence, not fallback behavior correctness.

Primary files:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl.rs`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl/tests.rs`
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs` (docs)
- `/Users/elea/Documents/GitHub/miniextendr/rpkg/tests/testthat/test-class-systems.R` (runtime check)

---

## Contract Decision (Task 0, must happen first)
Decide and document fallback semantics.

### Recommended contract
`#[miniextendr(s7(fallback))]` means:
- dispatch target is `S7::class_any`
- wrapper should **not** fail with raw slot-access error (`@`)
- non-compatible objects may still fail in Rust conversion, but with explicit conversion/type errors

This keeps fallback useful as a dispatch catch-all while preserving typed Rust method backends.

**Done when**:
- Contract is written in docs in `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs`.

---

## Task 1: Fix fallback self-expression in generated R wrapper
Problem: generated fallback methods call `.Call(..., x@.ptr, ...)` unconditionally.

### Change
In `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl.rs`:
- move call-expression construction until after method attrs are available
- build self expression conditionally:

```rust
// Sketch only
let self_expr = if method_attrs.s7_fallback {
    "tryCatch(x@.ptr, error = function(e) x)"
} else {
    "x@.ptr"
};
let call = ctx.instance_call(self_expr);
```

This removes the immediate `@` failure for non-slot objects.

**Done when**:
- generated fallback method does not contain raw `x@.ptr` as the sole argument path.
- fallback call path still works for normal miniextendr S7 objects.

---

## Task 2: Apply fallback class in generic-override branch
Problem: `s7_fallback` only affects non-override branch.

### Change
In `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl.rs`:
- compute `method_class` once (before override branching):

```rust
let method_class = if method_attrs.s7_fallback {
    "S7::class_any".to_string()
} else {
    class_name.clone()
};
```

- use `method_class` in **both** branches:
  - generic override branch (`S7::method({gen_name}, {method_class})`)
  - non-override branch (current behavior already uses it)

**Done when**:
- `generic=...` + `fallback` generates `S7::method(..., S7::class_any)`.

---

## Task 3: Strengthen unit tests for generated wrapper text
Problem: current test only checks for `class_any` token.

### Change
Update tests in `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/miniextendr_impl/tests.rs`:

1. Expand existing `s7_generic_fallback` assertions:
- assert fallback method class is `S7::class_any`
- assert wrapper call expression is not the brittle raw `x@.ptr` pattern

2. Add new test: `s7_generic_override_fallback`
- method with `#[miniextendr(s7(generic = "base::print", fallback))]`
- assert generated method class is `S7::class_any`

3. Add regression assertion:
- ensure fallback wrapper includes safe self extraction expression (e.g. `tryCatch(...)`)

**Done when**:
- tests fail on old implementation and pass on fixed implementation.

---

## Task 4: Add runtime-facing regression test
Problem: generator text tests do not verify error quality at runtime.

### Change
In `/Users/elea/Documents/GitHub/miniextendr/rpkg/tests/testthat/test-class-systems.R`:
- add test that calling fallback generic on non-compatible object errors cleanly
- explicitly assert the message does **not** include slot-access failure text

Example shape:
```r
test_that("S7 fallback does not fail with slot-access error on ordinary objects", {
  msg <- tryCatch(
    { describe_any(1L); NA_character_ },
    error = function(e) conditionMessage(e)
  )

  expect_false(grepl("no applicable method for `@`", msg, fixed = TRUE))
  expect_true(grepl("externalptr|type|convert|S7Strict", msg))
})
```

**Done when**:
- runtime fallback failure mode is controlled and informative.

---

## Task 5: Documentation alignment
Problem: docs imply catch-all semantics but do not describe typed backend constraints.

### Change
Update docs in:
- `/Users/elea/Documents/GitHub/miniextendr/miniextendr-macros/src/lib.rs`

Clarify:
- `s7(fallback)` dispatches as `class_any`
- backend method may still require a compatible object
- non-compatible objects return conversion/type errors, not slot-access errors

**Done when**:
- docs match actual generated behavior.

---

## Task 6: Verification Checklist
Run:
```sh
cargo test -p miniextendr-macros miniextendr_impl::tests::s7_generic_fallback -- --nocapture
cargo test -p miniextendr-macros miniextendr_impl::tests::s7_generic_override_fallback -- --nocapture
```

And for runtime package test:
```sh
just devtools-test FILTER="class-systems"
```

**Done when all pass**.

---

## Acceptance Criteria
- No fallback-generated wrapper relies solely on raw `x@.ptr` slot access.
- `generic + fallback` produces `class_any` method registration.
- Unit tests explicitly guard both issues.
- Runtime test confirms ordinary-object fallback errors are controlled (no raw `@` failure).
- Docs describe actual semantics.
