//! RefCountedArena vs ProtectScope benchmarks.
//!
//! Implemented groups:
//! - `raw_preserve`: R_PreserveObject/R_ReleaseObject baseline (O(n) release)
//! - `protect_multi`: ProtectScope for N objects
//! - `hashmap_arena`: HashMapArena protect/release/refcount
//! - `threadlocal_arena`: ThreadLocalArena protect/release/refcount
//! - `threadlocal_hash_arena`: ThreadLocalHashArena protect/release/refcount
//! - `arena_comparison`: head-to-head at scale (1k, 5k, 10k objects)
//! - `release_scaling`: O(n) raw release vs O(1) arena release at scale
//! - `mixed_workload`: interleaved protect/release/refcount patterns
//!
//! Key finding: raw R_ReleaseObject is O(n) (scans precious list), making
//! protect+release cycles O(n²) at scale. RefCountedArena with hash table
//! provides O(1) lookup, dramatically faster for large object counts.
