# Protected Reassignment + Safe Container Insertion Plan

Date: 2026-01-12
Status: **IMPLEMENTED** (2026-01-12)

## Summary
We want explicit, safe APIs that:

- allow *reassigning* a protected value without growing the PROTECT stack (using `R_ProtectWithIndex` / `R_Reprotect`), and
- allow *inserting* freshly allocated children into a protected container (list/STRSXP/etc) without risking GC or overflowing the protect stack.

Key decisions:

- **Safe defaults** for container insertion (internal protect/unprotect).
- **Internal** representation of "protected by R" (no public types).
- **ReprotectSlot** is the explicit user-facing mechanism for whole-object reassignment.

This plan keeps existing abstractions, adds safe helper APIs, and clarifies the protection model in docs.

## Background / Constraints

- R's PROTECT stack is LIFO. RAII drops cannot safely "unprotect the old value" after assignment without careful ordering.
- `OwnedProtect` uses `UNPROTECT(1)` (top-of-stack), so `a = OwnedProtect::new(...)` can unprotect the new value instead of the old.
- `R_ProtectWithIndex` + `R_Reprotect` provide *replace-in-place* without stack growth. This is already modeled by `ReprotectSlot`.
- A protected container only protects *reachable* children **after** insertion. A newly allocated child must be protected between allocation and insertion.
- Long vector materialization must avoid unbounded PROTECT stack growth.
- R errors (`longjmp`) bypass Rust destructors; RAII cannot guarantee cleanup unless wrapped by `R_UnwindProtect` at .Call boundaries.

## Goals

1. **Whole-object reassignment**: explicit, safe replacement of a protected value without stack growth.
2. **Safe container insertion**: container APIs that protect a child for the minimum window and unprotect immediately.
3. **Long vector support**: element-by-element construction with constant protect stack depth.
4. **Clear semantics**: internal model to distinguish "protected by R" vs "protected by ProtectScope".

## Non-Goals

- Automatic operator overloading for assignment (Rust does not support custom `=`).
- New public wrapper types for protection provenance.
- Changing core GC semantics or relying on preserving objects across calls.

## Proposed API Surface (Plan Only)

### 1) Whole-object reassignment via `ReprotectSlot`

Keep the existing API and document it as the canonical solution for reassignment:

```rust
let scope = ProtectScope::new();
let slot = scope.protect_with_index(Rf_allocVector(VECSXP, n));
slot.set(Rf_allocVector(VECSXP, n2)); // R_Reprotect, stack count stays 1
```

No new public types; emphasize `ReprotectSlot` in docs and examples.

### 2) Safe container insertion on List / StrVec

Add safe, explicit methods that protect a child only for the insertion window:

- `List::set_elt(i, child: SEXP)` (safe)
- `StrVec::set_charsxp(i, charsxp: SEXP)` (safe)
- `StrVec::set_str(i, &str)` (safe; creates CHARSXP then inserts)

Each safe method:

1. `PROTECT(child)`
2. `SET_*_ELT(container, i, child)`
3. `UNPROTECT(1)` (via `OwnedProtect` drop)

### 3) Unsafe escape hatch

Provide `unsafe` variants (or a single unsafe method name) when caller guarantees no allocation / GC between child creation and insertion:

- `unsafe fn set_elt_unchecked(i, child: SEXP)`
- `unsafe fn set_charsxp_unchecked(i, charsxp: SEXP)`

### 4) Internal "protected by R" model

Introduce an internal marker type (e.g., `RFrame`) used only inside the crate to make it explicit when a wrapper value is safe because R owns the protection during a call frame.

- Not exposed publicly.
- Used in docs and internal constructors to clarify lifetime/protection rules.

## Implementation Steps

1. **Audit current patterns**
   - Search for manual `PROTECT/UNPROTECT` around `SET_VECTOR_ELT`, `SET_STRING_ELT`, or similar.
   - Identify places where new helpers can reduce duplication.

2. **Add safe insertion helpers**
   - Implement `List::set_elt` (safe) and `List::set_elt_unchecked` (unsafe).
   - Implement `StrVec::set_charsxp` / `StrVec::set_str` (safe) + `*_unchecked` (unsafe).
   - Ensure these methods are `unsafe` at the same level as other raw R API usages.

3. **Document `ReprotectSlot` as the reassignment tool**
   - Update `gc_protect` docs and examples to explicitly describe reassignment use-case.
   - Add a short example that contrasts `OwnedProtect` vs `ReprotectSlot` for reassignments.

4. **Internal provenance marker**
   - Add a minimal `RFrame` type in a private module to represent call-frame protection (internal use only).
   - Use it in internal constructors or doc examples to signal that arguments are protected by R.

5. **Tests (R-initialized)**
   - Add tests that materialize long vectors using `List::set_elt` / `StrVec::set_*` and assert protect count stays constant.
   - Add regression tests that ensure `ReprotectSlot` count remains 1 while repeatedly `set`-ing.

6. **Docs updates**
   - `docs/SAFETY.md`: clarify child insertion rule and short-lived protection requirement.
   - `docs/docs.md`: mention the new safe insertion APIs and recommended patterns.
   - `docs/GAPS.md`: update entries about protection patterns and long vector construction.

7. **Benchmarks (optional)**
   - Add or update benchmarks to compare safe insertion vs manual PROTECT/UNPROTECT patterns.
   - Ensure no significant regression for common workloads.

## Safety Notes / Invariants

- Safe insertion helpers must avoid any R allocations between `PROTECT` and `SET_*_ELT`.
- For `StrVec::set_str`, any conversion / allocation must happen *before* protection of the final `CHARSXP` or must be encompassed by that protection guard.
- Methods should be `unsafe` if they rely on R main-thread restrictions or valid `SEXP` invariants (consistent with existing API style).
- RAII unprotects do not run across `R` errors; `.Call` entrypoints should already be wrapped in `R_UnwindProtect`.

## Migration / Compatibility

- New methods are additive; no breaking changes.
- Existing manual `PROTECT/UNPROTECT` can be left as-is; new helpers provide a safer alternative.
- Encourage use of `ReprotectSlot` for reassignment scenarios.

## Open Questions (for final review)

- Exact naming: `set_elt` vs `set_elt_safe` (plan prefers default-safe `set_elt`).
- Whether to expose a guard-accepting variant (e.g., `set_elt_guarded`) to avoid double-protect when caller already has `OwnedProtect`.
- Whether to add similar helpers for other container types (e.g., matrices, pairlists).

## Example Usage (final shape)

```rust
// Whole-object reassignment (no stack growth)
let scope = ProtectScope::new();
let slot = scope.protect_with_index(Rf_allocVector(VECSXP, 1));
slot.set(Rf_allocVector(VECSXP, 2));

// Long vector materialization (constant stack depth)
let list = List::from_raw(scope.protect_raw(Rf_allocVector(VECSXP, n)));
for i in 0..n {
    let child = make_child(i); // may allocate
    list.set_elt(i as isize, child); // protects/unprotects internally
}
```

---

## Implementation Notes (2026-01-12)

### What Was Implemented

1. **Critical Bug Fix**: Fixed `List::from_raw_pairs()` which had unprotected allocations
   - Added `OwnedProtect` guards for both list and names during construction
   - `Rf_mkCharLenCE` in the loop can trigger GC - now both containers are protected

2. **Safe Container Insertion for List**:
   - `List::set_elt(idx, child)` - protects child during insertion (safe default)
   - `List::set_elt_unchecked(idx, child)` - no protection (unsafe escape hatch)
   - `List::set_elt_with(idx, callback)` - callback-based allocation within protection window
   - `ListBuilder<'a>` - efficient batch construction with scope reference

3. **New StrVec Module** (`strvec.rs`):
   - `StrVec` wrapper for STRSXP with safe insertion methods
   - `StrVec::set_str(idx, s)` - safe string insertion
   - `StrVec::set_charsxp(idx, charsxp)` - safe CHARSXP insertion
   - `StrVec::set_na(idx)` - set NA_character_
   - `StrVecBuilder<'a>` - efficient batch construction

4. **Enhanced Documentation**:
   - Added comprehensive docs for `ReprotectSlot` including RAII pitfall warning
   - Added module-level docs for container insertion patterns in `gc_protect.rs`
   - Documented when to use `ReprotectSlot` vs `OwnedProtect`

5. **Tests** (`gc_protect_tests.rs`):
   - `test_list_builder_*` - ListBuilder correctness
   - `test_list_set_elt*` - List safe insertion methods
   - `test_strvec_*` - StrVec/StrVecBuilder methods
   - `test_reprotect_slot_*` - ReprotectSlot patterns

### Design Decisions

- **Naming**: Used `set_elt` (safe) and `set_elt_unchecked` (unsafe) following Rust conventions
- **No RFrame marker**: Decided against internal provenance marker as overkill
- **No guard-accepting variant**: Kept API simple; users can use `set_elt_unchecked` when child already protected
- **Matrices/pairlists**: Deferred - start with List and StrVec which are most common

### Files Modified

- `miniextendr-api/src/list.rs` - bug fix + new APIs
- `miniextendr-api/src/strvec.rs` - new module
- `miniextendr-api/src/gc_protect.rs` - enhanced documentation
- `miniextendr-api/src/lib.rs` - exports for new types
- `rpkg/src/rust/gc_protect_tests.rs` - new test module
- `rpkg/src/rust/lib.rs` - test module registration
