# Issue #59 — Deduplicate S7/S4 generic aliases

## Problem
R CMD check warns:
- Rd files with duplicated alias 'get_value': CounterTraitS7.Rd, S7TraitCounter.Rd
- Rd files with duplicated alias 's4_get_value': CounterTraitS4.Rd, S4TraitCounter.Rd
- Rd files with duplicated alias 'value': S7Celsius.Rd, S7Fahrenheit.Rd

## Phase 1 — confirm scope
Run `just devtools-check 2>&1 > /tmp/devtools-check.log` (sandbox bypass).
Read `rpkg/rpkg-check-output/miniextendr.Rcheck/00check.log` to confirm:
1. The warning still fires (and which classes are involved).
2. Whether s4_class.rs already deduplicates correctly via `@aliases method,class-method` (line 145).
3. Whether the bare `\alias{generic_name}` sneaks in via some path other than the class-qualified `@name`.

Document in `reviews/issue-59-alias-dedup-investigation.md`.

## Phase 2 — design and implement
Per the issue proposal:
1. Add `RWrapperPriority::SharedGeneric` (between Function and Class) in
   `miniextendr-api/src/registry.rs`.
2. For S7/S4 generics implemented by 2+ classes in the package, emit a single
   shared generic block with `@name {generic}` + `@export` (or `@rawNamespace
   export(generic)` for S7 per existing convention) before any class block.
3. Per-class instance methods drop `@name {generic}`, keep `@rdname {ClassName}`
   and `@aliases {ClassName}${generic}` for class-scoped alias visibility.
4. Update `collect_r_wrappers()` priority sort and add a dedup pass that detects
   multi-class generics.

Single-class generics keep the existing per-class block path.

## Tests
- Snapshot tests in `miniextendr-macros/src/miniextendr_impl/tests.rs` for two
  S7 classes sharing a generic, two S4 classes sharing a generic, and the
  single-class case (no shared block).
- Integration: regenerated rpkg `man/*.Rd` and `00check.log` show zero
  duplicated-alias warnings.

## Acceptance
- [ ] `just devtools-check 2>&1 > /tmp/devtools-check.log`: no duplicate-alias warnings.
- [ ] Generic Rd block exists for each multi-class generic.
- [ ] `?<generic_name>` resolves cleanly in R.
- [ ] Snapshot tests cover the new emission shape.
- [ ] `just check && just clippy && just devtools-test` clean.
