# miniextendr Benchmark Summary - 2025-12-30

## Key Performance Highlights

### FFI Overhead is Minimal

| Operation | Time | Notes |
|-----------|------|-------|
| `INTEGER_ELT` / `REAL_ELT` | 7-8ns | Element access |
| `INTEGER` / `REAL` (ptr) | 7-8ns | Pointer access |
| `Rf_xlength` | 7-8ns | Length query |
| `Rf_ScalarInteger` | 8-13ns | Scalar creation |
| `Rf_ScalarReal` | 10-14ns | Scalar creation |
| `PROTECT/UNPROTECT` | 14-18ns | Single protect |

### ExternalPtr is Fast

| Operation | Time | Notes |
|-----------|------|-------|
| `as_ref` / `deref` / `as_ptr` | 3.5-4ns | Essentially pointer dereference |
| `as_sexp` | ~0ns | Inlined, no-op |
| Create small (8B) | 144-181ns | Box + R allocation |
| Create medium (1KB) | 198-252ns | |
| Create large (64KB) | 166-541ns | Includes Box allocation |

**Comparison to Box:**
- `Box::new` small: 15ns
- `Box::new` medium: 24ns
- `Box::new` large: 1.5µs (includes 64KB allocation)

### ALTREP Overhead

| Operation | Plain Vector | ALTREP | Slowdown |
|-----------|-------------|--------|----------|
| `INTEGER_ELT` | 8-9ns | 196-240ns | ~25x |
| `REAL_ELT` | - | 208-387ns | - |
| `DATAPTR` | 8-9ns | 166-287ns | ~20-30x |

ALTREP trades speed for memory efficiency and laziness - the overhead is expected.

### Coercion: Rust vs R

| Operation | R Coerce | Rust Coerce | Winner |
|-----------|----------|-------------|--------|
| scalar int→real | 29-35ns | 23ns | Rust (1.3x) |
| vec[65536] int→real | 45-75µs | 7.7µs | Rust (6-10x) |
| vec[65536] raw→int | 18-20µs | 4.7µs | Rust (4x) |
| vec[65536] real→int | 36-51µs | 54µs | R (slight) |

### Protection Mechanisms

| Mechanism | Time | Notes |
|-----------|------|-------|
| `PROTECT/UNPROTECT` | 14-18ns | Traditional R approach |
| Preserve list (checked) | 53-66ns | Safe API |
| Preserve list (unchecked) | 28-40ns | Skip thread check |
| `R_UnwindProtect` | 32ns | Exception safety |

### Allocator Comparison

| Operation | System | RAllocator | Overhead |
|-----------|--------|------------|----------|
| alloc 8B | 11-16ns | 41-83ns | 4-5x |
| alloc 64B | 14-20ns | 66-97ns | 4-5x |
| alloc 1KB | 23ns | 138-194ns | 6-8x |
| alloc 8KB | 22ns | 390-625ns | 18-28x |
| alloc 64KB | 531-537ns | 276ns-3.7µs | Variable |
| batch 100×64B | 1.2µs | 7.5-8µs | 6x |
| vec growth 8→1KB | 232-257ns | 708-952ns | 3-4x |

RAllocator overhead comes from R's GC integration and preserve mechanism.

### Worker Thread Dispatch

| Operation | Time |
|-----------|------|
| `run_on_worker` (no R) | 3-6µs |
| `run_on_worker` (with R) | 5-11µs |
| `with_r_thread` main thread | 9-15ns |

Cross-thread dispatch is ~5-10µs; main thread fast-path is essentially free.

### String Performance

| Operation | Time | Notes |
|-----------|------|-------|
| UTF-8 to `&str` | 17ns | Zero-copy pointer |
| UTF-8 to `String` | 37ns | Allocation + copy |
| Latin-1 to `String` | 250-580ns | Requires translation (~10-15x slower) |

### List Access Patterns

| Operation | 16 elements | 256 elements | 4096 elements |
|-----------|-------------|--------------|---------------|
| `get_index` first | 32ns | 32ns | 32ns |
| `get_index` last | 33ns | 33ns | 34ns |
| `get_named` first | 52ns | 52ns | 52ns |
| `get_named` last | 230ns | 2.9µs | 53µs |

Named lookup is O(n) - use positional access when possible.

### List Derive Performance

| Operation | Time |
|-----------|------|
| `IntoList` (named, 4 fields) | 125-286ns |
| `IntoList` (tuple, 3 fields) | 51-76ns |
| `TryFromList` (named, 4 fields) | 163-167ns |
| `TryFromList` (tuple, 3 fields) | 52-53ns |

### RMatrix Access Patterns

| Access Pattern | 64×64 (4K) | 256×256 (64K) | Notes |
|----------------|------------|---------------|-------|
| `as_slice` sum | 3.2µs | 51µs | Fastest - contiguous |
| `column` slices | 5.8µs | 57µs | Per-column overhead |
| `get_rc` | 159µs | 2.6ms | Per-element overhead (~50x slower) |
| `to_vec` copy | 417ns | 7µs | memcpy cost |

### Collection Conversions (from_r)

| Target Type | 16 elements | 256 elements | 65536 elements |
|-------------|-------------|--------------|----------------|
| `HashSet<i32>` | 189ns | 2.3µs | 1ms |
| `BTreeSet<i32>` | 182ns | 1.2µs | 259µs |
| `HashMap<String, i32>` | 1µs | - | - |
| `BTreeMap<String, i32>` | 1.5µs | - | - |

Named list sizes: 16, 256, 4096 elements.

### Vec Conversion Performance

| Size | Zero-copy slice | Vec memcpy | Vec coerce (i32→i64) |
|------|-----------------|------------|---------------------|
| 1 | 21ns | 35-41ns | 33ns |
| 16 | 21ns | 37-43ns | 47-50ns |
| 256 | 21ns | 54-57ns | 56-62ns |
| 4096 | 21ns | 229-246ns | 375-470ns |
| 65536 | 21ns | 3.8-3.9µs | 7.0-7.8µs |

**Key insight:** Zero-copy slices are O(1) regardless of size.

## Recommendations

1. **Prefer slices over owned Vecs** - `&[i32]` / `&[f64]` are O(1) zero-copy
2. **Use positional list access** - `get_index` is O(1), `get_named` is O(n)
3. **Avoid per-element matrix access** - `get_rc` is ~50x slower than `as_slice`
4. **UTF-8 strings are fast** - Latin-1 translation is the bottleneck
5. **Use unchecked FFI in hot paths** - saves ~3-5ns per call after validation
6. **ALTREP is for large/lazy data** - overhead is amortized over many elements
7. **BTreeSet outperforms HashSet** for R conversions (4x faster at 65K elements)

## Test Environment

- Platform: Darwin 25.2.0 (macOS)
- Timer precision: 41ns
- Benchmark framework: divan 0.1.21

## Notes

- ExternalPtr erased downcast benchmarks failed due to a bug (needs investigation)
- Size indices 0-4 map to: 1, 16, 256, 4096, 65536 elements
- Matrix size indices 0-1 map to: 64×64, 256×256
