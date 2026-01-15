# Benchmark Summary - 2026-01-12

This document summarizes benchmark results for miniextendr, covering all benchmark categories.

## Table of Contents

- [GC Protection Mechanisms](#gc-protection-mechanisms)
- [PPSize Analysis](#ppsize-analysis)
- [FFI and R Interop](#ffi-and-r-interop)
- [Type Conversions](#type-conversions)
- [Memory and Allocation](#memory-and-allocation)
- [ALTREP](#altrep)
- [ExternalPtr](#externalptr)
- [String Operations](#string-operations)
- [Worker Thread](#worker-thread)
- [Miscellaneous](#miscellaneous)

---

## GC Protection Mechanisms

### Key Findings

R's protect stack (`--max-ppsize`) has these limits:

- **Minimum**: 10,000
- **Default**: 50,000
- **Maximum**: 500,000

**Critical Discovery**: R uses ~30-40 protect slots at initialization, leaving ~49,960 available in the default configuration.

### ProtectScope vs Arena Implementations

| Implementation | 10k | 50k | 100k | 200k | 300k | 400k | 500k |
|----------------|-----|-----|------|------|------|------|------|
| ProtectScope | 115µs | N/A | N/A | N/A | N/A | N/A | N/A |
| RefCountedArena (BTreeMap) | 845µs | 4.2ms | 8.5ms | 17.7ms | 29.6ms | **37.9ms** | **47.5ms** |
| HashMapArena | 747µs | 3.5ms | 7.5ms | 17.3ms | 35.6ms | 40.8ms | 58.1ms |
| ThreadLocalArena (BTreeMap) | 761µs | 3.9ms | 8.3ms | **17.5ms** | **28.0ms** | 40.4ms | **47.4ms** |
| ThreadLocalHashArena | **440µs** | **2.6ms** | **5.6ms** | 17.3ms | 34.9ms | 47.9ms | 72.2ms |

### Crossover Analysis

- **< 150k protections**: ThreadLocalHashArena wins (HashMap O(1) operations)
- **~150-200k**: Crossover point - BTreeMap and HashMap roughly equivalent
- **> 200k protections**: BTreeMap variants win due to better cache locality
- **At 500k**: ThreadLocalArena is 52% faster than ThreadLocalHashArena (47ms vs 72ms)

### Recommendations

1. **For < 50k protections**: Use `ProtectScope` when possible (fastest, but limited by ppsize)
2. **For 10k-150k protections**: Use `ThreadLocalHashArena` (fastest arena implementation)
3. **For > 150k protections**: Use `ThreadLocalArena` (BTreeMap) - wins at scale due to better cache locality

### Detailed Protection Benchmarks

#### Single Protection Operations

| Operation | Time |
|-----------|------|
| ProtectScope single | 14.5-18.6 ns |
| RefCountedArena single | 49-97 ns |
| ThreadLocal single | 38.8-39.8 ns |
| ThreadLocalHash single | 124-211 ns |
| Raw protect/unprotect | 13.7-21 ns |

#### Reference Counting (same value)

| Count | ProtectScope | RefCountedArena | ThreadLocalArena |
|-------|--------------|-----------------|------------------|
| 10 | 47-65 ns | 71-115 ns | 89-91 ns |
| 100 | 396-534 ns | 226-290 ns | 432-451 ns |
| 1000 | 3.9-10 µs | 1.8-2.3 µs | 3.8-3.9 µs |

**Note**: RefCountedArena wins for repeated protections of the same value due to efficient reference counting.

---

## PPSize Analysis

### ProtectScope at ppsize Boundaries

R's protect stack is shared across all operations. When running after other benchmarks, available slots decrease.

| Protections Attempted | Median Time | Notes |
|----------------------|-------------|-------|
| 10,000 | 141µs | Stable |
| 20,000 | 259µs | Stable |
| 30,000 | 344µs | Stable |
| 40,000 | 435µs | Stable |
| 49,000 | 580µs | Near limit |
| 49,500 | 573µs | Near limit |
| 49,900 | 549µs | At limit (~40 slots used by R init) |

**Formula**: `max_available = 50000 - ~40 (R init) = ~49,960 protections`

### Arena Performance Across ppsize Range

Full comparison at extended scale (median times):

| Protections | RefCountedArena | HashMapArena | ThreadLocalArena | ThreadLocalHashArena | Winner |
|-------------|-----------------|--------------|------------------|----------------------|--------|
| 10k | 845µs | 747µs | 761µs | **440µs** | TL-Hash |
| 50k | 4.2ms | 3.5ms | 3.9ms | **2.6ms** | TL-Hash |
| 100k | 8.5ms | 7.5ms | 8.3ms | **5.6ms** | TL-Hash |
| 200k | 17.7ms | 17.3ms | **17.5ms** | 17.3ms | Tie |
| 300k | 29.6ms | 35.6ms | **28.0ms** | 34.9ms | TL-BTree |
| 400k | **37.9ms** | 40.8ms | 40.4ms | 47.9ms | RC-BTree |
| 500k | **47.5ms** | 58.1ms | 47.4ms | 72.2ms | TL-BTree |

**Key insight**: HashMap's O(1) operations are faster for small counts, but at scale (>200k), BTreeMap's predictable memory layout and better cache locality overcome the theoretical O(log n) disadvantage.

**Observation**: Per-protection cost increases at scale due to hash table resizing and memory pressure.

---

## FFI and R Interop

### Basic R FFI Calls

| Operation | Time |
|-----------|------|
| Scalar integer | 9.2-11.6 ns |
| Scalar logical | 3.5-3.7 ns |
| Scalar real | 9.2-11.3 ns |
| xlength | 7.5-7.6 ns |
| INTEGER_ELT | 7.5-7.6 ns |
| REAL_ELT | 7.4-7.7 ns |
| INTEGER_PTR | 7.5-7.7 ns |

### Vector Allocation

| Type | Size | Time |
|------|------|------|
| INTSXP | 1 | 8.3-10.9 ns |
| INTSXP | 256 | 75-96 ns |
| INTSXP | 4096 | 586-799 ns |
| REALSXP | 1 | 6.6-10.3 ns |
| REALSXP | 4096 | 208ns-3.6µs |
| RAWSXP | 1 | 7.3-9.8 ns |
| RAWSXP | 4096 | 204-271 ns |

### Checked vs Unchecked FFI

| Operation | Checked | Unchecked |
|-----------|---------|-----------|
| Scalar integer | 7.4-9.0 ns | 6.1-10.3 ns |
| xlength | 8.9-9.7 ns | 6.7-7.5 ns |
| Alloc vector (256) | 73-89 ns | 41-159 ns |

**Conclusion**: Checked FFI adds ~1-2 ns overhead per call, negligible for most use cases.

---

## Type Conversions

### Rust to R (into_r)

| Type | Size=1 | Size=256 | Size=65536 |
|------|--------|----------|------------|
| `i32` | 9.6-12 ns | 63-108 ns | 3.6-47µs |
| `f64` | 7.2-12 ns | 153-210 ns | 6.8-35µs |
| `u8` | 13-18 ns | 36-48 ns | 3.8-5.5µs |
| `bool` | 3.6-3.8 ns | - | - |
| `String` | 89-104 ns | 11.9-13.5µs | 3.5-3.7ms |
| `&str` | 49-58 ns | 4.9-5.1µs | 1.2-1.3ms |
| `Option<i32>` (no NA) | 34-36 ns | 168-173 ns | 27-38µs |
| `Option<i32>` (50% NA) | 34-38 ns | 165-177 ns | 27-28µs |

### R to Rust (from_r)

| Type | Size=1 | Size=256 | Size=65536 |
|------|--------|----------|------------|
| `i32` slice | 21.2-22.7 ns | 21.2-26.6 ns | 21.2-26.6 ns |
| `f64` slice | 21.2-24.6 ns | 21.2-26.6 ns | 21.2-26.6 ns |
| `u8` slice | 26.7-40 ns | 29-34 ns | 31-37 ns |
| Scalar `i32` | 26.9-35.2 ns | - | - |
| Scalar `f64` | 24.8-30.4 ns | - | - |
| `String` (UTF-8) | 43.7-49.8 ns | - | - |
| `String` (Latin-1) | 250ns-15µs | - | - |

**Key Insight**: Slice access is essentially zero-copy (~21 ns regardless of size).

### Type Coercion

| Conversion | R coerce | Rust coerce |
|------------|----------|-------------|
| int → real (scalar) | 29-36 ns | 22.5-23 ns |
| real → int (scalar) | 28-38 ns | 22.5-23 ns |
| int → real (64k) | 36-81µs | 7.7-9.4µs |
| real → int (64k) | 36-50µs | 53-56µs |
| raw → int (64k) | 18-27µs | 4.7-5.0µs |

**Recommendation**: Use Rust coercion for numeric conversions - 3-10x faster than R.

---

## Memory and Allocation

### R Allocator vs System Allocator

| Size | R Allocator | System Allocator | Ratio |
|------|-------------|------------------|-------|
| 8 bytes | 60-75 ns | 11-15 ns | 5-6x |
| 64 bytes | 60-84 ns | 14-17 ns | 4-5x |
| 1024 bytes | 136-177 ns | 23-24 ns | 6-7x |
| 8192 bytes | 380-520 ns | 22 ns | 17-24x |
| 65536 bytes | 656-960 ns | 515-540 ns | 1.3x |

**Conclusion**: System allocator is significantly faster for small allocations. R allocator overhead is due to GC tracking.

### Zeroed Allocation

| Size | R Allocator | System Allocator |
|------|-------------|------------------|
| 8 bytes | 61-71 ns | 11-16 ns |
| 1024 bytes | 78-170 ns | 20-40 ns |
| 65536 bytes | 3.5-4.3µs | 791-901 ns |

---

## ALTREP

### ALTREP vs Plain Vectors

| Operation | Plain | ALTREP (no expand) | ALTREP (expanded) |
|-----------|-------|--------------------|--------------------|
| INTEGER_ELT | 9.0-9.1 ns | 192-247 ns | 13.6-17µs |
| DATAPTR | 9.3-10.7 ns | 166-208 ns | 13.2-17.8µs |
| REAL_ELT | 9.0-9.1 ns | 166-283 ns | 27.5-36.4µs |

**Key Insight**: ALTREP has ~20-25x overhead for element access. When materialized (expanded), overhead increases to ~1500x due to full vector creation.

### ALTREP Iteration

| Operation | No expansion | After 2 expansions | After 4 expansions |
|-----------|--------------|--------------------|--------------------|
| INTEGER_ELT iteration | 375-416 ns | 411-495 ns | 531-708 ns |
| xlength | 268-313 ns | 307-365 ns | 427-530 ns |

---

## ExternalPtr

### Creation and Access

| Operation | Time |
|-----------|------|
| Create (small payload) | 186-213 ns |
| Create (medium payload) | 224-287 ns |
| Create (large payload) | 208-637 ns |
| Access as ref | 3.7-4.3 ns |
| Access as ptr | 3.7-4.0 ns |
| Deref | 3.8-4.0 ns |
| as_sexp | 0.002-0.019 ns |
| get_tag | 3.8-3.9 ns |
| set_protected | 15.4-16.0 ns |

### Type-Erased Operations

| Operation | Time |
|-----------|------|
| erased_is (hit) | 122.7-124 ns |
| erased_is (miss) | 123.4-125 ns |
| erased_downcast_ref (hit) | 127.3-131 ns |
| erased_downcast_mut (hit) | 127.3-131 ns |

### Baseline Comparisons

| Operation | ExternalPtr | Box (Rust) |
|-----------|-------------|------------|
| Small payload | 186-213 ns | 13.5-14.2 ns |
| Medium payload | 224-287 ns | 20-87 ns |
| Large payload | 208-637 ns | 1.8-2.1µs |

**Note**: ExternalPtr creation is ~10-15x slower than Box for small payloads due to R object creation overhead.

---

## String Operations

### String Conversion Performance

| Operation | Short (1 char) | Medium (256 char) | Long (65536 char) |
|-----------|----------------|-------------------|-------------------|
| mkCharLen | 8.1-8.3 ns | 220-226 ns | 60-65µs |
| from_r CStr | 12.7-13.1 ns | 99-101 ns | 1.1-1.2µs |
| Translate (UTF-8) | 7.5-7.6 ns | - | - |
| Translate (Latin-1) | 208-431 ns | - | - |

### String Interning

| Operation | Time |
|-----------|------|
| Empty string (R_BlankString) | 0.002 ns |
| Empty string (mkCharLen) | 41-158 ns |

**Recommendation**: Use `R_BlankString` for empty strings - effectively free.

---

## Worker Thread

### Thread Dispatch Overhead

| Operation | Time |
|-----------|------|
| run_on_worker (no R) | 2.6-4.3µs |
| run_on_worker (with R thread) | 5.1-6.8µs |
| with_r_thread (main) | 11.9-14.4 ns |

**Key Insight**: Worker thread dispatch adds ~3-5µs overhead. Main thread R access is essentially free.

---

## Miscellaneous

### List Operations

| Operation | Size | Time |
|-----------|------|------|
| Derive into_list (named) | - | 125-300 ns |
| Derive into_list (tuple) | - | 64-78 ns |
| Derive try_from_list (named) | - | 175-177 ns |
| Derive try_from_list (tuple) | - | 53-54 ns |
| List get by index | - | 32 ns |
| List get by name (first) | - | 52-53 ns |
| List get by name (last, 65536 elements) | - | 53-58µs |

### Factor Operations

| Operation | Time |
|-----------|------|
| Single factor (cached) | 42-52 ns |
| Single factor (uncached) | 362-386 ns |
| 100 factors (cached) | 4.3-6.0µs |
| 100 factors (uncached) | 37-41µs |

### Unwind Protect

| Operation | Time |
|-----------|------|
| Direct noop | 0.002 ns |
| R_UnwindProtect noop | 33.9-34.5 ns |

**Key Insight**: `R_UnwindProtect` adds ~34 ns overhead per call.

### Trait ABI (Cross-Package)

| Operation | Time |
|-----------|------|
| mx_query_vtable | 0.78-0.81 ns |
| query_view_value | 25.9-30.5 ns |
| view_value_only | 24.6-30.0 ns |
| baseline_direct | 0.002 ns |

---

## Summary Recommendations

1. **GC Protection**:
   - Use `ProtectScope` for < 50k protections (fastest, but limited)
   - Use `ThreadLocalHashArena` for 10k-100k protections
   - Use `ThreadLocalArena` for > 100k protections

2. **Type Conversions**:
   - Prefer Rust coercion over R's `Rf_coerceVector` (3-10x faster)
   - Use slice views instead of copying when possible (zero-copy)

3. **Strings**:
   - Use `R_BlankString` for empty strings
   - UTF-8 strings are ~30x faster than Latin-1 (no translation needed)

4. **Memory**:
   - System allocator is 5-20x faster for small allocations
   - Consider using Rust vectors and converting at boundaries

5. **Worker Thread**:
   - Batch operations to amortize thread dispatch overhead (~3-5µs per call)
   - Main thread R access is essentially free (~14 ns)

6. **ALTREP**:
   - Avoid materializing ALTREP vectors unless necessary
   - Element access has ~20-25x overhead vs plain vectors
