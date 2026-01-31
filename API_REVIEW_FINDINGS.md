# miniextendr-api Public API Review Findings

## Summary

This document addresses API design concerns in `miniextendr-api/src/lib.rs` related to public surface area, semver stability, and user experience.

## Findings

### 1. Wildcard Worker Export (Medium Priority)

**Issue:** [lib.rs:291](../miniextendr-api/src/lib.rs#L291)
```rust
pub use worker::*;
```

**Current exports:**
- `has_worker_context()`
- `Sendable<T>`
- `is_r_main_thread()`
- `assert_r_main_thread_for_pointer_api()`
- `panic_payload_to_string()`
- `panic_message_to_r_error()`
- `panic_message_to_r_errorcall()`
- `with_r_thread()`
- `run_on_worker()`

**Problems:**
- Any new worker item becomes a semver commitment
- Risk of name collisions with user code
- Unclear which items are intended for public use
- Makes API harder to reason about

**Recommendation:**

**Option A: Explicit exports** (Recommended)
```rust
pub use worker::{
    is_r_main_thread,
    with_r_thread,
    Sendable,
};
// Keep worker:: namespace available for advanced use
```

**Option B: Keep namespaced**
```rust
// Remove wildcard, keep module public
pub mod worker;

// Users access as: miniextendr_api::worker::is_r_main_thread()
```

**Impact:** Breaking change if users rely on root-level worker items.

**Suggested items for root export:**
- `is_r_main_thread` - commonly used
- `with_r_thread` - commonly used
- `Sendable` - commonly used with worker thread pattern

**Suggested to keep namespaced:**
- `panic_payload_to_string()` - internal/advanced
- `panic_message_to_r_error()` - internal/advanced
- `assert_r_main_thread_for_pointer_api()` - internal/advanced
- `has_worker_context()` - internal/advanced
- `run_on_worker()` - internal (used by macro-generated code)

---

### 2. Extensive Optional Type Re-exports (Medium Priority)

**Issue:** [lib.rs:565-697](../miniextendr-api/src/lib.rs#L565-L697)

Root re-exports of ~50+ types from optional dependencies (rayon, ndarray, nalgebra, num-bigint, rust_decimal, ordered-float, uuid, regex, etc.).

**Problems:**
- Large public surface area
- Couples semver to external crates
- Risk of name collisions (e.g., `Regex`, `Url`, `Uuid` are common names)
- Bloats documentation
- Unclear which items are core vs optional

**Recommendation:**

**Option A: Prelude module** (Recommended for this use case)
```rust
pub mod prelude {
    //! Commonly used items for convenient imports
    #[cfg(feature = "rayon")]
    pub use crate::optionals::{RParallelExtend, RParallelIterator};

    #[cfg(feature = "ndarray")]
    pub use crate::optionals::{Array1, Array2, ArrayView1, ArrayView2};

    // etc. - curated subset
}

// Usage:
use miniextendr_api::prelude::*;
```

**Option B: Doc-hidden root aliases**
```rust
#[doc(hidden)]
#[cfg(feature = "regex")]
pub use optionals::Regex;
// etc.
```

**Option C: Remove root exports entirely**
```rust
// Users access as: miniextendr_api::optionals::regex_impl::Regex
pub mod optionals;
```

**Impact:**
- Option A: Minimal (prelude is additive)
- Option B: Minimal (still accessible but doc-hidden)
- Option C: Breaking change

**Recommendation:** Use Option A (prelude) for ergonomics while keeping the API surface manageable.

---

### 3. Name Duplication Across Namespaces (Low Priority)

**Issue:** Same names used for macros and types

Examples:
- `ExternalPtr` (macro) vs `externalptr::ExternalPtr` (type) - [lib.rs:177-199](../miniextendr-api/src/lib.rs#L177-L199)
- `RFactor` (macro) vs `factor::RFactor` (trait) - [lib.rs:377-390](../miniextendr-api/src/lib.rs#L377-L390)
- Similar for `ROrdered`, `RUnordered`

**Problems:**
- Confusing when using `use miniextendr_api::*;`
- Unclear which is which in documentation
- Can surprise users

**Recommendation:**

**Option A: Document the dual meaning**
```rust
/// Derive macro for implementing TypedExternal.
/// See also: [`externalptr::ExternalPtr`] type.
pub use miniextendr_macros::ExternalPtr;

/// Box-like owned pointer for Rust objects in R.
/// Derive with [`ExternalPtr`] macro.
pub use externalptr::ExternalPtr;
```

**Option B: Rename macros**
```rust
pub use miniextendr_macros::ExternalPtr as DeriveExternalPtr;
pub use miniextendr_macros::RFactor as DeriveRFactor;
```

**Option C: Discourage glob imports**
Add to CLAUDE.md and docs:
```
**Note:** Avoid `use miniextendr_api::*` due to name collisions.
Prefer explicit imports or module paths.
```

**Recommendation:** Option A + Option C (document and discourage globs). Name duplication is acceptable when well-documented.

---

### 4. Dual Module Paths (Low Priority)

**Issue:** [lib.rs:353-369](../miniextendr-api/src/lib.rs#L353-L369)

Modules re-exported at root level:
```rust
pub mod convert;
pub use convert::*;  // Makes types available at both paths

pub mod list;
pub use list::*;

pub mod typed_list;
pub use typed_list::*;
```

**Problems:**
- Two canonical paths: `miniextendr_api::List` vs `miniextendr_api::list::List`
- Unclear which to use
- Complicates documentation

**Recommendation:**

**Option A: Choose one canonical path**
```rust
// Remove wildcard, keep module public
pub mod list;

// Document canonical path
/// For list types, use [`list::List`], [`list::ListBuilder`], etc.
```

**Option B: Re-export selectively**
```rust
pub mod list;
pub use list::{List, ListBuilder};  // Only common types
// Users access less common items as: miniextendr_api::list::...
```

**Option C: Status quo with documentation**
Document that both paths work but recommend one:
```rust
/// List types. Prefer `miniextendr_api::List` over `miniextendr_api::list::List`.
pub mod list;
pub use list::*;
```

**Recommendation:** Option B (selective re-exports). This balances ergonomics with API clarity.

---

## Recommended Action Plan

### Phase 1: Non-Breaking Improvements (Do Now)

1. **Document the issues** ✅ (this file)

2. **Add prelude module**
   ```rust
   pub mod prelude {
       //! Convenience imports for common workflows
       pub use crate::{miniextendr, List, IntoR, IntoList};

       #[cfg(feature = "rayon")]
       pub use crate::optionals::{RParallelExtend, RParallelIterator};
       // etc.
   }
   ```

3. **Improve documentation**
   - Add doc comments explaining macro vs type duplication
   - Document canonical import paths
   - Add "Public API" section to README

4. **Add to CLAUDE.md**
   ```markdown
   ## Import Guidelines
   - Avoid `use miniextendr_api::*` (name collisions)
   - Prefer explicit imports or `use miniextendr_api::prelude::*`
   - Use module paths for clarity: `worker::is_r_main_thread()`
   ```

### Phase 2: Breaking Changes (Next Major Version)

1. **Replace worker wildcard**
   ```rust
   - pub use worker::*;
   + pub use worker::{is_r_main_thread, with_r_thread, Sendable};
   ```

2. **Selective module re-exports**
   ```rust
   pub mod list;
   - pub use list::*;
   + pub use list::{List, ListBuilder};
   ```

3. **Consider deprecating root optional exports**
   ```rust
   #[deprecated(note = "Use miniextendr_api::optionals::Array1 or prelude")]
   #[cfg(feature = "ndarray")]
   pub use optionals::Array1;
   ```

---

## Current Status

**As of 2026-01-30:**
- Issues documented ✅
- No breaking changes applied yet
- Preserving current API for compatibility

**Next steps:**
- Decide on phase 1 vs phase 2 approach
- Implement chosen strategy
- Update documentation accordingly

---

## Related

- Original discussion: (this session)
- Version: 0.1.0 (unreleased, breaking changes acceptable)
- Principle: "No backwards compatibility" per CLAUDE.md
