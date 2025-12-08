# miniextendr Review Plan

## Goal

**Reduce redundancy and superfluous code** - NOT reduce code at all costs.

The objective is to identify:

- Duplicate implementations serving the same purpose
- Over-engineered solutions where simpler alternatives exist
- Abandoned experiments that were never cleaned up
- Dead code paths that are no longer used

Code that is necessary for correctness, safety, or future extensibility should be kept.

## Investigation Approach

Review all commits chronologically to understand:

1. What problems were being solved
2. What approaches were tried and abandoned
3. What evolved into the current architecture
4. What might have been left behind during refactoring

## Commit History Analysis

### Phase 1: Initial Setup (commits 1-30)

- `1c2ed5b` - `b6a96a2`: Project scaffolding, build system, R package structure
- `33a875b`: "prototype error handling done"
- `556e49f`: **"reduce in complexity"** - early simplification attempt

### Phase 2: Proc-Macro Development (commits 31-60)

- `42003db` - `363834f`: proc-macro work
- `5643b63`: Added `...` (dots) prototype
- `61b4c03`: Module code added
- `e80ce9c`: R wrapper generation

### Phase 3: Error Handling & Unwinding (commits 61-90)

- `03fbdbf`: "working on those unwinding tests"
- `f029a30`: "this is it.. this is how this should work"
- `aa55c2d`: "trampolined, guarded unwind_protect"
- `45f9c42`: "ensured closure deallocated even if Rf_error runs"
- `b4d126b`: **"remove: I've removed all the previous experiments"**

### Phase 4: Worker Thread Pattern (commits 91-110)

- `6f67072`: "first iteration of rust worker pattern"
- `52ec0b8`: "unsafe, sync SEXP" - flagged as "very unsafe to use"
- `73cd735`: "persistent worker thread"
- `f073cab`: "r_guard using persistent thread"
- `41435a1`: "finally this works properly"

### Phase 5: ALTREP Development (commits 111-140)

- `1923e03` - `d00e3fa`: Multiple "wp: altrep" commits (iterative development)
- `3361c82`: "altrep now less verbose"
- `82c08de`: **"removed all mentions of alttrep"** (typo: alttrep)
- `b49e228`: "altrep looks very solid now"

### Phase 6: Recent Review (commits 141-147)

- `c5f1df3`: **"reviewing externalptr.rs... doesn't look good"**
- `87c9f7c`: **"removed ErasedExternalPtr.. ExternalPtr<()> is good enough"**
- `59e147e`: "added idiomatic pointer cast"

## Key Investigation Points

### 1. ExternalPtr Module

Already flagged in recent commits. Need to review what remains after `ErasedExternalPtr` removal.

### 2. ALTREP Implementation

Many iterative commits suggest evolution. Check for:

- Leftover code from earlier approaches
- Multiple paths to same functionality

### 3. Worker Thread vs Main Thread

Two execution strategies exist. Verify both are necessary.

### 4. Dots Module

Prototype added at `5643b63`, status unclear.

### 5. Build System Complexity

Multiple commits tweaking configure scripts. Check if all complexity is justified.

## Files to Deep-Review

- [ ] `miniextendr-api/src/externalptr.rs` - flagged by author
- [ ] `miniextendr-api/src/altrep.rs` - largest module
- [ ] `miniextendr-api/src/altrep_std_impls.rs` - overlap?
- [ ] `miniextendr-api/src/worker.rs` - core pattern
- [ ] `miniextendr-api/src/dots.rs` - completion status
- [ ] `miniextendr-macros/src/lib.rs` - R code generation

## Status

- [x] Phase 1 commits reviewed
- [x] Phase 2 commits reviewed
- [x] Phase 3 commits reviewed
- [x] Phase 4 commits reviewed
- [x] Phase 5 commits reviewed
- [x] Phase 6 commits reviewed
- [x] Conclusions documented

---

## Investigation Findings

### File Size Distribution (miniextendr-api/src/)

| File | LOC | % of total |
|------|-----|------------|
| altrep.rs | 1,702 | 27% |
| externalptr.rs | 1,387 | 22% |
| altrep_std_impls.rs | 855 | 14% |
| ffi.rs | 510 | 8% |
| altrep_bridge.rs | 394 | 6% |
| worker.rs | 352 | 6% |
| altrep_traits.rs | 212 | 3% |
| macro_coverage.rs | 168 | 3% |
| unwind_protect.rs | 162 | 3% |
| Other (8 files) | 474 | 8% |
| **Total** | **6,216** | 100% |

### Key Findings

#### 1. ALTREP: Two Parallel Approaches (NOT Redundant)

The codebase has **two distinct ALTREP systems** that serve different purposes:

**Backend Traits** (`altrep.rs` + `altrep_std_impls.rs`):
- Simpler API: `IntBackend`, `RealBackend`, `StringBackend`, etc.
- Single class per type (6 global handles)
- Used by standard implementations: `CompactIntSeq`, `OwnedReal`, etc.
- Good for: common cases with less boilerplate

**Method Traits** (`altrep_traits.rs` + `altrep_bridge.rs` + proc-macro):
- Full control: `Altrep`, `AltVec`, `AltInteger`, etc.
- Each struct gets its own ALTREP class
- Used by custom classes: `ConstantIntClass`, `VecIntAltrepClass`
- Good for: when you need custom ALTREP behavior

**Verdict**: Both approaches serve valid use cases. NOT redundant.

#### 2. ExternalPtr: StableTypeId Complexity

The `StableTypeId` design stores:
- Hash of type name (u64)
- Length of name (usize)
- Raw pointer to name string (*const u8)

There's a TODO questioning this: "i don't know why the name is just not given by a &str"

**Potential simplification**: Could use `&'static str` directly since names are already static.

**Verdict**: Worth reviewing but not blocking.

#### 3. dots.rs: Minimal but Complete

Only 12 LOC - just a struct definition:
```rust
pub struct Dots { pub inner: SEXP }
```

The actual handling is in the proc-macro. The TODO "finish the dots module" may be outdated since the struct exists and works.

**Verdict**: NOT superfluous. Module is intentionally minimal.

#### 4. macro_coverage.rs: Intentional Test Code

168 LOC of test functions annotated with `#[miniextendr]` to ensure macro coverage. Comment explains purpose: "so `cargo expand` can be used as a living catalog".

**Verdict**: Useful for testing/documentation. Keep.

#### 5. backtrace.rs: Small Utility

25 LOC. Configurable panic hook via `MINIEXTENDR_BACKTRACE` env var.

**Verdict**: Useful. Keep.

#### 6. SendableSexp / SendablePtr Wrappers

Two newtype wrappers exist to implement `Send` for cross-thread transfer:
- `SendableSexp(SEXP)` - for SEXP pointers
- `SendablePtr<T>(NonNull<T>)` - for typed pointers

**Verdict**: Necessary for the worker thread pattern. NOT superfluous.

### Clean-Up Already Done

Commit history shows previous clean-ups:
- `b4d126b`: "removed all previous experiments" (-219 lines)
- `87c9f7c`: "removed ErasedExternalPtr" (-160 lines, simplified to type alias)

### Remaining TODOs in Code

Only 2 TODOs found:
1. `externalptr.rs:122` - StableTypeId name storage question
2. `lib.rs:108` - "finish the dots module" (may be outdated)

### Summary

**NOT Redundant/Superfluous**:
- Two ALTREP approaches (different use cases)
- macro_coverage.rs (testing infrastructure)
- SendableSexp/SendablePtr (required for threading)
- dots.rs (minimal by design)
- backtrace.rs (useful utility)

**Worth Reviewing**:
- `StableTypeId` in externalptr.rs - could potentially be simplified
- The TODO about dots module - verify if still valid

**No Major Redundancy Found**: The codebase appears well-maintained with previous clean-ups already done. Most apparent "complexity" serves legitimate purposes.
