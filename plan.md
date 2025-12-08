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

- [ ] Phase 1 commits reviewed
- [ ] Phase 2 commits reviewed
- [ ] Phase 3 commits reviewed
- [ ] Phase 4 commits reviewed
- [ ] Phase 5 commits reviewed
- [ ] Phase 6 commits reviewed
- [ ] Conclusions documented
