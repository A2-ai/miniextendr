# Materialization Tracking (removed, potential future feature)

## What it was

A feature-gated (`materialization-tracking`) module in `miniextendr-api` that logged
every ALTREP `Dataptr` call -- the moment R forces a lazy/compact vector to materialize
into a contiguous memory buffer.

## Why it was removed

- **No R interface**: The module documented R functions (`altrep_materialization_count()`,
  `altrep_materialization_reset()`) that were never actually exported or created.
- **No tests**: No test exercised the feature.
- **Not enabled anywhere**: No downstream crate activated the feature.
- **Tiny scope**: 44 lines of code + 2 `#[cfg]` call sites in ALTREP macros.
- **Incomplete prototype**: Just an atomic counter + `eprintln!` -- no structured output,
  no way to inspect *which* vector materialized or *why*.

## Why it could be useful

ALTREP's performance benefit comes from *not* materializing. If user code (or R internals)
accidentally triggers `DATAPTR()`, the entire vector gets allocated and copied, negating
the benefit. A tracking mechanism would help users diagnose:

1. **Unexpected materializations**: "I expected my ALTREP vector to stay lazy, but
   something forced it."
2. **Hot paths**: "Which code path materializes most often?"
3. **Regression detection**: "Did a new R version start materializing where the old one
   didn't?"

## Design sketch for a proper implementation

### Requirements

- Zero-cost when disabled (feature gate or const bool).
- Structured output (not just `eprintln!`).
- Accessible from R (exported functions, not `:::` internal).
- Per-type tracking (know *which* ALTREP class materialized).
- Optional backtrace capture for debugging.

### Possible API

```rust
// Rust side (miniextendr-api)
pub struct MaterializationEvent {
    pub type_name: &'static str,
    pub writable: bool,
    pub timestamp: std::time::Instant,
    // Optional: backtrace if enabled
}

/// Get all materialization events since last reset.
pub fn materialization_events() -> Vec<MaterializationEvent> { ... }

/// Get count only (cheap).
pub fn materialization_count() -> usize { ... }

/// Reset tracking state.
pub fn materialization_reset() { ... }
```

```r
# R side (exported)
altrep_materialization_count()
altrep_materialization_events()  # returns data.frame
altrep_materialization_reset()
```

### Integration points

The ALTREP `Dataptr` trampoline (in `__impl_altvec_dataptr!` and
`__impl_altvec_string_dataptr!` macros) would call the tracking function
before dispatching to the user's implementation.

### Open questions

- Should tracking be per-instance or per-class?
- Should it capture R call stack (expensive but diagnostic)?
- Should there be a callback hook instead of/in addition to a log?
- Should it integrate with R's profiling infrastructure?
