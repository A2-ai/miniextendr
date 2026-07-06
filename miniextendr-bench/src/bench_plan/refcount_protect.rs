//! RefCountedArena vs ProtectScope benchmarks.
//!
//! Implemented groups:
//! - `raw_preserve`: R_PreserveObject/R_ReleaseObject baseline (O(n) release)
//! - `protect_multi`: ProtectScope for N objects
//! - `refcount_arena`: RefCountedArena protect/release/refcount
//! - `threadlocal_arena`: ThreadLocalArena protect/release/refcount
//! - `arena_comparison`: head-to-head at scale (1k, 5k, 10k objects)
//! - `release_scaling`: O(n) raw release vs O(1) arena release at scale
//! - `mixed_workload`: interleaved protect/release/refcount patterns
//!
//! Key finding: raw R_ReleaseObject is O(n) (scans precious list), making
//! protect+release cycles O(n²) at scale. RefCountedArena's BTreeMap
//! provides O(log n) lookup, dramatically faster than O(n) release at scale.
